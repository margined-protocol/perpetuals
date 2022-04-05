use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Attribute, Binary, ContractResult, Deps, DepsMut, Env, Event, MessageInfo, Reply,
    Response, StdError, StdResult, SubMsgExecutionResponse, Uint128,
};
use margined_common::{integer::Integer, validate::validate_ratio};
use margined_perp::margined_engine::{ExecuteMsg, InstantiateMsg, QueryMsg};
#[cfg(not(feature = "library"))]
use std::str::FromStr;

use crate::error::ContractError;
use crate::{
    handle::{
        close_position, deposit_margin, liquidate, open_position, pay_funding, set_pause,
        update_config, withdraw_margin,
    },
    query::{
        query_config, query_cumulative_premium_fraction, query_margin_ratio, query_position,
        query_state, query_trader_balance_with_funding_payment,
        query_trader_position_with_funding_payment, query_unrealized_pnl,
    },
    reply::{
        close_position_reply, decrease_position_reply, increase_position_reply, liquidate_reply,
        partial_liquidation_reply, pay_funding_reply, reverse_position_reply,
    },
    state::{store_config, store_state, store_vamm, Config, State},
};

pub const SWAP_INCREASE_REPLY_ID: u64 = 1;
pub const SWAP_DECREASE_REPLY_ID: u64 = 2;
pub const SWAP_REVERSE_REPLY_ID: u64 = 3;
pub const SWAP_CLOSE_REPLY_ID: u64 = 4;
pub const SWAP_LIQUIDATE_REPLY_ID: u64 = 5;
pub const SWAP_PARTIAL_LIQUIDATION_REPLY_ID: u64 = 6;
pub const PAY_FUNDING_REPLY_ID: u64 = 7;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // validate the ratios, note this assumes the decimals is correct
    let decimals = Uint128::from(10u128.pow(msg.decimals as u32));
    validate_ratio(msg.initial_margin_ratio, decimals)?;
    validate_ratio(msg.maintenance_margin_ratio, decimals)?;
    validate_ratio(msg.liquidation_fee, decimals)?;

    // verify message addresses
    let eligible_collateral = deps.api.addr_validate(&msg.eligible_collateral)?;
    let insurance_fund = deps.api.addr_validate(&msg.insurance_fund)?;
    let fee_pool = deps.api.addr_validate(&msg.fee_pool)?;

    // config parameters
    let config = Config {
        owner: info.sender,
        insurance_fund,
        fee_pool,
        eligible_collateral,
        decimals,
        initial_margin_ratio: msg.initial_margin_ratio,
        maintenance_margin_ratio: msg.maintenance_margin_ratio,
        partial_liquidation_margin_ratio: Uint128::zero(), // set as zero by default
        liquidation_fee: msg.liquidation_fee,
    };

    store_config(deps.storage, &config)?;

    // store default state
    store_state(
        deps.storage,
        &State {
            open_interest_notional: Uint128::zero(),
            bad_debt: Uint128::zero(),
            pause: false,
        },
    )?;

    // store default vamms
    store_vamm(deps, &msg.vamm)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
            insurance_fund,
            fee_pool,
            eligible_collateral,
            decimals,
            initial_margin_ratio,
            maintenance_margin_ratio,
            partial_liquidation_margin_ratio,
            liquidation_fee,
        } => update_config(
            deps,
            info,
            owner,
            insurance_fund,
            fee_pool,
            eligible_collateral,
            decimals,
            initial_margin_ratio,
            maintenance_margin_ratio,
            partial_liquidation_margin_ratio,
            liquidation_fee,
        ),
        ExecuteMsg::OpenPosition {
            vamm,
            side,
            quote_asset_amount,
            leverage,
            base_asset_limit,
        } => {
            let trader = info.sender.clone();
            open_position(
                deps,
                env,
                info,
                vamm,
                trader.to_string(),
                side,
                quote_asset_amount,
                leverage,
                base_asset_limit,
            )
        }
        ExecuteMsg::ClosePosition {
            vamm,
            quote_asset_limit,
        } => {
            let trader = info.sender.clone();
            close_position(deps, env, info, vamm, trader.to_string(), quote_asset_limit)
        }
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
        } => to_binary(&query_unrealized_pnl(deps, vamm, trader, calc_option)?),
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
        ContractResult::Ok(response) => match msg.id {
            SWAP_INCREASE_REPLY_ID => {
                let (input, output) = parse_swap(response);
                let response = increase_position_reply(deps, env, input, output)?;
                Ok(response)
            }
            SWAP_DECREASE_REPLY_ID => {
                let (input, output) = parse_swap(response);
                let response = decrease_position_reply(deps, env, input, output)?;
                Ok(response)
            }
            SWAP_REVERSE_REPLY_ID => {
                let (input, output) = parse_swap(response);
                let response = reverse_position_reply(deps, env, input, output)?;
                Ok(response)
            }
            SWAP_CLOSE_REPLY_ID => {
                let (input, output) = parse_swap(response);
                let response = close_position_reply(deps, env, input, output)?;
                Ok(response)
            }
            SWAP_LIQUIDATE_REPLY_ID => {
                let (input, output) = parse_swap(response);
                let response = liquidate_reply(deps, env, input, output)?;
                Ok(response)
            }
            SWAP_PARTIAL_LIQUIDATION_REPLY_ID => {
                let (input, output) = parse_swap(response);
                let response = partial_liquidation_reply(deps, env, input, output)?;
                Ok(response)
            }
            PAY_FUNDING_REPLY_ID => {
                let (premium_fraction, sender) = parse_pay_funding(response);
                let response = pay_funding_reply(deps, env, premium_fraction, sender)?;
                Ok(response)
            }
            _ => Err(StdError::generic_err(format!(
                "reply (id {:?}) invalid",
                msg.id
            ))),
        },
        ContractResult::Err(e) => Err(StdError::generic_err(format!(
            "reply (id {:?}) error {:?}",
            msg.id, e
        ))),
    }
}

