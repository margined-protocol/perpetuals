use cosmwasm_std::{
    Addr, Deps, Env, Event, MessageInfo, Response, StdError, StdResult, Storage, SubMsg,
    SubMsgResponse, Uint128,
};

use std::str::FromStr;

use margined_common::{
    asset::{Asset, AssetInfo},
    integer::Integer,
};
use margined_perp::margined_engine::{
    PnlCalcOption, Position, PositionUnrealizedPnlResponse, RemainMarginResponse, Side,
};
use margined_perp::margined_vamm::Direction;

use crate::{
    messages::execute_insurance_fund_withdrawal,
    querier::{
        query_insurance_is_vamm, query_vamm_config, query_vamm_output_amount,
        query_vamm_output_twap, query_vamm_state, query_vamm_underlying_price,
    },
    query::query_cumulative_premium_fraction,
    state::{read_config, read_position, read_vamm_map, State},
};

// reads position from storage but also handles the case where there is no
// previously stored position, i.e. a new position
pub fn get_position(
    env: Env,
    storage: &dyn Storage,
    vamm: &Addr,
    trader: &Addr,
    side: Side,
) -> Position {
    // read the position for the trader from vamm
    let mut position = read_position(storage, vamm, trader).unwrap();

    // so if the position returned is None then its new
    if position.vamm == Addr::unchecked("") {
        // update the default position
        position.vamm = vamm.clone();
        position.trader = trader.clone();
        position.direction = side_to_direction(side);
        position.block_number = env.block.height;
    }

    position
}

// Creates an asset from the eligible collateral and msg sent
pub fn get_asset(info: MessageInfo, eligible_collateral: AssetInfo) -> Asset {
    match eligible_collateral.clone() {
        AssetInfo::Token { .. } => Asset {
            info: eligible_collateral,
            amount: Uint128::zero(),
        },
        AssetInfo::NativeToken { denom } => {
            let sent = match info.funds.iter().find(|x| x.denom == *denom) {
                Some(coin) => coin.amount,
                None => Uint128::zero(),
            };
            Asset {
                info: eligible_collateral,
                amount: sent,
            }
        }
    }
}

pub fn realize_bad_debt(
    deps: Deps,
    bad_debt: Uint128,
    messages: &mut Vec<SubMsg>,
    state: &mut State,
) -> StdResult<Response> {
    if state.bad_debt.is_zero() {
        // create transfer from message
        messages.push(execute_insurance_fund_withdrawal(deps, bad_debt).unwrap());
        state.bad_debt = bad_debt;
    } else {
        state.bad_debt = Uint128::zero();
    };

    Ok(Response::new())
}

// this blocks trades if open interest is too high, required during the bootstrapping of the project
pub fn update_open_interest_notional(
    deps: &Deps,
    state: &mut State,
    vamm: Addr,
    amount: Integer,
) -> StdResult<Response> {
    let cap = query_vamm_config(deps, vamm.to_string())?.open_interest_notional_cap;

    let mut updated_open_interest =
        amount.checked_add(Integer::new_positive(state.open_interest_notional))?;

    if updated_open_interest.is_negative() {
        updated_open_interest = Integer::zero();
    }

    // check if the cap has been exceeded
    if !cap.is_zero() && amount.is_positive() && updated_open_interest > Integer::new_positive(cap)
    {
        return Err(StdError::generic_err("open interest exceeds cap"));
    }

    state.open_interest_notional = updated_open_interest.value;

    Ok(Response::new())
}

// this blocks trades if user's position exceeds max base asset holding cap
pub fn check_base_asset_holding_cap(deps: &Deps, vamm: Addr, size: Uint128) -> StdResult<Response> {
    let cap = query_vamm_config(deps, vamm.to_string())?.base_asset_holding_cap;

    // check if the cap has been exceeded
    // TODO add concept of whitelist
    if !cap.is_zero() && size > cap {
        return Err(StdError::generic_err("base asset holding exceeds cap"));
    }

    Ok(Response::new())
}

pub fn get_margin_ratio_calc_option(
    deps: Deps,
    vamm: String,
    trader: String,
    calc_option: PnlCalcOption,
) -> StdResult<Integer> {
    let config = read_config(deps.storage)?;

    // retrieve the latest position
    let position = read_position(
        deps.storage,
        &deps.api.addr_validate(&vamm)?,
        &deps.api.addr_validate(&trader)?,
    )
    .unwrap();

    if position.size.is_zero() {
        return Ok(Integer::zero());
    }

    let PositionUnrealizedPnlResponse {
        position_notional,
        unrealized_pnl,
    } = get_position_notional_unrealized_pnl(deps, &position, calc_option)?;

    let remain_margin = calc_remain_margin_with_funding_payment(deps, position, unrealized_pnl)?;

    let margin_ratio = ((Integer::new_positive(remain_margin.margin)
        - Integer::new_positive(remain_margin.bad_debt))
        * Integer::new_positive(config.decimals))
        / Integer::new_positive(position_notional);

    Ok(margin_ratio)
}

