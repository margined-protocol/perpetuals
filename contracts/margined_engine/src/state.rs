use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Api, DepsMut, StdResult, Storage, Uint128};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, Singleton,
};
use cw_storage_plus::Item;

use margined_common::integer::Integer;
use margined_perp::margined_engine::Side;
use margined_perp::margined_vamm::Direction;

use sha3::{Digest, Sha3_256};
use terraswap::asset::AssetInfo;

pub static KEY_CONFIG: &[u8] = b"config";
pub static KEY_POSITION: &[u8] = b"position";
pub static KEY_STATE: &[u8] = b"state";
pub static KEY_TMP_SWAP: &[u8] = b"tmp-swap";
pub static KEY_TMP_LIQUIDATOR: &[u8] = b"tmp-liquidator";
pub static KEY_VAMM_MAP: &[u8] = b"vamm-map";
pub const VAMM_LIST: Item<VammList> = Item::new("vamm-list");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub insurance_fund: Addr,
    pub fee_pool: Addr,
    pub eligible_collateral: AssetInfo,
    pub decimals: Uint128,
    pub initial_margin_ratio: Uint128,
    pub maintenance_margin_ratio: Uint128,
    pub partial_liquidation_margin_ratio: Uint128,
    pub liquidation_fee: Uint128,
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, KEY_CONFIG).load()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub open_interest_notional: Uint128,
    pub bad_debt: Uint128,
    pub pause: bool,
}

pub fn store_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    singleton(storage, KEY_STATE).save(state)
}

