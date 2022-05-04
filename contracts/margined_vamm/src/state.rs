use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Order, StdResult, Storage, Timestamp, Uint128};
use cosmwasm_storage::{bucket, bucket_read, singleton, singleton_read, ReadonlyBucket};

use margined_common::integer::Integer;

pub static KEY_CONFIG: &[u8] = b"config";
pub static KEY_STATE: &[u8] = b"state";
pub static KEY_RESERVE_SNAPSHOT: &[u8] = b"reserve_snapshot";
pub static KEY_RESERVE_SNAPSHOT_COUNTER: &[u8] = b"reserve_snapshot_counter";

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub margin_engine: Addr,
    pub pricefeed: Addr,
    pub quote_asset: String,
    pub base_asset: String,
    pub base_asset_holding_cap: Uint128,
    pub open_interest_notional_cap: Uint128,
    pub decimals: Uint128,
    pub toll_ratio: Uint128,
    pub spread_ratio: Uint128,
    pub fluctuation_limit_ratio: Uint128,
    pub spot_price_twap_interval: u64,
    pub funding_period: u64,
    pub funding_buffer_period: u64,
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, KEY_CONFIG).load()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub open: bool,
    pub quote_asset_reserve: Uint128,
    pub base_asset_reserve: Uint128,
    pub total_position_size: Integer,
    pub funding_rate: Uint128,
    pub next_funding_time: u64,
}

pub fn store_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    singleton(storage, KEY_STATE).save(state)
}

pub fn read_state(storage: &dyn Storage) -> StdResult<State> {
    singleton_read(storage, KEY_STATE).load()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct ReserveSnapshot {
    pub quote_asset_reserve: Uint128,
    pub base_asset_reserve: Uint128,
    pub timestamp: Timestamp,
    pub block_height: u64,
}

pub fn read_reserve_snapshot(storage: &dyn Storage, height: u64) -> StdResult<ReserveSnapshot> {
    bucket_read(storage, KEY_RESERVE_SNAPSHOT).load(&height.to_be_bytes())
}

/// reads multiple reserve snapshots and returns an array
pub fn read_reserve_snapshots(
    storage: &dyn Storage,
    start: Option<u64>,
    limit: Option<u32>,
) -> StdResult<Vec<ReserveSnapshot>> {
    let limit: usize = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let snapshots: ReadonlyBucket<ReserveSnapshot> =
        ReadonlyBucket::new(storage, KEY_RESERVE_SNAPSHOT);

    snapshots
        .range(calc_range_start(start).as_deref(), None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (_, v) = item?;
            Ok(v)
        })
        .collect()
}

pub fn store_reserve_snapshot(
    storage: &mut dyn Storage,
    reserve_snapshot: &ReserveSnapshot,
) -> StdResult<()> {
    increment_reserve_snapshot_counter(storage)?;

    let height = read_reserve_snapshot_counter(storage)?;

    bucket(storage, KEY_RESERVE_SNAPSHOT).save(&height.to_be_bytes(), reserve_snapshot)?;

    Ok(())
}

pub fn update_reserve_snapshot(
    storage: &mut dyn Storage,
    reserve_snapshot: &ReserveSnapshot,
) -> StdResult<()> {
    let height = read_reserve_snapshot_counter(storage)?;

    bucket(storage, KEY_RESERVE_SNAPSHOT).save(&height.to_be_bytes(), reserve_snapshot)?;

    Ok(())
}

pub fn read_reserve_snapshot_counter(storage: &dyn Storage) -> StdResult<u64> {
    Ok(singleton_read(storage, KEY_RESERVE_SNAPSHOT_COUNTER)
        .may_load()?
        .unwrap_or_default())
}

pub fn increment_reserve_snapshot_counter(storage: &mut dyn Storage) -> StdResult<()> {
    let val = read_reserve_snapshot_counter(storage)? + 1;

    singleton(storage, KEY_RESERVE_SNAPSHOT_COUNTER).save(&val)
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_start(start_after: Option<u64>) -> Option<Vec<u8>> {
    start_after.map(|id| {
        let mut v = id.to_be_bytes().to_vec();
        v.push(1);
        v
    })
}
