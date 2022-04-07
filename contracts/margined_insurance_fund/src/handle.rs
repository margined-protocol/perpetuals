use cosmwasm_std::{DepsMut, MessageInfo, Response, Addr};

use crate::{
    error::ContractError,
    state::{read_config, store_config, Config, read_vamm, save_vamm},
};

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
) -> Result<Response, ContractError> {
    let mut config: Config = read_config(deps.storage)?;

    // check permission
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // change owner of amm
    if let Some(owner) = owner {
        config.owner = deps.api.addr_validate(owner.as_str())?;
    }

    store_config(deps.storage, &config)?;

    Ok(Response::default())
}

pub fn add_amm(
    deps: DepsMut,
    info: MessageInfo,
    amm: Addr,
) -> Result<Response, ContractError> {
    let mut config: Config = read_config(deps.storage)?;

    // check permission
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    } 

    // check if the address is actually an amm
    // TODO

    // add the amm
    save_vamm(deps, amm);

    Ok(Response::default())
}

pub fn remove_amm(
    deps: DepsMut,
    info: MessageInfo,
    amm: Addr,
) -> Result<Response, ContractError> {
    let mut config: Config = read_config(deps.storage)?;

    // check permission
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // pop the vamm off here
    let mut vamms = read_vamm(deps.storage)?.vamms;


    // check if the amm is there
    

    Ok(Response::default())
}