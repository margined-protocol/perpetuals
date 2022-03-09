#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use margined_perp::margined_vamm::{ExecuteMsg, InstantiateMsg, QueryMsg};

use crate::error::ContractError;
use crate::query::{
    query_calc_fee, query_input_twap, query_output_price, query_output_twap, query_spot_price,
    query_twap_price,
};
use crate::state::{store_reserve_snapshot, ReserveSnapshot};
use crate::{
    handle::{swap_input, swap_output, update_config},
    query::{query_config, query_state},
    state::{store_config, store_state, Config, State},
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        owner: info.sender,
        quote_asset: msg.quote_asset,
        base_asset: msg.base_asset,
        toll_ratio: msg.toll_ratio,
        spread_ratio: msg.spread_ratio,
        decimals: Uint128::from(10u128.pow(msg.decimals as u32)),
    };

    store_config(deps.storage, &config)?;

    let state = State {
        base_asset_reserve: msg.base_asset_reserve,
        quote_asset_reserve: msg.quote_asset_reserve,
        funding_rate: Uint128::zero(), // Initialise the funding rate as 0
        funding_period: msg.funding_period, // Funding period in seconds
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
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
            toll_ratio,
            spread_ratio,
        } => update_config(deps, info, owner, toll_ratio, spread_ratio),
        ExecuteMsg::SwapInput {
            direction,
            quote_asset_amount,
        } => swap_input(deps, env, info, direction, quote_asset_amount),
        ExecuteMsg::SwapOutput {
            direction,
            base_asset_amount,
        } => swap_output(deps, env, info, direction, base_asset_amount),
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
