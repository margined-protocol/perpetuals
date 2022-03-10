use cosmwasm_std::{Deps, Env, StdResult, Uint128};
use margined_perp::margined_vamm::{CalcFeeResponse, ConfigResponse, Direction, StateResponse};

use crate::{
    handle::{get_input_price_with_reserves, get_output_price_with_reserves},
    state::{
        read_config, read_reserve_snapshot, read_reserve_snapshot_counter, read_state, Config,
        State,
    },
};

const FIFTEEN_MINUTES: u64 = 15 * 60;

/// Queries contract Config
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner,
        quote_asset: config.quote_asset,
        base_asset: config.base_asset,
        toll_ratio: config.toll_ratio,
        spread_ratio: config.spread_ratio,
        decimals: config.decimals,
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

/// Queries input price
pub fn _query_input_price(deps: Deps, direction: Direction, amount: Uint128) -> StdResult<Uint128> {
    let state: State = read_state(deps.storage)?;

    let res = get_input_price_with_reserves(
        deps,
        &direction,
        amount,
        state.quote_asset_reserve,
        state.base_asset_reserve,
    )?;

    Ok(res)
}

/// Queries output price
pub fn query_output_price(deps: Deps, direction: Direction, amount: Uint128) -> StdResult<Uint128> {
    let state: State = read_state(deps.storage)?;

    let res = get_output_price_with_reserves(
        deps,
        &direction,
        amount,
        state.quote_asset_reserve,
        state.base_asset_reserve,
    )?;

    Ok(res)
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
    calc_twap(deps, env, interval)
}

/// Queries twap price of the vAMM, using the reserve snapshots
pub fn query_input_twap(
    deps: Deps,
    env: Env,
    direction: Direction,
    amount: Uint128,
) -> StdResult<Uint128> {
    calc_twap_input_asset(deps, env, amount, true, &direction, FIFTEEN_MINUTES)
}

/// Queries twap price of the vAMM, using the reserve snapshots
pub fn query_output_twap(
    deps: Deps,
    env: Env,
    direction: Direction,
    amount: Uint128,
) -> StdResult<Uint128> {
    println!("query_output_twap");
    calc_twap_input_asset(deps, env, amount, false, &direction, FIFTEEN_MINUTES)
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

/// Calculates the TWAP of the AMM reserves
fn calc_twap(deps: Deps, env: Env, interval: u64) -> StdResult<Uint128> {
    println!("calc_twap");
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

        println!("period: {}", period);
        period = period.checked_add(delta_timestamp).unwrap();
        println!("period: {}", period);
        previous_timestamp = current_snapshot.timestamp.seconds();
    }

    Ok(weighted_price.checked_div(Uint128::from(interval))?)
}

/// TODO TODO TODO
/// Please clean this function up and amalgamate with that above **IF**
/// possible.
/// Calculates the TWAP of the AMM reserves with an input
fn calc_twap_input_asset(
    deps: Deps,
    env: Env,
    amount: Uint128,
    quote: bool,
    direction: &Direction,
    interval: u64,
) -> StdResult<Uint128> {
    println!("calc_twap_input_asset");
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

    println!("prev timestamp: {}", previous_timestamp);
    println!("period: {}", period);
    let mut weighted_price = current_price.checked_mul(period)?;

    loop {
        println!("counter: {}", counter);
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
            println!("output");
            current_price = get_output_price_with_reserves(
                deps,
                direction,
                amount,
                current_snapshot.quote_asset_reserve,
                current_snapshot.base_asset_reserve,
            )?;
        }
        println!("Here?");
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

        println!("period: {}", period);
        period = period.checked_add(delta_timestamp).unwrap();
        println!("period: {}", period);
        previous_timestamp = current_snapshot.timestamp.seconds();
    }

    Ok(weighted_price.checked_div(Uint128::from(interval))?)
}
