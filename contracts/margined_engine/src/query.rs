use cosmwasm_std::{Deps, StdResult, Uint128};
use margined_perp::margined_engine::{
    ConfigResponse, PositionResponse,
};

use crate::state::{
    Config, read_config,
    read_position, read_vamm,
};

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


/// Queries traders position across all vamms
pub fn query_trader_balance_with_funding_payment(
    deps: Deps,
    trader: String
) -> StdResult<Uint128> {
    let mut margin = Uint128::zero();
    let vamm_list = read_vamm(deps.storage)?;
    for vamm in vamm_list.vamm.iter() {
        let position = query_position(deps, vamm.to_string(), trader.clone())?;
        margin = margin.checked_add(position.margin)?;

    }

    Ok(margin)
}