use cosmwasm_std::{Deps, StdResult};
use margined_perp::margined_engine::{
    ConfigResponse, PositionResponse,
};

use crate::state::{Config, read_config, read_position};

/// Queries contract Config
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;

    Ok(
        ConfigResponse {
            owner: config.owner,
            eligible_collateral: config.eligible_collateral,
        }
    )
}

/// Queries contract State
pub fn query_position(deps: Deps, vamm: String, trader: String) -> StdResult<PositionResponse> {
    // read the msg.senders position
    let position = read_position(
        deps.storage,
        &deps.api.addr_validate(&vamm)?,
        &deps.api.addr_validate(&trader)?,
    )?.unwrap();

    Ok(
        PositionResponse {
            size: position.size,
            margin: position.margin,
            notional: position.notional,
            premium_fraction: position.premium_fraction,
            liquidity_history_index: position.liquidity_history_index,
            timestamp: position.timestamp,
        }
    )
}