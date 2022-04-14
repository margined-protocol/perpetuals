use cosmwasm_std::{
    Addr, Deps, Env, MessageInfo, Response, StdError, StdResult, Storage, SubMsg, Uint128,
};
use terraswap::asset::{Asset, AssetInfo};

use crate::{
    messages::execute_transfer_from,
    querier::{
        query_vamm_calc_fee, query_vamm_config, query_vamm_output_price, query_vamm_output_twap,
    },
    query::query_cumulative_premium_fraction,
    state::{
        read_config, read_position, read_state, read_vamm, read_vamm_map, store_state, Position,
        State, VammList,
    },
};

use margined_common::integer::Integer;
use margined_perp::margined_engine::{
    PnlCalcOption, PositionUnrealizedPnlResponse, RemainMarginResponse, Side,
};
use margined_perp::margined_vamm::{CalcFeeResponse, Direction};

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

pub fn realize_bad_debt(
    storage: &mut dyn Storage,
    contract_address: Addr,
    bad_debt: Uint128,
    messages: &mut Vec<SubMsg>,
) -> StdResult<Response> {
    let config = read_config(storage)?;
    let mut state = read_state(storage)?;

    if state.bad_debt.is_zero() {
        // create transfer from message
        messages.push(
            execute_transfer_from(
                storage,
                &config.insurance_fund,
                &contract_address,
                bad_debt, // Uint128::from(1000000000u64),
            )
            .unwrap(),
        );
        state.bad_debt = bad_debt;
    } else {
        state.bad_debt = Uint128::zero();
    };

    store_state(storage, &state)?;

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

    if !cap.is_zero() {
        let mut updated_open_interest =
            amount.checked_add(Integer::new_positive(state.open_interest_notional))?;

        if updated_open_interest.is_negative() {
            updated_open_interest = Integer::zero();
        }

        if amount.is_positive() && updated_open_interest > Integer::new_positive(cap) {
            return Err(StdError::generic_err("over limit"));
        }

        state.open_interest_notional = updated_open_interest.value;
    }

    Ok(Response::new())
}

pub fn get_position_notional_unrealized_pnl(
    deps: Deps,
    position: &Position,
    calc_option: PnlCalcOption,
) -> StdResult<PositionUnrealizedPnlResponse> {
    let mut output_notional = Uint128::zero();
    let mut unrealized_pnl = Integer::zero();

    let position_size = position.size;
    if !position_size.is_zero() {
        match calc_option {
            PnlCalcOption::TWAP => {
                output_notional = query_vamm_output_twap(
                    &deps,
                    position.vamm.to_string(),
                    position.direction.clone(),
                    position_size.value,
                )?;
            }
            PnlCalcOption::SPOTPRICE => {
                output_notional = query_vamm_output_price(
                    &deps,
                    position.vamm.to_string(),
                    position.direction.clone(),
                    position_size.value,
                )?;
            }
            PnlCalcOption::ORACLE => {}
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

// ensures that sufficient native token is sent inclusive the fees, TODO consider tax
pub fn require_native_token_sent(
    deps: &Deps,
    info: MessageInfo,
    vamm: Addr,
    amount: Uint128,
    leverage: Uint128,
) -> StdResult<Response> {
    let config = read_config(deps.storage)?;

    if let AssetInfo::NativeToken { .. } = config.eligible_collateral.clone() {
        let quote_asset_amount = amount.checked_mul(leverage)?.checked_div(config.decimals)?;

        let CalcFeeResponse {
            spread_fee,
            toll_fee,
        } = query_vamm_calc_fee(deps, vamm.into_string(), quote_asset_amount)?;

        let total_amount = amount.checked_add(spread_fee)?.checked_add(toll_fee)?;

        let token = Asset {
            info: config.eligible_collateral,
            amount: total_amount,
        };

        token.assert_sent_native_token_balance(&info)?;
    };

    Ok(Response::new())
}

pub fn require_vamm(storage: &dyn Storage, vamm: &Addr) -> StdResult<Response> {
    // check that it is a registered vamm
    let vamm_list: VammList = read_vamm(storage)?;
    if !vamm_list.is_vamm(vamm.as_ref()) {
        return Err(StdError::generic_err("vAMM is not registered"));
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
pub fn require_margin(margin_ratio: Uint128, base_margin: Uint128) -> StdResult<Response> {
    let remaining_margin_ratio =
        Integer::new_positive(margin_ratio) - Integer::new_positive(base_margin);
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
        return Err(StdError::generic_err("margin engine is paused"));
    }

    Ok(Response::new())
}

// takes the side (buy|sell) and returns the direction (long|short)
pub fn side_to_direction(side: Side) -> Direction {
    match side {
        Side::BUY => Direction::AddToAmm,
        Side::SELL => Direction::RemoveFromAmm,
    }
}

// takes the direction (long|short) and returns the side (buy|sell)
pub fn direction_to_side(direction: Direction) -> Side {
    match direction {
        Direction::AddToAmm => Side::BUY,
        Direction::RemoveFromAmm => Side::SELL,
    }
}
