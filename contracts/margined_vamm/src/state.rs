use cosmwasm_schema::cw_serde;
use cosmwasm_std::{from_slice, to_vec, StdError, StdResult, Storage, Timestamp, Uint128};

use margined_perp::margined_vamm::{ConfigResponse, StateResponse};

pub static KEY_CONFIG: &[u8] = b"config";
pub static KEY_STATE: &[u8] = b"state";
pub static KEY_RESERVE_SNAPSHOT: &[u8] = b"reserve_snapshot";
pub static KEY_RESERVE_SNAPSHOT_COUNTER: &[u8] = b"reserve_snapshot_counter";

// Has the same fields
pub type State = StateResponse;

pub type Config = ConfigResponse;

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    Ok(storage.set(KEY_CONFIG, &to_vec(config)?))
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    match storage.get(KEY_CONFIG) {
        Some(data) => from_slice(&data),
        None => Err(StdError::generic_err("Config not found")),
    }
}

pub fn store_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    Ok(storage.set(KEY_STATE, &to_vec(state)?))
}

pub fn read_state(storage: &dyn Storage) -> StdResult<State> {
    match storage.get(KEY_STATE) {
        Some(data) => from_slice(&data),
        None => Err(StdError::generic_err("State not found")),
    }
}

#[cw_serde]
pub struct ReserveSnapshot {
    pub quote_asset_reserve: Uint128,
    pub base_asset_reserve: Uint128,
    pub timestamp: Timestamp,
    pub block_height: u64,
}

pub fn read_reserve_snapshot(storage: &dyn Storage, height: u64) -> StdResult<ReserveSnapshot> {
    match storage.get(&[KEY_RESERVE_SNAPSHOT, &height.to_be_bytes()].concat()) {
        Some(data) => from_slice(&data),
        None => Err(StdError::generic_err("Reserve snapshot not found")),
    }
}

/// Stores a new reserve snapshot
pub fn store_reserve_snapshot(
    storage: &mut dyn Storage,
    reserve_snapshot: &ReserveSnapshot,
) -> StdResult<()> {
    increment_reserve_snapshot_counter(storage)?;

    update_current_reserve_snapshot(storage, reserve_snapshot)
}

/// Updates the current reserve snapshot
pub fn update_current_reserve_snapshot(
    storage: &mut dyn Storage,
    reserve_snapshot: &ReserveSnapshot,
) -> StdResult<()> {
    let height = read_reserve_snapshot_counter(storage)?;

    Ok(storage.set(
        &[KEY_RESERVE_SNAPSHOT, &height.to_be_bytes()].concat(),
        &to_vec(reserve_snapshot)?,
    ))
}

pub fn read_reserve_snapshot_counter(storage: &dyn Storage) -> StdResult<u64> {
    Ok(match storage.get(KEY_RESERVE_SNAPSHOT_COUNTER) {
        Some(data) => from_slice(&data)?,
        None => 0,
    })
}

pub fn increment_reserve_snapshot_counter(storage: &mut dyn Storage) -> StdResult<()> {
    let val = read_reserve_snapshot_counter(storage)? + 1;

    Ok(storage.set(KEY_RESERVE_SNAPSHOT_COUNTER, &to_vec(&val)?))
}
