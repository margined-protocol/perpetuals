use cosmwasm_schema::cw_serde;
use cosmwasm_std::{from_slice, to_vec, Addr, StdError, StdResult, Storage, Uint128};
use std::cmp::Ordering;

use margined_common::{asset::Asset, integer::Integer};
use margined_perp::margined_engine::{ConfigResponse, Position, Side};

pub static KEY_CONFIG: &[u8] = b"config";
pub static KEY_POSITION: &[u8] = b"position";
pub static KEY_STATE: &[u8] = b"state";
pub static KEY_SENT_FUNDS: &[u8] = b"sent-funds";
pub static KEY_TMP_SWAP: &[u8] = b"tmp-swap";
pub static KEY_TMP_LIQUIDATOR: &[u8] = b"tmp-liquidator";
pub static KEY_VAMM_MAP: &[u8] = b"vamm-map";

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

#[cw_serde]
pub struct State {
    pub open_interest_notional: Uint128,
    pub prepaid_bad_debt: Uint128,
    pub pause: bool,
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

pub fn store_position(storage: &mut dyn Storage, key: &[u8], position: &Position) -> StdResult<()> {
    // hash the vAMM and trader together to get a unique position key
    Ok(storage.set(&[KEY_POSITION, key].concat(), &to_vec(position)?))
}

pub fn remove_position(storage: &mut dyn Storage, key: &[u8]) {
    // hash the vAMM and trader together to get a unique position key
    storage.remove(&[KEY_POSITION, key].concat())
}

pub fn read_position(storage: &dyn Storage, key: &[u8]) -> StdResult<Position> {
    // hash the vAMM and trader together to get a unique position key
    match storage.get(&[KEY_POSITION, key].concat()) {
        Some(data) => from_slice(&data),
        None => Ok(Position::default()),
    }
}

/// Used to monitor that transferred native tokens are sufficient when opening a
/// new position or relevant operations
#[cw_serde]
pub struct SentFunds {
    pub asset: Asset,
    pub required: Uint128,
}

impl SentFunds {
    /// throws an error if the required funds is less than the asset amount
    pub fn are_sufficient(&self) -> StdResult<()> {
        // this should only pass if asset.amount == required
        match self.asset.amount.cmp(&self.required) {
            Ordering::Greater => Err(StdError::generic_err("sent funds are excessive")),
            Ordering::Less => Err(StdError::generic_err("sent funds are insufficient")),
            _ => Ok(()),
        }
    }
}

pub fn store_sent_funds(storage: &mut dyn Storage, funds: &SentFunds) -> StdResult<()> {
    Ok(storage.set(KEY_SENT_FUNDS, &to_vec(funds)?))
}

pub fn remove_sent_funds(storage: &mut dyn Storage) {
    storage.remove(KEY_SENT_FUNDS)
}

pub fn read_sent_funds(storage: &dyn Storage) -> StdResult<SentFunds> {
    match storage.get(KEY_SENT_FUNDS) {
        Some(data) => from_slice(&data),
        None => Err(StdError::generic_err("No sent funds are stored")),
    }
}

#[cw_serde]
pub struct TmpSwapInfo {
    pub vamm: Addr,
    pub trader: Addr,
    pub side: Side,                 // buy or sell
    pub margin_amount: Uint128,     // amount of quote asset being supplied
    pub leverage: Uint128,          // leverage of new position
    pub open_notional: Uint128,     // notional of position being opened
    pub position_notional: Uint128, // notional of existing position, inclusing funding
    pub unrealized_pnl: Integer,    // any pnl due
    pub margin_to_vault: Integer,   // margin to be sent to vault
    pub fees_paid: bool, // true if fees have been paid, used in case of reversing position
}

pub fn store_tmp_swap(storage: &mut dyn Storage, swap: &TmpSwapInfo) -> StdResult<()> {
    Ok(storage.set(KEY_TMP_SWAP, &to_vec(swap)?))
}

pub fn remove_tmp_swap(storage: &mut dyn Storage) {
    storage.remove(KEY_TMP_SWAP)
}

pub fn read_tmp_swap(storage: &dyn Storage) -> StdResult<TmpSwapInfo> {
    match storage.get(KEY_TMP_SWAP) {
        Some(data) => from_slice(&data),
        None => Err(StdError::generic_err("Temporary Swap not found")),
    }
}

pub fn store_tmp_liquidator(storage: &mut dyn Storage, liquidator: &Addr) -> StdResult<()> {
    Ok(storage.set(KEY_TMP_LIQUIDATOR, &to_vec(liquidator)?))
}

pub fn remove_tmp_liquidator(storage: &mut dyn Storage) {
    storage.remove(KEY_TMP_LIQUIDATOR)
}

pub fn read_tmp_liquidator(storage: &dyn Storage) -> StdResult<Addr> {
    match storage.get(KEY_TMP_LIQUIDATOR) {
        Some(data) => from_slice(&data),
        None => Err(StdError::generic_err("Addr not found")),
    }
}

#[cw_serde]
#[derive(Default)]
pub struct VammMap {
    pub last_restriction_block: u64,
    pub cumulative_premium_fractions: Vec<Integer>,
}

pub fn store_vamm_map(storage: &mut dyn Storage, vamm: Addr, vamm_map: &VammMap) -> StdResult<()> {
    Ok(storage.set(
        &[KEY_VAMM_MAP, vamm.as_bytes()].concat(),
        &to_vec(vamm_map)?,
    ))
}

pub fn read_vamm_map(storage: &dyn Storage, vamm: &Addr) -> StdResult<VammMap> {
    match storage.get(&[KEY_VAMM_MAP, vamm.as_bytes()].concat()) {
        Some(data) => from_slice(&data),
        None => Ok(VammMap::default()),
    }
}

/// Accumulates the premium fractions at each settlement payment so that eventually users take
/// their P&L
pub fn append_cumulative_premium_fraction(
    storage: &mut dyn Storage,
    vamm: Addr,
    premium_fraction: Integer,
) -> StdResult<()> {
    let mut vamm_map = read_vamm_map(storage, &vamm)?;

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
    let mut vamm_map = read_vamm_map(storage, &vamm)?;

    vamm_map.last_restriction_block = block_height;

    store_vamm_map(storage, vamm, &vamm_map)
}
