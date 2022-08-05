use cosmwasm_std::{DepsMut, MessageInfo, Response, Uint128};

use crate::{
    error::ContractError,
    state::{read_config, store_config, store_price_data, Config},
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

    // change owner of pricefeed
    if let Some(owner) = owner {
        config.owner = deps.api.addr_validate(owner.as_str())?;
    }

    store_config(deps.storage, &config)?;

    Ok(Response::default().add_attribute("action", "update_config"))
}

/// this is a mock function that enables storage of data
/// by the contract owner will be replaced by integration
/// with on-chain price oracles in the future.
pub fn append_price(
    deps: DepsMut,
    info: MessageInfo,
    key: String,
    price: Uint128,
    timestamp: u64,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;

    // check permission
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    store_price_data(deps.storage, key, price, timestamp)?;

    Ok(Response::default().add_attribute("action", "append_price"))
}

/// this is a mock function that enables storage of data
/// by the contract owner will be replaced by integration
/// with on-chain price oracles in the future.
pub fn append_multiple_price(
    deps: DepsMut,
    info: MessageInfo,
    key: String,
    prices: Vec<Uint128>,
    timestamps: Vec<u64>,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;

    // check permission
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // prices and timestamps are the same length
    if prices.len() != timestamps.len() {
        return Err(ContractError::Unauthorized {});
    }

    for index in 0..prices.len() {
        store_price_data(deps.storage, key.clone(), prices[index], timestamps[index])?;
    }

    Ok(Response::default())
}
