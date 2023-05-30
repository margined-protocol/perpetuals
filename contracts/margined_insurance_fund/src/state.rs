use cosmwasm_std::{from_slice, to_vec, Addr, StdError, StdResult, Storage};
use margined_perp::margined_insurance_fund::ConfigResponse;

pub static KEY_CONFIG: &[u8] = b"config";
pub const VAMM_LIST: &[u8] = b"vamm-list";
pub const VAMM_LIMIT: usize = 3usize;

pub type Config = ConfigResponse;

// function checks if an addr is already added and adds it if not
// We also check that we have not reached the limit of vAMMs here
pub fn save_vamm(storage: &mut dyn Storage, input: Addr) -> StdResult<()> {
    // check if there is a vector
    let mut vamm_list: Vec<Addr> = match storage.get(VAMM_LIST) {
        None => vec![],
        Some(data) => from_slice(&data)?,
    };

    // check if we already added the vamm
    if vamm_list.contains(&input) {
        return Err(StdError::generic_err("This vAMM is already added"));
    };

    // check if we have reached the capacity
    if vamm_list.len() >= VAMM_LIMIT {
        return Err(StdError::generic_err(
            "The vAMM capacity is already reached",
        ));
    };

    // add the vamm to the vector
    vamm_list.push(input);
    Ok(storage.set(VAMM_LIST, &to_vec(&vamm_list)?))
}

// this function reads Addrs stored in the VAMM_LIST.
// note that this function ONLY takes the first VAMM_LIMIT terms
pub fn read_vammlist(storage: &dyn Storage, limit: usize) -> StdResult<Vec<Addr>> {
    match storage.get(VAMM_LIST) {
        None => Err(StdError::generic_err("No vAMMs are stored")),
        Some(data) => {
            let mut list: Vec<Addr> = from_slice(&data)?;
            if limit < list.len() {
                list.truncate(limit);
            }
            Ok(list)
        }
    }
}

// this function checks whether the vamm is stored already
pub fn is_vamm(storage: &dyn Storage, input: Addr) -> bool {
    match storage.get(VAMM_LIST) {
        None => false,
        Some(data) => from_slice::<Vec<Addr>>(&data)
            .map(|list| list.contains(&input))
            .unwrap_or_default(),
    }
}

// this function deletes the entry under the given key
pub fn remove_vamm(storage: &mut dyn Storage, input: Addr) -> StdResult<()> {
    // check if there are any vamms stored
    let mut vamm_list = match storage.get(VAMM_LIST) {
        None => return Err(StdError::generic_err("No vAMMs are stored")),
        Some(data) => from_slice::<Vec<Addr>>(&data)?,
    };

    // change vamm_list
    if let Some(index) = vamm_list.clone().iter().position(|x| x.eq(&input)) {
        vamm_list.swap_remove(index);
    } else {
        return Err(StdError::generic_err("This vAMM has not been added"));
    }

    // saves the updated vamm_list
    Ok(storage.set(VAMM_LIST, &to_vec(&vamm_list)?))
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    Ok(storage.set(KEY_CONFIG, &to_vec(config)?))
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    match storage.get(KEY_CONFIG) {
        Some(data) => from_slice(&data),
        None => Err(StdError::generic_err("Config not found")),
    }
}
