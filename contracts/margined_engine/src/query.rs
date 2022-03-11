use cosmwasm_std::{Deps, StdResult, Uint128};
use margined_perp::margined_engine::{
    ConfigResponse, PnlCalcOption, PositionResponse, PositionUnrealizedPnlResponse,
};

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
pub fn query_unrealized_pnl(deps: Deps, vamm: String, trader: String) -> StdResult<Uint128> {
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
pub fn query_trader_balance_with_funding_payment(deps: Deps, trader: String) -> StdResult<Uint128> {
    let mut margin = Uint128::zero();
    let vamm_list = read_vamm(deps.storage)?;
    for vamm in vamm_list.vamm.iter() {
        let position = query_position(deps, vamm.to_string(), trader.clone())?;
        margin = margin.checked_add(position.margin)?;
    }

    Ok(margin)
}

/// Queries the margin ratio of a trader
pub fn query_margin_ratio(deps: Deps, vamm: String, trader: String) -> StdResult<Uint128> {
    let config: Config = read_config(deps.storage)?;

    // retrieve the latest position
    let position = read_position(
        deps.storage,
        &deps.api.addr_validate(&vamm)?,
        &deps.api.addr_validate(&trader)?,
    )?
    .unwrap();

    if position.size.is_zero() {
        return Ok(Uint128::zero());
    }

    // TODO think how the side can be used
    // currently it seems only losses have been
    // tested but it cant be like that forever...
    let PositionUnrealizedPnlResponse {
        position_notional: mut notional,
        unrealized_pnl: mut pnl,
        side: _side,
    } = get_position_notional_unrealized_pnl(deps, &position, PnlCalcOption::SPOTPRICE)?;
    let PositionUnrealizedPnlResponse {
        position_notional: twap_notional,
        unrealized_pnl: twap_pnl,
        side: _twap_side,
    } = get_position_notional_unrealized_pnl(deps, &position, PnlCalcOption::TWAP)?;

    // calculate and return margin
    if pnl > twap_pnl {
        pnl = twap_pnl;
        notional = twap_notional;
    }

    let update_margin = if position.margin > pnl {
        position.margin.checked_sub(pnl)?
    } else {
        pnl.checked_sub(position.margin)?
    };

    let margin = update_margin
        .checked_mul(config.decimals)?
        .checked_div(notional)?;

    Ok(margin)
}
