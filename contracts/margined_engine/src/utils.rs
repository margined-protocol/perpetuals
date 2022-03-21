use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, Deps, Env, ReplyOn, Response, StdError, StdResult, Storage, SubMsg,
    Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use crate::{
    querier::{query_vamm_calc_fee, query_vamm_output_price, query_vamm_output_twap},
    state::{read_config, read_position, read_state, read_vamm, store_state, Position, VammList},
};

use margined_common::integer::Integer;
use margined_perp::margined_engine::{
    Pnl, PnlCalcOption, PnlResponse, PositionUnrealizedPnlResponse, RemainMarginResponse, Side,
};
use margined_perp::margined_vamm::{CalcFeeResponse, Direction};

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
            execute_transfer_from(storage, &config.insurance_fund, &contract_address, bad_debt)
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

pub fn require_vamm(storage: &dyn Storage, vamm: &Addr) -> StdResult<Response> {
    // check that it is a registered vamm
    let vamm_list: VammList = read_vamm(storage)?;
    if !vamm_list.is_vamm(vamm.as_ref()) {
        return Err(StdError::generic_err("vAMM is not registered"));
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
pub fn _switch_direction(dir: Direction) -> Direction {
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

// Checks that margin ratio is greater than base margin
pub fn require_margin(base_margin: Uint128, margin_ratio: Uint128) -> StdResult<Response> {
    if margin_ratio < base_margin {
        return Err(StdError::generic_err("Position is undercollateralized"));
    }

    Ok(Response::new())
}

pub fn require_insufficient_margin(
    base_margin: Uint128,
    margin_ratio: Uint128,
    polarity: bool,
) -> StdResult<Response> {
    if margin_ratio > base_margin && polarity {
        return Err(StdError::generic_err("Position is overcollateralized"));
    }

    Ok(Response::new())
}

pub fn get_position_notional_unrealized_pnl(
    deps: Deps,
    position: &Position,
    calc_option: PnlCalcOption,
) -> StdResult<PositionUnrealizedPnlResponse> {
    let mut position_notional = Uint128::zero();
    let mut unrealized_pnl = Uint128::zero();

    let position_size = position.size;
    if !position_size.is_zero() {
        match calc_option {
            PnlCalcOption::TWAP => {
                position_notional = query_vamm_output_twap(
                    &deps,
                    position.vamm.to_string(),
                    position.direction.clone(),
                    position_size.value,
                )?;
            }
            PnlCalcOption::SPOTPRICE => {
                position_notional = query_vamm_output_price(
                    &deps,
                    position.vamm.to_string(),
                    position.direction.clone(),
                    position_size.value,
                )?;
            }
            PnlCalcOption::ORACLE => {}
        }
        if position.notional > position_notional {
            unrealized_pnl = position.notional.checked_sub(position_notional)?;
        } else {
            unrealized_pnl = position_notional.checked_sub(position.notional)?;
        }
    }

    Ok(PositionUnrealizedPnlResponse {
        position_notional,
        unrealized_pnl,
    })
}

pub fn calc_pnl(
    output: Uint128,
    previous_notional: Uint128,
    direction: Direction,
) -> StdResult<PnlResponse> {
    // calculate delta from the trade
    let profit_loss: Pnl;
    let value: Uint128 = if output > previous_notional {
        if direction == Direction::AddToAmm {
            profit_loss = Pnl::Profit;
        } else {
            profit_loss = Pnl::Loss;
        }
        output.checked_sub(previous_notional)?
    } else {
        if direction == Direction::AddToAmm {
            profit_loss = Pnl::Loss;
        } else {
            profit_loss = Pnl::Profit;
        }
        previous_notional.checked_sub(output)?
    };

    Ok(PnlResponse { value, profit_loss })
}

pub fn calc_remain_margin_with_funding_payment(
    position: &Position,
    pnl: PnlResponse,
) -> StdResult<RemainMarginResponse> {
    // calculate the funding payment

    // calculate the remaining margin
    let mut bad_debt = Uint128::zero();
    let remaining_margin: Uint128 = if pnl.profit_loss == Pnl::Profit {
        position.margin.checked_add(pnl.value)?
    } else if pnl.value < position.margin {
        position.margin.checked_sub(pnl.value)?
    } else {
        // if the delta is bigger than margin we
        // will have some bad debt and margin out is gonna
        // be zero
        bad_debt = pnl.value.checked_sub(position.margin)?;
        Uint128::zero()
    };

    // if the remain is negative, set it to zero
    // and set the rest to
    Ok(RemainMarginResponse {
        funding_payment: Uint128::zero(),
        margin: remaining_margin,
        bad_debt,
    })
}

// negative means trader pays and vice versa
pub fn calc_funding_payment(
    storage: &dyn Storage,
    position: Position,
    latest_premium_fraction: Uint128,
) -> Integer {
    let config = read_config(storage).unwrap();
    if !position.size.is_zero() {
        let signed_premium_fraction = Integer::new_positive(latest_premium_fraction);
        let signed_prev_premium_fraction: Integer =
            Integer::new_positive(position.last_updated_premium_fraction);

        (signed_premium_fraction - signed_prev_premium_fraction) * position.size
            / Integer::new_positive(config.decimals)
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
    position.last_updated_premium_fraction = Uint128::zero();
    position.timestamp = env.block.time;

    Ok(position)
}