pub fn get_position_notional_unrealized_pnl(
    deps: Deps,
    position: &Position,
    calc_option: PnlCalcOption,
) -> StdResult<PositionUnrealizedPnlResponse> {
    let mut output_notional = Uint128::zero();
    let mut unrealized_pnl = Integer::zero();

    if !position.size.is_zero() {
        match calc_option {
            PnlCalcOption::Twap => {
                output_notional = query_vamm_output_twap(
                    &deps,
                    position.vamm.to_string(),
                    position.direction.clone(),
                    position.size.value,
                )?;
            }
            PnlCalcOption::SpotPrice => {
                output_notional = query_vamm_output_amount(
                    &deps,
                    position.vamm.to_string(),
                    position.direction.clone(),
                    position.size.value,
                )?;
            }
            PnlCalcOption::Oracle => {
                let config = read_config(deps.storage)?;
                let oracle_price: Uint128 =
                    query_vamm_underlying_price(&deps, position.vamm.to_string())?;

                output_notional = oracle_price
                    .checked_mul(position.size.value)?
                    .checked_div(config.decimals)?;
            }
        }

        // we are short if the size of the position is less than 0
        unrealized_pnl = if position.direction == Direction::AddToAmm {
            Integer::new_positive(output_notional) - Integer::new_positive(position.notional)
        } else {
            Integer::new_positive(position.notional) - Integer::new_positive(output_notional)
        };
    }

    Ok(PositionUnrealizedPnlResponse {
        position_notional: output_notional,
        unrealized_pnl,
    })
}

pub fn calc_remain_margin_with_funding_payment(
    deps: Deps,
    position: Position,
    margin_delta: Integer,
) -> StdResult<RemainMarginResponse> {
    let config = read_config(deps.storage)?;

    // calculate the funding payment
    let latest_premium_fraction =
        query_cumulative_premium_fraction(deps, position.vamm.to_string())?;
    let funding_payment = (latest_premium_fraction - position.last_updated_premium_fraction)
        * position.size
        / Integer::new_positive(config.decimals);

    // calculate the remaining margin
    let mut remaining_margin: Integer =
        margin_delta - funding_payment + Integer::new_positive(position.margin);
    let mut bad_debt = Integer::zero();

    if remaining_margin.is_negative() {
        bad_debt = remaining_margin.invert_sign();
        remaining_margin = Integer::zero();
    }

    // if the remain is negative, set it to zero
    // and set the rest to
    Ok(RemainMarginResponse {
        funding_payment,
        margin: remaining_margin.value,
        bad_debt: bad_debt.value,
        latest_premium_fraction,
    })
}

// negative means trader pays and vice versa
pub fn calc_funding_payment(
    position: Position,
    latest_premium_fraction: Integer,
    decimals: Uint128,
) -> Integer {
    if !position.size.is_zero() {
        (latest_premium_fraction - position.last_updated_premium_fraction) * position.size
            / Integer::new_positive(decimals)
            * Integer::new_negative(1u64)
    } else {
        Integer::ZERO
    }
}

// this resets the main variables of a position
pub fn clear_position(env: Env, mut position: Position) -> StdResult<Position> {
    position.size = Integer::zero();
    position.margin = Uint128::zero();
    position.notional = Uint128::zero();
    position.last_updated_premium_fraction = Integer::zero();
    position.block_number = env.block.height;

    Ok(position)
}

pub fn require_vamm(deps: Deps, insurance: &Addr, vamm: &Addr) -> StdResult<Response> {
    // check that it is a registered vamm
    if !query_insurance_is_vamm(&deps, insurance.to_string(), vamm.to_string())?.is_vamm {
        return Err(StdError::generic_err("vAMM is not registered"));
    }

    // check that vamm is open
    if !query_vamm_state(&deps, vamm.to_string())?.open {
        return Err(StdError::generic_err("vAMM is not open"));
    }

    Ok(Response::new())
}

// Check no bad debt
pub fn require_bad_debt(bad_debt: Uint128) -> StdResult<Response> {
    if !bad_debt.is_zero() {
        return Err(StdError::generic_err("Insufficient margin"));
    }

    Ok(Response::new())
}

// Checks that position isn't zero
pub fn require_position_not_zero(size: Uint128) -> StdResult<Response> {
    if size.is_zero() {
        return Err(StdError::generic_err("Position is zero"));
    }

    Ok(Response::new())
}

