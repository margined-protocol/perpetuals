use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{Addr, StdResult, Storage, Uint128};
use cosmwasm_storage::{singleton, singleton_read};
use cw_storage_plus::Item;

pub static KEY_CONFIG: &[u8] = b"config";
pub static KEY_STATE: &[u8] = b"state";

pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub decimals: u8,
    pub quote_asset: String,
    pub base_asset: String,
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, KEY_CONFIG).load()
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub quote_asset_reserve: Uint256,
    pub base_asset_reserve: Uint256,
    pub funding_rate: Uint256,
    pub funding_period: Uint128,
}

pub fn store_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    singleton(storage, KEY_STATE).save(state)
}

pub fn read_state(storage: &dyn Storage) -> StdResult<State> {
    println!("ALALALA");
    singleton_read(storage, KEY_STATE).load()
}
