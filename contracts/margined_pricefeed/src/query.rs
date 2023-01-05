use cosmwasm_std::{Deps, Env, StdError, StdResult, Uint128};
use margined_perp::margined_pricefeed::{ConfigResponse, OwnerResponse, PriceData};

use crate::{contract::OWNER, state::read_price_data};

/// Queries contract Config
pub fn query_config(_deps: Deps) -> StdResult<ConfigResponse> {
    Ok(ConfigResponse {})
}

/// Queries contract owner from the admin
pub fn query_owner(deps: Deps) -> StdResult<OwnerResponse> {
    if let Some(owner) = OWNER.get(deps)? {
        Ok(OwnerResponse { owner })
    } else {
        Err(StdError::generic_err("No owner set"))
    }
}

/// Queries latest price for pair stored with key
pub fn query_get_price(deps: Deps, key: String) -> StdResult<PriceData> {
    let prices = read_price_data(deps.storage, key)?;

    if let Some(price) = prices.last() {
        return Ok(price.clone());
    }

    Err(StdError::generic_err("No price found"))
}

/// Queries previous price for pair stored with key
pub fn query_get_previous_price(
    deps: Deps,
    key: String,
    num_round_back: u64,
) -> StdResult<PriceData> {
    let mut prices = read_price_data(deps.storage, key)?;
    prices.sort_by(|a, b| a.round_id.cmp(&b.round_id));

    // check ind to get last previous price ind by num_round_back
    if let Some(ind) = prices.len().checked_sub((num_round_back + 1) as usize) {
        return Ok(prices[ind].clone());
    }

    Err(StdError::generic_err("Not enough history"))
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

    if latest_round.round_id == 0u64 {
        return Err(StdError::generic_err("Insufficient history"));
    }

    // if latest updated timestamp is earlier than target timestamp, return the latest price.
    if latest_round.timestamp.seconds() < base_timestamp || latest_round.round_id == 1u64 {
        return Ok(latest_round.price);
    }

    let mut cumulative_time =
        Uint128::from(env.block.time.seconds().checked_sub(timestamp).unwrap());

    let mut weighted_price = latest_round.price.checked_mul(cumulative_time)?;

    loop {
        if latest_round.round_id == 1u64 {
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
