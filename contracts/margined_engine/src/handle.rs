use cosmwasm_std::{
    Addr, attr, CosmosMsg, DepsMut, Env, MessageInfo, Response,
    ReplyOn, StdError, StdResult, SubMsg, to_binary, Uint128,
    WasmMsg,
};

use margined_perp::margined_vamm::{Direction, ExecuteMsg};
use margined_perp::margined_engine::Side;
use crate::{
    contract::{SWAP_EXECUTE_REPLY_ID},
    error::ContractError,
    state::{
        Config, read_config, store_config,
        Position, read_position, store_position,
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
    _env: Env,
    _info: MessageInfo,
    vamm: String,
    trader: String,
    side: Side,
    quote_asset_amount: Uint128,
    _leverage: Uint128,
) -> StdResult<Response> {
    let vamm = deps.api.addr_validate(&vamm)?;
    let trader = deps.api.addr_validate(&trader)?;

    // read the position for the trader from vamm
    let position = read_position(deps.storage, &vamm, &trader)?;

    // so if the position returned is None then its new
    if position.is_none() {
        swap_input(
            vamm,
            side,
            quote_asset_amount,
        );
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

    Ok(Response::new().add_attributes(vec![("action", "open_position")]))
}

fn swap_input(
    vamm: Addr, 
    side: Side, 
    input_amount: Uint128, 
) -> StdResult<Response> {
    let mut direction = Direction::LONG;
    if side == Side::SELL {
        direction = Direction::SHORT;
    }
    let swap_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: vamm.to_string(),
        funds: vec![],
        msg: to_binary(&ExecuteMsg::SwapInput {
            direction: direction,
            quote_asset_amount: input_amount,
        })?,
    });

    let execute_submsg = SubMsg {
        msg: swap_msg,
        gas_limit: None, // probably should set a limit in the config
        id: SWAP_EXECUTE_REPLY_ID,
        reply_on: ReplyOn::Always,
    };

    Ok(Response::new()
        .add_submessage(execute_submsg)
        .add_attributes(vec![
            attr("action", "execute_swap"),
            attr("amount_in", input_amount),
        ]))
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
