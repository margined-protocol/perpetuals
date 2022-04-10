use cosmwasm_std::{
    to_binary, Addr, BankMsg, Coin, CosmosMsg, Deps, Env, MessageInfo, ReplyOn, Response, StdError,
    StdResult, Storage, SubMsg, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use terraswap::asset::{Asset, AssetInfo};

use crate::{
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
use margined_perp::margined_vamm::{CalcFeeResponse, Direction};
use margined_perp::{
    margined_engine::{PnlCalcOption, PositionUnrealizedPnlResponse, RemainMarginResponse, Side},
    querier::query_token_balance,
};

pub fn execute_transfer_from(
    storage: &dyn Storage,
    owner: &Addr,
    receiver: &Addr,
    amount: Uint128,
) -> StdResult<SubMsg> {
    let config = read_config(storage)?;

    let msg: CosmosMsg = match config.eligible_collateral {
        AssetInfo::NativeToken { denom } => CosmosMsg::Bank(BankMsg::Send {
            to_address: receiver.to_string(),
            amount: vec![Coin { denom, amount }],
        }),
        AssetInfo::Token { contract_addr } => CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                owner: owner.to_string(),
                recipient: receiver.to_string(),
                amount,
            })?,
        }),
    };

    let transfer_msg = SubMsg {
        msg,
        gas_limit: None, // probably should set a limit in the config
        id: 0u64,
        reply_on: ReplyOn::Never,
    };

    Ok(transfer_msg)
}

pub fn execute_transfer(
    storage: &dyn Storage,
    receiver: &Addr,
    amount: Uint128,
) -> StdResult<SubMsg> {
    let config = read_config(storage)?;
    let msg = WasmMsg::Execute {
        contract_addr: config.eligible_collateral.to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: receiver.to_string(),
            amount,
        })?,
    };

    let transfer_msg = SubMsg {
        msg: CosmosMsg::Wasm(msg),
        gas_limit: None, // probably should set a limit in the config
        id: 0u64,
        reply_on: ReplyOn::Never,
    };

    Ok(transfer_msg)
}

pub fn execute_transfer_to_insurance_fund(
    deps: Deps,
    env: Env,
    amount: Uint128,
) -> StdResult<SubMsg> {
    let config = read_config(deps.storage)?;

    let token_balance = query_token_balance(
        deps,
        config.eligible_collateral.clone(),
        env.contract.address,
    )?;

    let amount_to_send = if token_balance < amount {
        token_balance
    } else {
        amount
    };

    let msg = WasmMsg::Execute {
        contract_addr: config.eligible_collateral.to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: config.insurance_fund.to_string(),
            amount: amount_to_send,
        })?,
    };

    let transfer_msg = SubMsg {
        msg: CosmosMsg::Wasm(msg),
        gas_limit: None, // probably should set a limit in the config
        id: 0u64,
        reply_on: ReplyOn::Never,
    };

    Ok(transfer_msg)
}

// Transfers the toll and spread fees to the the insurance fund and fee pool
pub fn transfer_fees(
    deps: Deps,
    from: Addr,
    vamm: Addr,
    notional: Uint128,
) -> StdResult<Vec<SubMsg>> {
    let config = read_config(deps.storage)?;

    let CalcFeeResponse {
        spread_fee,
        toll_fee,
    } = query_vamm_calc_fee(&deps, vamm.into_string(), notional)?;

    let mut messages: Vec<SubMsg> = vec![];

    if !spread_fee.is_zero() {
        let msg =
            execute_transfer_from(deps.storage, &from, &config.insurance_fund, spread_fee).unwrap();
        messages.push(msg);
    };

    if !toll_fee.is_zero() {
        let msg = execute_transfer_from(deps.storage, &from, &config.fee_pool, toll_fee).unwrap();
        messages.push(msg);
    };

    Ok(messages)
}

pub fn withdraw(
    deps: Deps,
    env: Env,
    state: &mut State,
    receiver: &Addr,
    insurance_fund: &Addr,
    eligible_collateral: AssetInfo,
    amount: Uint128,
) -> StdResult<Vec<SubMsg>> {
    let token_balance =
        query_token_balance(deps, eligible_collateral, env.contract.address.clone())?;
    let mut messages: Vec<SubMsg> = vec![];

    let mut shortfall = Uint128::zero();

    if token_balance < amount {
        shortfall = amount.checked_sub(token_balance)?;

        messages.push(
            execute_transfer_from(
                deps.storage,
                insurance_fund,
                &env.contract.address,
                shortfall,
            )
            .unwrap(),
        );
    }
    messages.push(execute_transfer(deps.storage, receiver, amount).unwrap());

    // add any shortfall to bad_debt
    state.bad_debt += shortfall;

    Ok(messages)
}

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

    println!("\nmargin delta: {}", margin_delta);
    println!("funding_payment: {}", funding_payment);
    println!("position.margin: {}\n", position.margin);

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
