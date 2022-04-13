use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, DepsMut, StdError, StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read};
use cw_storage_plus::Map;

pub static KEY_CONFIG: &[u8] = b"config";
pub const VAMM_LIST: Map<Addr, bool> = Map::new("vamm-list");

// function checks if an addr is already added and adds it if not
pub fn save_vamm(deps: DepsMut, input: Addr) -> StdResult<()> {
    // we match because the data might not exist yet
    // In the case there is data, we force an error
    // In the case there is not data, we add the Addr and set its bool to true
    match VAMM_LIST.may_load(deps.storage, input.clone())? {
        Some(_is_vamm) => {
            return Err(StdError::GenericErr {
                msg: "This vAMM is already added".to_string(),
            })
        }
        None => {}
    };
    VAMM_LIST.save(deps.storage, input, &true)
}

// this function checks whether the vamm is stored
pub fn is_vamm(storage: &dyn Storage, input: Addr) -> bool {
    VAMM_LIST.has(storage, input)
}

// this function deletes the entry under the given key
pub fn delist_vamm(deps: DepsMut, input: Addr) -> StdResult<()> {
    // first check that there is data there
    match VAMM_LIST.may_load(deps.storage, input.clone())? {
        Some(_is_vamm) => {}
        None => {
            return Err(StdError::GenericErr {
                msg: "This vAMM has not been added".to_string(),
            })
        }
    };
    // removes the entry from the Map
    Ok(VAMM_LIST.remove(deps.storage, input))
}

// function changes the bool stored under an address to 'false'
// note that that means this can only be given an *existing* vamm
pub fn vamm_off(deps: DepsMut, input: Addr) -> StdResult<()> {
    // first check that there is data there
    match VAMM_LIST.may_load(deps.storage, input.clone())? {
        Some(_is_vamm) => {}
        None => {
            return Err(StdError::GenericErr {
                msg: "This vAMM has not been added".to_string(),
            })
        }
    };
    VAMM_LIST.save(deps.storage, input, &false)
}

// this function reads the bool stored under an addr, and if there is no addr stored there then throws an error
// use this function when you want to check the 'on/off' status of a vAmm
pub fn read_vamm(storage: &dyn Storage, input: Addr) -> StdResult<bool> {
    VAMM_LIST
        .load(storage, input)
        .map_err(|_e| StdError::GenericErr {
            msg: "No vAMM stored".to_string(),
        })
}

/*
pub fn map_validate(api: &dyn Api, input: &[String]) -> StdResult<Vec<Addr>> {
    input.iter().map(|addr| api.addr_validate(addr)).collect()
}

pub fn store_vamm(deps: DepsMut, input: &[String]) -> StdResult<()> {
    let cfg = VammList {
        vamms: map_validate(deps.api, input)?,
    };
    VAMM_LIST.save(deps.storage, &cfg)
}
*/
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
