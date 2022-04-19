use crate::error::ContractError;
use crate::{
    handle::{add_vamm, remove_vamm, update_config, withdraw, switch_vamm_off, switch_vamm_on, shutdown_all_vamm},
    query::{query_config, query_is_vamm, query_mult_vamm, query_status_mult_vamm, query_vamm_status},
    state::{store_config, Config},
};

#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use margined_perp::margined_insurance_fund::{ExecuteMsg, InstantiateMsg, QueryMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        owner: info.sender,
        beneficiary: Addr::unchecked(""),
    };

    store_config(deps.storage, &config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig { owner, beneficiary } => {
            update_config(deps, info, owner, beneficiary)
        }
        ExecuteMsg::AddVamm { vamm } => add_vamm(deps, info, vamm),
        ExecuteMsg::RemoveVamm { vamm } => remove_vamm(deps, info, vamm),
        ExecuteMsg::Withdraw { token, amount } => withdraw(deps, info, token, amount),
        ExecuteMsg::SwitchVammOff { vamm } => switch_vamm_off(deps, info, vamm),
        ExecuteMsg::SwitchVammOn { vamm } => switch_vamm_on(deps, info, vamm),
        ExecuteMsg::ShutdownAllVamm {} => shutdown_all_vamm(deps, info),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::IsVamm { vamm } => to_binary(&query_is_vamm(deps, vamm)?),
        QueryMsg::GetAllVamm {} => to_binary(&query_mult_vamm(deps)?),
        QueryMsg::GetVammStatus { vamm } => to_binary(&query_vamm_status(deps, vamm)?),
        QueryMsg::GetAllVammStatus {} => to_binary(&query_status_mult_vamm(deps)?),
    }
}
