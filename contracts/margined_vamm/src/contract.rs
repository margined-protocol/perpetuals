#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use margined_common::integer::Integer;
use margined_perp::margined_engine;
use margined_perp::margined_vamm::{ExecuteMsg, InstantiateMsg, QueryMsg};

use crate::error::ContractError;
use crate::query::{
    query_calc_fee, query_input_twap, query_output_price, query_output_twap, query_spot_price,
    query_twap_price,
};
use crate::{
    handle::{settle_funding, swap_input, swap_output, update_config},
    query::{query_config, query_state},
    state::{store_config, store_state, Config, State, store_reserve_snapshot, ReserveSnapshot},
};

pub const ONE_HOUR_IN_SECONDS: u64 = 60 * 60;
pub const ONE_DAY_IN_SECONDS: u64 = 24 * 60 * 60;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let mut config = Config {
        owner: info.sender,
        margin_engine: Addr::unchecked("".to_string()), // default to nothing, must be set
        quote_asset: msg.quote_asset,
        base_asset: msg.base_asset,
        toll_ratio: msg.toll_ratio,
        spread_ratio: msg.spread_ratio,
        pricefeed: deps.api.addr_validate(&msg.pricefeed).unwrap(),
        decimals: Uint128::from(10u128.pow(msg.decimals as u32)),
        spot_price_twap_interval: ONE_HOUR_IN_SECONDS, // default 1 hr
        funding_period: msg.funding_period,            // Funding period in seconds
        funding_buffer_period: msg.funding_period / 2u64,
    };

    // set and update margin engine
    let margin_engine = msg.margin_engine;
    if let Some(margin_engine) = margin_engine {
        config.margin_engine = deps.api.addr_validate(margin_engine.as_str())?;
    }

    store_config(deps.storage, &config)?;

    let state = State {
        base_asset_reserve: msg.base_asset_reserve,
        quote_asset_reserve: msg.quote_asset_reserve,
        total_position_size: Integer::default(), // it's 0 btw
        funding_rate: Uint128::zero(),           // Initialise the funding rate as 0
        next_funding_time: env.block.time.seconds()
            + msg.funding_period / ONE_HOUR_IN_SECONDS * ONE_HOUR_IN_SECONDS,
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
            toll_ratio,
            spread_ratio,
            margin_engine,
            pricefeed,
        } => update_config(
            deps,
            info,
            owner,
            toll_ratio,
            spread_ratio,
            margin_engine,
            pricefeed,
        ),
        ExecuteMsg::SwapInput {
            direction,
            quote_asset_amount,
        } => swap_input(deps, env, info, direction, quote_asset_amount),
        ExecuteMsg::SwapOutput {
            direction,
            base_asset_amount,
        } => swap_output(deps, env, info, direction, base_asset_amount),
        ExecuteMsg::SettleFunding {} => settle_funding(deps, env, info),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State {} => to_binary(&query_state(deps)?),
        QueryMsg::OutputPrice { direction, amount } => {
            to_binary(&query_output_price(deps, direction, amount)?)
        }
        QueryMsg::InputTwap { direction, amount } => {
            to_binary(&query_input_twap(deps, env, direction, amount)?)
        }
        QueryMsg::OutputTwap { direction, amount } => {
            to_binary(&query_output_twap(deps, env, direction, amount)?)
        }
        QueryMsg::CalcFee { quote_asset_amount } => {
            to_binary(&query_calc_fee(deps, quote_asset_amount)?)
        }
        QueryMsg::SpotPrice {} => to_binary(&query_spot_price(deps)?),
        QueryMsg::TwapPrice { interval } => to_binary(&query_twap_price(deps, env, interval)?),
    }
}
