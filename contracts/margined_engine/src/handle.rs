use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, ReplyOn, Response, StdError, StdResult,
    SubMsg, Uint128, WasmMsg,
};

use crate::{
    contract::{
        PAY_FUNDING_REPLY_ID, SWAP_CLOSE_REPLY_ID, SWAP_DECREASE_REPLY_ID, SWAP_INCREASE_REPLY_ID,
        SWAP_LIQUIDATE_REPLY_ID, SWAP_REVERSE_REPLY_ID,
    },
    querier::query_vamm_output_price,
    query::query_margin_ratio,
    state::{
        read_config, read_position, read_state, store_config, store_position, store_tmp_liquidator,
        store_tmp_swap, Config, Position, State, Swap,
    },
    utils::{
        calc_remain_margin_with_funding_payment, direction_to_side, execute_transfer_from,
        get_position, require_bad_debt, require_insufficient_margin, require_margin,
        require_position_not_zero, require_vamm, side_to_direction, withdraw,
    },
};
use margined_common::integer::Integer;
use margined_perp::margined_engine::Side;
use margined_perp::margined_vamm::{Direction, ExecuteMsg};

pub fn update_config(deps: DepsMut, info: MessageInfo, owner: String) -> StdResult<Response> {
    let config = read_config(deps.storage)?;
    if info.sender != config.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    let new_config = Config {
        owner: deps.api.addr_validate(&owner).unwrap(),
        ..config
    };

    store_config(deps.storage, &new_config)?;

    Ok(Response::default())
}

// Opens a position
// TODO - refactor arguments into a struct
#[allow(clippy::too_many_arguments)]
pub fn open_position(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    vamm: String,
    trader: String,
    side: Side,
    quote_asset_amount: Uint128,
    leverage: Uint128,
) -> StdResult<Response> {
    let config: Config = read_config(deps.storage)?;

    let vamm = deps.api.addr_validate(&vamm)?;
    let trader = deps.api.addr_validate(&trader)?;

    let margin_ratio = Uint128::from(1_000_000_000u64)
        .checked_mul(config.decimals)?
        .checked_div(leverage)?;

    require_vamm(deps.storage, &vamm)?;
    require_margin(margin_ratio, config.initial_margin_ratio)?;

    // calc the input amount wrt to leverage and decimals
    let open_notional = quote_asset_amount
        .checked_mul(leverage)?
        .checked_div(config.decimals)?;

    let position: Position = get_position(env.clone(), deps.storage, &vamm, &trader, side.clone());

    let mut is_increase: bool = true;
    if !(position.direction == Direction::AddToAmm && side == Side::BUY
        || position.direction == Direction::RemoveFromAmm && side == Side::SELL)
    {
        is_increase = false;
    }

    let msg: SubMsg = if is_increase {
        internal_increase_position(vamm.clone(), side.clone(), open_notional).unwrap()
    } else {
        open_reverse_position(
            &deps,
            env,
            vamm.clone(),
            trader.clone(),
            side.clone(),
            open_notional,
            false,
        )
    };

    store_tmp_swap(
        deps.storage,
        &Swap {
            vamm,
            trader,
            side,
            quote_asset_amount,
            leverage,
            open_notional,
        },
    )?;

    Ok(Response::new()
        .add_submessage(msg)
        .add_attributes(vec![("action", "open_position")]))
}

pub fn close_position(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    vamm: String,
    trader: String,
) -> StdResult<Response> {
    // validate address inputs
    let vamm = deps.api.addr_validate(&vamm)?;
    let trader = deps.api.addr_validate(&trader)?;

    // read the position for the trader from vamm
    let position = read_position(deps.storage, &vamm, &trader).unwrap();

    // check the position isn't zero
    require_position_not_zero(position.size.value)?;

    let msg = internal_close_position(deps, &position, SWAP_CLOSE_REPLY_ID)?;

    Ok(Response::new().add_submessage(msg))
}

