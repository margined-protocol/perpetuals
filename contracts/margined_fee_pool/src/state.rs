use cosmwasm_std::{from_slice, to_vec, StdError, StdResult, Storage};
use margined_common::asset::AssetInfo;
use margined_perp::margined_fee_pool::ConfigResponse;

pub static KEY_CONFIG: &[u8] = b"config";
pub const TOKEN_LIST: &[u8] = b"token-list";
pub const TOKEN_LIMIT: usize = 3usize;

pub type Config = ConfigResponse;

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    Ok(storage.set(KEY_CONFIG, &to_vec(config)?))
}

// function checks if an addr is already added and adds it if not
// We also check that we have not reached the limit of tokens here
pub fn save_token(storage: &mut dyn Storage, input: AssetInfo) -> StdResult<()> {
    // check if the list exists already
    let mut token_list: Vec<AssetInfo> = match storage.get(TOKEN_LIST) {
        None => vec![],
        Some(data) => from_slice(&data)?,
    };

    // check if we already added the token
    if token_list.contains(&input) {
        return Err(StdError::generic_err("This token is already added"));
    };

    // check if we have reached the capacity
    if token_list.len() >= TOKEN_LIMIT {
        return Err(StdError::generic_err(
            "The token capacity is already reached",
        ));
    };

    // add the token
    token_list.push(input);

    Ok(storage.set(TOKEN_LIST, &to_vec(&token_list)?))
}

// this function reads Addrs stored in the TOKEN_LIST.
// note that this function ONLY takes the first TOKEN_LIMIT terms
pub fn read_token_list(storage: &dyn Storage, limit: usize) -> StdResult<Vec<AssetInfo>> {
    match storage.get(TOKEN_LIST) {
        None => Err(StdError::generic_err("No tokens are stored")),
        Some(data) => {
            let mut list: Vec<AssetInfo> = from_slice(&data)?;
            if limit < list.len() {
                list.truncate(limit);
            }
            Ok(list)
        }
    }
}

// this function checks whether the token is stored already
pub fn is_token(storage: &dyn Storage, token: AssetInfo) -> bool {
    match storage.get(TOKEN_LIST) {
        None => false,
        Some(data) => from_slice::<Vec<AssetInfo>>(&data)
            .map(|list| list.contains(&token))
            .unwrap_or_default(),
    }
}

// this function deletes the entry under the given key
pub fn remove_token(storage: &mut dyn Storage, token: AssetInfo) -> StdResult<()> {
    // check if the list exists
    let mut token_list: Vec<AssetInfo> = match storage.get(TOKEN_LIST) {
        None => return Err(StdError::generic_err("No tokens are stored")),
        Some(data) => from_slice(&data)?,
    };

    // change token_list
    if let Some(index) = token_list.clone().iter().position(|x| x.eq(&token)) {
        token_list.swap_remove(index);
    } else {
        return Err(StdError::generic_err("This token has not been added"));
    }

    // saves the updated token_list
    Ok(storage.set(TOKEN_LIST, &to_vec(&token_list)?))
}
