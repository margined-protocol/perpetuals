use std::str::FromStr;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, from_binary, Binary, ContractResult, Deps, DepsMut, Env, Event, MessageInfo,
    Reply, Response, StdResult, StdError, Uint128, SubMsgExecutionResponse, Attribute,
};
use cw20::{Cw20ReceiveMsg};
use margined_perp::margined_engine::{
    ExecuteMsg, InstantiateMsg, QueryMsg, Cw20HookMsg,
};

use crate::error::ContractError;
use crate::{
    handle::{
        update_config, increase_position_reply, decrease_position_reply, reverse_position_reply,
        open_position, close_position,
    },
    query::{
        query_config, query_position, query_trader_balance_with_funding_payment,
    },
    state::{Config, read_config, store_config, store_vamm},
};

pub const SWAP_INCREASE_REPLY_ID: u64 = 1;
pub const SWAP_DECREASE_REPLY_ID: u64 = 2;
pub const SWAP_REVERSE_REPLY_ID: u64 = 3;
pub const SWAP_CLOSE_REPLY_ID: u64 = 4;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let decimals = Uint128::from(10u128.pow(msg.decimals as u32));
    let eligible_collateral = deps.api.addr_validate(&msg.eligible_collateral)?;

    // config parameters
    let config = Config {
        owner: info.sender.clone(),
        eligible_collateral: eligible_collateral,
        decimals: decimals,
        initial_margin_ratio: msg.initial_margin_ratio,
        maintenance_margin_ratio: msg.maintenance_margin_ratio,
        liquidation_fee: msg.liquidation_fee,
    };
    
    store_config(deps.storage, &config)?;

    // store default vamms
    store_vamm(deps, &msg.vamm)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(
            deps,
            env,
            info.clone(),
            msg
        ),
        ExecuteMsg::UpdateConfig {
            owner,
        } => {
            update_config(
                deps,
                info.clone(),
                owner,
            )
        },
        ExecuteMsg::OpenPosition {
            vamm,
            side,
            quote_asset_amount,
            leverage,
         } => {
             let trader = info.sender.clone();
         open_position(
            deps,
            env,
            info.clone(),
            vamm,
            trader.to_string(),
            side,
            quote_asset_amount,
            leverage,
        )},
        ExecuteMsg::ClosePosition {
            vamm,
         } => {
             let trader = info.sender.clone();
         close_position(
            deps,
            env,
            info.clone(),
            vamm,
            trader.to_string(),
            SWAP_CLOSE_REPLY_ID,
        )},
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    // only asset contract can execute this message
    let config: Config = read_config(deps.storage)?;
    if config.eligible_collateral != deps.api.addr_validate(info.sender.as_str())? {
        return Err(StdError::generic_err("unauthorized"));
    }
    
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::OpenPosition {
            vamm,
            side,
            leverage,
        }) => open_position(
            deps,
            env,
            info,
            vamm,
            cw20_msg.sender,
            side,
            cw20_msg.amount, // not needed, we should take from deposited amount or validate
            leverage,
        ),
        Err(_) => Err(StdError::generic_err("invalid cw20 hook message")),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Position {
            vamm,
            trader,
        } => to_binary(&query_position(deps, vamm, trader)?),
        QueryMsg::TraderBalance {
            trader,
        } => to_binary(&query_trader_balance_with_funding_payment(deps, trader)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    match msg.result {
        ContractResult::Ok(response) => {
            match msg.id {
                SWAP_INCREASE_REPLY_ID => {
                    let (input, output) = parse_swap(response);
                    let response = increase_position_reply(
                        deps,
                        env,
                        input,
                        output,
                    )?;
                    Ok(response)
                },
                SWAP_DECREASE_REPLY_ID => {
                    let (input, output) = parse_swap(response);
                    let response = decrease_position_reply(
                        deps,
                        env,
                        input,
                        output,
                    )?;
                    Ok(response)
                },
                SWAP_REVERSE_REPLY_ID => {
                    let (input, output) = parse_swap(response);
                    let response = reverse_position_reply(
                        deps,
                        env,
                        input,
                        output,
                    )?;
                    Ok(response)
                },
                CLOSE_POSITION_REPLY_ID => {
                    let (input, output) = parse_swap(response);
                    let response = reverse_position_reply(
                        deps,
                        env,
                        input,
                        output,
                    )?;
                    Ok(response)
                },
                _ => Err(StdError::generic_err(format!(
                    "reply (id {:?}) invalid",
                    msg.id
                ))),
            }
        }
        ContractResult::Err(e) => Err(StdError::generic_err(format!(
            "reply (id {:?}) error {:?}",
            msg.id, e
        ))),
    }
}

fn parse_swap(
    response: SubMsgExecutionResponse,
) -> (Uint128, Uint128) {
    // Find swap inputs and output events
    println!("{:?}", response);
    let wasm = response.events.iter().find(|&e| e.ty == "wasm");
    let wasm = wasm.unwrap();
    let input_str = read_event("input".to_string(), wasm).value;
    let input: Uint128 = Uint128::from_str(&input_str).unwrap();

    let output_str = read_event("output".to_string(), wasm).value;
    let output: Uint128 = Uint128::from_str(&output_str).unwrap();

    return (input, output)
}

fn read_event(
    key: String,
    event: &Event,
) -> Attribute {
    let result = event.attributes.iter().find(|&attr| attr.key == key).unwrap();
    return result.clone()
}