use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError,
    StdResult, SubMsgResult, Uint128,
};
use cw2::set_contract_version;
use cw_controllers::Admin;
use margined_common::validate::{
    validate_decimal_places, validate_eligible_collateral, validate_margin_ratios, validate_ratio,
};
use margined_perp::margined_engine::{ExecuteMsg, InstantiateMsg, QueryMsg};

use crate::error::ContractError;
use crate::{
    handle::{
        close_position, deposit_margin, liquidate, open_position, pay_funding, set_pause,
        update_config, update_pauser, withdraw_margin,
    },
    query::{
        query_all_positions, query_config, query_cumulative_premium_fraction,
        query_free_collateral, query_margin_ratio, query_pauser, query_position,
        query_position_notional_unrealized_pnl, query_state,
        query_trader_balance_with_funding_payment, query_trader_position_with_funding_payment,
    },
    reply::{
        close_position_reply, liquidate_reply, partial_close_position_reply,
        partial_liquidation_reply, pay_funding_reply, reverse_position_reply,
        update_position_reply,
    },
    state::{store_config, store_state, Config, State},
    utils::{parse_pay_funding, parse_swap},
};

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "crates.io:margined-engine";
/// Contract version that is used for migration.
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Admin controller for the pauser role
pub const PAUSER: Admin = Admin::new("pauser");

pub const INCREASE_POSITION_REPLY_ID: u64 = 1;
pub const DECREASE_POSITION_REPLY_ID: u64 = 2;
pub const REVERSE_POSITION_REPLY_ID: u64 = 3;
pub const CLOSE_POSITION_REPLY_ID: u64 = 4;
pub const PARTIAL_CLOSE_POSITION_REPLY_ID: u64 = 5;
pub const LIQUIDATION_REPLY_ID: u64 = 6;
pub const PARTIAL_LIQUIDATION_REPLY_ID: u64 = 7;
pub const PAY_FUNDING_REPLY_ID: u64 = 8;

