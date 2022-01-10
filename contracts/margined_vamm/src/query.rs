use cosmwasm_std::{Deps, StdResult};
use margined_perp::margined_vamm::{
    ConfigResponse,
};

use crate::state::{Config, CONFIG};

/// Queries contract Config
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = CONFIG.load(deps.storage)?;

    config.as_res(deps.api)
}