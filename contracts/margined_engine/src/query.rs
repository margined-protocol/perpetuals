use cosmwasm_std::{Deps, StdError, StdResult, Uint128, Order as OrderBy};
use margined_common::integer::Integer;
use margined_perp::margined_engine::{
    ConfigResponse, PauserResponse, PnlCalcOption, Position, PositionUnrealizedPnlResponse,
    StateResponse,
};
use margined_utils::contracts::helpers::InsuranceFundController;

use crate::{
    contract::PAUSER,
    state::{read_config, read_position, read_state, read_vamm_map, read_positions},
    utils::{
        calc_funding_payment, calc_remain_margin_with_funding_payment,
        get_position_notional_unrealized_pnl, keccak_256,
    },
};

/// Queries contract Config
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    read_config(deps.storage)
}

/// Queries contract State
pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = read_state(deps.storage)?;

    Ok(StateResponse {
        open_interest_notional: state.open_interest_notional,
        bad_debt: state.prepaid_bad_debt,
    })
}

/// Queries pauser from the admin
pub fn query_pauser(deps: Deps) -> StdResult<PauserResponse> {
    if let Some(pauser) = PAUSER.get(deps)? {
        Ok(PauserResponse { pauser })
    } else {
        Err(StdError::generic_err("No pauser set"))
    }
}

/// Queries user position
pub fn query_position(deps: Deps, vamm: String, position_id: u64) -> StdResult<Position> {
    // if vamm and trader are not correct, position_key will throw not found error
    let position_key = keccak_256(&[vamm.as_bytes()].concat());
    let position = read_position(deps.storage, &position_key, position_id)?;

    Ok(position)
}

/// Queries and returns users position for all registered vamms
pub fn query_all_positions(
    deps: Deps,
    trader: String,
    start_after: Option<u64>,
    limit: Option<u32>,
    order_by: Option<i32>
) -> StdResult<Vec<Position>> {
    let config = read_config(deps.storage)?;
    let order_by = order_by.map_or(None, |val| OrderBy::try_from(val).ok());
    let mut response: Vec<Position> = vec![];

    let vamms = match config.insurance_fund {
        Some(insurance_fund) => {
            let insurance_controller = InsuranceFundController(insurance_fund);
            insurance_controller
                .all_vamms(&deps.querier, None)?
                .vamm_list
        }
        None => return Err(StdError::generic_err("insurance fund is not registered")),
    };

    for vamm in vamms.iter() {
        println!("query_all_positions - vamm: {:?}", vamm);
        let position_key = keccak_256(&[vamm.as_bytes()].concat());
        let positions = read_positions(deps.storage, &position_key, start_after, limit, order_by).unwrap();

        for position in positions {
            println!("query_all_positions - position: {:?}", position);
            // a default is returned if no position found with no trader set
            if position.trader == trader {
                response.push(position)
            }
        }
    }

    Ok(response)
}

/// Queries user position
pub fn query_position_notional_unrealized_pnl(
    deps: Deps,
    vamm: String,
    position_id: u64,
    calc_option: PnlCalcOption,
) -> StdResult<PositionUnrealizedPnlResponse> {
    let position_key = keccak_256(&[vamm.as_bytes()].concat());
    // read the msg.senders position
    let position = read_position(deps.storage, &position_key, position_id)?;

    let result = get_position_notional_unrealized_pnl(deps, &position, calc_option)?;

    Ok(result)
}

/// Queries cumulative premium fractions
pub fn query_cumulative_premium_fraction(deps: Deps, vamm: String) -> StdResult<Integer> {
    // retrieve vamm data
    let vamm_map = read_vamm_map(deps.storage, &deps.api.addr_validate(&vamm)?)?;

    let result = match vamm_map.cumulative_premium_fractions.len() {
        0 => Integer::zero(),
        n => vamm_map.cumulative_premium_fractions[n - 1],
    };

    println!("query_cumulative_premium_fraction - result: {}", result);
    Ok(result)
}

/// Queries traders balance across all vamms with funding payment
pub fn query_trader_balance_with_funding_payment(deps: Deps, position_id: u64) -> StdResult<Uint128> {
    let config = read_config(deps.storage)?;

    let mut margin = Uint128::zero();

    let vamms = match config.insurance_fund {
        Some(insurance_fund) => {
            let insurance_controller = InsuranceFundController(insurance_fund);
            insurance_controller
                .all_vamms(&deps.querier, None)?
                .vamm_list
        }
        None => return Err(StdError::generic_err("insurance fund is not registered")),
    };

    for vamm in vamms.iter() {
        let position =
            query_trader_position_with_funding_payment(deps, vamm.to_string(), position_id)?;
        margin = margin.checked_add(position.margin)?;
    }

    Ok(margin)
}

/// Queries traders position across all vamms with funding payments
pub fn query_trader_position_with_funding_payment(
    deps: Deps,
    vamm: String,
    position_id: u64,
) -> StdResult<Position> {
    let config = read_config(deps.storage)?;

    let position_key = keccak_256(&[vamm.as_bytes()].concat());

    // retrieve latest user position
    let mut position = read_position(deps.storage, &position_key, position_id)?;

    let latest_cumulative_premium_fraction =
        query_cumulative_premium_fraction(deps, vamm.to_string())?;

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
pub fn query_margin_ratio(deps: Deps, vamm: String, position_id: u64) -> StdResult<Integer> {
    let position_key = keccak_256(&[vamm.as_bytes()].concat());
    // retrieve the latest position
    let position = read_position(deps.storage, &position_key, position_id)?;

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

    let config = read_config(deps.storage)?;
    let margin_ratio = ((Integer::new_positive(remain_margin.margin)
        - Integer::new_positive(remain_margin.bad_debt))
        * Integer::new_positive(config.decimals))
        / Integer::new_positive(position_notional);

    Ok(margin_ratio)
}

/// Queries the withdrawable collateral of a trader
pub fn query_free_collateral(deps: Deps, vamm: String, position_id: u64) -> StdResult<Integer> {
    // retrieve the latest position
    let position = query_trader_position_with_funding_payment(deps, vamm, position_id)?;

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

    let config = read_config(deps.storage)?;

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