pub fn liquidate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    vamm: String,
    trader: String,
) -> StdResult<Response> {
    let config: Config = read_config(deps.storage)?;

    // validate address inputs
    let vamm = deps.api.addr_validate(&vamm)?;
    let trader = deps.api.addr_validate(&trader)?;

    // store the liquidator
    store_tmp_liquidator(deps.storage, &info.sender)?;

    // check if margin ratio has been
    let margin = query_margin_ratio(deps.as_ref(), vamm.to_string(), trader.to_string())?;

    require_vamm(deps.storage, &vamm)?;
    require_insufficient_margin(
        config.maintenance_margin_ratio,
        margin.ratio,
        margin.polarity,
    )?;

    // read the position for the trader from vamm
    let position = read_position(deps.storage, &vamm, &trader).unwrap();

    // TODO First we should see if it is a partial or full liqudiation, but not today
    let msg: SubMsg;
    let mut response = Response::default();
    if false {
        // NOTHING in future this condition will be there to see if the liquidation is partial
    } else {
        msg = internal_close_position(deps, &position, SWAP_LIQUIDATE_REPLY_ID)?;
        response = response.add_submessage(msg);
    }

    Ok(response)
}

pub fn pay_funding(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    vamm: String,
) -> StdResult<Response> {
    // validate address inputs
    let vamm = deps.api.addr_validate(&vamm)?;

    // check its a valid vamm
    require_vamm(deps.storage, &vamm)?;

    let funding_msg = SubMsg {
        msg: CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: vamm.to_string(),
            funds: vec![],
            msg: to_binary(&ExecuteMsg::SettleFunding {})?,
        }),
        gas_limit: None, // probably should set a limit in the config
        id: PAY_FUNDING_REPLY_ID,
        reply_on: ReplyOn::Always,
    };

    Ok(Response::new().add_submessage(funding_msg))
}

/// Enables a user to directly deposit margin into their position
pub fn deposit_margin(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vamm: String,
    amount: Uint128,
) -> StdResult<Response> {
    let vamm = deps.api.addr_validate(&vamm)?;
    let trader = info.sender;

    // first try to execute the transfer
    let msg = execute_transfer_from(deps.storage, &trader, &env.contract.address, amount)?;

    // read the position for the trader from vamm
    let mut position = read_position(deps.storage, &vamm, &trader).unwrap();
    position.margin = position.margin.checked_add(amount)?;

    store_position(deps.storage, &position)?;

    Ok(Response::new().add_submessage(msg).add_attributes(vec![
        ("action", "deposit_margin"),
        ("trader", &trader.to_string()),
        ("amount", &amount.to_string()),
    ]))
}

/// Enables a user to directly withdraw excess margin from their position
pub fn withdraw_margin(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vamm: String,
    amount: Uint128,
) -> StdResult<Response> {
    let config: Config = read_config(deps.storage)?;
    let mut state: State = read_state(deps.storage)?;

    // get and validate address inputs
    let vamm = deps.api.addr_validate(&vamm)?;
    let trader = info.sender;

    require_vamm(deps.storage, &vamm)?;

    // read the position for the trader from vamm
    let mut position = read_position(deps.storage, &vamm, &trader).unwrap();

    // TODO this can be changed to an integer
    let margin_delta = Integer::new_negative(amount);

    let remain_margin =
        calc_remain_margin_with_funding_payment(deps.as_ref(), position.clone(), margin_delta)?;
    require_bad_debt(remain_margin.bad_debt)?;

    position.margin = remain_margin.margin;
    position.last_updated_premium_fraction = remain_margin.latest_premium_fraction;

    store_position(deps.storage, &position)?;

    // check if margin ratio has been
    let margin = query_margin_ratio(deps.as_ref(), vamm.to_string(), trader.to_string())?;

    require_margin(margin.ratio, config.initial_margin_ratio)?;

    // try to execute the transfer
    let msgs = withdraw(
        deps.as_ref(),
        env,
        &mut state,
        &trader,
        &config.insurance_fund,
        config.eligible_collateral,
        amount,
    )
    .unwrap();

    Ok(Response::new().add_submessages(msgs).add_attributes(vec![
        ("action", "withdraw_margin"),
        ("trader", &trader.to_string()),
        ("amount", &amount.to_string()),
    ]))
}

