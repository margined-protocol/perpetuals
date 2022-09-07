use cosmwasm_std::{Deps, Env, StdError, StdResult, Uint128};
use margined_common::integer::Integer;
use margined_perp::margined_vamm::{CalcFeeResponse, ConfigResponse, Direction, StateResponse};

use crate::{
    contract::MAX_ORACLE_SPREAD_RATIO,
    handle::{get_input_price_with_reserves, get_output_price_with_reserves},
    querier::query_underlying_price,
    state::{read_config, read_reserve_snapshot_counter, read_state, Config, State},
    utils::{
        calc_twap, price_boundaries_of_last_block, TwapCalcOption, TwapInputAsset,
        TwapPriceCalcParams,
    },
};

const FIFTEEN_MINUTES: u64 = 15 * 60;

/// Queries contract Config
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner,
        base_asset_holding_cap: config.base_asset_holding_cap,
        open_interest_notional_cap: config.open_interest_notional_cap,
        quote_asset: config.quote_asset,
        base_asset: config.base_asset,
        toll_ratio: config.toll_ratio,
        spread_ratio: config.spread_ratio,
        fluctuation_limit_ratio: config.fluctuation_limit_ratio,
        decimals: config.decimals,
        margin_engine: config.margin_engine,
        insurance_fund: config.insurance_fund,
        pricefeed: config.pricefeed,
        funding_period: config.funding_period,
    })
}

/// Queries contract State
pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state: State = read_state(deps.storage)?;

    Ok(StateResponse {
        open: state.open,
        quote_asset_reserve: state.quote_asset_reserve,
        base_asset_reserve: state.base_asset_reserve,
        total_position_size: state.total_position_size,
        funding_rate: state.funding_rate,
        next_funding_time: state.next_funding_time,
    })
}

/// Queries input price
pub fn query_input_price(deps: Deps, direction: Direction, amount: Uint128) -> StdResult<Uint128> {
    let config: Config = read_config(deps.storage)?;
    let state: State = read_state(deps.storage)?;

    let output = get_input_price_with_reserves(
        deps,
        &direction,
        amount,
        state.quote_asset_reserve,
        state.base_asset_reserve,
    )?;

    let price = amount.checked_mul(config.decimals)?.checked_div(output)?;

    Ok(price)
}

/// Queries output price
pub fn query_output_price(deps: Deps, direction: Direction, amount: Uint128) -> StdResult<Uint128> {
    let config: Config = read_config(deps.storage)?;
    let state: State = read_state(deps.storage)?;

    let output = get_output_price_with_reserves(
        deps,
        &direction,
        amount,
        state.quote_asset_reserve,
        state.base_asset_reserve,
    )?;

    let price = amount.checked_mul(config.decimals)?.checked_div(output)?;

    Ok(price)
}

/// Queries input amount
pub fn query_input_amount(deps: Deps, direction: Direction, amount: Uint128) -> StdResult<Uint128> {
    let state: State = read_state(deps.storage)?;

    let output = get_input_price_with_reserves(
        deps,
        &direction,
        amount,
        state.quote_asset_reserve,
        state.base_asset_reserve,
    )?;

    Ok(output)
}

/// Queries output amount
pub fn query_output_amount(
    deps: Deps,
    direction: Direction,
    amount: Uint128,
) -> StdResult<Uint128> {
    let state: State = read_state(deps.storage)?;

    let output = get_output_price_with_reserves(
        deps,
        &direction,
        amount,
        state.quote_asset_reserve,
        state.base_asset_reserve,
    )?;

    Ok(output)
}

/// Queries spot price of the vAMM
pub fn query_spot_price(deps: Deps) -> StdResult<Uint128> {
    let config: Config = read_config(deps.storage)?;
    let state: State = read_state(deps.storage)?;

    let res = state
        .quote_asset_reserve
        .checked_mul(config.decimals)?
        .checked_div(state.base_asset_reserve)?;

    Ok(res)
}

/// Queries twap price of the vAMM, using the reserve snapshots
pub fn query_twap_price(deps: Deps, env: Env, interval: u64) -> StdResult<Uint128> {
    let snapshot_index = read_reserve_snapshot_counter(deps.storage).unwrap();
    let params = TwapPriceCalcParams {
        opt: TwapCalcOption::Reserve,
        snapshot_index,
        asset: None,
    };
    calc_twap(deps, env, params, interval)
}

