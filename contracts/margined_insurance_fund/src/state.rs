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

// this function reads Addrs stored in the VAMM_LIST TODO: case there is no info in VAMM_LIST
pub fn read_vammlist(deps: Deps, storage: &dyn Storage) -> StdResult<Vec<Addr>> {
    let keys = VAMM_LIST
        .keys(storage, None, None, Order::Ascending)
        .map(|x| deps.api.addr_validate(&String::from_utf8(x)?))
        .collect();
    keys
}

// function changes the bool stored under an address to 'false'
// note that that means this can only be given an *existing* vamm
pub fn vamm_off(deps: DepsMut, input: Addr) -> StdResult<()> {
    // first check that there is data there
    if VAMM_LIST.may_load(deps.storage, &input)?.is_none() {
        return Err(StdError::GenericErr {
            msg: "This vAMM has not been added".to_string(),
        });
    };
    VAMM_LIST.save(deps.storage, &input, &false)
}

// function changes the bool stored under an address to 'true'
// note that that means this can only be given an *existing* vamm
pub fn vamm_on(deps: DepsMut, input: Addr) -> StdResult<()> {
    // first check that there is data there
    if VAMM_LIST.may_load(deps.storage, &input)?.is_none() {
        return Err(StdError::GenericErr {
            msg: "This vAMM has not been added".to_string(),
        });
    };
    VAMM_LIST.save(deps.storage, &input, &true)
}

// this function reads the bool stored under an addr, and if there is no addr stored there then throws an error
// use this function when you want to check the 'on/off' status of a vAMM
pub fn read_vamm_status(storage: &dyn Storage, input: Addr) -> StdResult<bool> {
    VAMM_LIST
        .load(storage, &input)
        .map_err(|_| StdError::GenericErr {
            msg: "No vAMM stored".to_string(),
        })
}

// this function reads the bools stored in the Map, and if there are no vAMMs stored, returns empty vec
// use this function when you want to check the 'on/off' status of a vAMM
pub fn read_all_vamm_status(storage: &dyn Storage) -> StdResult<Vec<(Addr, bool)>> {
    let status_vec = VAMM_LIST
        .range(storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(Vec<u8>, bool)>>>()?
        .iter()
        .map(|tup| {
            (
                Addr::unchecked(&String::from_utf8(tup.0.clone()).unwrap()),
                tup.1,
            )
        })
        .collect();

    // This takes the Map in storage, loads the key-value pairs but the keys are still UTF-8 encoded
    // We collect into a StdResult<Vec> of key-value pairs but the keys are still Vec<u8>
    // Unwrap the Result and then transform back to an iterator so we can map (Vec<u8>, bool) -> (Addr, bool)
    // finally re-collect into a vec

    Ok(status_vec)
}

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
    pub beneficiary: Addr,
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, KEY_CONFIG).load()
}
