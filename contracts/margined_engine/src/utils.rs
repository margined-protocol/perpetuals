use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, Deps, Env, ReplyOn, Response, StdError, StdResult, Storage, SubMsg,
    Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use crate::{
    querier::{query_vamm_calc_fee, query_vamm_output_price, query_vamm_output_twap},
    query::query_cumulative_premium_fraction,
    state::{
        read_config, read_position, read_state, read_vamm, store_state, Position, State, VammList,
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
    let msg = WasmMsg::Execute {
        contract_addr: config.eligible_collateral.to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
            owner: owner.to_string(),
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
pub fn transfer_fee(
    deps: Deps,
    from: Addr,
    vamm: Addr,
    notional: Uint128,
) -> StdResult<Vec<WasmMsg>> {
    let config = read_config(deps.storage)?;

    let CalcFeeResponse {
        spread_fee,
        toll_fee,
    } = query_vamm_calc_fee(&deps, vamm.into_string(), notional)?;

    let mut messages: Vec<WasmMsg> = vec![];

    if !spread_fee.is_zero() {
        let msg = WasmMsg::Execute {
            contract_addr: config.eligible_collateral.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                owner: from.to_string(),
                recipient: config.insurance_fund.to_string(),
                amount: spread_fee,
            })?,
        };
        messages.push(msg);
    };

    if !toll_fee.is_zero() {
        let msg = WasmMsg::Execute {
            contract_addr: config.eligible_collateral.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                owner: from.to_string(),
                recipient: config.fee_pool.to_string(),
                amount: toll_fee,
            })?,
        };
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
    eligible_collateral: Addr,
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

    // this is unecessary but need to find a better way to do it
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
        position.timestamp = env.block.time;
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

    // TODO think some more why this logic is incorrect
    // when I did it this way I always had some dust left over
    // if state.bad_debt > bad_debt {
    //     state.bad_debt = state.bad_debt.checked_sub(bad_debt)?;
    // } else {
    //     // create transfer from message

    //     let msg = execute_transfer_from(
    //         storage,
    //         &config.insurance_fund,
    //         &contract_address,
    //         bad_debt.checked_sub(state.bad_debt)?,
    //     )
    //     .unwrap();
    //     messages.push(msg);
    //     state.bad_debt = Uint128::zero();
    // };

    store_state(storage, &state)?;

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
        let direction = if position.size < Integer::zero() {
            Direction::RemoveFromAmm
        } else {
            Direction::AddToAmm
        };

        match calc_option {
            PnlCalcOption::TWAP => {
                output_notional = query_vamm_output_twap(
                    &deps,
                    position.vamm.to_string(),
                    position.direction.clone(),
                    // direction,
                    position_size.value,
                )?;
            }
            PnlCalcOption::SPOTPRICE => {
                output_notional = query_vamm_output_price(
                    &deps,
                    position.vamm.to_string(),
                    position.direction.clone(),
                    // direction,
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

    // let margin = if position.size < Integer::zero() {
    //     println!("negative");
    //     Integer::new_negative(position.margin)
    // } else {
    //     println!("positive");
    //     Integer::new_positive(position.margin)
    // };

    // calculate the remaining margin
    let mut remaining_margin: Integer =
        margin_delta - funding_payment + Integer::new_positive(position.margin);
    let mut bad_debt = Integer::zero();

    // println!("{:?}", margin);
    // println!("{:?}", margin_delta);
    // println!("{:?}", remaining_margin);

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
    position.timestamp = env.block.time;

    Ok(position)
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

// takes the side (buy|sell) and returns opposite (short|long)
// this is useful when closing/reversing a position
pub fn switch_direction(dir: Direction) -> Direction {
    match dir {
        Direction::RemoveFromAmm => Direction::AddToAmm,
        Direction::AddToAmm => Direction::RemoveFromAmm,
    }
}

// takes the side (buy|sell) and returns opposite (short|long)
// this is useful when closing/reversing a position
pub fn _switch_side(dir: Side) -> Side {
    match dir {
        Side::BUY => Side::SELL,
        Side::SELL => Side::BUY,
    }
}
