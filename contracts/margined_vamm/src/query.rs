use cosmwasm_std::{Deps, StdResult};
use margined_perp::margined_vamm::{
    ConfigResponse, StateResponse,
};

use crate::state::{Config, CONFIG, State, STATE};

/// Queries contract Config
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = CONFIG.load(deps.storage)?;

    config.as_res(deps.api)
}

/// Queries contract State
pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state: State = STATE.load(deps.storage)?;

    state.as_res(deps.api)
}