use cosmwasm_std::{Addr, Env, Response, StdError, StdResult, Storage, Uint128};

use crate::state::{
    read_reserve_snapshot, read_reserve_snapshot_counter, store_reserve_snapshot,
    update_reserve_snapshot, ReserveSnapshot,
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