fn parse_swap(response: SubMsgExecutionResponse) -> (Uint128, Uint128) {
    // Find swap inputs and output events
    let wasm = response.events.iter().find(|&e| e.ty == "wasm");
    let wasm = wasm.unwrap();

    let swap = read_event("action".to_string(), wasm).value;

    let input: Uint128;
    let output: Uint128;
    match swap.as_str() {
        "swap_input" => {
            let input_str = read_event("quote_asset_amount".to_string(), wasm).value;
            input = Uint128::from_str(&input_str).unwrap();

            let output_str = read_event("base_asset_amount".to_string(), wasm).value;
            output = Uint128::from_str(&output_str).unwrap();
        }
        "swap_output" => {
            let input_str = read_event("base_asset_amount".to_string(), wasm).value;
            input = Uint128::from_str(&input_str).unwrap();

            let output_str = read_event("quote_asset_amount".to_string(), wasm).value;
            output = Uint128::from_str(&output_str).unwrap();
        }
        // TODO this is bad bit need to deal with it properly
        _ => {
            input = Uint128::zero();
            output = Uint128::zero();
        }
    }

    (input, output)
}

fn parse_pay_funding(response: SubMsgExecutionResponse) -> (Integer, String) {
    // Find swap inputs and output events
    let wasm = response.events.iter().find(|&e| e.ty == "wasm");
    let wasm = wasm.unwrap();

    let premium_str = read_event("premium_fraction".to_string(), wasm).value;
    let premium: Integer = Integer::from_str(&premium_str).unwrap();

    let sender = read_event("_contract_addr".to_string(), wasm).value;

    (premium, sender)
}

fn read_event(key: String, event: &Event) -> Attribute {
    let result = event
        .attributes
        .iter()
        .find(|&attr| attr.key == key)
        .unwrap();
    result.clone()
}
