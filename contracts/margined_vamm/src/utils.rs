use cosmwasm_std::{Addr, Env, Response, StdError, StdResult, Storage, Uint128};
use margined_perp::margined_vamm::Direction;

use crate::state::{
    read_config, read_reserve_snapshot, read_reserve_snapshot_counter, read_state,
    store_reserve_snapshot, update_reserve_snapshot, ReserveSnapshot,
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

/// Does the modulus (%) operator on Uint128.
/// However it follows the design of the perpetual protocol decimals
/// https://github.com/perpetual-protocol/perpetual-protocol/blob/release/v2.1.x/src/utils/Decimal.sol
pub(crate) fn modulo(a: Uint128, b: Uint128, decimals: Uint128) -> Uint128 {
    let a_decimals = a.checked_mul(decimals).unwrap();
    let integral = a_decimals / b;
    a_decimals - (b * integral)
}
