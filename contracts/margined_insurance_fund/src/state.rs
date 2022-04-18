use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Deps, DepsMut, Order, StdError, StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read};
use cw_storage_plus::Map;

pub static KEY_CONFIG: &[u8] = b"config";
pub const VAMM_LIST: Map<&Addr, bool> = Map::new("vamm-list");

// function checks if an addr is already added and adds it if not
pub fn save_vamm(deps: DepsMut, input: Addr) -> StdResult<()> {
    // we match because the data might not exist yet
    // In the case there is data, we force an error
    // In the case there is not data, we add the Addr and set its bool to true
    if VAMM_LIST.may_load(deps.storage, &input.clone())?.is_some() {
        return Err(StdError::GenericErr {
            msg: "This vAMM is already added".to_string(),
        });
    };
    VAMM_LIST.save(deps.storage, &input, &true)
}

// this function reads Addrs stored in the VAMM_LIST (hopefully)...
pub fn read_vammlist(deps: Deps, storage: &dyn Storage) -> StdResult<Vec<Addr>> {
    let keys = VAMM_LIST
        .keys(storage, None, None, Order::Ascending)
        .map(|x| deps.api.addr_validate(&String::from_utf8(x)?))
        .collect();
    keys
}
//Addr::unchecked
// this function checks whether the vamm is stored
pub fn is_vamm(storage: &dyn Storage, input: Addr) -> bool {
    VAMM_LIST.has(storage, &input)
}

// this function deletes the entry under the given key
pub fn remove_vamm(deps: DepsMut, input: Addr) -> StdResult<()> {
    // first check that there is data there
    if VAMM_LIST.may_load(deps.storage, &input.clone())?.is_none() {
        return Err(StdError::GenericErr {
            msg: "This vAMM has not been added".to_string(),
        });
    };

    // removes the entry from the Map
    VAMM_LIST.remove(deps.storage, &input);
    Ok(())
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
