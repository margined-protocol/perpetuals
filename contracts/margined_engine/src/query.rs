use cosmwasm_std::{Deps, StdError, StdResult, Uint128};
use margined_common::integer::Integer;
use margined_perp::margined_engine::{
    ConfigResponse, PnlCalcOption, Position, PositionUnrealizedPnlResponse, StateResponse,
};

use crate::{
    querier::query_insurance_all_vamm,
    state::{read_config, read_position, read_state, read_vamm_map, Config, State},
    utils::{
        calc_funding_payment, calc_remain_margin_with_funding_payment,
        get_position_notional_unrealized_pnl,
    },
};

/// Queries contract Config
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner,
        pauser: config.pauser,
        insurance_fund: config.insurance_fund,
        fee_pool: config.fee_pool,
        eligible_collateral: config.eligible_collateral,
        decimals: config.decimals,
        initial_margin_ratio: config.initial_margin_ratio,
        maintenance_margin_ratio: config.maintenance_margin_ratio,
        partial_liquidation_ratio: config.partial_liquidation_ratio,
        liquidation_fee: config.liquidation_fee,
    })
}

/// Queries contract State
pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state: State = read_state(deps.storage)?;

    Ok(StateResponse {
        open_interest_notional: state.open_interest_notional,
        bad_debt: state.prepaid_bad_debt,
    })
}

/// Queries user position
pub fn query_position(deps: Deps, vamm: String, trader: String) -> StdResult<Position> {
    let position = read_position(
        deps.storage,
        &deps.api.addr_validate(&vamm)?,
        &deps.api.addr_validate(&trader)?,
    )
    .unwrap();

    // a default is returned if no position found with no trader set
    if position.trader != trader {
        return Err(StdError::generic_err("No position found"));
    }

    Ok(position)
}

/// Queries and returns users position for all registered vamms
pub fn query_all_positions(deps: Deps, trader: String) -> StdResult<Vec<Position>> {
    let config = read_config(deps.storage).unwrap();

    let mut response: Vec<Position> = vec![];

    let vamms = query_insurance_all_vamm(&deps, config.insurance_fund.to_string(), None)?.vamm_list;
    for vamm in vamms.iter() {
        let position =
            read_position(deps.storage, vamm, &deps.api.addr_validate(&trader)?).unwrap();

        // a default is returned if no position found with no trader set
        if position.trader == trader {
            response.push(position)
        }
    }

    Ok(response)
}

/// Queries user position
pub fn query_position_notional_unrealized_pnl(
    deps: Deps,
    vamm: String,
    trader: String,
    calc_option: PnlCalcOption,
) -> StdResult<PositionUnrealizedPnlResponse> {
    // read the msg.senders position
    let position = read_position(
        deps.storage,
        &deps.api.addr_validate(&vamm)?,
        &deps.api.addr_validate(&trader)?,
    )
    .unwrap();

    let result = get_position_notional_unrealized_pnl(deps, &position, calc_option)?;

    Ok(result)
}

/// Queries cumulative premium fractions
pub fn query_cumulative_premium_fraction(deps: Deps, vamm: String) -> StdResult<Integer> {
    // retrieve vamm data
    let vamm_map = read_vamm_map(deps.storage, deps.api.addr_validate(&vamm)?).unwrap();

    let result = match vamm_map.cumulative_premium_fractions.len() {
        0 => Integer::zero(),
        n => vamm_map.cumulative_premium_fractions[n - 1],
    };

    Ok(result)
}

/// Queries traders balance across all vamms with funding payment
pub fn query_trader_balance_with_funding_payment(deps: Deps, trader: String) -> StdResult<Uint128> {
    let config = read_config(deps.storage).unwrap();

    let mut margin = Uint128::zero();
    let vamms = query_insurance_all_vamm(&deps, config.insurance_fund.to_string(), None)?.vamm_list;
    for vamm in vamms.iter() {
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
) -> StdResult<Position> {
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

    Ok(position)
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

    let PositionUnrealizedPnlResponse {
        position_notional: spot_notional,
        unrealized_pnl: spot_pnl,
    } = get_position_notional_unrealized_pnl(deps, &position, PnlCalcOption::SpotPrice)?;
    let PositionUnrealizedPnlResponse {
        position_notional: twap_notional,
        unrealized_pnl: twap_pnl,
    } = get_position_notional_unrealized_pnl(deps, &position, PnlCalcOption::Twap)?;

    // calculate and return margin
    let PositionUnrealizedPnlResponse {
        position_notional,
        unrealized_pnl,
    } = if spot_pnl.abs() > twap_pnl.abs() {
        PositionUnrealizedPnlResponse {
            position_notional: twap_notional,
            unrealized_pnl: twap_pnl,
        }
    } else {
        PositionUnrealizedPnlResponse {
            position_notional: spot_notional,
            unrealized_pnl: spot_pnl,
        }
    };

    let remain_margin = calc_remain_margin_with_funding_payment(deps, position, unrealized_pnl)?;

    let margin_ratio = ((Integer::new_positive(remain_margin.margin)
        - Integer::new_positive(remain_margin.bad_debt))
        * Integer::new_positive(config.decimals))
        / Integer::new_positive(position_notional);

    Ok(margin_ratio)
}

/// Queries the withdrawable collateral of a trader
pub fn query_free_collateral(deps: Deps, vamm: String, trader: String) -> StdResult<Integer> {
    let config: Config = read_config(deps.storage)?;

    // retrieve the latest position
    let position = query_trader_position_with_funding_payment(deps, vamm, trader)?;

    // get trader's unrealized PnL and choose the least beneficial one for the trader
    let PositionUnrealizedPnlResponse {
        position_notional: spot_notional,
        unrealized_pnl: spot_pnl,
    } = get_position_notional_unrealized_pnl(deps, &position, PnlCalcOption::SpotPrice)?;
    let PositionUnrealizedPnlResponse {
        position_notional: twap_notional,
        unrealized_pnl: twap_pnl,
    } = get_position_notional_unrealized_pnl(deps, &position, PnlCalcOption::Twap)?;

    // calculate and return margin
    let PositionUnrealizedPnlResponse {
        position_notional,
        unrealized_pnl,
    } = if spot_pnl.abs() > twap_pnl.abs() {
        PositionUnrealizedPnlResponse {
            position_notional: twap_notional,
            unrealized_pnl: twap_pnl,
        }
    } else {
        PositionUnrealizedPnlResponse {
            position_notional: spot_notional,
            unrealized_pnl: spot_pnl,
        }
    };

    // min(margin + funding, margin + funding + unrealized PnL) - position value * initMarginRatio
    let account_value = unrealized_pnl.checked_add(Integer::new_positive(position.margin))?;
    let minimum_collateral = if account_value
        .checked_sub(Integer::new_positive(position.margin))?
        .is_positive()
    {
        Integer::new_positive(position.margin)
    } else {
        account_value
    };

    let margin_requirement = if position.size.is_positive() {
        position
            .notional
            .checked_mul(config.initial_margin_ratio)?
            .checked_div(config.decimals)?
    } else {
        position_notional
            .checked_mul(config.initial_margin_ratio)?
            .checked_div(config.decimals)?
    };

    Ok(minimum_collateral.checked_sub(Integer::new_positive(margin_requirement))?)
}
