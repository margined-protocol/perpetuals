use cosmwasm_std::{Deps, StdResult, Uint128};
use margined_perp::margined_vamm::{ConfigResponse, Direction, StateResponse};

use crate::{
    handle::{get_input_price_with_reserves, get_output_price_with_reserves},
    state::{read_config, read_state, Config, State},
};

/// Queries contract Config
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner,
        quote_asset: config.quote_asset,
        base_asset: config.base_asset,
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
        decimals: state.decimals,
    })
}

/// Queries output price
pub fn query_output_price(deps: Deps, direction: Direction, amount: Uint128) -> StdResult<Uint128> {
    let state: State = read_state(deps.storage)?;
    let res = get_output_price_with_reserves(&state, &direction, amount)?;

    Ok(res)
}

// /// Queries output price
// pub fn query_calc_fee(deps: Deps, quote_asset_amount: Uint128) -> StdResult<Uint128> {
//     let state: State = read_state(deps.storage)?;
//     let res = get_output_price_with_reserves(&state, &direction, amount)?;

//     Ok(res)
// }
