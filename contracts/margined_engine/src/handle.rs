use cosmwasm_std::{
    Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response,
    ReplyOn, StdError, StdResult, SubMsg, to_binary, Uint128,
    WasmMsg, Storage,
};

use margined_perp::margined_vamm::{Direction, ExecuteMsg};
use margined_perp::margined_engine::{Side};
use crate::{
    contract::{SWAP_INCREASE_REPLY_ID, SWAP_DECREASE_REPLY_ID, SWAP_REVERSE_REPLY_ID},
    state::{
        Config, read_config, store_config,
        Position, read_position, Swap, store_tmp_swap,
    },
    utils::{
        require_vamm, side_to_direction, direction_to_side, switch_direction,
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

    // calc the input amount wrt to leverage and decimals
    let open_notional = quote_asset_amount
                .checked_mul(leverage)?
                .checked_div(config.decimals)?;

    let position: Position = get_position(env.clone(), deps.storage, &vamm, &trader, side.clone());

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

    Ok(Response::new()
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

// Increase the position, just basically wraps swap input though it may do more in the future
pub fn internal_increase_position(
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
        println!("close and open reverse position");        
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

pub fn get_position(
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

// this resets the main variables of a position
pub fn clear_position(
    env: Env,
    mut position: Position,
) -> StdResult<Position> {
    position.size = Uint128::zero();
    position.margin = Uint128::zero();
    position.notional = Uint128::zero();
    position.timestamp = env.block.time;

    Ok(position)
}
