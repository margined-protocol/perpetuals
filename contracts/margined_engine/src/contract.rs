use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError,
    StdResult, SubMsgResult, Uint128,
};
use cw2::set_contract_version;
use cw_controllers::{Admin, Hooks};
use margined_common::validate::{
    validate_decimal_places, validate_eligible_collateral, validate_margin_ratios, validate_ratio,
};
use margined_perp::margined_engine::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

use crate::error::ContractError;
use crate::handle::{update_tp_sl, trigger_tp_sl};
use crate::state::init_last_position_id;
use crate::{
    handle::{
        close_position, deposit_margin, liquidate, open_position, pay_funding, update_config,
        withdraw_margin,
    },
    query::{
        query_all_positions, query_config, query_cumulative_premium_fraction,
        query_free_collateral, query_margin_ratio, query_pauser, query_position,
        query_position_notional_unrealized_pnl, query_state,
        query_trader_balance_with_funding_payment, query_trader_position_with_funding_payment,
    },
    reply::{
        close_position_reply, liquidate_reply, partial_close_position_reply,
        partial_liquidation_reply, pay_funding_reply,
        update_position_reply,
    },
    state::{store_config, store_state, Config, State},
    utils::{
        add_whitelist, parse_pay_funding, parse_swap, remove_whitelist, set_pause, update_pauser,
    },
};

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "crates.io:margined-engine";
/// Contract version that is used for migration.
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Admin controller for the pauser role
pub const PAUSER: Admin = Admin::new("pauser");
/// Hooks controller for the base asset holding whitelist
pub const WHITELIST: Hooks = Hooks::new("whitelist");

