use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128};
use margined_common::validate::validate_eligible_collateral as validate_funds;
use margined_perp::querier::query_token_balance;

use crate::{
    contract::OWNER,
    messages::execute_transfer,
    state::{is_token, remove_token as remove_token_from_list, save_token},
};

pub fn update_owner(deps: DepsMut, info: MessageInfo, owner: String) -> StdResult<Response> {
    // validate the address
    let valid_owner = deps.api.addr_validate(&owner)?;

    OWNER
        .execute_update_admin::<(), _>(deps, info, Some(valid_owner))
        .map_err(|error| StdError::generic_err(format!("{}", error)))?;

    Ok(Response::default().add_attribute("action", "update_owner"))
}

pub fn add_token(deps: DepsMut, info: MessageInfo, token: String) -> StdResult<Response> {
    // check permission
    if !OWNER.is_admin(deps.as_ref(), &info.sender)? {
        return Err(StdError::generic_err("unauthorized"));
    }

    // validate address
    let valid_token = validate_funds(deps.as_ref(), token)?;

    // add the token
    save_token(deps, valid_token)?;

    Ok(Response::default())
}

pub fn remove_token(deps: DepsMut, info: MessageInfo, token: String) -> StdResult<Response> {
    // check permission
    if !OWNER.is_admin(deps.as_ref(), &info.sender)? {
        return Err(StdError::generic_err("unauthorized"));
    }

    // validate address
    let valid_token = validate_funds(deps.as_ref(), token)?;

    // remove token here
    remove_token_from_list(deps, valid_token)?;

    Ok(Response::default())
}

pub fn send_token(
    deps: Deps,
    env: Env,
    info: MessageInfo,
    token: String,
    amount: Uint128,
    recipient: String,
) -> StdResult<Response> {
    // check permissions to send the message
    if !OWNER.is_admin(deps, &info.sender)? {
        return Err(StdError::generic_err("unauthorized"));
    }

    // validate the token we want to send (this also tells us if it is native token or not)
    let valid_token = validate_funds(deps, token)?;

    // validate the recipient address
    let valid_recipient = deps.api.addr_validate(&recipient)?;

    // check that the token is in the token list
    if !is_token(deps.storage, valid_token.clone()) {
        return Err(StdError::generic_err("This token is not supported"));
    };

    // query the balance of the given token that this contract holds
    let balance = query_token_balance(deps, valid_token.clone(), env.contract.address)?;

    // check that the balance is sufficient to pay the amount
    if balance < amount {
        return Err(StdError::generic_err("Insufficient funds"));
    }
    Ok(
        Response::default().add_submessage(execute_transfer(
            valid_token,
            &valid_recipient,
            amount,
        )?),
    )
}
