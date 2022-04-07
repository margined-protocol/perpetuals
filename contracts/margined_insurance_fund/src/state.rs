use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, StdResult, Storage, DepsMut, Api};
use cosmwasm_storage::{singleton, singleton_read};
use cw_storage_plus::Item;

pub static KEY_CONFIG: &[u8] = b"config";
pub const VAMM_LIST: Item<VammList> = Item::new("vamm-list");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VammList {
    pub vamms: Vec<Addr>,
}

impl VammList {
    /// returns true if the address is a registered vamm
    pub fn is_vamm(&self, addr: &str) -> bool {
        self.vamms.iter().any(|a| a.as_ref() == addr)
    }
}

pub fn save_vamm(deps: DepsMut, input: Addr) -> StdResult<()> {
    let mut amm_list = VAMM_LIST.load(deps.storage)?;
    amm_list.vamms.push(input);

    VAMM_LIST.save(deps.storage, &amm_list)
}

pub fn read_vamm(storage: &dyn Storage) -> StdResult<VammList> {
    VAMM_LIST.load(storage)
}

pub fn map_validate(api: &dyn Api, input: &[String]) -> StdResult<Vec<Addr>> {
    input.iter().map(|addr| api.addr_validate(addr)).collect()
}

pub fn store_vamm(deps: DepsMut, input: &[String]) -> StdResult<()> {
    let cfg = VammList {
        vamms: map_validate(deps.api, input)?,
    };
    VAMM_LIST.save(deps.storage, &cfg)
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, KEY_CONFIG).load()
}