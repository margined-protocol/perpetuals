#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;
use margined_common::{
    integer::Integer,
    validate::{validate_decimal_places, validate_ratio},
};
use margined_perp::margined_vamm::{ExecuteMsg, InstantiateMsg, QueryMsg};

use crate::error::ContractError;
use crate::querier::{query_underlying_price, query_underlying_twap_price};
use crate::{
    handle::{set_open, settle_funding, swap_input, swap_output, update_config},
    query::{
        query_calc_fee, query_config, query_input_amount, query_input_price, query_input_twap,
        query_is_over_spread_limit, query_output_amount, query_output_price, query_output_twap,
        query_spot_price, query_state, query_twap_price,
    },
    state::{store_config, store_reserve_snapshot, store_state, Config, ReserveSnapshot, State},
};

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "crates.io:margined-vamm";
/// Contract version that is used for migration.
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const MAX_ORACLE_SPREAD_RATIO: u64 = 100_000_000; // 0.1 i.e. 10%
pub const ONE_HOUR_IN_SECONDS: u64 = 60 * 60;
pub const ONE_DAY_IN_SECONDS: u64 = 24 * 60 * 60;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // check the decimal places supplied and ensure it is at least 6
    let decimals = validate_decimal_places(msg.decimals)?;

    validate_ratio(msg.toll_ratio, decimals)?;
    validate_ratio(msg.spread_ratio, decimals)?;
    validate_ratio(msg.fluctuation_limit_ratio, decimals)?;

    let mut config = Config {
        owner: info.sender,
        margin_engine: Addr::unchecked("".to_string()), // default to nothing, must be set
        quote_asset: msg.quote_asset,
        base_asset: msg.base_asset,
        base_asset_holding_cap: Uint128::zero(),
        open_interest_notional_cap: Uint128::zero(),
        toll_ratio: msg.toll_ratio,
        spread_ratio: msg.spread_ratio,
        fluctuation_limit_ratio: msg.fluctuation_limit_ratio,
        pricefeed: deps.api.addr_validate(&msg.pricefeed).unwrap(),
        decimals,
        spot_price_twap_interval: ONE_HOUR_IN_SECONDS,
        funding_period: msg.funding_period,
        funding_buffer_period: msg.funding_period / 2u64,
    };

    // set and update margin engine
    let margin_engine = msg.margin_engine;
    if let Some(margin_engine) = margin_engine {
        config.margin_engine = deps.api.addr_validate(margin_engine.as_str())?;
    }

    store_config(deps.storage, &config)?;

    let state = State {
        open: false,
        base_asset_reserve: msg.base_asset_reserve,
        quote_asset_reserve: msg.quote_asset_reserve,
        total_position_size: Integer::zero(),
        funding_rate: Integer::zero(),
        next_funding_time: 0u64,
    };

    store_state(deps.storage, &state)?;

    let reserve = ReserveSnapshot {
        base_asset_reserve: msg.base_asset_reserve,
        quote_asset_reserve: msg.quote_asset_reserve,
        timestamp: env.block.time,
        block_height: env.block.height,
    };

    store_reserve_snapshot(deps.storage, &reserve)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
            base_asset_holding_cap,
            open_interest_notional_cap,
            toll_ratio,
            spread_ratio,
            fluctuation_limit_ratio,
            margin_engine,
            pricefeed,
            spot_price_twap_interval,
        } => update_config(
            deps,
            info,
            owner,
            base_asset_holding_cap,
            open_interest_notional_cap,
            toll_ratio,
            spread_ratio,
            fluctuation_limit_ratio,
            margin_engine,
            pricefeed,
            spot_price_twap_interval,
        ),
        ExecuteMsg::SwapInput {
            direction,
            quote_asset_amount,
            can_go_over_fluctuation,
            base_asset_limit,
        } => swap_input(
            deps,
            env,
            info,
            direction,
            quote_asset_amount,
            base_asset_limit,
            can_go_over_fluctuation,
        ),
        ExecuteMsg::SwapOutput {
            direction,
            base_asset_amount,
            quote_asset_limit,
        } => swap_output(
            deps,
            env,
            info,
            direction,
            base_asset_amount,
            quote_asset_limit,
        ),
        ExecuteMsg::SettleFunding {} => settle_funding(deps, env, info),
        ExecuteMsg::SetOpen { open } => set_open(deps, env, info, open),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State {} => to_binary(&query_state(deps)?),
        QueryMsg::InputPrice { direction, amount } => {
            to_binary(&query_input_price(deps, direction, amount)?)
        }
        QueryMsg::OutputPrice { direction, amount } => {
            to_binary(&query_output_price(deps, direction, amount)?)
        }
        QueryMsg::InputAmount { direction, amount } => {
            to_binary(&query_input_amount(deps, direction, amount)?)
        }
        QueryMsg::OutputAmount { direction, amount } => {
            to_binary(&query_output_amount(deps, direction, amount)?)
        }
        QueryMsg::InputTwap { direction, amount } => {
            to_binary(&query_input_twap(deps, env, direction, amount)?)
        }
        QueryMsg::OutputTwap { direction, amount } => {
            to_binary(&query_output_twap(deps, env, direction, amount)?)
        }
        QueryMsg::UnderlyingPrice {} => to_binary(&query_underlying_price(&deps)?),
        QueryMsg::UnderlyingTwapPrice { interval } => {
            to_binary(&query_underlying_twap_price(&deps, interval)?)
        }
        QueryMsg::CalcFee { quote_asset_amount } => {
            to_binary(&query_calc_fee(deps, quote_asset_amount)?)
        }
        QueryMsg::SpotPrice {} => to_binary(&query_spot_price(deps)?),
        QueryMsg::TwapPrice { interval } => to_binary(&query_twap_price(deps, env, interval)?),
        QueryMsg::IsOverSpreadLimit {} => to_binary(&query_is_over_spread_limit(deps)?),
    }
}
