use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, StdResult, Storage};
use cosmwasm_bignumber::{Decimal256};
use crate::{
    error::ContractError,
    state::{read_config, read_state, store_config, store_state, Config, State},
};
use margined_perp::margined_vamm::Direction;

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    toll_ratio: Option<Decimal256>,
    spread_ratio: Option<Decimal256>,
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
    _env: Env,
    _info: MessageInfo,
    direction: Direction,
    quote_asset_amount: Decimal256,
) -> Result<Response, ContractError> {
    let base_asset_amount =
        get_input_price_with_reserves(deps.as_ref(), &direction, quote_asset_amount)?;

    update_reserve(
        deps.storage,
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
    _env: Env,
    _info: MessageInfo,
    direction: Direction,
    base_asset_amount: Decimal256,
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
    quote_asset_amount: Decimal256,
) -> StdResult<Decimal256> {
    let state: State = read_state(deps.storage)?;
    // let config: Config = read_config(deps.storage)?;

    if quote_asset_amount == Decimal256::zero() {
        Decimal256::zero();
    }

    // k = x * y (divided by decimal places)
    let invariant_k = state.quote_asset_reserve * state.base_asset_reserve;

    let quote_asset_after: Decimal256;
    let base_asset_after: Decimal256;

    match direction {
        Direction::AddToAmm => {
            quote_asset_after = state.quote_asset_reserve + quote_asset_amount;
        }
        Direction::RemoveFromAmm => {
            quote_asset_after = state.quote_asset_reserve - quote_asset_amount;
        }
    }

    base_asset_after = invariant_k / quote_asset_after;

    let mut base_asset_bought = if base_asset_after > state.base_asset_reserve {
        base_asset_after - state.base_asset_reserve
    } else {
        state.base_asset_reserve - base_asset_after
    };

    let remainder = modulo(invariant_k, quote_asset_after);
    if remainder != Decimal256::zero() {
        if *direction == Direction::AddToAmm {
            base_asset_bought = base_asset_bought - Decimal256::one();
        } else {
            base_asset_bought = base_asset_bought + Decimal256::one();
        }
    }

    Ok(base_asset_bought)
}

pub fn get_output_price_with_reserves(
    deps: Deps,
    direction: &Direction,
    base_asset_amount: Decimal256,
) -> StdResult<Decimal256> {
    let state: State = read_state(deps.storage)?;
    // let config: Config = read_config(deps.storage)?;

    if base_asset_amount == Decimal256::zero() {
        Decimal256::zero();
    }
    let invariant_k = state
        .quote_asset_reserve * state.base_asset_reserve;

    let quote_asset_after: Decimal256;
    let base_asset_after: Decimal256;

    match direction {
        Direction::AddToAmm => {
            base_asset_after = state.base_asset_reserve + base_asset_amount;
        }
        Direction::RemoveFromAmm => {
            base_asset_after = state.base_asset_reserve - base_asset_amount;
        }
    }
    quote_asset_after = invariant_k / base_asset_after;

    let mut quote_asset_sold = if quote_asset_after > state.quote_asset_reserve {
        quote_asset_after - state.quote_asset_reserve
    } else {
        state.quote_asset_reserve - quote_asset_after
    };

    let remainder = modulo(invariant_k, base_asset_after);
    if remainder != Decimal256::zero() {
        if *direction == Direction::AddToAmm {
            quote_asset_sold = quote_asset_sold - Decimal256::one();
        } else {
            quote_asset_sold = quote_asset_sold + Decimal256::one();
        }
    }
    Ok(quote_asset_sold)
}

fn update_reserve(
    storage: &mut dyn Storage,
    direction: Direction,
    quote_asset_amount: Decimal256,
    base_asset_amount: Decimal256,
) -> StdResult<Response> {
    let state: State = read_state(storage)?;
    let mut update_state = state.clone();

    println!("State before:\n{:?}\n", state);

    match direction {
        Direction::AddToAmm => {
            update_state.quote_asset_reserve = update_state
                .quote_asset_reserve
                + quote_asset_amount;
            update_state.base_asset_reserve =
                state.base_asset_reserve - base_asset_amount;
        }
        Direction::RemoveFromAmm => {
            update_state.base_asset_reserve = update_state
                .base_asset_reserve
                + base_asset_amount;
            update_state.quote_asset_reserve =
                state.quote_asset_reserve - quote_asset_amount;
        }
    }

    store_state(storage, &update_state)?;
    println!("State after:\n{:?}\n", update_state);

    Ok(Response::new().add_attributes(vec![("action", "update_reserve")]))
}

/// Does the modulus (%) operator on Decimal256.
/// However it follows the design of the perpertual protocol decimals
/// https://github.com/perpetual-protocol/perpetual-protocol/blob/release/v2.1.x/src/utils/Decimal.sol
fn modulo(a: Decimal256, b: Decimal256) -> Decimal256 {
    // TODO the decimals are currently hardcoded to 9dp, this needs to change in the future but without
    // needing to pass the entire world to this function, i.e. access to storage
    // let decimal : Uint256 = Uint256::from(1_000_000_000u128);
    // let a_decimals = a;
    let integral = a / b;
    a - (b * integral)
}
