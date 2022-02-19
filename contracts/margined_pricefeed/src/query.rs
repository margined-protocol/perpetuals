use cosmwasm_std::{Deps, Env, StdError, StdResult, Uint128};
use margined_perp::margined_pricefeed::ConfigResponse;

use crate::state::{read_config, read_price_data, Config, PriceData};

/// Queries contract Config
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner,
        decimals: config.decimals,
    })
}

/// Queries latest price for pair stored with key
pub fn query_get_price(deps: Deps, key: String) -> StdResult<PriceData> {
    let prices_response = read_price_data(deps.storage, key);

    let prices = prices_response.unwrap();
    let price = prices.last().unwrap();

    Ok(price.clone())
}

/// Queries previous price for pair stored with key
pub fn query_get_previous_price(
    deps: Deps,
    key: String,
    num_round_back: Uint128,
) -> StdResult<PriceData> {
    let prices_response = read_price_data(deps.storage, key);

    let prices = prices_response.unwrap();
    let latest_price = prices.last().unwrap();

    if num_round_back > latest_price.round_id {
        return Err(StdError::generic_err("Not enough history"));
    }

    let mut previous_prices = prices.clone();

    // obvs not the most efficient way to do this but this is
    // just a placeholder while we build the twap logic and
    // do the integration with the rest of the project.
    let mut i = 0;
    while i < num_round_back.u128() {
        previous_prices.pop();
        i += 1;
    }

    let previous_price = previous_prices.last().unwrap();

    Ok(previous_price.clone())
}

/// Queries contract Config
pub fn query_get_twap_price(
    deps: Deps,
    env: Env,
    key: String,
    interval: u64,
) -> StdResult<Uint128> {
    if interval == 0 {
        return Err(StdError::generic_err("Interval can't be zero"));
    }

    let base_timestamp = env.block.time.seconds().checked_sub(interval).unwrap();
    let prices_response = read_price_data(deps.storage, key);

    // get the current data
    let mut prices = prices_response.unwrap();
    let mut latest_round = prices.last().unwrap();
    let mut timestamp = latest_round.timestamp.seconds();

    let mut cumulative_time =
        Uint128::from(env.block.time.seconds().checked_sub(timestamp).unwrap());

    let mut weighted_price = latest_round.price.checked_mul(cumulative_time)?;

    loop {
        if latest_round.round_id == Uint128::from(1u128) {
            let twap = weighted_price.checked_div(cumulative_time).unwrap();
            return Ok(twap);
        }

        prices.pop();
        latest_round = prices.last().unwrap();

        if latest_round.timestamp.seconds() <= base_timestamp {
            let delta_timestamp = Uint128::from(timestamp.checked_sub(base_timestamp).unwrap());

            weighted_price = weighted_price
                .checked_add(latest_round.price.checked_mul(delta_timestamp).unwrap())
                .unwrap();

            break;
        }

        let delta_timestamp = Uint128::from(
            timestamp
                .checked_sub(latest_round.timestamp.seconds())
                .unwrap(),
        );
        weighted_price = weighted_price
            .checked_add(latest_round.price.checked_mul(delta_timestamp).unwrap())
            .unwrap();

        cumulative_time = cumulative_time.checked_add(delta_timestamp).unwrap();
        timestamp = latest_round.timestamp.seconds();
    }

    let twap = weighted_price.checked_div(Uint128::from(interval)).unwrap();

    Ok(twap)
}
