use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Deps, DepsMut, StdError::GenericErr, StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read};
use margined_common::asset::AssetInfo;

pub static KEY_CONFIG: &[u8] = b"config";
pub const TOKEN_LIST: &[u8] = b"token-list";
pub const TOKEN_LIMIT: usize = 3usize;

#[cw_serde]
pub struct Config {}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

// function checks if an addr is already added and adds it if not
// We also check that we have not reached the limit of tokens here
pub fn save_token(deps: DepsMut, input: AssetInfo) -> StdResult<()> {
    // check if the list exists already
    let mut token_list: Vec<AssetInfo> = singleton_read(deps.storage, TOKEN_LIST)
        .may_load()?
        .unwrap_or_default();

    // check if we already added the token
    if token_list.contains(&input) {
        return Err(GenericErr {
            msg: "This token is already added".to_string(),
        });
    };

    // check if we have reached the capacity
    if token_list.len() >= TOKEN_LIMIT {
        return Err(GenericErr {
            msg: "The token capacity is already reached".to_string(),
        });
    };

    // add the token
    token_list.push(input);

    singleton(deps.storage, TOKEN_LIST).save(&token_list)
}

// this function reads Addrs stored in the TOKEN_LIST.
// note that this function ONLY takes the first TOKEN_LIMIT terms
pub fn read_token_list(deps: Deps, limit: usize) -> StdResult<Vec<AssetInfo>> {
    match singleton_read::<Vec<AssetInfo>>(deps.storage, TOKEN_LIST).may_load()? {
        None => Err(GenericErr {
            msg: "No tokens are stored".to_string(),
        }),
        Some(list) => {
            let take = limit.min(list.len());
            Ok(list[..take].to_vec())
        }
    }
}

// this function checks whether the token is stored already
pub fn is_token(storage: &dyn Storage, token: AssetInfo) -> bool {
    if let Ok(list) = singleton_read::<Vec<AssetInfo>>(storage, TOKEN_LIST).load() {
        return list.contains(&token);
    }
    false
}

// this function deletes the entry under the given key
pub fn remove_token(deps: DepsMut, token: AssetInfo) -> StdResult<()> {
    // check if the list exists
    let mut token_list: Vec<AssetInfo> =
        match singleton_read(deps.storage, TOKEN_LIST).may_load()? {
            None => {
                return Err(GenericErr {
                    msg: "No tokens are stored".to_string(),
                })
            }
            Some(value) => value,
        };

    // change token_list
    if let Some(index) = token_list.clone().iter().position(|x| x.eq(&token)) {
        token_list.swap_remove(index);
    } else {
        return Err(GenericErr {
            msg: "This token has not been added".to_string(),
        });
    }

    // saves the updated token_list
    singleton(deps.storage, TOKEN_LIST).save(&token_list)
}
