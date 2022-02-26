use cosmwasm_std::{Deps, StdResult};
use cosmwasm_bignumber::Decimal256;
use margined_perp::margined_engine::{ConfigResponse, PnlCalcOption, PositionResponse};

use crate::{
    handle::get_position_notional_unrealized_pnl,
    state::{read_config, read_position, read_vamm, Config},
};

/// Queries contract Config
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner,
        eligible_collateral: config.eligible_collateral,
    })
}

/// Queries user position
pub fn query_position(deps: Deps, vamm: String, trader: String) -> StdResult<PositionResponse> {
    // read the msg.senders position
    let position = read_position(
        deps.storage,
        &deps.api.addr_validate(&vamm)?,
        &deps.api.addr_validate(&trader)?,
    )?
    .unwrap();

    Ok(PositionResponse {
        size: position.size,
        margin: position.margin,
        notional: position.notional,
        premium_fraction: position.premium_fraction,
        liquidity_history_index: position.liquidity_history_index,
        timestamp: position.timestamp,
    })
}

/// Queries user position
pub fn query_unrealized_pnl(deps: Deps, vamm: String, trader: String) -> StdResult<Decimal256> {
    // read the msg.senders position
    let position = read_position(
        deps.storage,
        &deps.api.addr_validate(&vamm)?,
        &deps.api.addr_validate(&trader)?,
    )?
    .unwrap();

    let result = get_position_notional_unrealized_pnl(deps, &position, PnlCalcOption::SPOTPRICE)?;

    Ok(result.unrealized_pnl)
}

/// Queries traders position across all vamms
pub fn query_trader_balance_with_funding_payment(deps: Deps, trader: String) -> StdResult<Decimal256> {
    let mut margin = Decimal256::zero();
    let vamm_list = read_vamm(deps.storage)?;
    for vamm in vamm_list.vamm.iter() {
        let position = query_position(deps, vamm.to_string(), trader.clone())?;
        margin = margin + position.margin;
    }

    Ok(margin)
}
