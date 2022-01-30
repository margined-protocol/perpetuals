#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, to_binary, from_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response,
    StdResult, StdError, Uint128, SubMsg, CosmosMsg, WasmMsg,
};
use cw20::{Cw20ReceiveMsg, Cw20ExecuteMsg};
use margined_perp::margined_engine::{ExecuteMsg, InstantiateMsg, QueryMsg, Cw20HookMsg};

use crate::error::ContractError;
use crate::{
    handle::{update_config, update_position, open_position},
    query::{query_config, query_position},
    state::{Config, read_config, store_config},
};

pub const SWAP_EXECUTE_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let decimals = Uint128::from(10u128.pow(msg.decimals as u32));
    let eligible_collateral = deps.api.addr_validate(&msg.eligible_collateral)?;

    let config = Config {
        owner: info.sender.clone(),
        eligible_collateral: eligible_collateral,
        decimals: decimals,
        initial_margin_ratio: msg.initial_margin_ratio,
        maintenance_margin_ratio: msg.maintenance_margin_ratio,
        liquidation_fee: msg.liquidation_fee,
    };
    
    store_config(deps.storage, &config)?;

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
            info,
            msg
        ),
        ExecuteMsg::UpdateConfig {
            owner,
        } => {
            update_config(
                deps,
                info,
                owner,
            )
        },
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
            quote_asset_amount,
            leverage,
        }) => open_position(
            deps,
            env,
            info,
            vamm,
            cw20_msg.sender,
            side,
            quote_asset_amount, // not needed, we should take from deposited amount or validate
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
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        SWAP_EXECUTE_REPLY_ID => {
            let response = update_position(deps, env)?;
            Ok(response)
        }
        _ => {
            println!("{:?}", msg.id);
            Err(StdError::generic_err("reply id is invalid"))
        },
    }
}