// Increase the position, just basically wraps swap input though it may do more in the future
pub fn internal_increase_position(
    vamm: Addr,
    side: Side,
    open_notional: Uint128,
) -> StdResult<SubMsg> {
    swap_input(&vamm, side, open_notional, false, SWAP_INCREASE_REPLY_ID)
}
pub fn internal_close_position(deps: DepsMut, position: &Position, id: u64) -> StdResult<SubMsg> {
    let swap_msg = WasmMsg::Execute {
        contract_addr: position.vamm.to_string(),
        funds: vec![],
        msg: to_binary(&ExecuteMsg::SwapOutput {
            direction: position.direction.clone(),
            base_asset_amount: position.size.value,
        })?,
    };

    store_tmp_swap(
        deps.storage,
        &Swap {
            vamm: position.vamm.clone(),
            trader: position.trader.clone(),
            side: direction_to_side(position.direction.clone()),
            quote_asset_amount: position.size.value,
            leverage: Uint128::zero(),
            open_notional: position.notional,
        },
    )?;

    Ok(SubMsg {
        msg: CosmosMsg::Wasm(swap_msg),
        gas_limit: None, // probably should set a limit in the config
        id,
        reply_on: ReplyOn::Always,
    })
}

// Increase the position, just basically wraps swap input though it may do more in the future
fn open_reverse_position(
    deps: &DepsMut,
    env: Env,
    vamm: Addr,
    trader: Addr,
    side: Side,
    open_notional: Uint128,
    can_go_over_fluctuation: bool,
) -> SubMsg {
    let position: Position = get_position(env, deps.storage, &vamm, &trader, side.clone());
    let current_notional = query_vamm_output_price(
        &deps.as_ref(),
        vamm.to_string(),
        position.direction.clone(),
        position.size.value,
    )
    .unwrap();

    // if position.notional > open_notional {
    let msg: SubMsg = if current_notional > open_notional {
        // then we are opening a new position or adding to an existing
        swap_input(
            &vamm,
            side,
            open_notional,
            can_go_over_fluctuation,
            SWAP_DECREASE_REPLY_ID,
        )
        .unwrap()
    } else {
        // first close position swap out the entire position
        swap_output(
            &vamm,
            direction_to_side(position.direction.clone()),
            position.size.value,
            SWAP_REVERSE_REPLY_ID,
        )
        .unwrap()
    };

    msg
}

fn swap_input(
    vamm: &Addr,
    side: Side,
    open_notional: Uint128,
    can_go_over_fluctuation: bool,
    id: u64,
) -> StdResult<SubMsg> {
    let direction: Direction = side_to_direction(side);

    let msg = WasmMsg::Execute {
        contract_addr: vamm.to_string(),
        funds: vec![],
        msg: to_binary(&ExecuteMsg::SwapInput {
            direction,
            quote_asset_amount: open_notional,
            can_go_over_fluctuation,
        })?,
    };

    let execute_submsg = SubMsg {
        msg: CosmosMsg::Wasm(msg),
        gas_limit: None, // probably should set a limit in the config
        id,
        reply_on: ReplyOn::Always,
    };

    Ok(execute_submsg)
}

fn swap_output(vamm: &Addr, side: Side, open_notional: Uint128, id: u64) -> StdResult<SubMsg> {
    let direction: Direction = side_to_direction(side);

    let swap_msg = WasmMsg::Execute {
        contract_addr: vamm.to_string(),
        funds: vec![],
        msg: to_binary(&ExecuteMsg::SwapOutput {
            direction,
            base_asset_amount: open_notional,
        })?,
    };

    let execute_submsg = SubMsg {
        msg: CosmosMsg::Wasm(swap_msg),
        gas_limit: None, // probably should set a limit in the config
        id,
        reply_on: ReplyOn::Always,
    };

    Ok(execute_submsg)
}
