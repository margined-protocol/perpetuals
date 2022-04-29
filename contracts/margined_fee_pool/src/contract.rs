use crate::error::ContractError;
use crate::{
    handle::{add_token, remove_token, send_token, update_config},
    query::{query_all_token, query_config, query_is_token, query_token_list_length},
    state::{store_config, Config},
};

#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use margined_common::validate::validate_eligible_collateral as validate_funds;
use margined_perp::margined_fee_pool::{ExecuteMsg, InstantiateMsg, QueryMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let valid_funds = validate_funds(deps.as_ref(), msg.funds)?;

    let config = Config {
        owner: info.sender,
        funds: valid_funds,
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
        ExecuteMsg::UpdateConfig { owner } => update_config(deps, info, owner),
        ExecuteMsg::AddToken { token } => add_token(deps, info, token),
        ExecuteMsg::RemoveToken { token } => remove_token(deps, info, token),
        ExecuteMsg::SendToken {
            token,
            amount,
            recipient,
        } => send_token(deps, env, info, token, amount, recipient),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::IsToken { token } => to_binary(&query_is_token(deps, token)?),
        QueryMsg::GetTokenList { limit } => to_binary(&query_all_token(deps, limit)?),
        QueryMsg::GetTokenLength {} => to_binary(&query_token_list_length(deps)?),
    }
}
