use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, Deps, DepsMut, Env, ReplyOn, Response, StdError, StdResult,
    Storage, SubMsg, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use crate::{
    handle::{
        calc_pnl, calc_remain_margin_with_funding_payment, clear_position, get_position,
        get_position_notional_unrealized_pnl, internal_increase_position,
    },
    querier::query_vamm_calc_fee,
    state::{
        read_config, read_state, read_tmp_swap, remove_tmp_swap, store_position, store_state,
        store_tmp_swap,
    },
    utils::side_to_direction,
};

use margined_perp::margined_engine::{Pnl, PnlCalcOption};
use margined_perp::margined_vamm::CalcFeeResponse;
use margined_perp::querier::query_token_balance;

// Increases position after successful execution of the swap
pub fn increase_position_reply(
    deps: DepsMut,
    env: Env,
    _input: Uint128,
    output: Uint128,
) -> StdResult<Response> {
    let config = read_config(deps.storage)?;
    let tmp_swap = read_tmp_swap(deps.storage)?;
    if tmp_swap.is_none() {
        return Err(StdError::generic_err("no temporary position"));
    }

    let swap = tmp_swap.unwrap();
    let mut position = get_position(
        env.clone(),
        deps.storage,
        &swap.vamm,
        &swap.trader,
        swap.side.clone(),
    );

    // now update the position
    position.size = position.size.checked_add(output)?;
    position.notional = position.notional.checked_add(swap.open_notional)?;
    position.direction = side_to_direction(swap.side);

    // TODO make my own decimal math lib
    position.margin = position
        .notional
        .checked_mul(config.decimals)?
        .checked_div(swap.leverage)?;

    store_position(deps.storage, &position)?;

    // create transfer message
    let msg = execute_transfer_from(
        deps.storage,
        &swap.trader,
        &env.contract.address,
        position.margin,
    )
    .unwrap();

    // create messages to pay for toll and spread fees
    let fee_msgs = transfer_fee(deps.as_ref(), swap.trader, swap.vamm, position.notional).unwrap();

    remove_tmp_swap(deps.storage);
    Ok(Response::new().add_submessage(msg).add_messages(fee_msgs))
}

// Decreases position after successful execution of the swap
pub fn decrease_position_reply(
    deps: DepsMut,
    env: Env,
    _input: Uint128,
    output: Uint128,
) -> StdResult<Response> {
    let tmp_swap = read_tmp_swap(deps.storage)?;
    if tmp_swap.is_none() {
        return Err(StdError::generic_err("no temporary position"));
    }

    let swap = tmp_swap.unwrap();
    let mut position = get_position(
        env,
        deps.storage,
        &swap.vamm,
        &swap.trader,
        swap.side.clone(),
    );

    // now update the position
    position.size = position.size.checked_sub(output)?;
    position.notional = position.notional.checked_sub(swap.open_notional)?;

    store_position(deps.storage, &position)?;

    // remove the tmp position
    remove_tmp_swap(deps.storage);

    Ok(Response::new())
}

// Decreases position after successful execution of the swap
pub fn reverse_position_reply(
    deps: DepsMut,
    env: Env,
    _input: Uint128,
    output: Uint128,
) -> StdResult<Response> {
    let response: Response = Response::new();
    let tmp_swap = read_tmp_swap(deps.storage)?;
    if tmp_swap.is_none() {
        return Err(StdError::generic_err("no temporary position"));
    }

    let mut swap = tmp_swap.unwrap();
    let mut position = get_position(
        env.clone(),
        deps.storage,
        &swap.vamm,
        &swap.trader,
        swap.side.clone(),
    );
    let margin_amount = position.margin;

    position = clear_position(env, position)?;

    let msg: SubMsg;
    // now increase the position again if there is additional position
    let open_notional: Uint128;
    if swap.open_notional > output {
        open_notional = swap.open_notional.checked_sub(output)?;
        swap.open_notional = swap.open_notional.checked_sub(output)?;
    } else {
        open_notional = output.checked_sub(swap.open_notional)?;
        swap.open_notional = output.checked_sub(swap.open_notional)?;
    }
    if open_notional.checked_div(swap.leverage)? == Uint128::zero() {
        // create transfer message
        msg = execute_transfer(deps.storage, &swap.trader, margin_amount).unwrap();
        remove_tmp_swap(deps.storage);
    } else {
        store_tmp_swap(deps.storage, &swap)?;

        msg = internal_increase_position(swap.vamm, swap.side, open_notional)?
    }

    store_position(deps.storage, &position)?;

    Ok(response.add_submessage(msg))
}

