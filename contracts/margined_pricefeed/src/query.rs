use cosmwasm_std::{Deps, StdResult};
use margined_perp::margined_pricefeed::{ConfigResponse};

use crate::{
    state::{read_config, Config},
};

/// Queries contract Config
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner,
        decimals: config.decimals,
    })
}
