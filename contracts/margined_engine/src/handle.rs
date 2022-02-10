use cosmwasm_std::{
    Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response,
    ReplyOn, StdError, StdResult, SubMsg, to_binary, Uint128,
    WasmMsg, Storage,
};
use cw20::{Cw20ExecuteMsg};

use margined_perp::margined_vamm::{Direction, ExecuteMsg};
use margined_perp::margined_engine::Side;
use crate::{
    contract::{SWAP_INCREASE_REPLY_ID, SWAP_DECREASE_REPLY_ID, SWAP_REVERSE_REPLY_ID},
    state::{
        Config, read_config, store_config,
        Position, read_position, store_position,
        store_tmp_position, read_tmp_position, remove_tmp_position,
        VammList, read_vamm,
    },
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
    validate_vamm(deps.storage, &vamm)?;
    
    let config: Config = read_config(deps.storage)?;
    
    // create a response, so that we can assign relevant submessages
    let mut response = Response::new();

    // calc the input amount wrt to leverage and decimals
    let open_notional = quote_asset_amount
                .checked_mul(leverage)?
                .checked_div(config.decimals)?;

    let mut position = get_position(env, deps.storage, &vamm, &trader, side);

    let mut is_increase: bool = true;
    if !(position.direction == Direction::AddToAmm && side.clone() == Side::BUY) &&
            !(position.direction == Direction::RemoveFromAmm && side.clone() == Side::SELL) {
        is_increase = false;
    }

    if is_increase {
        println!("increase");
        // then we are opening a new position or adding to an existing
        let swap_msg = swap_input(
            &vamm,
            side.clone(),
            open_notional,
            SWAP_INCREASE_REPLY_ID
        ).unwrap();

        // increase the margin, notional etc...
        position.margin = position.margin.checked_add(quote_asset_amount)?;
        position.notional = position.notional.checked_add(open_notional)?;

        let transfer_msg = execute_transfer_from(
            deps.storage,
            &trader.clone(),
            &env.contract.address,
            position.margin,
        ).unwrap();

        // Add the submessage to the response
        response = response
            .add_submessage(transfer_msg)
            .add_submessage(swap_msg);

    } else {
        // TODO make this a function maybe called, open_reverse_position
        // if old position is greater then we don't need to reverse just reduce the position
        println!("{}, {}", position.notional, open_notional);
        if position.notional > open_notional {
            println!("decrease");
            // then we are opening a new position or adding to an existing
            let msg = swap_input(
                &vamm,
                side.clone(),
                open_notional,
                SWAP_DECREASE_REPLY_ID
            ).unwrap();

            position.notional = position.notional.checked_sub(open_notional)?;

            // Add the submessage to the response
            response = response.add_submessage(msg);
        } else {    
            println!("reverse");        
            let amount = position.size;

            // then we are opening a new position or adding to an existing
            let msg = swap_output(
                &vamm,
                direction_to_side(position.direction.clone()),
                amount,
                SWAP_REVERSE_REPLY_ID
            ).unwrap();

            // Add the submessage to the response
            response = response.add_submessage(msg);
        }
    }

    store_tmp_position(deps.storage, &position)?;

    Ok(
        response.add_attributes(vec![
            ("action", "open_position"),
            ("open_notional", &open_notional.to_string()),
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

    store_tmp_position(deps.storage, &position)?;

    Ok(Response::new()
        .add_attributes(vec![("action", "close_position")])
        .add_submessage(msg)
    )
}

// Closes position returning funds after successful execution of the swap out
pub fn finalize_close_position(
    deps: DepsMut,
    _env: Env,
    _input: Uint128,
    output: Uint128,
) -> StdResult<Response> {
    let tmp_position = read_tmp_position(deps.storage)?;
    if tmp_position.is_none() {
        return Err(StdError::generic_err("no temporary position"));
    }

    // TODO update the position with what actually happened in the
    // swap, probably later this requires to check if long, short,buy, sell
    // but for now lets just implement the long case
    let mut position: Position = tmp_position.unwrap();
    position.size = position.size.checked_add(output)?;

    // store the updated position
    store_position(deps.storage, &position)?;

    // remove the tmp position
    remove_tmp_position(deps.storage);

    Ok(Response::new())
}

// Increases position after successful execution of the swap
pub fn increase_position(
    deps: DepsMut,
    _env: Env,
    _input: Uint128,
    output: Uint128,
) -> StdResult<Response> {
    let tmp_position = read_tmp_position(deps.storage)?;
    if tmp_position.is_none() {
        return Err(StdError::generic_err("no temporary position"));
    }

    // TODO update the position with what actually happened in the
    // swap, probably later this requires to check if long, short,buy, sell
    // but for now lets just implement the long case
    let mut position: Position = tmp_position.unwrap();
    position.size = position.size.checked_add(output)?;

    // store the updated position
    store_position(deps.storage, &position)?;

    // remove the tmp position
    remove_tmp_position(deps.storage);

    Ok(Response::new())
}

// Decreases position after successful execution of the swap
pub fn decrease_position(
    deps: DepsMut,
    _env: Env,
    _input: Uint128,
    output: Uint128,
) -> StdResult<Response> {
    let tmp_position = read_tmp_position(deps.storage)?;
    if tmp_position.is_none() {
        return Err(StdError::generic_err("no temporary position"));
    }
    // TODO update the position with what actually happened in the
    // swap, probably later this requires to check if long, short,buy, sell
    // but for now lets just implement the long case
    let mut position: Position = tmp_position.unwrap();
    position.size = position.size.checked_sub(output)?;

    // store the updated position
    store_position(deps.storage, &position)?;

    // remove the tmp position
    remove_tmp_position(deps.storage);

    Ok(Response::new())
}

// Decreases position after successful execution of the swap
pub fn reverse_position(
    deps: DepsMut,
    env: Env,
    _input: Uint128,
    output: Uint128,
) -> StdResult<Response> {
    let mut response: Response = Response::new();
    println!("reverse_position");
    let tmp_position = read_tmp_position(deps.storage)?;
    if tmp_position.is_none() {
        return Err(StdError::generic_err("no temporary position"));
    }

    let open_notional = tmp_position.clone().unwrap().notional;
    let amount = open_notional
        .checked_sub(output)?;

    println!("{}", amount);
    // so if the position to reverse is large then we do something, if it is smaller than a few wei
    // just reset the position and move on with life
    let mut position = clear_position(env, tmp_position.clone().unwrap())?;
    store_position(deps.storage, &position)?;

    // TODO, this is hardcoded to close and clear if the amount is less than the smallest 4dp of you precision
    // not for production
    if amount > Uint128::from(1000u128) {
        let direction = switch_direction(position.direction.clone());

        // then we are opening a new position or adding to an existing
        let msg = swap_input(
            &position.vamm,
            direction_to_side(direction),
            amount,
            SWAP_INCREASE_REPLY_ID
        ).unwrap();
    
        // increase the margin, notional etc...
        position.margin = position.margin.checked_add(amount)?;
        position.notional = position.notional.checked_add(open_notional)?;
    
        // store the updated position
        store_tmp_position(deps.storage, &position)?;

        // add the response
        response = response.add_submessage(msg);
    
    }
    
    Ok(response)
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
    sender: &Addr,
    receiver: &Addr, 
    amount: Uint128, 
) -> StdResult<SubMsg> {
    let config = read_config(storage)?;
    let msg = WasmMsg::Execute {
        contract_addr: config.eligible_collateral.to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
            owner: sender.to_string(),
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
    let current_position = read_position(storage, &vamm, &trader)?;
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

fn validate_vamm(storage: &dyn Storage, vamm: &Addr) -> StdResult<Response> {
    // check that it is a registered vamm
    let vamm_list: VammList = read_vamm(storage)?;
    if !vamm_list.is_vamm(&vamm.to_string()) {
        return Err(StdError::generic_err("vAMM is not registered"));
    }

    Ok(Response::new())

}

// takes the side (buy|sell) and returns the direction (long|short)
fn side_to_direction(
    side: Side,
) -> Direction {
    let direction: Direction = match side {
            Side::BUY => Direction::AddToAmm,
            Side::SELL => Direction::RemoveFromAmm,
    };

    return direction
}

// takes the direction (long|short) and returns the side (buy|sell)
fn direction_to_side(
    direction: Direction,
) -> Side {
    let side: Side = match direction {
            Direction::AddToAmm => Side::BUY,
            Direction::RemoveFromAmm => Side::SELL,
    };

    return side
}

// takes the side (buy|sell) and returns opposite (short|long)
// this is useful when closing/reversing a position
fn switch_direction(
    dir: Direction,
) -> Direction {
    return match dir {
            Direction::RemoveFromAmm => Direction::AddToAmm,
            Direction::AddToAmm => Direction::RemoveFromAmm,
    };
}
