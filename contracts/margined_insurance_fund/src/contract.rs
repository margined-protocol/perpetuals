#[cfg(not(feature = "library"))]
use crate::error::ContractError;
use crate::{
    handle::{add_vamm, remove_vamm, shutdown_all_vamm, update_config, withdraw},
    query::{
        query_all_vamm, query_config, query_is_vamm, query_status_all_vamm, query_vamm_status,
    },
    state::{store_config, Config},
};
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use margined_perp::margined_insurance_fund::{ExecuteMsg, InstantiateMsg, QueryMsg};

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "crates.io:margined-insurance-fund";
/// Contract version that is used for migration.
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        owner: info.sender,
        beneficiary: Addr::unchecked(""),
    };

    store_config(deps.storage, &config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig { owner, beneficiary } => {
            update_config(deps, info, owner, beneficiary)
        }
        ExecuteMsg::AddVamm { vamm } => add_vamm(deps, info, vamm),
        ExecuteMsg::RemoveVamm { vamm } => remove_vamm(deps, info, vamm),
        ExecuteMsg::Withdraw { token, amount } => withdraw(deps, info, token, amount),
        ExecuteMsg::ShutdownVamms {} => shutdown_all_vamm(deps, env, info),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::IsVamm { vamm } => to_binary(&query_is_vamm(deps, vamm)?),
        QueryMsg::GetAllVamm { limit } => to_binary(&query_all_vamm(deps, limit)?),
        QueryMsg::GetVammStatus { vamm } => to_binary(&query_vamm_status(deps, vamm)?),
        QueryMsg::GetAllVammStatus { limit } => to_binary(&query_status_all_vamm(deps, limit)?),
    }
}
