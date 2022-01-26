use cosmwasm_std::{
    Addr, DepsMut, Env, MessageInfo, Response, StdResult, Storage, Uint128,
};

use margined_perp::margined_vamm::ExecuteMsg::{SwapInput};
use margined_perp::margined_engine::Side;
use crate::{
    error::ContractError,
    state::{
        Config, read_config, store_config,
        Position, read_position, store_position,
    },
};

pub fn update_config(
    deps: DepsMut,
    _info: MessageInfo,
    owner: String,
) -> Result<Response, ContractError> {
    let config = read_config(deps.storage)?;
    if info.sender != config.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    let new_config = Config {
        owner: deps.api.addr_validate(owner).unwrap(),
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
    vamm: Addr,
    trader: Addr,
    side: Side,
    quote_asset_amount: Uint128,
    leverage: Uint128,
) -> Result<Response, ContractError> {
    // read the position for the trader from vamm
    if let Some(position) = read_position(deps.storage, &vamm, &trader)? {
        println!("Matched {:?}!", position);
    } else {
        println!("no position exists");
    }


    Ok(Response::new().add_attributes(vec![("action", "open_position")]))
}

// fn increase_position(
//     deps: DepsMut,
//     side: Side,
//     position: Position,
// ) -> Result<Response, ContractError> {

// }

fn swap_input(
    vamm: Addr, 
    side: Side, 
    input_amount: Uint128, 
) -> StdResult<SwapInputResult> {
    if side == Side.Buy {}
    let swap_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: vamm.to_string(),
        funds: vec![],
        msg: to_binary(&ExecuteMsg::SwapInput {
            side: side,
            quote_asset_amount: input_amount,
        })?,
    });

    let response = Response::default()
        .add_message(swap_msg)
        .add_attribute("minted_amount", amount_to_mint);

    Ok(BalancerTransitionResult {
        state: None,
        response,
    })
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
