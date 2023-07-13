use cosmwasm_std::{Deps, Env, StdError, StdResult, Uint128};
use margined_common::integer::Integer;
use margined_perp::margined_vamm::{
    CalcFeeResponse, ConfigResponse, Direction, OwnerResponse, StateResponse,
};
use margined_utils::contracts::helpers::PricefeedController;

use crate::{
    contract::OWNER,
    handle::{get_input_price_with_reserves, get_output_price_with_reserves},
    state::{read_config, read_reserve_snapshot_counter, read_state},
    utils::{
        calc_twap, price_boundaries_of_last_block, TwapCalcOption, TwapInputAsset,
        TwapPriceCalcParams,
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
        open: state.open,
        quote_asset_reserve: state.quote_asset_reserve,
        base_asset_reserve: state.base_asset_reserve,
        total_position_size: state.total_position_size,
        funding_rate: state.funding_rate,
        next_funding_time: state.next_funding_time,
    })
}

/// Queries contract owner from the admin
pub fn query_owner(deps: Deps) -> StdResult<OwnerResponse> {
    if let Some(owner) = OWNER.get(deps)? {
        Ok(OwnerResponse { owner })
    } else {
        Err(StdError::generic_err("No owner set"))
    }
}

/// Queries input price
pub fn query_input_price(deps: Deps, direction: Direction, amount: Uint128) -> StdResult<Uint128> {
    let state = read_state(deps.storage)?;

    let output = get_input_price_with_reserves(
        deps,
        &direction,
        amount,
        state.quote_asset_reserve,
        state.base_asset_reserve,
    )?;

    if output.is_zero() {
        return Ok(Uint128::zero());
    }
    let config = read_config(deps.storage)?;
    let price = amount.checked_mul(config.decimals)?.checked_div(output)?;

    Ok(price)
}

/// Queries output price
pub fn query_output_price(deps: Deps, direction: Direction, amount: Uint128) -> StdResult<Uint128> {
    let state = read_state(deps.storage)?;

    let output = get_output_price_with_reserves(
        deps,
        &direction,
        amount,
        state.quote_asset_reserve,
        state.base_asset_reserve,
    )?;

    if output.is_zero() {
        return Ok(Uint128::zero());
    }

    let config = read_config(deps.storage)?;

    let price = amount.checked_mul(config.decimals)?.checked_div(output)?;

    Ok(price)
}

/// Queries input amount
pub fn query_input_amount(deps: Deps, direction: Direction, amount: Uint128) -> StdResult<Uint128> {
    let state = read_state(deps.storage)?;

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
    let state = read_state(deps.storage)?;
    println!("query_output_amount - state: {:?}", state);
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
    let config = read_config(deps.storage)?;
    let state = read_state(deps.storage)?;

    let res = state
        .quote_asset_reserve
        .checked_mul(config.decimals)?
        .checked_div(state.base_asset_reserve)?;

    Ok(res)
}

/// Queries twap price of the vAMM, using the reserve snapshots
pub fn query_twap_price(
    deps: Deps,
    env: Env,
    interval: u64,
    opt: TwapCalcOption,
    asset: Option<TwapInputAsset>,
) -> StdResult<Uint128> {
    let snapshot_index = read_reserve_snapshot_counter(deps.storage)?;
    let params = TwapPriceCalcParams {
        opt,
        snapshot_index,
        asset,
    };
    calc_twap(deps, env, params, interval)
}

/// Returns the total (i.e. toll + spread) fees for an amount
pub fn query_calc_fee(deps: Deps, quote_asset_amount: Uint128) -> StdResult<CalcFeeResponse> {
    let mut res = CalcFeeResponse {
        toll_fee: Uint128::zero(),
        spread_fee: Uint128::zero(),
    };

    if quote_asset_amount != Uint128::zero() {
        let config = read_config(deps.storage)?;

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
    let config: ConfigResponse = read_config(deps.storage)?;
    let pricefeed_controller = PricefeedController(config.pricefeed);

    // get price from the oracle
    let oracle_price = pricefeed_controller.get_price(&deps.querier, config.base_asset)?;
    println!("query_is_over_spread_limit - oracle_price: {:?}", oracle_price);
    if oracle_price.is_zero() {
        return Err(StdError::generic_err("underlying price is 0"));
    }

    // get the local market price of the vamm
    let market_price = query_spot_price(deps)?;
    println!("query_is_over_spread_limit - market_price: {:?}", market_price);

    let current_spread_ratio = (Integer::new_positive(market_price)
        - Integer::new_positive(oracle_price))
        * Integer::new_positive(config.decimals)
        / Integer::new_positive(oracle_price);
    println!("query_is_over_spread_limit - current_spread_ratio: {:?}", current_spread_ratio);

    let max_oracle_spread_ratio =
        Integer::new_positive(config.decimals).checked_div(Integer::from(10u128))?; // 0.1 i.e. 10%
    println!("query_is_over_spread_limit - max_oracle_spread_ratio: {:?}", max_oracle_spread_ratio);
    Ok(current_spread_ratio.abs() >= max_oracle_spread_ratio)
}

/// Returns bool to show is fluctuation limit has been exceeded
pub fn query_is_over_fluctuation_limit(
    deps: Deps,
    env: Env,
    direction: Direction,
    base_asset_amount: Uint128,
) -> StdResult<bool> {
    let config = read_config(deps.storage)?;

    if config.fluctuation_limit_ratio.is_zero() {
        return Ok(false);
    };

    let (upper_limit, lower_limit) = price_boundaries_of_last_block(deps.storage, env)?;

    let quote_asset_amount = query_output_amount(deps, direction.clone(), base_asset_amount)?;

    let state = read_state(deps.storage)?;

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

    Ok(price > upper_limit || price < lower_limit)
}
