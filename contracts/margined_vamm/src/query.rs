use cosmwasm_std::{Deps, Env, StdResult};
use cosmwasm_bignumber::{Decimal256, Uint256};
use margined_perp::margined_vamm::{CalcFeeResponse, ConfigResponse, Direction, StateResponse};

use crate::{
    handle::get_output_price_with_reserves,
    state::{
        read_config, read_reserve_snapshot, read_reserve_snapshot_counter, read_state, Config,
        State,
    },
};

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

/// Queries output price
pub fn query_output_price(deps: Deps, direction: Direction, amount: Decimal256) -> StdResult<Decimal256> {
    let res = get_output_price_with_reserves(deps, &direction, amount)?;

    Ok(res)
}

/// Queries spot price of the vAMM
pub fn query_spot_price(deps: Deps) -> StdResult<Decimal256> {
    // let config: Config = read_config(deps.storage)?;
    let state: State = read_state(deps.storage)?;

    let res = state
        .quote_asset_reserve
        / state.base_asset_reserve;
        // .checked_mul(config.decimals)?

    Ok(res)
}

/// Queries twap price of the vAMM, using the reserve snapshots
pub fn query_twap_price(deps: Deps, env: Env, interval: u64) -> StdResult<Decimal256> {
    calc_reserve_twap(deps, env, interval)
}

/// Returns the total (i.e. toll + spread) fees for an amount
pub fn query_calc_fee(deps: Deps, quote_asset_amount: Decimal256) -> StdResult<CalcFeeResponse> {
    let mut res = CalcFeeResponse {
        toll_fee: Decimal256::zero(),
        spread_fee: Decimal256::zero(),
    };

    if quote_asset_amount != Decimal256::zero() {
        let config: Config = read_config(deps.storage)?;

        res.toll_fee = quote_asset_amount
            * config.toll_ratio;
            // .checked_div(config.decimals)?;
        res.spread_fee = quote_asset_amount
            * config.spread_ratio;
            // .checked_div(config.decimals)?;
    }

    Ok(res)
}

/// Calculates the TWAP of the AMM reserves
fn calc_reserve_twap(deps: Deps, env: Env, interval: u64) -> StdResult<Decimal256> {
    // let config: Config = read_config(deps.storage)?;
    let mut counter = read_reserve_snapshot_counter(deps.storage).unwrap();
    let current_snapshot = read_reserve_snapshot(deps.storage, counter);
    let mut current_snapshot = current_snapshot.unwrap();

    let mut current_price = current_snapshot
        .quote_asset_reserve
        / current_snapshot.base_asset_reserve;
        // .checked_mul(config.decimals)?
    if interval == 0 {
        return Ok(current_price);
    }

    let base_timestamp = env.block.time.seconds().checked_sub(interval).unwrap();

    if counter == 1 || current_snapshot.timestamp.seconds() <= base_timestamp {
        return Ok(current_price);
    }

    let mut previous_timestamp = current_snapshot.timestamp.seconds();
    let mut period = Decimal256::from_uint256(Uint256::from(
        env.block
            .time
            .seconds()
            .checked_sub(previous_timestamp)
            .unwrap(),
    ));
    // let mut weighted_price = current_price.checked_mul(period)?;
    let mut weighted_price = current_price * period;

    loop {
        counter -= 1;
        // if snapshot history is too short
        if counter == 0 {
            return Ok(weighted_price / period);
        }
        current_snapshot = read_reserve_snapshot(deps.storage, counter).unwrap();
        current_price = current_snapshot
            .quote_asset_reserve
            / current_snapshot.base_asset_reserve;
            // .checked_mul(config.decimals)?

        if current_snapshot.timestamp.seconds() <= base_timestamp {
            let delta_timestamp =
                previous_timestamp - base_timestamp;

            weighted_price = weighted_price + current_price * Decimal256::from_uint256(Uint256::from(delta_timestamp));
            break;
        }

        let delta_timestamp = Decimal256::from_uint256(Uint256::from(
            previous_timestamp
                .checked_sub(current_snapshot.timestamp.seconds())
                .unwrap(),
        ));
        weighted_price = weighted_price + (current_price * delta_timestamp);

        period = period + delta_timestamp;
        previous_timestamp = current_snapshot.timestamp.seconds();
    }

    Ok(weighted_price / Decimal256::from_uint256(Uint256::from(interval)))
}
