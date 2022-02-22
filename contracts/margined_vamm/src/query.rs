use cosmwasm_std::{Deps, StdResult};
use cosmwasm_bignumber::{Decimal256};
use margined_perp::margined_vamm::{CalcFeeResponse, ConfigResponse, Direction, StateResponse};

use crate::{
    handle::get_output_price_with_reserves,
    state::{read_config, read_state, Config, State},
};

/// Queries contract Config
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner,
        quote_asset: config.quote_asset,
        base_asset: config.base_asset,
        toll_ratio: config.toll_ratio,
        spread_ratio: config.spread_ratio
    })
}

/// Queries contract State
pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state: State = read_state(deps.storage)?;

    Ok(StateResponse {
        quote_asset_reserve: state.quote_asset_reserve,
        base_asset_reserve: state.base_asset_reserve,
        funding_rate: state.funding_rate,
        funding_period: state.funding_period,
    })
}

/// Queries output price
pub fn query_output_price(deps: Deps, direction: Direction, amount: Decimal256) -> StdResult<Decimal256> {
    let res = get_output_price_with_reserves(deps, &direction, amount)?;

    Ok(res)
}

/// Returns the total (i.e. toll + spread) fees for an amount
pub fn query_calc_fee(deps: Deps, quote_asset_amount: Decimal256) -> StdResult<CalcFeeResponse> {
    let mut res = CalcFeeResponse {
        toll_fee: Decimal256::zero(),
        spread_fee: Decimal256::zero(),
    };

    if quote_asset_amount != Decimal256::zero() {
        let config: Config = read_config(deps.storage)?;

        res.toll_fee = quote_asset_amount * config.toll_ratio;
        res.spread_fee = quote_asset_amount * config.spread_ratio;
    }

    Ok(res)
}