pub fn read_state(storage: &dyn Storage) -> StdResult<State> {
    singleton_read(storage, KEY_STATE).load()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VammList {
    pub vamm: Vec<Addr>,
}

impl VammList {
    /// returns true if the address is a registered vamm
    pub fn is_vamm(&self, addr: &str) -> bool {
        self.vamm.iter().any(|a| a.as_ref() == addr)
    }
}

pub fn store_vamm(deps: DepsMut, input: &[String]) -> StdResult<()> {
    let cfg = VammList {
        vamm: map_validate(deps.api, input)?,
    };
    VAMM_LIST.save(deps.storage, &cfg)
}

pub fn read_vamm(storage: &dyn Storage) -> StdResult<VammList> {
    VAMM_LIST.load(storage)
}

pub fn map_validate(api: &dyn Api, input: &[String]) -> StdResult<Vec<Addr>> {
    input.iter().map(|addr| api.addr_validate(addr)).collect()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Position {
    pub vamm: Addr,
    pub trader: Addr,
    pub direction: Direction,
    pub size: Integer,
    pub margin: Uint128,
    pub notional: Uint128,
    pub last_updated_premium_fraction: Integer,
    pub liquidity_history_index: Uint128,
    pub block_number: u64,
}

impl Default for Position {
    fn default() -> Position {
        Position {
            vamm: Addr::unchecked(""),
            trader: Addr::unchecked(""),
            direction: Direction::AddToAmm,
            size: Integer::zero(),
            margin: Uint128::zero(),
            notional: Uint128::zero(),
            last_updated_premium_fraction: Integer::zero(),
            liquidity_history_index: Uint128::zero(),
            block_number: 0u64,
        }
    }
}

fn position_bucket(storage: &mut dyn Storage) -> Bucket<Position> {
    bucket(storage, KEY_POSITION)
}

fn position_bucket_read(storage: &dyn Storage) -> ReadonlyBucket<Position> {
    bucket_read(storage, KEY_POSITION)
}

pub fn store_position(storage: &mut dyn Storage, position: &Position) -> StdResult<()> {
    // hash the vAMM and trader together to get a unique position key
    let mut hasher = Sha3_256::new();

    // write input message
    hasher.update(position.vamm.as_bytes());
    hasher.update(position.trader.as_bytes());

    // read hash digest
    let hash = hasher.finalize();

    position_bucket(storage).save(&hash, position)
}

pub fn read_position(storage: &dyn Storage, vamm: &Addr, trader: &Addr) -> StdResult<Position> {
    // hash the vAMM and trader together to get a unique position key
    let mut hasher = Sha3_256::new();

    // write input message
    hasher.update(vamm.as_bytes());
    hasher.update(trader.as_bytes());

    // read hash digest
    let hash = hasher.finalize();
    let result = position_bucket_read(storage)
        .may_load(&hash)?
        .unwrap_or_default();

    Ok(result)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Swap {
    pub vamm: Addr,
    pub trader: Addr,
    pub side: Side,
    pub quote_asset_amount: Uint128,
    pub leverage: Uint128,
    pub open_notional: Uint128,
    pub position_notional: Uint128,
    pub unrealized_pnl: Integer,
    pub margin_to_vault: Integer,
}

pub fn store_tmp_swap(storage: &mut dyn Storage, swap: &Swap) -> StdResult<()> {
    singleton(storage, KEY_TMP_SWAP).save(swap)
}

pub fn remove_tmp_swap(storage: &mut dyn Storage) {
    let mut store: Singleton<Swap> = singleton(storage, KEY_TMP_SWAP);
    store.remove()
}

pub fn read_tmp_swap(storage: &dyn Storage) -> StdResult<Option<Swap>> {
    singleton_read(storage, KEY_TMP_SWAP).may_load()
}

pub fn store_tmp_liquidator(storage: &mut dyn Storage, liquidator: &Addr) -> StdResult<()> {
    singleton(storage, KEY_TMP_LIQUIDATOR).save(liquidator)
}

pub fn remove_tmp_liquidator(storage: &mut dyn Storage) {
    let mut store: Singleton<Addr> = singleton(storage, KEY_TMP_LIQUIDATOR);
    store.remove()
}

pub fn read_tmp_liquidator(storage: &dyn Storage) -> StdResult<Option<Addr>> {
    singleton_read(storage, KEY_TMP_LIQUIDATOR).may_load()
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, JsonSchema)]
pub struct VammMap {
    pub last_restriction_block: u64,
    pub cumulative_premium_fractions: Vec<Integer>,
}

fn vamm_map_bucket(storage: &mut dyn Storage) -> Bucket<VammMap> {
    bucket(storage, KEY_VAMM_MAP)
}

fn vamm_map_bucket_read(storage: &dyn Storage) -> ReadonlyBucket<VammMap> {
    bucket_read(storage, KEY_VAMM_MAP)
}

pub fn store_vamm_map(storage: &mut dyn Storage, vamm: Addr, vamm_map: &VammMap) -> StdResult<()> {
    vamm_map_bucket(storage).save(vamm.as_bytes(), vamm_map)
}

pub fn read_vamm_map(storage: &dyn Storage, vamm: Addr) -> StdResult<VammMap> {
    let result = vamm_map_bucket_read(storage)
        .may_load(vamm.as_bytes())?
        .unwrap_or_default();

    Ok(result)
}

/// Accumulates the premium fractions at each settlement payment so that eventually users take
/// their P&L
pub fn append_cumulative_premium_fraction(
    storage: &mut dyn Storage,
    vamm: Addr,
    premium_fraction: Integer,
) -> StdResult<()> {
    let mut vamm_map = read_vamm_map(storage, vamm.clone())?;

    // we push the first premium fraction to an empty array
    // else we add them together prior to pushing
    match vamm_map.cumulative_premium_fractions.len() {
        0 => vamm_map.cumulative_premium_fractions.push(premium_fraction),
        n => {
            let current_premium_fraction = vamm_map.cumulative_premium_fractions[n - 1];
            let latest_premium_fraction = premium_fraction + current_premium_fraction;
            vamm_map
                .cumulative_premium_fractions
                .push(latest_premium_fraction)
        }
    }

    store_vamm_map(storage, vamm, &vamm_map)
}

pub fn enter_restriction_mode(
    storage: &mut dyn Storage,
    vamm: Addr,
    block_height: u64,
) -> StdResult<()> {
    let mut vamm_map = read_vamm_map(storage, vamm.clone())?;

    vamm_map.last_restriction_block = block_height;

    store_vamm_map(storage, vamm, &vamm_map)
}
