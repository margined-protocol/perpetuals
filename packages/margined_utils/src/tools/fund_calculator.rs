use cosmwasm_std::{
    to_binary, Addr, Coin, Empty, Querier, QuerierWrapper, QueryRequest, StdResult, Uint128,
    WasmQuery,
};
use margined_common::integer::Integer;
use margined_perp::margined_engine::QueryMsg as EngineQueryMsg;
use margined_perp::margined_engine::{
    PnlCalcOption, Position, PositionUnrealizedPnlResponse, Side,
};
use margined_perp::margined_vamm::{CalcFeeResponse, Direction, QueryMsg as VammQueryMsg};

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
) -> StdResult<Vec<Coin>> {
    let new_notional = quote_asset_amount
        .checked_mul(leverage)?
        .checked_div(SIX_D_P)?;

    // pull the fees for the vamm that the position will be taken on; note that this will be shifted however many digits
    let fee_amount: Uint128 = query_vamm_fees(querier, vamm.to_string(), new_notional)?;

    // check if they have an existing position so we can calculate if someone owes margin
    let mut position: Position = query_existing_position(
        querier,
        engine.to_string(),
        vamm.to_string(),
        trader.to_string(),
    )?;

    // Check the unrealised PnL and add it to: (existing position + margin)
    let unrealised_pnl = query_existing_position_pnl(
        querier,
        engine.to_string(),
        vamm.to_string(),
        trader.to_string(),
    )?;

    // Check if unrealised_pnl is positive or negative
    // if it is negative, we check that it is smaller than position.margin to prevent overflow
    let mut margin_owed: Integer = Integer::zero();
    match unrealised_pnl.unrealized_pnl.is_negative() {
        true => {
            position.margin = if position.margin > unrealised_pnl.unrealized_pnl.value {
                position.margin - unrealised_pnl.unrealized_pnl.value
            } else {
                margin_owed =
                    Integer::new_positive(unrealised_pnl.unrealized_pnl.value - position.margin);
                Uint128::zero()
            }
        }
        false => position.margin += unrealised_pnl.unrealized_pnl.value,
    };

    // First we check if they are increasing the position or not
    let is_increase: bool = position.direction == Direction::AddToAmm && side == Side::Buy
        || position.direction == Direction::RemoveFromAmm && side == Side::Sell;

    // If we are increasing, then we simply add the margin for the latest trade (quote_asset_amount)
    // If we are not increasing, then we need to calculate whether the position is reversing or not
    // If the position doesnt reverse, we don't need any more margin (we just decrease leverage + notional)
    // If the position does reverse, then we need to calculate the new notional position and
    // divide it by the leverage to find the margin required (first summand)
    match is_increase {
        true => {
            margin_owed += Integer::new_positive(quote_asset_amount);
        }
        false => {
            if unrealised_pnl.position_notional < new_notional {
                margin_owed += Integer::new_positive(
                    (new_notional - unrealised_pnl.position_notional)
                        .checked_mul(SIX_D_P)?
                        .checked_div(leverage)?,
                ) - Integer::new_positive(position.margin);
            }
        }
    };

    // calculate the final funds owed.
    // if margin_owed is negative, then the trader only pays fee_amount.
    // if margin_owed is positive then the trader pays fee_amount + margin_owed
    let funds_owed = if margin_owed.is_positive() {
        fee_amount + margin_owed.value
    } else {
        fee_amount
    };

    if funds_owed.is_zero() {
        Ok(vec![])
    } else {
        Ok(vec![Coin::new(funds_owed.u128(), "uusd")])
    }
}

// to query the given vamm's fees for use in the fund calculator
pub fn query_vamm_fees<Q: Querier>(
    querier: &Q,
    vamm_addr: String,
    quote_asset_amount: Uint128,
) -> StdResult<Uint128> {
    Ok(QuerierWrapper::<Empty>::new(querier)
        .query::<CalcFeeResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: vamm_addr,
            msg: to_binary(&VammQueryMsg::CalcFee { quote_asset_amount })?,
        }))?
        .toll_fee)
}

// to query the position of the given trader, on the given vamm for use in the fund calculator
pub fn query_existing_position<Q: Querier>(
    querier: &Q,
    engine: String,
    vamm: String,
    trader: String,
) -> StdResult<Position> {
    let position = QuerierWrapper::<Empty>::new(querier)
        .query::<Position>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: engine,
            msg: to_binary(&EngineQueryMsg::Position { vamm, trader })?,
        }))
        .unwrap_or_default();

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