pub const INCREASE_POSITION_REPLY_ID: u64 = 1;
pub const CLOSE_POSITION_REPLY_ID: u64 = 2;
pub const PARTIAL_CLOSE_POSITION_REPLY_ID: u64 = 3;
pub const LIQUIDATION_REPLY_ID: u64 = 4;
pub const PARTIAL_LIQUIDATION_REPLY_ID: u64 = 5;
pub const PAY_FUNDING_REPLY_ID: u64 = 6;
pub const TAKE_PROFIT_REPLY_ID: u64 = 7;
pub const STOP_LOSS_REPLY_ID: u64 = 8;
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
    let insurance_fund = match msg.insurance_fund {
        Some(addr) => Some(deps.api.addr_validate(&addr)?),
        None => None,
    };

    let fee_pool = deps.api.addr_validate(&msg.fee_pool)?;

    // validate eligible collateral
    let eligible_collateral = validate_eligible_collateral(deps.as_ref(), msg.eligible_collateral)?;

    // find decimals of asset
    let decimal_response = eligible_collateral.get_decimals(&deps.querier)?;
    println!("instantiate margined engine - decimal_response: {}", decimal_response);

    // validate decimal places are correct, and return ratio max.
    let decimals = validate_decimal_places(decimal_response)?;

    // validate the ratios conform to the decimals
    validate_ratio(msg.initial_margin_ratio, decimals)?;
    validate_ratio(msg.maintenance_margin_ratio, decimals)?;
    validate_ratio(msg.liquidation_fee, decimals)?;

    // validate that the maintenance margin is not greater than the initial
    validate_margin_ratios(msg.initial_margin_ratio, msg.maintenance_margin_ratio)?;
    println!("instantiate margined engine - initial_margin_ratio: {}", msg.initial_margin_ratio);
    println!("instantiate margined engine - maintenance_margin_ratio: {}", msg.maintenance_margin_ratio);
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

    // Initialize last position id
    init_last_position_id(deps.storage)?;

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
        ExecuteMsg::AddWhitelist { address } => add_whitelist(deps, info, address),
        ExecuteMsg::RemoveWhitelist { address } => remove_whitelist(deps, info, address),
        ExecuteMsg::OpenPosition {
            vamm,
            side,
            margin_amount,
            leverage,
            take_profit,
            stop_loss,
            base_asset_limit,
        } => open_position(
            deps,
            env,
            info,
            vamm,
            side,
            margin_amount,
            leverage,
            take_profit,
            stop_loss,
            base_asset_limit,
        ),
        ExecuteMsg::UpdateTpSl {
            vamm,
            position_id,
            take_profit,
            stop_loss
        } => update_tp_sl(deps, env, info, vamm, position_id, take_profit, stop_loss),
        ExecuteMsg::ClosePosition {
            vamm,
            position_id,
            quote_asset_limit,
        } => close_position(deps, env, info, vamm, position_id, quote_asset_limit),
        ExecuteMsg::Liquidate {
            vamm,
            trader,
            position_id,
            quote_asset_limit,
        } => liquidate(deps, env, info, vamm, position_id, trader, quote_asset_limit),
        ExecuteMsg::TriggerTpSl { vamm, position_id, quote_asset_limit } => trigger_tp_sl(deps, env, info, vamm, position_id, quote_asset_limit),
        ExecuteMsg::PayFunding { vamm } => pay_funding(deps, env, info, vamm),
        ExecuteMsg::DepositMargin { vamm, position_id, amount } => deposit_margin(deps, env, info, vamm, position_id, amount),
        ExecuteMsg::WithdrawMargin { vamm, position_id, amount } => {
            withdraw_margin(deps, env, info, vamm, position_id, amount)
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
        QueryMsg::IsWhitelisted { address } => to_binary(&WHITELIST.query_hook(deps, address)?),
        QueryMsg::GetWhitelist {} => to_binary(&WHITELIST.query_hooks(deps)?),
        QueryMsg::AllPositions {
            trader,
            start_after,
            limit,
            order_by,
        } => to_binary(&query_all_positions(deps, trader, start_after, limit, order_by)?),
        QueryMsg::Position { vamm, position_id } => to_binary(&query_position(deps, vamm, position_id)?),
        QueryMsg::MarginRatio { vamm, position_id } => {
            to_binary(&query_margin_ratio(deps, vamm, position_id)?)
        }
        QueryMsg::CumulativePremiumFraction { vamm } => {
            to_binary(&query_cumulative_premium_fraction(deps, vamm)?)
        }
        QueryMsg::UnrealizedPnl {
            vamm,
            position_id,
            calc_option,
        } => to_binary(&query_position_notional_unrealized_pnl(
            deps,
            vamm,
            position_id,
            calc_option,
        )?),
        QueryMsg::FreeCollateral { vamm, position_id } => {
            to_binary(&query_free_collateral(deps, vamm, position_id)?)
        }
        QueryMsg::BalanceWithFundingPayment { position_id} => {
            to_binary(&query_trader_balance_with_funding_payment(deps, position_id)?)
        }
        QueryMsg::PositionWithFundingPayment { vamm, position_id } => to_binary(
            &query_trader_position_with_funding_payment(deps, vamm, position_id)?,
        ),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    match &msg.result {
        SubMsgResult::Ok(response) => match msg.id {
            INCREASE_POSITION_REPLY_ID => {
                let (input, output, position_id) = parse_swap(response)?;
                println!("INCREASE_POSITION_REPLY_ID - input: {:?} output: {:?} position_id: {:?}", input, output, position_id);
                let response =
                    update_position_reply(deps, env, input, output, position_id, INCREASE_POSITION_REPLY_ID)?;
                Ok(response)
            }
            CLOSE_POSITION_REPLY_ID => {
                let (input, output, position_id) = parse_swap(response)?;
                println!("CLOSE_POSITION_REPLY_ID - input: {:?} output: {:?} position_id: {:?}", input, output, position_id);
                let response = close_position_reply(deps, env, input, output, position_id)?;
                Ok(response)
            }
            PARTIAL_CLOSE_POSITION_REPLY_ID => {
                let (input, output, position_id) = parse_swap(response)?;
                println!("PARTIAL_CLOSE_POSITION_REPLY_ID - input: {:?} output: {:?} position_id: {:?}", input, output, position_id);
                let response = partial_close_position_reply(deps, env, input, output, position_id)?;
                Ok(response)
            }
            LIQUIDATION_REPLY_ID => {
                let (input, output, position_id) = parse_swap(response)?;
                println!("LIQUIDATION_REPLY_ID - input: {:?} output: {:?} position_id: {:?}", input, output, position_id);
                let response = liquidate_reply(deps, env, input, output, position_id)?;
                Ok(response)
            }
            PARTIAL_LIQUIDATION_REPLY_ID => {
                let (input, output, position_id) = parse_swap(response)?;
                println!("PARTIAL_LIQUIDATION_REPLY_ID - input: {:?} output: {:?} position_id: {:?}", input, output, position_id);
                let response = partial_liquidation_reply(deps, env, input, output, position_id)?;
                Ok(response)
            }
            PAY_FUNDING_REPLY_ID => {
                let (premium_fraction, sender) = parse_pay_funding(response)?;
                println!("PAY_FUNDING_REPLY_ID - premium_fraction {:?}", premium_fraction);
                let response = pay_funding_reply(deps, env, premium_fraction, sender)?;
                Ok(response)
            }
            TAKE_PROFIT_REPLY_ID => {
                let (input, output, position_id) = parse_swap(response)?;
                println!("TAKE_PROFIT_REPLY_ID - input: {:?} output: {:?} position_id: {:?}", input, output, position_id);
                let response = close_position_reply(deps, env, input, output, position_id)?;
                Ok(response)
            }
            STOP_LOSS_REPLY_ID => {
                let (input, output, position_id) = parse_swap(response)?;
                println!("STOP_LOSS_REPLY_ID - input: {:?} output: {:?} position_id: {:?}", input, output, position_id);
                let response = close_position_reply(deps, env, input, output, position_id)?;
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
            TAKE_PROFIT_REPLY_ID => Err(StdError::generic_err(format!(
                "take profit failure - reply (id {:?})",
                msg.id
            ))),
            STOP_LOSS_REPLY_ID => Err(StdError::generic_err(format!(
                "stop loss failure - reply (id {:?})",
                msg.id
            ))),
            _ => Err(StdError::generic_err(format!(
                "reply (id {:?}) error {:?}",
                msg.id, e
            ))),
        },
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new())
}
