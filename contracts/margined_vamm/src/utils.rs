use cosmwasm_std::{Addr, Deps, Env, Response, StdError, StdResult, Storage, Uint128};
use margined_perp::margined_vamm::Direction;

use crate::{
    handle::{get_input_price_with_reserves, get_output_price_with_reserves},
    state::{
        read_config, read_reserve_snapshot, read_reserve_snapshot_counter, read_state,
        store_reserve_snapshot, update_reserve_snapshot, Config, ReserveSnapshot, State,
    },
};

pub fn require_margin_engine(sender: Addr, margin_engine: Addr) -> StdResult<Response> {
    // check that it is a registered vamm
    if sender != margin_engine {
        return Err(StdError::generic_err("sender not margin engine"));
    }

    Ok(Response::new())
}

pub fn require_open(open: bool) -> StdResult<Response> {
    // check that it is a registered vamm
    if !open {
        return Err(StdError::generic_err("amm is closed"));
    }

    Ok(Response::new())
}

pub(crate) fn check_is_over_block_fluctuation_limit(
    storage: &mut dyn Storage,
    env: Env,
    direction: Direction,
    quote_asset_amount: Uint128,
    base_asset_amount: Uint128,
    can_go_over_limit: bool,
) -> StdResult<Response> {
    let config = read_config(storage)?;
    let state = read_state(storage)?;

    if config.fluctuation_limit_ratio.is_zero() {
        return Ok(Response::new());
    }

    // calculate the price boundary of the previous block
    let height = read_reserve_snapshot_counter(storage)?;
    let mut latest_snapshot = read_reserve_snapshot(storage, height)?;

    if latest_snapshot.block_height == env.block.height && height > 1 {
        latest_snapshot = read_reserve_snapshot(storage, height - 1u64)?;
    }

    let last_price = latest_snapshot
        .quote_asset_reserve
        .checked_mul(config.decimals)?
        .checked_div(latest_snapshot.base_asset_reserve)?;
    let upper_limit = last_price
        .checked_mul(config.decimals + config.fluctuation_limit_ratio)?
        .checked_div(config.decimals)?;
    let lower_limit = last_price
        .checked_mul(config.decimals - config.fluctuation_limit_ratio)?
        .checked_div(config.decimals)?;

    let current_price = state
        .quote_asset_reserve
        .checked_mul(config.decimals)?
        .checked_div(state.base_asset_reserve)?;

    // ensure that the latest price isn't over the limit which would restrict any further
    // swaps from occurring in this block
    if current_price > upper_limit || current_price < lower_limit {
        return Err(StdError::generic_err(
            "price is already over fluctuation limit",
        ));
    }

    if !can_go_over_limit {
        let price = if direction == Direction::AddToAmm {
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
        if price > upper_limit || price < lower_limit {
            return Err(StdError::generic_err("price is over fluctuation limit"));
        }
    }

    Ok(Response::new())
}

pub fn add_reserve_snapshot(
    storage: &mut dyn Storage,
    env: Env,
    quote_asset_reserve: Uint128,
    base_asset_reserve: Uint128,
) -> StdResult<Response> {
    let height = read_reserve_snapshot_counter(storage)?;
    let current_snapshot = read_reserve_snapshot(storage, height)?;

    if current_snapshot.block_height == env.block.height {
        let new_snapshot = ReserveSnapshot {
            quote_asset_reserve,
            base_asset_reserve,
            timestamp: current_snapshot.timestamp,
            block_height: current_snapshot.block_height,
        };

        update_reserve_snapshot(storage, &new_snapshot)?;
    } else {
        let new_snapshot = ReserveSnapshot {
            quote_asset_reserve,
            base_asset_reserve,
            timestamp: env.block.time,
            block_height: env.block.height,
        };

        store_reserve_snapshot(storage, &new_snapshot)?;
    }

    Ok(Response::default())
}

/// Calculates the TWAP of the AMM reserves
pub fn calc_twap(deps: Deps, env: Env, interval: u64) -> StdResult<Uint128> {
    let config: Config = read_config(deps.storage)?;
    let mut counter = read_reserve_snapshot_counter(deps.storage).unwrap();
    let current_snapshot = read_reserve_snapshot(deps.storage, counter);
    let mut current_snapshot = current_snapshot.unwrap();

    let mut current_price = current_snapshot
        .quote_asset_reserve
        .checked_mul(config.decimals)?
        .checked_div(current_snapshot.base_asset_reserve)?;

    if interval == 0 {
        return Ok(current_price);
    }

    let base_timestamp = env.block.time.seconds().checked_sub(interval).unwrap();

    if counter == 1 || current_snapshot.timestamp.seconds() <= base_timestamp {
        return Ok(current_price);
    }

    let mut previous_timestamp = current_snapshot.timestamp.seconds();
    let mut period = Uint128::from(
        env.block
            .time
            .seconds()
            .checked_sub(previous_timestamp)
            .unwrap(),
    );

    let mut weighted_price = current_price.checked_mul(period)?;

    loop {
        counter -= 1;
        // if snapshot history is too short
        if counter == 0 {
            return Ok(weighted_price.checked_div(period)?);
        }
        current_snapshot = read_reserve_snapshot(deps.storage, counter).unwrap();
        current_price = current_snapshot
            .quote_asset_reserve
            .checked_mul(config.decimals)?
            .checked_div(current_snapshot.base_asset_reserve)?;

        if current_snapshot.timestamp.seconds() <= base_timestamp {
            let delta_timestamp =
                Uint128::from(previous_timestamp.checked_sub(base_timestamp).unwrap());

            weighted_price = weighted_price
                .checked_add(current_price.checked_mul(delta_timestamp).unwrap())
                .unwrap();

            break;
        }

        let delta_timestamp = Uint128::from(
            previous_timestamp
                .checked_sub(current_snapshot.timestamp.seconds())
                .unwrap(),
        );
        weighted_price = weighted_price
            .checked_add(current_price.checked_mul(delta_timestamp).unwrap())
            .unwrap();

        period = period.checked_add(delta_timestamp).unwrap();
        previous_timestamp = current_snapshot.timestamp.seconds();
    }

    Ok(weighted_price.checked_div(Uint128::from(interval))?)
}

// fn get_price_with_specific_snapshot() {}

// TODO TODO TODO
// Please clean this function up and amalgamate with that above **IF**
// possible.
/// Calculates the TWAP of the AMM reserves with an input
pub fn calc_twap_input_asset(
    deps: Deps,
    env: Env,
    amount: Uint128,
    quote: bool,
    direction: &Direction,
    interval: u64,
) -> StdResult<Uint128> {
    let state: State = read_state(deps.storage)?;
    let mut counter = read_reserve_snapshot_counter(deps.storage).unwrap();
    let current_snapshot = read_reserve_snapshot(deps.storage, counter);
    let mut current_snapshot = current_snapshot.unwrap();

    let mut current_price: Uint128 = if quote {
        get_input_price_with_reserves(
            deps,
            direction,
            amount,
            state.quote_asset_reserve,
            state.base_asset_reserve,
        )?
    } else {
        get_output_price_with_reserves(
            deps,
            direction,
            amount,
            state.quote_asset_reserve,
            state.base_asset_reserve,
        )?
    };

    if interval == 0 {
        return Ok(current_price);
    }

    let base_timestamp = env.block.time.seconds().checked_sub(interval).unwrap();

    if counter == 1 || current_snapshot.timestamp.seconds() <= base_timestamp {
        return Ok(current_price);
    }

    let mut previous_timestamp = current_snapshot.timestamp.seconds();
    let mut period = Uint128::from(
        env.block
            .time
            .seconds()
            .checked_sub(previous_timestamp)
            .unwrap(),
    );

    let mut weighted_price = current_price.checked_mul(period)?;

    loop {
        counter -= 1;
        // if snapshot history is too short
        if counter == 0 {
            return Ok(weighted_price.checked_div(period)?);
        }
        current_snapshot = read_reserve_snapshot(deps.storage, counter).unwrap();
        if quote {
            current_price = get_input_price_with_reserves(
                deps,
                direction,
                amount,
                current_snapshot.quote_asset_reserve,
                current_snapshot.base_asset_reserve,
            )?;
        } else {
            current_price = get_output_price_with_reserves(
                deps,
                direction,
                amount,
                current_snapshot.quote_asset_reserve,
                current_snapshot.base_asset_reserve,
            )?;
        }
        if current_snapshot.timestamp.seconds() <= base_timestamp {
            let delta_timestamp =
                Uint128::from(previous_timestamp.checked_sub(base_timestamp).unwrap());

            weighted_price = weighted_price
                .checked_add(current_price.checked_mul(delta_timestamp).unwrap())
                .unwrap();

            break;
        }

        let delta_timestamp = Uint128::from(
            previous_timestamp
                .checked_sub(current_snapshot.timestamp.seconds())
                .unwrap(),
        );
        weighted_price = weighted_price
            .checked_add(current_price.checked_mul(delta_timestamp).unwrap())
            .unwrap();

        period = period.checked_add(delta_timestamp).unwrap();
        previous_timestamp = current_snapshot.timestamp.seconds();
    }

    Ok(weighted_price.checked_div(Uint128::from(interval))?)
}

/// Does the modulus (%) operator on Uint128.
/// However it follows the design of the perpetual protocol decimals
/// https://github.com/perpetual-protocol/perpetual-protocol/blob/release/v2.1.x/src/utils/Decimal.sol
pub(crate) fn modulo(a: Uint128, b: Uint128, decimals: Uint128) -> Uint128 {
    let a_decimals = a.checked_mul(decimals).unwrap();
    let integral = a_decimals / b;
    a_decimals - (b * integral)
}
