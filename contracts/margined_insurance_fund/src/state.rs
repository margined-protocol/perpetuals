use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Api, DepsMut, StdError, StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read};
use cw_storage_plus::Item; //, Map

pub static KEY_CONFIG: &[u8] = b"config";
pub const VAMM_LIST: Item<VammList> = Item::new("vamm-list");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VammList {
    pub vamms: Vec<Addr>,
}

/*
impl VammList {
    /// returns true if the address is a registered vamm
    pub fn is_vamm(&self, addr: &str) -> bool {
        self.vamms.iter().any(|a| a.as_ref() == addr)
    }
}
*/

// function saves a given Addr by either pushing it into the existing Vec or instantiating a new Vec

pub fn save_vamm(deps: DepsMut, input: Addr) -> StdResult<()> {
    // match because the data might not exist yet
    let amm_list = match VAMM_LIST.may_load(deps.storage)? {
        Some(mut loaded_list) => {
            loaded_list.vamms.push(input);
            loaded_list
        }
        None => VammList { vamms: vec![input] },
    };
    VAMM_LIST.save(deps.storage, &amm_list)
}

// function removes a given Addr - one issue is pulling the index of the Addr

pub fn remove_vamm(deps: DepsMut, input: Addr) -> StdResult<()> {
    // first check that there is data there
    let mut amm_list = VAMM_LIST.load(deps.storage)?;

    // find the index (possible that the data isn't in the vec) and swap_remove it
    let index = match amm_list.vamms.iter().position(|x| x.eq(&input)) {
        Some(value) => value,
        None => {
            return Err(StdError::NotFound {
                kind: "AMM".to_string(),
            })
        }
    };
    amm_list.vamms.swap_remove(index);

    VAMM_LIST.save(deps.storage, &amm_list)
}

pub fn read_vamm(storage: &dyn Storage) -> StdResult<VammList> {
    VAMM_LIST.load(storage)
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