/// Queries twap price of the vAMM, using the reserve snapshots
pub fn query_input_twap(
    deps: Deps,
    env: Env,
    direction: Direction,
    amount: Uint128,
) -> StdResult<Uint128> {
    let snapshot_index = read_reserve_snapshot_counter(deps.storage).unwrap();

    let asset = TwapInputAsset {
        direction,
        amount,
        quote: true,
    };

    let params = TwapPriceCalcParams {
        opt: TwapCalcOption::Input,
        snapshot_index,
        asset: Some(asset),
    };

    calc_twap(deps, env, params, FIFTEEN_MINUTES)
}

/// Queries twap price of the vAMM, using the reserve snapshots
pub fn query_output_twap(
    deps: Deps,
    env: Env,
    direction: Direction,
    amount: Uint128,
) -> StdResult<Uint128> {
    let snapshot_index = read_reserve_snapshot_counter(deps.storage).unwrap();

    let asset = TwapInputAsset {
        direction,
        amount,
        quote: false,
    };

    let params = TwapPriceCalcParams {
        opt: TwapCalcOption::Input,
        snapshot_index,
        asset: Some(asset),
    };

    calc_twap(deps, env, params, FIFTEEN_MINUTES)
}

/// Returns the total (i.e. toll + spread) fees for an amount
pub fn query_calc_fee(deps: Deps, quote_asset_amount: Uint128) -> StdResult<CalcFeeResponse> {
    let mut res = CalcFeeResponse {
        toll_fee: Uint128::zero(),
        spread_fee: Uint128::zero(),
    };

    if quote_asset_amount != Uint128::zero() {
        let config: Config = read_config(deps.storage)?;

        res.toll_fee = quote_asset_amount
            .checked_mul(config.toll_ratio)?
            .checked_div(config.decimals)?;
        res.spread_fee = quote_asset_amount
            .checked_mul(config.spread_ratio)?
            .checked_div(config.decimals)?;
    }

    Ok(res)
}

/// Returns bool to show is spread limit has been exceeded
pub fn query_is_over_spread_limit(deps: Deps) -> StdResult<bool> {
    let config: Config = read_config(deps.storage)?;

    // get price from the oracle
    let oracle_price = query_underlying_price(&deps)?;
    if oracle_price.is_zero() {
        return Err(StdError::generic_err("underlying price is 0"));
    }

    // get the local market price of the vamm
    let market_price = query_spot_price(deps)?;

    let current_spread_ratio = (Integer::new_positive(market_price)
        - Integer::new_positive(oracle_price))
        * Integer::new_positive(config.decimals)
        / Integer::new_positive(oracle_price);

    Ok(current_spread_ratio.abs() >= Integer::new_positive(MAX_ORACLE_SPREAD_RATIO))
}

/// Returns bool to show is fluctuation limit has been exceeded
pub fn query_is_over_fluctuation_limit(
    deps: Deps,
    env: Env,
    direction: Direction,
    base_asset_amount: Uint128,
) -> StdResult<bool> {
    let config: Config = read_config(deps.storage)?;
    let state: State = read_state(deps.storage)?;

    if config.fluctuation_limit_ratio.is_zero() {
        return Ok(false);
    };

    let (upper_limit, lower_limit) = price_boundaries_of_last_block(deps.storage, env)?;

    let quote_asset_amount = query_output_amount(deps, direction.clone(), base_asset_amount)?;

    let price = if direction == Direction::RemoveFromAmm {
        state
            .quote_asset_reserve
            .checked_add(quote_asset_amount)?
            .checked_mul(config.decimals)?
            .checked_div(state.base_asset_reserve.checked_sub(base_asset_amount)?)
    } else {
        state
            .quote_asset_reserve
            .checked_sub(quote_asset_amount)?
            .checked_mul(config.decimals)?
            .checked_div(state.base_asset_reserve.checked_add(base_asset_amount)?)
    }?;

    if price <= upper_limit && price >= lower_limit {
        return Ok(false);
    }

    Ok(true)
}
