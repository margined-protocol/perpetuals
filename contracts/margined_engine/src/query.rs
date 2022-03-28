use cosmwasm_std::{Deps, StdResult, Uint128};
use margined_common::integer::Integer;
use margined_perp::margined_engine::{
    ConfigResponse, PnlCalcOption, PositionResponse, PositionUnrealizedPnlResponse,
};

use crate::{
    state::{read_config, read_position, read_vamm, read_vamm_map, Config},
    utils::{calc_funding_payment, get_position_notional_unrealized_pnl},
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
    )
    .unwrap();

    Ok(PositionResponse {
        size: position.size,
        margin: position.margin,
        notional: position.notional,
        last_updated_premium_fraction: position.last_updated_premium_fraction,
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
    )
    .unwrap();

    let result = get_position_notional_unrealized_pnl(deps, &position, PnlCalcOption::SPOTPRICE)?;

    Ok(result.unrealized_pnl)
}

/// Queries user position
pub fn query_cumulative_premium_fraction(deps: Deps, vamm: String) -> StdResult<Integer> {
    // read the msg.senders position
    let vamm_map = read_vamm_map(deps.storage, deps.api.addr_validate(&vamm)?).unwrap();

    let result = match vamm_map.cumulative_premium_fractions.len() {
        0 => Integer::zero(),
        n => vamm_map.cumulative_premium_fractions[n - 1],
    };

    Ok(result)
}

/// Queries traders balanmce across all vamms with funding payment
pub fn query_trader_balance_with_funding_payment(deps: Deps, trader: String) -> StdResult<Uint128> {
    let mut margin = Uint128::zero();
    let vamm_list = read_vamm(deps.storage)?;
    for vamm in vamm_list.vamm.iter() {
        let position =
            query_trader_position_with_funding_payment(deps, vamm.to_string(), trader.clone())?;
        margin = margin.checked_add(position.margin)?;
    }

    Ok(margin)
}

/// Queries traders position across all vamms with funding payments
pub fn query_trader_position_with_funding_payment(
    deps: Deps,
    vamm: String,
    trader: String,
) -> StdResult<PositionResponse> {
    let config = read_config(deps.storage).unwrap();

    let vamm = deps.api.addr_validate(&vamm)?;
    let trader = deps.api.addr_validate(&trader)?;

    // retrieve latest user position
    let mut position = read_position(deps.storage, &vamm, &trader)?;

    let latest_cumulative_premium_fraction =
        query_cumulative_premium_fraction(deps, vamm.to_string()).unwrap();

    let funding_payment = calc_funding_payment(
        position.clone(),
        latest_cumulative_premium_fraction,
        config.decimals,
    );

    let margin_with_funding_payment = Integer::new_positive(position.margin) + funding_payment;

    if margin_with_funding_payment.is_positive() {
        position.margin = margin_with_funding_payment.value;
    } else {
        position.margin = Uint128::zero();
    }

    Ok(PositionResponse {
        size: position.size,
        margin: position.margin,
        notional: position.notional,
        last_updated_premium_fraction: position.last_updated_premium_fraction,
        liquidity_history_index: position.liquidity_history_index,
        timestamp: position.timestamp,
    })
}

/// Queries the margin ratio of a trader
pub fn query_margin_ratio(deps: Deps, vamm: String, trader: String) -> StdResult<Integer> {
    let config: Config = read_config(deps.storage)?;

    // retrieve the latest position
    let position = read_position(
        deps.storage,
        &deps.api.addr_validate(&vamm)?,
        &deps.api.addr_validate(&trader)?,
    )
    .unwrap();

    if position.size.is_zero() {
        return Ok(Integer::zero());
    }

    // TODO think how the side can be used
    // currently it seems only losses have been
    // tested but it cant be like that forever...
    let PositionUnrealizedPnlResponse {
        position_notional: mut notional,
        unrealized_pnl: mut pnl,
    } = get_position_notional_unrealized_pnl(deps, &position, PnlCalcOption::SPOTPRICE)?;
    let PositionUnrealizedPnlResponse {
        position_notional: twap_notional,
        unrealized_pnl: twap_pnl,
    } = get_position_notional_unrealized_pnl(deps, &position, PnlCalcOption::TWAP)?;

    // calculate and return margin
    if pnl > twap_pnl {
        pnl = twap_pnl;
        notional = twap_notional;
    }

    let margin_ratio = Integer::new_positive(position.margin) - Integer::new_positive(pnl);

    Ok((margin_ratio * Integer::new_positive(config.decimals)) / Integer::new_positive(notional))
}
