use cosmwasm_std::{
    to_binary, Addr, Empty, Querier, QuerierWrapper, QueryRequest, StdResult, Uint128, WasmQuery,
};
use margined_common::integer::Integer;
use margined_perp::margined_engine::QueryMsg as EngineQueryMsg;
use margined_perp::margined_engine::{
    PnlCalcOption, Position, PositionUnrealizedPnlResponse, Side,
};
use margined_perp::margined_vamm::{ConfigResponse, Direction, QueryMsg as VammQueryMsg};

//////////////////////////////////////////////////////////////////////////
/// This is a tool for calculating the total funds needed when we want
/// to send a transaction using the native token, as the total amount  
/// needs to be known beforehand. This function need to know the       
/// existing position of the trader, and the vamm (to pull fees) which
/// they want to open a position on.                                   
//////////////////////////////////////////////////////////////////////////

// note that this tool calculates if someone is owed margin, but does not deduct the amount from the fees owned

pub const SIX_D_P: Uint128 = Uint128::new(1_000_000u128); // this is 6d.p.

// this function needs to be used before a (new) position is opened
pub fn calculate_funds_needed<Q: Querier>(
    querier: &Q,
    engine: Addr,
    trader: Addr,
    quote_asset_amount: Uint128,
    leverage: Uint128,
    side: Side,
    vamm: Addr,
) -> StdResult<Uint128> {
    // pull the fees for the vamm that the position will be taken on; note that this will be shifted however many digits
    let fee_rate: Uint128 = query_vamm_fees(querier, vamm.to_string())?;

    // calculate the fees wrt the notional position
    let fee_amount = fee_rate
        .checked_mul(quote_asset_amount)?
        .checked_div(SIX_D_P)? // to correct for the fee_rate
        .checked_mul(leverage)? // this gives us the notional position
        .checked_div(SIX_D_P)?; // to correct for the leverage

    // check if they have an existing position so we can calculate if someone owes margin
    let mut position: Position = query_existing_position(
        querier,
        engine.to_string(),
        vamm.to_string(),
        trader.to_string(),
    )?;

    // If there is no previous position, then we only require these funds
    if position == Position::default() {
        return Ok(fee_amount + quote_asset_amount);
    };

    println!("HERE");

    // initialise variable for use below
    let mut margin_owed: Integer = Integer::zero();

    // here we check the unrealised PnL and add it to the existing position + margin
    let unrealised_pnl_response = query_existing_position_pnl(
        querier,
        engine.to_string(),
        vamm.to_string(),
        trader.to_string(),
    )?;

    // update the notional with PnL
    position.notional = unrealised_pnl_response.position_notional;
    println!("{}", position.notional);

    // here we add the pnl to the margin
    // we check if unrealised_pnl is positive or negative to decide whether to add it or not
    // if it is negative, we check that it is smaller than position.margin to prevent overflow (we don't liquidate here)
    match unrealised_pnl_response.unrealized_pnl.negative {
        true => {
            position.margin = if position.margin > unrealised_pnl_response.unrealized_pnl.value {
                position.margin - unrealised_pnl_response.unrealized_pnl.value
            } else {
                margin_owed = Integer::new_positive(
                    unrealised_pnl_response.unrealized_pnl.value - position.margin,
                );
                Uint128::zero()
            }
        }
        false => position.margin += unrealised_pnl_response.unrealized_pnl.value,
    };

    // First we check if they are increasing the position or not
    let is_increase: bool = position.direction == Direction::AddToAmm && side == Side::Buy
        || position.direction == Direction::RemoveFromAmm && side == Side::Sell;

    // this is the notional value of the position they want to open - if they have an existing position, this won't be the final notional value
    let new_notional = quote_asset_amount
        .checked_mul(leverage)?
        .checked_div(SIX_D_P)?;

    // If we are increasing, then we simply add the margin for the latest trade (quote_asset_amount)
    // If we are not increasing, then we need to calculate whether the position is reversing or not
    // If the position doesnt reverse, we don't need any more margin (we just decrease leverage + notional)
    // If the position does reverse, then we need to calculate the new notional position and
    // divide it by the leverage to find the margin required (first summand)
    if is_increase {
        margin_owed += Integer::new_positive(quote_asset_amount);
    } else if position.notional > new_notional {
        margin_owed += Integer::zero();
    } else {
        margin_owed += Integer::new_positive(
            (new_notional - position.notional)
                .checked_mul(SIX_D_P)?
                .checked_div(leverage)?,
        ) - Integer::new_positive(position.margin);
    };

    // calculate the final funds owed.
    // if margin_owed is negative, then the trader only pays fee_amount.
    // if margin_owed is positive then the trader pays fee_amount + margin_owed
    let funds_owed = if margin_owed.is_positive() {
        fee_amount + margin_owed.value
    } else {
        fee_amount
    };

    Ok(funds_owed)
}

// to query the given vamm's fees for use in the fund calculator
pub fn query_vamm_fees<Q: Querier>(querier: &Q, vamm_addr: String) -> StdResult<Uint128> {
    let fee_rate = QuerierWrapper::<Empty>::new(querier)
        .query::<ConfigResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: vamm_addr,
            msg: to_binary(&VammQueryMsg::Config {})?,
        }))?
        .toll_ratio;
    Ok(fee_rate)
}

// to query the position of the given trader, on the given vamm for use in the fund calculator
pub fn query_existing_position<Q: Querier>(
    querier: &Q,
    engine: String,
    vamm: String,
    trader: String,
) -> StdResult<Position> {
    let position = QuerierWrapper::<Empty>::new(querier).query::<Position>(&QueryRequest::Wasm(
        WasmQuery::Smart {
            contract_addr: engine,
            msg: to_binary(&EngineQueryMsg::Position { vamm, trader })?,
        },
    ))?;
    Ok(position)
}

// to query the PnL of the given trader's position on the given vamm, for use in the fund calculator
pub fn query_existing_position_pnl<Q: Querier>(
    querier: &Q,
    engine: String,
    vamm: String,
    trader: String,
) -> StdResult<PositionUnrealizedPnlResponse> {
    let pnl_response = QuerierWrapper::<Empty>::new(querier)
        .query::<PositionUnrealizedPnlResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: engine,
            msg: to_binary(&EngineQueryMsg::UnrealizedPnl {
                vamm,
                trader,
                calc_option: PnlCalcOption::SpotPrice,
            })?,
        }))?;
    Ok(pnl_response)
}
