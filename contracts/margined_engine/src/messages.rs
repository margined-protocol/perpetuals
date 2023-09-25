use cosmwasm_std::{Addr, Deps, Env, StdError, StdResult, Storage, SubMsg, Uint128};

use crate::{
    contract::TRANSFER_FAILURE_REPLY_ID,
    state::{read_config, State},
};

use margined_common::{asset::AssetInfo, messages::wasm_execute};
use margined_perp::margined_insurance_fund::ExecuteMsg as InsuranceFundExecuteMessage;

pub fn execute_transfer_from(
    storage: &dyn Storage,
    owner: &Addr,
    receiver: &Addr,
    amount: Uint128,
) -> StdResult<SubMsg> {
    let config = read_config(storage)?;
    let msg = config.eligible_collateral.into_msg(
        receiver.to_string(),
        amount,
        Some(owner.to_string()),
    )?;

    Ok(SubMsg::reply_on_error(msg, TRANSFER_FAILURE_REPLY_ID))
}

pub fn execute_transfer(
    storage: &dyn Storage,
    receiver: &Addr,
    amount: Uint128,
) -> StdResult<SubMsg> {
    let config = read_config(storage)?;

    let msg = config
        .eligible_collateral
        .into_msg(receiver.to_string(), amount, None)?;

    Ok(SubMsg::reply_on_error(msg, TRANSFER_FAILURE_REPLY_ID))
}

pub fn execute_transfer_to_insurance_fund(
    deps: Deps,
    env: Env,
    amount: Uint128,
) -> StdResult<SubMsg> {
    let config = read_config(deps.storage)?;

    let token_balance = config
        .eligible_collateral
        .query_balance(&deps.querier, env.contract.address)?;

    let amount_to_send = Uint128::min(
        token_balance,
        amount,
    );

    match config.insurance_fund {
        Some(insurance_fund) => execute_transfer(deps.storage, &insurance_fund, amount_to_send),
        None => return Err(StdError::generic_err("insurance fund is not registered")),
    }
}

pub fn execute_insurance_fund_withdrawal(deps: Deps, amount: Uint128) -> StdResult<SubMsg> {
    let config = read_config(deps.storage)?;

    let insurance_fund = match config.insurance_fund {
        Some(insurance_fund) => insurance_fund,
        None => return Err(StdError::generic_err("insurance fund is not registered")),
    };

    let msg = wasm_execute(
        insurance_fund,
        &InsuranceFundExecuteMessage::Withdraw {
            token: config.eligible_collateral,
            amount,
        },
        vec![],
    )?;

    Ok(SubMsg::reply_on_error(msg, TRANSFER_FAILURE_REPLY_ID))
}

// Transfers the toll and spread fees to the the insurance fund and fee pool
pub fn transfer_fees(
    deps: Deps,
    from: Addr,
    spread_fee: Uint128,
    toll_fee: Uint128,
    open_position: bool
) -> StdResult<Vec<SubMsg>> {
    let mut messages: Vec<SubMsg> = vec![];

    let config = read_config(deps.storage)?;

    if Some(config.insurance_fund.clone()).is_none() {
        return Err(StdError::generic_err("insurance fund is not registered"));
    }

    if !spread_fee.is_zero() {
        let msg = match open_position {
            true => {
                execute_transfer_from(deps.storage, &from, &config.insurance_fund.unwrap(), spread_fee)?
            }
            false => {
                execute_transfer(deps.storage, &config.insurance_fund.unwrap(), spread_fee)?
            },
        };

        messages.push(msg);
    };

    if !toll_fee.is_zero() {
        let msg = match open_position {
            true => {
                execute_transfer_from(deps.storage, &from, &config.fee_pool, toll_fee)?
            }
            false => {
                execute_transfer(deps.storage, &config.fee_pool, toll_fee)?
            },
        };
        messages.push(msg);
    };
    Ok(messages)
}

pub fn withdraw(
    deps: Deps,
    env: Env,
    state: &mut State,
    receiver: &Addr,
    eligible_collateral: AssetInfo,
    amount: Uint128,
    fees: Uint128,
    pre_paid_shortfall: Uint128,
) -> StdResult<Vec<SubMsg>> {
    let token_balance = eligible_collateral.query_balance(&deps.querier, env.contract.address)?;

    let mut messages: Vec<SubMsg> = vec![];

    if token_balance.checked_add(pre_paid_shortfall)? < amount.checked_add(fees)? {
        let shortfall = amount.checked_add(fees)?.checked_sub(token_balance.checked_add(pre_paid_shortfall)?)?;

        // add any shortfall to bad_debt
        state.prepaid_bad_debt = state.prepaid_bad_debt.checked_add(shortfall)?;
        messages.push(execute_insurance_fund_withdrawal(deps, shortfall)?);
    }

    messages.push(execute_transfer(deps.storage, receiver, amount)?);
    Ok(messages)
}
