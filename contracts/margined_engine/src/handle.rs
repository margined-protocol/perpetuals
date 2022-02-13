use std::str::FromStr;
use cosmwasm_std::{
    Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response,
    ReplyOn, StdError, StdResult, SubMsg, to_binary, Uint128,
    WasmMsg, Storage,
};
use cw20::{Cw20ExecuteMsg};

use margined_perp::margined_vamm::{Direction, ExecuteMsg};
use margined_perp::margined_engine::{Side};
use crate::{
    contract::{SWAP_INCREASE_REPLY_ID, SWAP_DECREASE_REPLY_ID, SWAP_REVERSE_REPLY_ID},
    state::{
        Config, read_config, store_config,
        Position, read_position, store_position,
        Swap, store_tmp_swap, read_tmp_swap, remove_tmp_swap,
    },
    utils::{
        require_vamm, side_to_direction, direction_to_side, switch_direction,
        switch_side,
    }
};

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: String,
) -> StdResult<Response> {
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
    let vamm = deps.api.addr_validate(&vamm)?;
    let trader = deps.api.addr_validate(&trader)?;
    require_vamm(deps.storage, &vamm)?;
    
    let config: Config = read_config(deps.storage)?;
    
    // create a response, so that we can assign relevant submessages
    let mut response = Response::new();

    // calc the input amount wrt to leverage and decimals
    let open_notional = quote_asset_amount
                .checked_mul(leverage)?
                .checked_div(config.decimals)?;

    let mut position: Position = get_position(env.clone(), deps.storage, &vamm, &trader, side.clone());

    let mut is_increase: bool = true;
    if !(position.direction == Direction::AddToAmm && side.clone() == Side::BUY) &&
            !(position.direction == Direction::RemoveFromAmm && side.clone() == Side::SELL) {
        is_increase = false;
    }

    let msg: SubMsg;
    if is_increase {
        msg = internal_increase_position(vamm.clone(), side.clone(), open_notional);

    } else {
        // TODO make this a function maybe called, open_reverse_position
        // if old position is greater then we don't need to reverse just reduce the position
        println!("{}, {}", position.notional, open_notional);
        msg = open_reverse_position(
            &deps,
            env,
            vamm.clone(),
            trader.clone(),
            side.clone(),
            open_notional
        );
    }

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

    Ok(response
        .add_submessage(msg)
        .add_attributes(vec![
            ("action", "open_position")
        ])
    )
}

pub fn close_position(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    vamm: String,
    trader: String,
    id: u64,
) -> StdResult<Response> {
    // validate address inputs
    let vamm = deps.api.addr_validate(&vamm)?;
    let trader = deps.api.addr_validate(&trader)?;

    // read the position for the trader from vamm
    let position = read_position(deps.storage, &vamm, &trader)?.unwrap();

    let direction: Direction = switch_direction(position.direction.clone());
    let amount = position.size;

    let swap_msg = WasmMsg::Execute {
        contract_addr: vamm.to_string(),
        funds: vec![],
        msg: to_binary(&ExecuteMsg::SwapOutput {
            direction: direction,
            base_asset_amount: amount,
        })?,
    };

    let msg = SubMsg {
        msg: CosmosMsg::Wasm(swap_msg),
        gas_limit: None, // probably should set a limit in the config
        id: id,
        reply_on: ReplyOn::Always,
    };

    // tmp_store_swap(deps.storage, &position)?;

    Ok(Response::new()
        .add_attributes(vec![("action", "close_position")])
        .add_submessage(msg)
    )
}

// Increases position after successful execution of the swap
pub fn increase_position_reply(
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
        swap.side.clone()
    );

    // now update the position
    position.size = position.size.checked_add(output)?;
    position.margin = position.margin.checked_add(swap.quote_asset_amount)?;
    position.notional = position.notional.checked_add(swap.open_notional)?;
    position.direction = side_to_direction(swap.side);

    store_position(deps.storage, &position)?;

    // create transfer message
    let msg = execute_transfer_from(
        deps.storage,
        &swap.trader,
        &env.contract.address,
        position.margin,
    ).unwrap();

    remove_tmp_swap(deps.storage);

    Ok(Response::new().add_submessage(msg))
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
        env.clone(),
        deps.storage,
        &swap.vamm,
        &swap.trader,
        swap.side.clone()
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
    let mut response: Response = Response::new();
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
        swap.side.clone()
    );
    let margin_amount = position.margin;

    position = clear_position(env.clone(), position)?;

    let msg: SubMsg;
    // now increase the position again if there is additional position
    let margin: Uint128;
    if swap.open_notional > output {
        margin = swap.open_notional.checked_sub(output)?;
    } else {
        margin = output.checked_sub(swap.open_notional)?;
    }

    if margin.checked_div(swap.leverage)? == Uint128::zero() {
        // create transfer message
        msg = execute_transfer(
            deps.storage,
            &swap.trader,
            margin_amount,
        ).unwrap();
    } else {
        // reverse the position and increase
        msg = internal_increase_position(swap.vamm, switch_side(swap.side), margin)
    }

    store_position(deps.storage, &position)?;

    remove_tmp_swap(deps.storage);
    
    Ok(response.add_submessage(msg))
}

