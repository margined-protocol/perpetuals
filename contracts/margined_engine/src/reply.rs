use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, Deps, DepsMut, Env, ReplyOn, Response, StdError, StdResult,
    Storage, SubMsg, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use crate::{
    handle::{clear_position, get_position, internal_increase_position},
    querier::query_vamm_calc_fee,
    state::{read_config, read_tmp_swap, remove_tmp_swap, store_position, store_tmp_swap},
    utils::side_to_direction,
};

use margined_perp::margined_vamm::CalcFeeResponse;

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

    // calculate delta from the trade
    let delta: Uint128 = if output > swap.open_notional {
        output.checked_sub(swap.open_notional)?
    } else {
        swap.open_notional.checked_sub(output)?
    };

    let mut response = Response::new();

    // now calculate the margin to return and a potential shortfall is the
    // position is undercollateralised (note this may be taken from the insurance fund)
    let margin_returned: Uint128;
    // let mut shortfall = Uint128::zero();
    if delta < position.margin {
        margin_returned = position.margin.checked_sub(delta)?;

        // create transfer message
        let msg = execute_transfer(deps.storage, &swap.trader, margin_returned).unwrap();

        response = response.add_submessage(msg);
    }
    // TODO This should do something with the shortfall if there is insufficient collateral
    // probably transfer it from the insurance fund idk
    // else {
    //     margin_returned = Uint128::zero();
    //     // shortfall = delta.checked_sub(position.margin)?;
    // }

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
