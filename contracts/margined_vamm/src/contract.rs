#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cosmwasm_bignumber::{Uint256};
use margined_perp::margined_vamm::{ExecuteMsg, InstantiateMsg, QueryMsg};

use crate::error::ContractError;
use crate::{
    handle::{update_config, swap_input, swap_output},
    query::{query_config, query_state},
    state::{Config, store_config, State, store_state}
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        owner: info.sender.clone(),
        decimals: msg.decimals,
        quote_asset: msg.quote_asset,
        base_asset: msg.base_asset,
    };
    
    store_config(deps.storage, &config)?;

    let state = State {
        base_asset_reserve: msg.base_asset_reserve,
        quote_asset_reserve: msg.quote_asset_reserve,
        funding_rate: Uint256::zero(), // Initialise the funding rate as 0
        funding_period: msg.funding_period, // Funding period in seconds
    };

    store_state(deps.storage, &state)?;

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
            decimals,
        } => {
            update_config(
                deps,
                info,
                owner,
                decimals,
            )
        },
        ExecuteMsg::SwapInput {
            direction,
            quote_asset_amount,
        } => {
            swap_input(
                deps,
                env,
                info,
                direction,
                quote_asset_amount,
            )
        },
        ExecuteMsg::SwapOutput {
            direction,
            base_asset_amount,
        } => { 
            swap_output(
                deps,
                env,
                info,
                direction,
                base_asset_amount,
            )
        },
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State {} => to_binary(&query_state(deps)?),
    }
}
