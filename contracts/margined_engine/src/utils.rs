use cosmwasm_std::{
    Addr, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Storage, SubMsg,
    SubMsgResponse, Uint128,
};
use margined_utils::contracts::helpers::{InsuranceFundController, VammController};
use sha3::{Digest, Sha3_256};

use std::str::FromStr;

use margined_common::{
    asset::{Asset, AssetInfo},
    integer::Integer,
    messages::{read_event, read_response},
};
use margined_perp::margined_engine::{
    PnlCalcOption, Position, PositionUnrealizedPnlResponse, RemainMarginResponse, Side,
};
use margined_perp::margined_vamm::Direction;

use crate::{
    contract::{PAUSER, WHITELIST},
    messages::execute_insurance_fund_withdrawal,
    query::query_cumulative_premium_fraction,
    state::{read_config, read_position, read_state, read_vamm_map, store_state, State},
};

pub fn keccak_256(input: &[u8]) -> Vec<u8> {
    // create a SHA3-256 object
    let mut hasher = Sha3_256::new();

    // write input message
    hasher.update(input);

    // read hash digest
    hasher.finalize().to_vec()
}

// Creates an asset from the eligible collateral and msg sent
pub fn get_asset(info: MessageInfo, eligible_collateral: AssetInfo) -> Asset {
    match &eligible_collateral {
        AssetInfo::Token { .. } => Asset {
            info: eligible_collateral,
            amount: Uint128::zero(),
        },
        AssetInfo::NativeToken { denom } => {
            let sent = match info.funds.iter().find(|&x| x.denom.eq(denom)) {
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
) -> StdResult<Uint128> {
    if state.prepaid_bad_debt > bad_debt {
        // no need to move extra tokens because vault already prepay bad debt, only need to update the numbers
        state.prepaid_bad_debt = state.prepaid_bad_debt.checked_sub(bad_debt)?;
    } else {
        // in order to realize all the bad debt vault need extra tokens from insuranceFund
        let bad_debt_delta = bad_debt.checked_sub(state.prepaid_bad_debt)?;

        messages.push(execute_insurance_fund_withdrawal(deps, bad_debt_delta)?);

        state.prepaid_bad_debt = Uint128::zero();

        return Ok(bad_debt_delta);
    };

    Ok(Uint128::zero())
}

// this blocks trades if open interest is too high, required during the bootstrapping of the project
pub fn update_open_interest_notional(
    deps: &Deps,
    state: &mut State,
    vamm: Addr,
    amount: Integer,
    trader: Addr,
) -> StdResult<Response> {
    let vamm_controller = VammController(vamm);
    let cap = vamm_controller
        .config(&deps.querier)?
        .open_interest_notional_cap;
    println!("update_open_interest_notional - cap: {}", cap);
    let mut updated_open_interest =
        amount.checked_add(Integer::new_positive(state.open_interest_notional))?;

    if updated_open_interest.is_negative() {
        updated_open_interest = Integer::zero();
    }


    println!("update_open_interest_notional - updated_open_interest: {}", updated_open_interest);

    // check if the cap has been exceeded - if trader address is in whitelist this bypasses
    if (!cap.is_zero()
        && amount.is_positive()
        && updated_open_interest > Integer::new_positive(cap))
        && !WHITELIST.query_hook(deps.to_owned(), trader.to_string())?
    {
        return Err(StdError::generic_err("open interest exceeds cap"));
    }

    state.open_interest_notional = updated_open_interest.value;

    Ok(Response::new())
}

// this blocks trades if user's position exceeds max base asset holding cap
pub fn check_base_asset_holding_cap(
    deps: &Deps,
    vamm: Addr,
    size: Uint128,
    trader: Addr,
) -> StdResult<Response> {
    let vamm_controller = VammController(vamm);
    let cap = vamm_controller
        .config(&deps.querier)?
        .base_asset_holding_cap;
    println!("check_base_asset_holding_cap - size: {}", size);
    println!("check_base_asset_holding_cap - cap: {}", cap);
    // check if the cap has been exceeded - if trader address is in whitelist this bypasses
    if (!cap.is_zero() && size > cap)
        && !WHITELIST.query_hook(deps.to_owned(), trader.to_string())?
    {
        return Err(StdError::generic_err("base asset holding exceeds cap"));
    }

    Ok(Response::new())
}

pub fn get_margin_ratio_calc_option(
    deps: Deps,
    vamm: String,
    position_id: u64,
    calc_option: PnlCalcOption,
) -> StdResult<Integer> {
    let config = read_config(deps.storage)?;
    let position_key = keccak_256(&[vamm.as_bytes()].concat());
    // retrieve the latest position
    let position = read_position(deps.storage, &position_key, position_id)?;

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

    let vamm_controller = VammController(position.vamm.clone());

    if !position.size.is_zero() {
        match calc_option {
            PnlCalcOption::Twap => {
                output_notional = vamm_controller.output_twap(
                    &deps.querier,
                    position.direction.clone(),
                    position.size.value,
                )?;
            }
            PnlCalcOption::SpotPrice => {
                output_notional = vamm_controller.output_amount(
                    &deps.querier,
                    position.direction.clone(),
                    position.size.value,
                )?;
            }
            PnlCalcOption::Oracle => {
                let config = read_config(deps.storage)?;
                let oracle_price = vamm_controller.underlying_price(&deps.querier)?;

                output_notional = oracle_price
                    .checked_mul(position.size.value)?
                    .checked_div(config.decimals)?;
            }
        }
        println!("get_position_notional_unrealized_pnl - output_notional: {}", output_notional);
        println!("get_position_notional_unrealized_pnl - position.notional: {}", position.notional);
        // we are short if the size of the position is less than 0
        unrealized_pnl = if position.direction == Direction::AddToAmm {
            Integer::new_positive(output_notional) - Integer::new_positive(position.notional)
        } else {
            Integer::new_positive(position.notional) - Integer::new_positive(output_notional)
        };
        println!("get_position_notional_unrealized_pnl - unrealized_pnl: {}", unrealized_pnl);
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
    // calculate the funding payment
    let latest_premium_fraction =
        query_cumulative_premium_fraction(deps, position.vamm.to_string())?;
    let config = read_config(deps.storage)?;
    let funding_payment = (latest_premium_fraction - position.last_updated_premium_fraction)
        * position.size
        / Integer::new_positive(config.decimals);
    // println!("calc_remain_margin_with_funding_payment - latest_premium_fraction: {}", latest_premium_fraction);
    // println!("calc_remain_margin_with_funding_payment - position.last_updated_premium_fraction: {}", position.last_updated_premium_fraction);
    // println!("calc_remain_margin_with_funding_payment - position.size: {}", position.size);
    // println!("calc_remain_margin_with_funding_payment - funding_payment: {}", funding_payment);
    // calculate the remaining margin
    let mut remaining_margin: Integer =
        margin_delta - funding_payment + Integer::new_positive(position.margin);
    let mut bad_debt = Integer::zero();
    // println!("calc_remain_margin_with_funding_payment - remaining_margin: {}", remaining_margin);
    if remaining_margin.is_negative() {
        bad_debt = remaining_margin.invert_sign();
        remaining_margin = Integer::zero();
    }
    // println!("calc_remain_margin_with_funding_payment - bad_debt: {}", bad_debt);
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

pub fn update_pauser(deps: DepsMut, info: MessageInfo, pauser: String) -> StdResult<Response> {
    // validate the address
    let valid_pauser = deps.api.addr_validate(&pauser)?;

    PAUSER
        .execute_update_admin(deps, info, Some(valid_pauser))
        .map_err(|error| StdError::generic_err(error.to_string()))
}

// Adds an address to the whitelist for base asset holding cap
pub fn add_whitelist(deps: DepsMut, info: MessageInfo, address: String) -> StdResult<Response> {
    // validate the address
    let valid_addr = deps.api.addr_validate(&address)?;

    WHITELIST
        .execute_add_hook(&PAUSER, deps, info, valid_addr)
        .map_err(|error| StdError::generic_err(error.to_string()))
}

// Removes an address to the whitelist for base asset holding cap
pub fn remove_whitelist(deps: DepsMut, info: MessageInfo, address: String) -> StdResult<Response> {
    // validate the address
    let valid_addr = deps.api.addr_validate(&address)?;

    WHITELIST
        .execute_remove_hook(&PAUSER, deps, info, valid_addr)
        .map_err(|error| StdError::generic_err(error.to_string()))
}

pub fn set_pause(deps: DepsMut, _env: Env, info: MessageInfo, pause: bool) -> StdResult<Response> {
    let mut state = read_state(deps.storage)?;

    // check permission and if state matches
    // note: we could use `assert_admin` instead of `is_admin` except this would throw an `AdminError` and we would have to change the function sig
    if !PAUSER.is_admin(deps.as_ref(), &info.sender)? || state.pause == pause {
        return Err(StdError::generic_err("unauthorized"));
    }

    state.pause = pause;

    store_state(deps.storage, &state)?;

    Ok(Response::default().add_attribute("action", "set_pause"))
}

pub fn require_vamm(deps: Deps, insurance: &Option<Addr>, vamm: &Addr) -> StdResult<Response> {
    let insurance = match insurance {
        Some(arr) => arr,
        None => return Err(StdError::generic_err("insurance fund is not registered")),
    };

    let insurance_controller = InsuranceFundController(insurance.clone());

    // check that it is a registered vamm
    if !insurance_controller.is_vamm(&deps.querier, vamm.to_string())? {
        return Err(StdError::generic_err("vAMM is not registered"));
    }

    let vamm_controller = VammController(vamm.clone());
    // check that vamm is open
    if !vamm_controller.state(&deps.querier)?.open {
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
    println!("require_additional_margin - margin_ratio: {:?}", margin_ratio);
    println!("require_additional_margin - base_margin: {:?}", base_margin);
    if margin_ratio < Integer::new_positive(base_margin) {
        return Err(StdError::generic_err("Position is undercollateralized"));
    }

    Ok(Response::new())
}

pub fn require_insufficient_margin(
    margin_ratio: Integer,
    base_margin: Uint128,
) -> StdResult<Response> {
    if margin_ratio > Integer::new_positive(base_margin) {
        return Err(StdError::generic_err("Position is overcollateralized"));
    }

    Ok(Response::new())
}

pub fn require_not_restriction_mode(
    storage: &dyn Storage,
    vamm: &Addr,
    block_height: u64,
) -> StdResult<Response> {
    let vamm_map = read_vamm_map(storage, vamm)?;

    if vamm_map.last_restriction_block == block_height {
        return Err(StdError::generic_err("Only one action allowed"));
    }

    Ok(Response::new())
}

// check margin engine is not paused
pub fn require_not_paused(paused: bool) -> StdResult<Response> {
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

pub fn parse_swap(response: &SubMsgResponse) -> StdResult<(Uint128, Uint128, u64)> {
    // Find swap inputs and output events
    let wasm = read_response("wasm", response)?;
    let swap = read_event("type", wasm)?;
    println!("parse_swap - swap: {:?}", swap);
    match swap {
        "input" => {
            let input_str = read_event("quote_asset_amount", wasm)?;
            let output_str = read_event("base_asset_amount", wasm)?;
            let position_id = read_event("position_id", wasm)?;
            Ok((
                Uint128::from_str(input_str)?,
                Uint128::from_str(output_str)?,
                u64::from_str(position_id).unwrap(),
            ))
        }
        "output" => {
            let input_str = read_event("base_asset_amount", wasm)?;
            let output_str = read_event("quote_asset_amount", wasm)?;
            let position_id = read_event("position_id", wasm)?;
            Ok((
                Uint128::from_str(input_str)?,
                Uint128::from_str(output_str)?,
                u64::from_str(position_id).unwrap(),
            ))
        }
        _ => {
            return Err(StdError::generic_err("Cannot parse swap"));
        }
    }
}

pub fn parse_pay_funding<'a>(response: &'a SubMsgResponse) -> StdResult<(Integer, &'a str)> {
    // Find swap inputs and output events
    let wasm = read_response("wasm", response)?;
    let premium_str = read_event("premium_fraction", wasm)?;
    let premium = Integer::from_str(premium_str)?;

    let sender = read_event("_contract_address", wasm)?;

    Ok((premium, sender))
}

// takes the side (buy|sell) and returns the direction (long|short)
pub fn side_to_direction(side: &Side) -> Direction {
    match side {
        Side::Buy => Direction::AddToAmm,
        Side::Sell => Direction::RemoveFromAmm,
    }
}

// takes the direction (long|short) and returns the side (buy|sell)
pub fn direction_to_side(direction: &Direction) -> Side {
    match direction {
        Direction::AddToAmm => Side::Buy,
        Direction::RemoveFromAmm => Side::Sell,
    }
}

pub fn position_to_side(size: Integer) -> Side {
    if size > Integer::zero() {
        Side::Sell
    } else {
        Side::Buy
    }
}

// upper bound key by 1, for Order::Ascending
pub fn calc_range_start(start_after: Option<Vec<u8>>) -> Option<Vec<u8>> {
    start_after.map(|mut input| {
        // zero out all trailing 255, increment first that is not such
        for i in (0..input.len()).rev() {
            if input[i] == 255 {
                input[i] = 0;
            } else {
                input[i] += 1;
                break;
            }
        }
        input
    })
}