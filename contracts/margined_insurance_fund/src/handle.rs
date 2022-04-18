use cosmwasm_std::{
    to_binary, BankMsg, Coin, CosmosMsg, DepsMut, MessageInfo, ReplyOn, Response, SubMsg, Uint128,
    WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use terraswap::asset::AssetInfo;

use crate::{
    error::ContractError,
    state::{read_config, remove_amm, save_vamm, store_config, Config},
};

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    beneficiary: Option<String>,
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

    // change owner of insurance fund contract
    if let Some(beneficiary) = beneficiary {
        config.beneficiary = deps.api.addr_validate(beneficiary.as_str())?;
    }

    store_config(deps.storage, &config)?;

    Ok(Response::default())
}

pub fn add_vamm(deps: DepsMut, info: MessageInfo, vamm: String) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;

    // check permission
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // validate address
    let vamm_valid = deps.api.addr_validate(&vamm)?;

    // add the amm
    save_vamm(deps, vamm_valid)?;

    Ok(Response::default())
}

pub fn remove_vamm(
    deps: DepsMut,
    info: MessageInfo,
    vamm: String,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;

    // check permission
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // validate address
    let vamm_valid = deps.api.addr_validate(&vamm)?;

    // remove vamm here
    remove_amm(deps, vamm_valid)?;

    Ok(Response::default())
}

pub fn withdraw(
    deps: DepsMut,
    info: MessageInfo,
    token: AssetInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config: Config = read_config(deps.storage)?;

    // check permission
    if info.sender != config.beneficiary {
        return Err(ContractError::Unauthorized {});
    }

    // TODO: check that the asset is accepted

    // send tokens if native or cw20
    let msg: CosmosMsg = match token {
        AssetInfo::NativeToken { denom } => CosmosMsg::Bank(BankMsg::Send {
            to_address: config.beneficiary.to_string(),
            amount: vec![Coin { denom, amount }],
        }),
        AssetInfo::Token { contract_addr } => CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
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
