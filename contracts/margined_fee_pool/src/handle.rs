use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError::GenericErr, Uint128};
use margined_common::validate::validate_eligible_collateral as validate_funds;
use margined_perp::querier::query_token_balance;

use crate::{
    error::ContractError,
    messages::execute_transfer,
    state::{
        read_config, remove_token as remove_token_from_list, save_token, store_config, Config,
    },
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

    // change owner of insurance fund contract
    if let Some(owner) = owner {
        config.owner = deps.api.addr_validate(owner.as_str())?;
    }

    store_config(deps.storage, &config)?;

    Ok(Response::default())
}

pub fn add_token(
    deps: DepsMut,
    info: MessageInfo,
    token: String,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;

    // check permission
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // validate address
    let token_valid = deps.api.addr_validate(&token)?;

    // add the token
    save_token(deps, token_valid)?;

    Ok(Response::default())
}

pub fn remove_token(
    deps: DepsMut,
    info: MessageInfo,
    token: String,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;

    // check permission
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // validate address
    let token_valid = deps.api.addr_validate(&token)?;

    // remove token here
    remove_token_from_list(deps, token_valid)?;

    Ok(Response::default())
}

pub fn send_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token: String,
    amount: Uint128,
    recipient: String,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;

    // check permissions to send the message
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // validate the token we want to send
    let valid_token = validate_funds(deps.as_ref(), token)?;

    // validate the recipient address
    let valid_recipient = deps.as_ref().api.addr_validate(&recipient)?;

    // TODO: check that the token is in the token list?

    // query the balance of the given token that this contract holds
    let balance = query_token_balance(deps.as_ref(), valid_token, env.contract.address)?;

    // check that the balance is sufficient to pay the amount
    if balance < amount {
        return Err(ContractError::Std(GenericErr {
            msg: "Insufficient funds".to_string(),
        }));
    }
    Ok(Response::default().add_submessage(execute_transfer(
        deps.storage,
        &valid_recipient,
        balance,
    )?))
}