// Closes position after successful execution of the swap
pub fn close_position_reply(
    deps: DepsMut,
    env: Env,
    _input: Uint128,
    output: Uint128,
) -> StdResult<Response> {
    let config = read_config(deps.storage)?;

    let tmp_swap = read_tmp_swap(deps.storage)?;
    if tmp_swap.is_none() {
        return Err(StdError::generic_err("no temporary position"));
    }

    let swap = tmp_swap.unwrap();
    let mut position = get_position(
        env.clone(),
        deps.storage,
        &swap.vamm,
        &swap.trader,
        swap.side.clone(),
    );

    let _position_pnl =
        get_position_notional_unrealized_pnl(deps.as_ref(), &position, PnlCalcOption::SPOTPRICE)?;

    // calculate delta from trade and whether it was profitable or a loss
    let pnl = calc_pnl(output, swap.open_notional, position.direction.clone())?;

    let remain_margin =
        calc_remain_margin_with_funding_payment(&position, pnl.value, pnl.profit_loss.clone())?;

    let mut messages: Vec<SubMsg> = vec![];

    if pnl.profit_loss == Pnl::Profit {
        let token_balance = query_token_balance(
            deps.as_ref(),
            config.eligible_collateral,
            env.contract.address.clone(),
        )?;
        if remain_margin.remaining_margin <= token_balance {
            messages.push(
                execute_transfer(deps.storage, &swap.trader, remain_margin.remaining_margin)
                    .unwrap(),
            );
        } else {
            let short_fall = remain_margin.remaining_margin.checked_sub(token_balance)?;

            let mut state = read_state(deps.storage)?;

            messages.push(execute_transfer(deps.storage, &swap.trader, token_balance).unwrap());
            messages.push(
                execute_transfer_from(
                    deps.storage,
                    &config.insurance_fund,
                    &swap.trader,
                    short_fall,
                )
                .unwrap(),
            );
            state.bad_debt = short_fall;
            store_state(deps.storage, &state)?;
        }
    } else if pnl.value < position.margin {
        // create transfer message
        messages.push(
            execute_transfer(deps.storage, &swap.trader, remain_margin.remaining_margin).unwrap(),
        );
    } else {
        // TODO probably log prepaidBadDebt here
        let mut state = read_state(deps.storage)?;

        if state.bad_debt.is_zero() {
            // create transfer from message
            messages.push(
                execute_transfer_from(
                    deps.storage,
                    &config.insurance_fund,
                    &env.contract.address,
                    remain_margin.bad_debt,
                )
                .unwrap(),
            );
            state.bad_debt = remain_margin.bad_debt;
        } else {
            state.bad_debt = Uint128::zero();
        }
        store_state(deps.storage, &state)?;
    };

    // now start putting the response together
    let mut response = Response::new();
    response = response.add_submessages(messages);

    // create messages to pay for toll and spread fees
    let fee_msgs = transfer_fee(deps.as_ref(), swap.trader, swap.vamm, position.notional).unwrap();
    response = response.add_messages(fee_msgs);

    position = clear_position(env, position)?;

    // remove_position(deps.storage, &position)?;
    store_position(deps.storage, &position)?;

    remove_tmp_swap(deps.storage);

    Ok(response)
}

fn execute_transfer_from(
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

fn execute_transfer(storage: &dyn Storage, receiver: &Addr, amount: Uint128) -> StdResult<SubMsg> {
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
