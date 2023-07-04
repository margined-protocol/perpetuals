use cosmwasm_std::{Addr, Deps, Env, StdError, StdResult, Storage, SubMsg, Uint128};

use margined_utils::contracts::helpers::VammController;

use crate::{
    contract::TRANSFER_FAILURE_REPLY_ID,
    state::{read_config, State},
};

use margined_common::{asset::AssetInfo, messages::wasm_execute};
use margined_perp::margined_engine::TransferResponse;
use margined_perp::margined_insurance_fund::ExecuteMsg as InsuranceFundExecuteMessage;
use margined_perp::margined_vamm::CalcFeeResponse;

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
    println!("execute_transfer - msg: {:?}", msg);

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
    println!("execute_transfer_to_insurance_fund - token_balance: {:?}", token_balance);

    let amount_to_send = Uint128::min(
        token_balance,
        amount,
    );

    println!("execute_transfer_to_insurance_fund - amount_to_send: {:?}", amount_to_send);

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
    println!("execute_insurance_fund_withdrawal - msg: {:?}", msg);

    Ok(SubMsg::reply_on_error(msg, TRANSFER_FAILURE_REPLY_ID))
}

// Transfers the toll and spread fees to the the insurance fund and fee pool
pub fn transfer_fees(
    deps: Deps,
    from: Addr,
    vamm: Addr,
    notional: Uint128,
) -> StdResult<TransferResponse> {
    let vamm_controller = VammController(vamm);

    let CalcFeeResponse {
        spread_fee,
        toll_fee,
    } = vamm_controller.calc_fee(&deps.querier, notional)?;

    let mut messages: Vec<SubMsg> = vec![];

    let config = read_config(deps.storage)?;
    if !spread_fee.is_zero() {
        let msg = match config.insurance_fund {
            Some(insurance_fund) => {
                execute_transfer_from(deps.storage, &from, &insurance_fund, spread_fee)?
            }
            None => return Err(StdError::generic_err("insurance fund is not registered")),
        };

        messages.push(msg);
    };

    if !toll_fee.is_zero() {
        let msg = execute_transfer_from(deps.storage, &from, &config.fee_pool, toll_fee)?;
        messages.push(msg);
    };

    Ok(TransferResponse {
        messages,
        spread_fee,
        toll_fee,
    })
}

pub fn withdraw(
    deps: Deps,
    env: Env,
    state: &mut State,
    receiver: &Addr,
    eligible_collateral: AssetInfo,
    amount: Uint128,
    pre_paid_shortfall: Uint128,
) -> StdResult<Vec<SubMsg>> {
    let token_balance = eligible_collateral.query_balance(&deps.querier, env.contract.address)?;

    let mut messages: Vec<SubMsg> = vec![];

    if token_balance.checked_add(pre_paid_shortfall)? < amount {
        let shortfall = amount.checked_sub(token_balance.checked_add(pre_paid_shortfall)?)?;

        // add any shortfall to bad_debt
        state.prepaid_bad_debt = state.prepaid_bad_debt.checked_add(shortfall)?;

        messages.push(execute_insurance_fund_withdrawal(deps, shortfall)?);
    }

    messages.push(execute_transfer(deps.storage, receiver, amount)?);

    Ok(messages)
}
