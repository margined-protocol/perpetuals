use cosmwasm_std::{
    to_binary, BankMsg, Coin, CosmosMsg, DepsMut, MessageInfo, ReplyOn, Response, StdError,
    StdResult, SubMsg, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use cw_utils::maybe_addr;
use margined_common::asset::AssetInfo;

use crate::{
    contract::OWNER,
    messages::execute_vamm_shutdown,
    state::{read_config, read_vammlist, remove_vamm as remove_amm, save_vamm, Config, VAMM_LIMIT},
};

pub fn update_owner(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
) -> StdResult<Response> {
    // validate the address
    let valid_owner = maybe_addr(deps.api, owner)?;

    OWNER
        .execute_update_admin(deps, info, valid_owner)
        .map_err(|error| StdError::generic_err(format!("{}", error)))?;

    Ok(Response::default().add_attribute("action", "update_config"))
}

pub fn add_vamm(deps: DepsMut, info: MessageInfo, vamm: String) -> StdResult<Response> {
    // check permission
    if !OWNER.is_admin(deps.as_ref().clone(), &info.sender)? {
        return Err(StdError::generic_err("unauthorized"));
    }

    // validate address
    let vamm_valid = deps.api.addr_validate(&vamm)?;

    // add the amm
    save_vamm(deps, vamm_valid)?;

    Ok(Response::default())
}

pub fn remove_vamm(deps: DepsMut, info: MessageInfo, vamm: String) -> StdResult<Response> {
    // check permission
    if !OWNER.is_admin(deps.as_ref().clone(), &info.sender)? {
        return Err(StdError::generic_err("unauthorized"));
    }

    // validate address
    let vamm_valid = deps.api.addr_validate(&vamm)?;

    // remove vamm here
    remove_amm(deps, vamm_valid)?;

    Ok(Response::default())
}

pub fn shutdown_all_vamm(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
    // check permission
    if !OWNER.is_admin(deps.as_ref().clone(), &info.sender)? {
        return Err(StdError::generic_err("unauthorized"));
    }

    // initialise the submsgs vec
    let mut msgs = vec![];

    // construct all the shutdown messages
    let keys = read_vammlist(deps.as_ref(), VAMM_LIMIT)?;

    for vamm in keys.iter() {
        msgs.push(execute_vamm_shutdown(vamm.clone())?);
    }

    Ok(Response::default().add_submessages(msgs))
}

pub fn withdraw(
    deps: DepsMut,
    info: MessageInfo,
    token: AssetInfo,
    amount: Uint128,
) -> StdResult<Response> {
    let config: Config = read_config(deps.storage)?;

    // check permission
    if info.sender != config.beneficiary {
        return Err(StdError::generic_err("unauthorized"));
    }

    // send tokens if native or cw20
    let msg: CosmosMsg = match token {
        AssetInfo::NativeToken { denom } => CosmosMsg::Bank(BankMsg::Send {
            to_address: config.beneficiary.to_string(),
            amount: vec![Coin { denom, amount }],
        }),
        AssetInfo::Token { contract_addr } => CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: config.beneficiary.to_string(),
                amount,
            })?,
        }),
    };

    let transfer_msg = SubMsg {
        msg,
        gas_limit: None, // probably should set a limit in the config
        id: 0u64,
        reply_on: ReplyOn::Never,
    };

    Ok(Response::default()
        .add_submessage(transfer_msg)
        .add_attributes(vec![
            ("action", "insurance_withdraw"),
            ("amount", &amount.to_string()),
        ]))
}
