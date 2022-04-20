use cosmwasm_std::{Deps, StdResult};
use margined_perp::margined_insurance_fund::{
    AllVammResponse, AllVammStatusResponse, ConfigResponse, VammResponse, VammStatusResponse,
};

use crate::state::{
    is_vamm, read_all_vamm_status, read_config, read_vamm_status, read_vammlist, Config,
};

const DEFAULT_PAGINATION_LIMIT: u32 = 10u32;
const MAX_PAGINATION_LIMIT: u32 = 30u32;

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

/// Queries the list of vAMMs currently stored (not necessarily on)
pub fn query_mult_vamm(deps: Deps, limit: Option<u32>) -> StdResult<AllVammResponse> {
    // set the limit for pagination
    let limit = limit
        .unwrap_or(DEFAULT_PAGINATION_LIMIT)
        .min(MAX_PAGINATION_LIMIT) as usize;

    let list = read_vammlist(deps, deps.storage, limit)?;
    Ok(AllVammResponse { vamm_list: list })
}

/// Queries the status of multiple vAMMs, returning the vAMM address and whether it is on/off
pub fn query_status_mult_vamm(deps: Deps, limit: Option<u32>) -> StdResult<AllVammStatusResponse> {
    // set the limit for pagination
    let limit = limit
        .unwrap_or(DEFAULT_PAGINATION_LIMIT)
        .min(MAX_PAGINATION_LIMIT) as usize;

    let status_list = read_all_vamm_status(deps.storage, limit)?;
    Ok(AllVammStatusResponse {
        vamm_list_status: status_list,
    })
}