// // Closes position returning funds after successful execution of the swap out
// pub fn close_position_reply(
//     deps: DepsMut,
//     env: Env,
//     _input: Uint128,
//     output: Uint128,
// ) -> StdResult<Response> {
//     let response: Response = Response::new();
//     let tmp_swap = read_tmp_swap(deps.storage)?;
//     if tmp_swap.is_none() {
//         return Err(StdError::generic_err("no temporary position"));
//     }
//     println!("Close Output {}", output);
//     let swap = tmp_swap.unwrap();
//     let mut position = get_position(
//         env.clone(),
//         deps.storage,
//         &swap.vamm,
//         &swap.trader,
//         swap.side.clone()
//     );
    
//     position = clear_position(env, position)?;

//     store_position(deps.storage, &position)?;
    
//     remove_tmp_swap(deps.storage);

//     Ok(response)
// }

// Increase the position, just basically wraps swap input though it may do more in the future
fn internal_increase_position(
    vamm: Addr,
    side: Side,
    open_notional: Uint128,
) -> SubMsg {
    swap_input(
        &vamm,
        side.clone(),
        open_notional,
        SWAP_INCREASE_REPLY_ID
    ).unwrap()
}

// Increase the position, just basically wraps swap input though it may do more in the future
fn open_reverse_position(
    deps: &DepsMut,
    env: Env,
    vamm: Addr,
    trader: Addr,
    side: Side,
    open_notional: Uint128,
) -> SubMsg {
    let msg: SubMsg;
    let position: Position = get_position(env.clone(), deps.storage, &vamm, &trader, side.clone());
    println!("Position {}", position.notional);
    if position.notional > open_notional {
        println!("decrease");
        // then we are opening a new position or adding to an existing
        msg = swap_input(
            &vamm,
            side.clone(),
            open_notional,
            SWAP_DECREASE_REPLY_ID
        ).unwrap();

    } else {    
        println!("reverse");        
        // first close position swap out the entire position
        msg = swap_output(
            &vamm,
            direction_to_side(position.direction.clone()),
            position.size,
            SWAP_REVERSE_REPLY_ID
        ).unwrap();
    }

    return msg
}


// this resets the main variables of a position
fn clear_position(
    env: Env,
    mut position: Position,
) -> StdResult<Position> {
    position.size = Uint128::zero();
    position.margin = Uint128::zero();
    position.notional = Uint128::zero();
    position.timestamp = env.block.time;

    Ok(position)
}

fn swap_input(
    vamm: &Addr,
    side: Side, 
    open_notional: Uint128, 
    id: u64,
) -> StdResult<SubMsg> {
    let direction: Direction = side_to_direction(side);

    let swap_msg = WasmMsg::Execute {
        contract_addr: vamm.to_string(),
        funds: vec![],
        msg: to_binary(&ExecuteMsg::SwapInput {
            direction: direction,
            quote_asset_amount: open_notional,
        })?,
    };

    let execute_submsg = SubMsg {
        msg: CosmosMsg::Wasm(swap_msg),
        gas_limit: None, // probably should set a limit in the config
        id: id,
        reply_on: ReplyOn::Always,
    };

    Ok(execute_submsg)
}

fn swap_output(
    vamm: &Addr,
    side: Side, 
    open_notional: Uint128, 
    id: u64,
) -> StdResult<SubMsg> {
    let direction: Direction = side_to_direction(side);

    let swap_msg = WasmMsg::Execute {
        contract_addr: vamm.to_string(),
        funds: vec![],
        msg: to_binary(&ExecuteMsg::SwapOutput {
            direction: direction,
            base_asset_amount: open_notional,
        })?,
    };

    let execute_submsg = SubMsg {
        msg: CosmosMsg::Wasm(swap_msg),
        gas_limit: None, // probably should set a limit in the config
        id: id,
        reply_on: ReplyOn::Always,
    };

    Ok(execute_submsg)
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
            amount: amount,
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

fn execute_transfer(
    storage: &dyn Storage,
    receiver: &Addr, 
    amount: Uint128, 
) -> StdResult<SubMsg> {
    println!("LOL AUDREY MASSAGE ME");
    let config = read_config(storage)?;
    let msg = WasmMsg::Execute {
        contract_addr: config.eligible_collateral.to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: receiver.to_string(),
            amount: amount,
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

fn get_position(
    env: Env,
    storage: &dyn Storage,
    vamm: &Addr,
    trader: &Addr,
    side: Side,
) -> Position {
    // read the position for the trader from vamm
    let current_position = read_position(storage, &vamm, &trader).unwrap();
    let mut position = Position::default();

    // so if the position returned is None then its new
    if current_position.is_none() {
        let direction: Direction = side_to_direction(side);

        // update the default position
        position.vamm = vamm.clone();
        position.trader = trader.clone();
        position.direction = direction;
        position.timestamp = env.block.time;

    } else {
        position = current_position.unwrap();
    }

    position
    
}


