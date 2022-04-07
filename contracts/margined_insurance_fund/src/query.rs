use cosmwasm_std::{Deps, StdResult, Addr, StdError};
use margined_perp::margined_insurance_fund::{ConfigResponse, AmmResponse};

use crate::state::{read_config,Config, read_vamm};

/// Queries contract Config
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner,
    })
}

///Queries the AMM with given address
pub fn query_amm(deps: Deps, amm: Addr) -> StdResult<AmmResponse> {
    let amm_list = read_vamm(deps.storage)?.vamms;

    let amm_new: Addr = if amm_list.contains(&amm) { 
        amm
    } else {
        return Err(StdError::NotFound { kind: "AMM".to_string() })
    };

    Ok(AmmResponse {
        amm: amm_new
    })
}

//Queries all of the current AMMs