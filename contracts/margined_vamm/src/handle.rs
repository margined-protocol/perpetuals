use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, StdResult, Storage, Uint128};

use crate::{
    decimals::modulo,
    error::ContractError,
    state::{
        read_config, read_state, store_config, store_reserve_snapshot, store_state, Config,
        ReserveSnapshot, State,
    },
};
use margined_perp::margined_vamm::Direction;

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    toll_ratio: Option<Uint128>,
    spread_ratio: Option<Uint128>,
) -> Result<Response, ContractError> {
    let mut config: Config = read_config(deps.storage)?;

    // check permission
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // change owner of amm
    if let Some(owner) = owner {
        config.owner = deps.api.addr_validate(owner.as_str())?;
    }

    // change toll ratio
    if let Some(toll_ratio) = toll_ratio {
        config.toll_ratio = toll_ratio;
    }

    // change spread ratio
    if let Some(spread_ratio) = spread_ratio {
        config.spread_ratio = spread_ratio;
    }

    store_config(deps.storage, &config)?;

    Ok(Response::default())
}

// Function should only be called by the margin engine
pub fn swap_input(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    direction: Direction,
    quote_asset_amount: Uint128,
) -> Result<Response, ContractError> {
    let base_asset_amount =
        get_input_price_with_reserves(deps.as_ref(), &direction, quote_asset_amount)?;

    update_reserve(
        deps.storage,
        env,
        direction,
        quote_asset_amount,
        base_asset_amount,
    )?;

    Ok(Response::new().add_attributes(vec![
        ("action", "swap_input"),
        ("input", &quote_asset_amount.to_string()),
        ("output", &base_asset_amount.to_string()),
    ]))
}

// Function should only be called by the margin engine
pub fn swap_output(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    direction: Direction,
    base_asset_amount: Uint128,
) -> Result<Response, ContractError> {
    let quote_asset_amount =
        get_output_price_with_reserves(deps.as_ref(), &direction, base_asset_amount)?;

    // flip direction when updating reserve
    let mut update_direction = direction;
    if update_direction == Direction::AddToAmm {
        update_direction = Direction::RemoveFromAmm;
    } else {
        update_direction = Direction::AddToAmm;
    }

    update_reserve(
        deps.storage,
        env,
        update_direction,
        quote_asset_amount,
        base_asset_amount,
    )?;

    Ok(Response::new().add_attributes(vec![
        ("action", "swap_output"),
        ("input", &base_asset_amount.to_string()),
        ("output", &quote_asset_amount.to_string()),
    ]))
}

pub fn get_input_price_with_reserves(
    deps: Deps,
    direction: &Direction,
    quote_asset_amount: Uint128,
) -> StdResult<Uint128> {
    let state: State = read_state(deps.storage)?;
    let config: Config = read_config(deps.storage)?;



    if quote_asset_amount == Uint128::zero() {
        Uint128::zero();
    }

    // k = x * y (divided by decimal places)
    let invariant_k = state
        .quote_asset_reserve
        .checked_mul(state.base_asset_reserve)?
        .checked_div(config.decimals)?;

    let quote_asset_after: Uint128 = match direction {
        Direction::AddToAmm => {
            state.quote_asset_reserve.checked_add(quote_asset_amount)?
        }
        Direction::RemoveFromAmm => {
            state.quote_asset_reserve.checked_sub(quote_asset_amount)?
        }
    };

    let base_asset_after: Uint128 = invariant_k
        .checked_mul(config.decimals)?
        .checked_div(quote_asset_after)?;

    let mut base_asset_bought = if base_asset_after > state.base_asset_reserve {
        base_asset_after - state.base_asset_reserve
    } else {
        state.base_asset_reserve - base_asset_after
    };

    let remainder = modulo(invariant_k, quote_asset_after);
    if remainder != Uint128::zero() {
        if *direction == Direction::AddToAmm {
            base_asset_bought = base_asset_bought.checked_sub(Uint128::new(1u128))?;
        } else {
            base_asset_bought = base_asset_bought.checked_add(Uint128::from(1u128))?;
        }
    }

    Ok(base_asset_bought)
}

pub fn get_output_price_with_reserves(
    deps: Deps,
    direction: &Direction,
    base_asset_amount: Uint128,
) -> StdResult<Uint128> {
    let state: State = read_state(deps.storage)?;
    let config: Config = read_config(deps.storage)?;

    if base_asset_amount == Uint128::zero() {
        Uint128::zero();
    }
    let invariant_k = state
        .quote_asset_reserve
        .checked_mul(state.base_asset_reserve)?
        .checked_div(config.decimals)?;

    let base_asset_after: Uint128 = match direction {
        Direction::AddToAmm => {
            state.base_asset_reserve.checked_add(base_asset_amount)?
        }
        Direction::RemoveFromAmm => {
            state.base_asset_reserve.checked_sub(base_asset_amount)?
        }
    };
    
    let quote_asset_after: Uint128 = invariant_k
        .checked_mul(config.decimals)?
        .checked_div(base_asset_after)?;

    let mut quote_asset_sold = if quote_asset_after > state.quote_asset_reserve {
        quote_asset_after - state.quote_asset_reserve
    } else {
        state.quote_asset_reserve - quote_asset_after
    };

    let remainder = modulo(invariant_k, base_asset_after);
    if remainder != Uint128::zero() {
        if *direction == Direction::AddToAmm {
            quote_asset_sold = quote_asset_sold.checked_sub(Uint128::from(1u128))?;
        } else {
            quote_asset_sold = quote_asset_sold.checked_add(Uint128::new(1u128))?;
        }
    }
    Ok(quote_asset_sold)
}

fn update_reserve(
    storage: &mut dyn Storage,
    env: Env,
    direction: Direction,
    quote_asset_amount: Uint128,
    base_asset_amount: Uint128,
) -> StdResult<Response> {
    let state: State = read_state(storage)?;
    let mut update_state = state.clone();

    match direction {
        Direction::AddToAmm => {
            update_state.quote_asset_reserve = update_state
                .quote_asset_reserve
                .checked_add(quote_asset_amount)?;
            update_state.base_asset_reserve =
                state.base_asset_reserve.checked_sub(base_asset_amount)?;
        }
        Direction::RemoveFromAmm => {
            update_state.base_asset_reserve = update_state
                .base_asset_reserve
                .checked_add(base_asset_amount)?;
            update_state.quote_asset_reserve =
                state.quote_asset_reserve.checked_sub(quote_asset_amount)?;
        }
    }

    store_state(storage, &update_state)?;

    add_reserve_snapshot(
        storage,
        env,
        update_state.quote_asset_reserve,
        update_state.base_asset_reserve,
    )?;

    Ok(Response::new().add_attributes(vec![("action", "update_reserve")]))
}

fn add_reserve_snapshot(
    storage: &mut dyn Storage,
    env: Env,
    quote_asset_reserve: Uint128,
    base_asset_reserve: Uint128,
) -> StdResult<Response> {
    let new_snapshot = ReserveSnapshot {
        quote_asset_reserve,
        base_asset_reserve,
        timestamp: env.block.time,
        block_height: env.block.height,
    };
    store_reserve_snapshot(storage, &new_snapshot)?;

    Ok(Response::default())
}
