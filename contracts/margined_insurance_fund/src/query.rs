use cosmwasm_std::{Addr, Deps, StdError, StdResult};
use margined_perp::margined_insurance_fund::{AmmResponse, ConfigResponse};

use crate::state::{read_config, read_vamm, Config};

/// Queries contract config
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner,
    })
}

/// Queries the AMM with given address
pub fn query_amm(deps: Deps, amm: String) -> StdResult<AmmResponse> {
    let amm_valid = deps.api.addr_validate(&amm)?;

    let amm_list = read_vamm(deps.storage)?.vamms;

    let amm_new: Addr = if amm_list.contains(&amm_valid) {
        amm_valid
    } else {
        return Err(StdError::NotFound {
            kind: "AMM".to_string(),
        });
    };

    Ok(AmmResponse { amm: amm_new })
}

//Queries all of the current AMMs
