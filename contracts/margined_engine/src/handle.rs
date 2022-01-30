use cosmwasm_std::{
    Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response,
    ReplyOn, StdError, StdResult, SubMsg, to_binary, Uint128,
    WasmMsg,
};

use margined_perp::margined_vamm::{Direction, ExecuteMsg};
use margined_perp::margined_engine::Side;
use crate::{
    contract::{SWAP_EXECUTE_REPLY_ID},
    state::{
        Config, read_config, store_config,
        Position, read_position, store_position,
        store_tmp_position, read_tmp_position, remove_tmp_position,
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
    _leverage: Uint128,
) -> StdResult<Response> {
    // create a response, so that we can assign relevant submessages to
    // be executed
    let mut response = Response::new();

    // validate address inputs
    let vamm = deps.api.addr_validate(&vamm)?;
    let trader = deps.api.addr_validate(&trader)?;
    
    // read the position for the trader from vamm
    let position = read_position(deps.storage, &vamm, &trader)?;

    // so if the position returned is None then its new
    if position.is_none() {
        let msg = swap_input(
            &vamm,
            side,
            quote_asset_amount,
        ).unwrap();

        // Add the submessage to the response
        response = response.add_submessage(msg);

        // store the temporary position
        let position = Position {
            vamm: vamm,
            trader,
            direction: Direction::LONG,
            size: Uint128::zero(), // todo need to think how to tmp store
            margin: Uint128::zero(), // todo need to think how to tmp store
            notional: Uint128::zero(), // todo need to think how to tmp store
            premium_fraction: Uint128::zero(),
            liquidity_history_index: Uint128::zero(),
            timestamp: env.block.time,
        };

        store_tmp_position(deps.storage, &position)?;
    }

    // let is_increase: bool = false;
    // if (position.direction == Direction::LONG && side == Side::BUY) ||
    //         (position?.direction == Direction::SHORT && side == Side::SELL) {
    //     is_increase = true;
    // }

    // if position == None || is_increase {
    //     // then we are opening a new position or adding to an existing
    //     increase_position(
    //         de
    //     )
    // } else {
    //     // we are reversing
    //     println!("REVERSE REVERSE REVERSE");
    // }

    response = response.add_attributes(vec![("action", "open_position")]);
    Ok(response)
}

// Updates position after successful execution of the swap
pub fn update_position(
    deps: DepsMut,
    env: Env,
) -> StdResult<Response> {
    let tmp_position = read_tmp_position(deps.storage)?;
    if tmp_position.is_none() {
        return Err(StdError::generic_err("no temporary position"));
    }

    // TODO update the position with what actually happened in the
    // swap
    let tmp_position = tmp_position.unwrap();

    // store the updated position
    store_position(deps.storage, &tmp_position)?;

    // remove the tmp position
    remove_tmp_position(deps.storage);

    Ok(Response::new())
}

fn swap_input(
    vamm: &Addr, 
    side: Side, 
    input_amount: Uint128, 
) -> StdResult<SubMsg> {
    let mut direction = Direction::LONG;
    if side == Side::SELL {
        direction = Direction::SHORT;
    }

    let swap_msg = WasmMsg::Execute {
        contract_addr: vamm.to_string(),
        funds: vec![],
        msg: to_binary(&ExecuteMsg::SwapInput {
            direction: direction,
            quote_asset_amount: input_amount,
        })?,
    };

    let execute_submsg = SubMsg {
        msg: CosmosMsg::Wasm(swap_msg),
        gas_limit: None, // probably should set a limit in the config
        id: SWAP_EXECUTE_REPLY_ID,
        reply_on: ReplyOn::Always,
    };

    Ok(execute_submsg)
}

// fn adjust_position_for_liquidity_changed(
//     deps: DepsMut,
//     vamm: Addr,
//     trader: Addr
// ) -> StdResult<Position> {
//     // retrieve traders, existing position
//     let position = read_position(&deps.storage, vamm, trader);
// }

// /// Unit tests
// #[test]
// fn test_get_input_and_output_price() {}
