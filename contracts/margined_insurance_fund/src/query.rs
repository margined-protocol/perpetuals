use cosmwasm_std::{Deps, StdResult};
use margined_perp::margined_insurance_fund::{
    AllVammResponse, AllVammStatusResponse, ConfigResponse, VammResponse, VammStatusResponse,
};

use crate::state::{
    is_vamm, read_all_vamm_status, read_config, read_vamm_status, read_vammlist, Config,
};
/// Queries contract config
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner,
        beneficiary: config.beneficiary,
    })
}

/// Queries if the vAMM with given address is already stored
pub fn query_is_vamm(deps: Deps, vamm: String) -> StdResult<VammResponse> {
    // validate address
    let vamm_valid = deps.api.addr_validate(&vamm)?;

    // read the current storage and pull the vamm status corresponding to the given addr
    let vamm_bool = is_vamm(deps.storage, vamm_valid);

    Ok(VammResponse { is_vamm: vamm_bool })
}

/// Queries the status of the vAMM with given address
pub fn query_vamm_status(deps: Deps, vamm: String) -> StdResult<VammStatusResponse> {
    // validate address
    let vamm_valid = deps.api.addr_validate(&vamm)?;

    // read the current storage and pull the vamm list
    let vamm_bool = read_vamm_status(deps.storage, vamm_valid)?;

    Ok(VammStatusResponse {
        vamm_status: vamm_bool,
    })
}

//Queries multiple vAMMs TODO: add the option to query a slice
pub fn query_mult_vamm(deps: Deps) -> StdResult<AllVammResponse> {
    let list = read_vammlist(deps, deps.storage)?;
    Ok(AllVammResponse { vamm_list: list })
}

//Queries the status of multiple vAMMs TODO: add the option to query a slice
pub fn query_status_mult_vamm(deps: Deps) -> StdResult<AllVammStatusResponse> {
    let status_list = read_all_vamm_status(deps.storage)?;
    Ok(AllVammStatusResponse {
        vamm_list_status: status_list,
    })
}