// Checks that margin ratio is greater than base margin
pub fn require_additional_margin(
    margin_ratio: Integer,
    base_margin: Uint128,
) -> StdResult<Response> {
    let remaining_margin_ratio = margin_ratio - Integer::new_positive(base_margin);
    if remaining_margin_ratio < Integer::zero() {
        return Err(StdError::generic_err("Position is undercollateralized"));
    }

    Ok(Response::new())
}

pub fn require_insufficient_margin(
    margin_ratio: Integer,
    base_margin: Uint128,
) -> StdResult<Response> {
    let remaining_margin_ratio = margin_ratio - Integer::new_positive(base_margin);
    if remaining_margin_ratio > Integer::zero() {
        return Err(StdError::generic_err("Position is overcollateralized"));
    }

    Ok(Response::new())
}

pub fn require_not_restriction_mode(
    storage: &dyn Storage,
    vamm: &Addr,
    trader: &Addr,
    block_height: u64,
) -> StdResult<Response> {
    let vamm_map = read_vamm_map(storage, vamm.clone())?;
    let position = read_position(storage, vamm, trader).unwrap();

    if vamm_map.last_restriction_block == block_height && position.block_number == block_height {
        return Err(StdError::generic_err("Only one action allowed"));
    }

    Ok(Response::new())
}

pub fn require_not_paused(paused: bool) -> StdResult<Response> {
    // check margin engine is not paused
    if paused {
        return Err(StdError::generic_err("Margin engine is paused"));
    }

    Ok(Response::new())
}
// check an input is non-zero
pub fn require_non_zero_input(input: Uint128) -> StdResult<Response> {
    if input.is_zero() {
        return Err(StdError::generic_err("Input must be non-zero"));
    }

    Ok(Response::new())
}

pub fn parse_swap(response: SubMsgResponse) -> StdResult<(Uint128, Uint128)> {
    // Find swap inputs and output events
    let wasm = response.events.iter().find(|&e| e.ty == "wasm");

    let wasm = wasm.unwrap();

    let swap = read_event("type".to_string(), wasm)?;

    let input: Uint128;
    let output: Uint128;
    match swap.as_str() {
        "input" => {
            let input_str = read_event("quote_asset_amount".to_string(), wasm)?;
            let output_str = read_event("base_asset_amount".to_string(), wasm)?;

            input = Uint128::from_str(&input_str).unwrap();
            output = Uint128::from_str(&output_str).unwrap();
        }
        "output" => {
            let input_str = read_event("base_asset_amount".to_string(), wasm)?;
            let output_str = read_event("quote_asset_amount".to_string(), wasm)?;

            input = Uint128::from_str(&input_str).unwrap();
            output = Uint128::from_str(&output_str).unwrap();
        }
        _ => {
            return Err(StdError::generic_err("Cannot parse swap"));
        }
    }

    Ok((input, output))
}

pub fn parse_pay_funding(response: SubMsgResponse) -> StdResult<(Integer, String)> {
    // Find swap inputs and output events
    let wasm = response.events.iter().find(|&e| e.ty == "wasm");
    let wasm = wasm.unwrap();

    let premium_str = read_event("premium_fraction".to_string(), wasm)?;
    let premium: Integer = Integer::from_str(&premium_str).unwrap();

    let sender = read_contract_address(wasm).unwrap();

    Ok((premium, sender))
}

// Reads contract address from an event takes into account that there are
// inconistencies between cosmwasm versions and multitest etc...
fn read_contract_address(event: &Event) -> StdResult<String> {
    let mut result = event
        .attributes
        .iter()
        .find(|&attr| attr.key == *"_contract_addr".to_string());

    if result.is_none() {
        result = event
            .attributes
            .iter()
            .find(|&attr| attr.key == *"_contract_address".to_string());
    }

    let value = &result.unwrap().value;

    Ok(value.to_string())
}

fn read_event(key: String, event: &Event) -> StdResult<String> {
    let result = event.attributes.iter().find(|&attr| attr.key == key);

    if result.is_none() {
        return Err(StdError::generic_err(format!("No event found: {}", key)));
    }

    let value = &result.unwrap().value;

    Ok(value.to_string())
}

// takes the side (buy|sell) and returns the direction (long|short)
pub fn side_to_direction(side: Side) -> Direction {
    match side {
        Side::Buy => Direction::AddToAmm,
        Side::Sell => Direction::RemoveFromAmm,
    }
}

// takes the direction (long|short) and returns the side (buy|sell)
pub fn direction_to_side(direction: Direction) -> Side {
    match direction {
        Direction::AddToAmm => Side::Buy,
        Direction::RemoveFromAmm => Side::Sell,
    }
}
