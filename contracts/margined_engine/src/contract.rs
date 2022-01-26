#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128};
use margined_perp::margined_engine::{ExecuteMsg, InstantiateMsg, QueryMsg};

use crate::error::ContractError;
use crate::{
    handle::{update_config, swap_input},
    query::{query_config, query_position},
    state::{Config, store_config},
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {

    let decimals = Uint128::from(10u128.pow(msg.decimals as u32));

    let config = Config {
        owner: info.sender.clone(),
        decimals: decimals,
        initial_margin: msg.initial_margin,
        maintenance_margin: msg.maintenance_margin,
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
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
        } => {
            update_config(
                deps,
                info,
                owner,
            )
        },
        ExecuteMsg::OpenPosition {
            owner,
        } => {
            open_position(
                deps,
                info,
                owner,
            )
        },
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