pub const TRANSFER_FAILURE_REPLY_ID: u64 = 9;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // validate message addresses
    let valid_pauser = deps.api.addr_validate(&msg.pauser)?;
    let insurance_fund = deps.api.addr_validate(&msg.insurance_fund)?;
    let fee_pool = deps.api.addr_validate(&msg.fee_pool)?;

    // validate eligible collateral
    let eligible_collateral = validate_eligible_collateral(deps.as_ref(), msg.eligible_collateral)?;

    // find decimals of asset
    let decimal_response = eligible_collateral.get_decimals(deps.as_ref())?;

    // validate decimal places are correct, and return ratio max.
    let decimals = validate_decimal_places(decimal_response)?;

    // validate the ratios conform to the decimals
    validate_ratio(msg.initial_margin_ratio, decimals)?;
    validate_ratio(msg.maintenance_margin_ratio, decimals)?;
    validate_ratio(msg.liquidation_fee, decimals)?;

    // validate that the maintenance margin is not greater than the initial
    validate_margin_ratios(msg.initial_margin_ratio, msg.maintenance_margin_ratio)?;

    // config parameters
    let config = Config {
        owner: info.sender,
        insurance_fund,
        fee_pool,
        eligible_collateral,
        decimals,
        initial_margin_ratio: msg.initial_margin_ratio,
        maintenance_margin_ratio: msg.maintenance_margin_ratio,
        partial_liquidation_ratio: Uint128::zero(), // set as zero by default
        liquidation_fee: msg.liquidation_fee,
    };

    store_config(deps.storage, &config)?;

    // store default state
    store_state(
        deps.storage,
        &State {
            open_interest_notional: Uint128::zero(),
            prepaid_bad_debt: Uint128::zero(),
            pause: false,
        },
    )?;

    PAUSER.set(deps, Some(valid_pauser))?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
            insurance_fund,
            fee_pool,
            initial_margin_ratio,
            maintenance_margin_ratio,
            partial_liquidation_ratio,
            liquidation_fee,
        } => update_config(
            deps,
            info,
            owner,
            insurance_fund,
            fee_pool,
            initial_margin_ratio,
            maintenance_margin_ratio,
            partial_liquidation_ratio,
            liquidation_fee,
        ),
        ExecuteMsg::UpdatePauser { pauser } => update_pauser(deps, info, pauser),
        ExecuteMsg::OpenPosition {
            vamm,
            side,
            quote_asset_amount,
            leverage,
            base_asset_limit,
        } => open_position(
            deps,
            env,
            info,
            vamm,
            side,
            quote_asset_amount,
            leverage,
            base_asset_limit,
        ),
        ExecuteMsg::ClosePosition {
            vamm,
            quote_asset_limit,
        } => close_position(deps, env, info, vamm, quote_asset_limit),
        ExecuteMsg::Liquidate {
            vamm,
            trader,
            quote_asset_limit,
        } => liquidate(deps, env, info, vamm, trader, quote_asset_limit),
        ExecuteMsg::PayFunding { vamm } => pay_funding(deps, env, info, vamm),
        ExecuteMsg::DepositMargin { vamm, amount } => deposit_margin(deps, env, info, vamm, amount),
        ExecuteMsg::WithdrawMargin { vamm, amount } => {
            withdraw_margin(deps, env, info, vamm, amount)
        }
        ExecuteMsg::SetPause { pause } => set_pause(deps, env, info, pause),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State {} => to_binary(&query_state(deps)?),
        QueryMsg::GetPauser {} => to_binary(&query_pauser(deps)?),
        QueryMsg::AllPositions { trader } => to_binary(&query_all_positions(deps, trader)?),
        QueryMsg::Position { vamm, trader } => to_binary(&query_position(deps, vamm, trader)?),
        QueryMsg::MarginRatio { vamm, trader } => {
            to_binary(&query_margin_ratio(deps, vamm, trader)?)
        }
        QueryMsg::CumulativePremiumFraction { vamm } => {
            to_binary(&query_cumulative_premium_fraction(deps, vamm)?)
        }
        QueryMsg::UnrealizedPnl {
            vamm,
            trader,
            calc_option,
        } => to_binary(&query_position_notional_unrealized_pnl(
            deps,
            vamm,
            trader,
            calc_option,
        )?),
        QueryMsg::FreeCollateral { vamm, trader } => {
            to_binary(&query_free_collateral(deps, vamm, trader)?)
        }
        QueryMsg::BalanceWithFundingPayment { trader } => {
            to_binary(&query_trader_balance_with_funding_payment(deps, trader)?)
        }
        QueryMsg::PositionWithFundingPayment { vamm, trader } => to_binary(
            &query_trader_position_with_funding_payment(deps, vamm, trader)?,
        ),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    match msg.result {
        SubMsgResult::Ok(response) => match msg.id {
            INCREASE_POSITION_REPLY_ID => {
                let (input, output) = parse_swap(response).unwrap();
                let response =
                    update_position_reply(deps, env, input, output, INCREASE_POSITION_REPLY_ID)?;
                Ok(response)
            }
            DECREASE_POSITION_REPLY_ID => {
                let (input, output) = parse_swap(response).unwrap();
                let response =
                    update_position_reply(deps, env, input, output, DECREASE_POSITION_REPLY_ID)?;
                Ok(response)
            }
            REVERSE_POSITION_REPLY_ID => {
                let (input, output) = parse_swap(response).unwrap();
                let response = reverse_position_reply(deps, env, input, output)?;
                Ok(response)
            }
            CLOSE_POSITION_REPLY_ID => {
                let (input, output) = parse_swap(response).unwrap();
                let response = close_position_reply(deps, env, input, output)?;
                Ok(response)
            }
            PARTIAL_CLOSE_POSITION_REPLY_ID => {
                let (input, output) = parse_swap(response).unwrap();
                let response = partial_close_position_reply(deps, env, input, output)?;
                Ok(response)
            }
            LIQUIDATION_REPLY_ID => {
                let (input, output) = parse_swap(response).unwrap();
                let response = liquidate_reply(deps, env, input, output)?;
                Ok(response)
            }
            PARTIAL_LIQUIDATION_REPLY_ID => {
                let (input, output) = parse_swap(response).unwrap();
                let response = partial_liquidation_reply(deps, env, input, output)?;
                Ok(response)
            }
            PAY_FUNDING_REPLY_ID => {
                let (premium_fraction, sender) = parse_pay_funding(response).unwrap();
                let response = pay_funding_reply(deps, env, premium_fraction, sender)?;
                Ok(response)
            }
            _ => Err(StdError::generic_err(format!(
                "reply (id {:?}) invalid",
                msg.id
            ))),
        },
        SubMsgResult::Err(e) => match msg.id {
            TRANSFER_FAILURE_REPLY_ID => Err(StdError::generic_err(format!(
                "transfer failure - reply (id {:?})",
                msg.id
            ))),
            INCREASE_POSITION_REPLY_ID => Err(StdError::generic_err(format!(
                "increase position failure - reply (id {:?})",
                msg.id
            ))),
            DECREASE_POSITION_REPLY_ID => Err(StdError::generic_err(format!(
                "decrease position failure - reply (id {:?})",
                msg.id
            ))),
            REVERSE_POSITION_REPLY_ID => Err(StdError::generic_err(format!(
                "reverse position failure - reply (id {:?})",
                msg.id
            ))),
            CLOSE_POSITION_REPLY_ID => Err(StdError::generic_err(format!(
                "close position failure - reply (id {:?})",
                msg.id
            ))),
            PARTIAL_CLOSE_POSITION_REPLY_ID => Err(StdError::generic_err(format!(
                "partial close position failure - reply (id {:?})",
                msg.id
            ))),
            LIQUIDATION_REPLY_ID => Err(StdError::generic_err(format!(
                "liquidation failure - reply (id {:?})",
                msg.id
            ))),
            PARTIAL_LIQUIDATION_REPLY_ID => Err(StdError::generic_err(format!(
                "partial liquidation failure - reply (id {:?})",
                msg.id
            ))),
            PAY_FUNDING_REPLY_ID => Err(StdError::generic_err(format!(
                "funding payment failure - reply (id {:?})",
                msg.id
            ))),
            _ => Err(StdError::generic_err(format!(
                "reply (id {:?}) error {:?}",
                msg.id, e
            ))),
        },
    }
}
