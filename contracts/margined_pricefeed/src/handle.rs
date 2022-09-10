use cosmwasm_std::{DepsMut, MessageInfo, Response, StdError, Uint128};
use cw_utils::maybe_addr;

use crate::{contract::OWNER, error::ContractError, state::store_price_data};

pub fn update_owner(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
) -> Result<Response, ContractError> {
    // validate the address
    let valid_owner = maybe_addr(deps.api, owner)?;

    OWNER.execute_update_admin::<Response, _>(deps, info, valid_owner)?;

    Ok(Response::default().add_attribute("action", "update_owner"))
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
    // check permission
    OWNER.assert_admin(deps.as_ref(), &info.sender)?;

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
    // check permission
    OWNER.assert_admin(deps.as_ref(), &info.sender)?;

    // This throws if the prices and timestamps are not the same length
    if prices.len() != timestamps.len() {
        return Err(ContractError::Std(StdError::generic_err(
            "Prices and timestamps are not the same length",
        )));
    }

    for index in 0..prices.len() {
        store_price_data(deps.storage, key.clone(), prices[index], timestamps[index])?;
    }

    Ok(Response::default())
}
