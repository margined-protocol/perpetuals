use cosmwasm_std::{
    Addr, DepsMut, Env, MessageInfo, Response, StdResult, Storage, Uint128,
};

use margined_perp::margined_vamm::Direction;
use crate::{
    error::ContractError,
    state::{
        Config, read_config, store_config,
        State, read_state, store_state,
    },
};


pub fn update_config(
    deps: DepsMut,
    _info: MessageInfo,
    owner: String,
) -> Result<Response, ContractError> {
    let config = read_config(deps.storage)?;

    let new_config = Config {
        owner: Addr::unchecked(owner),
        quote_asset: config.quote_asset,
        base_asset: config.base_asset,
    };

    store_config(deps.storage, &new_config)?;

    Ok(Response::default())
}


// Function should only be called by the margin engine
pub fn swap_input(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    direction: Direction,
    quote_asset_amount: Uint128,
) -> Result<Response, ContractError> {
    let state: State = read_state(deps.storage)?;

    let base_asset_amount = get_input_price_with_reserves(
        &state,
        &direction,
        quote_asset_amount
    )?;

    update_reserve(
        deps.storage,
        direction,
        quote_asset_amount,
        base_asset_amount
    )?;

    Ok(Response::new()
        .add_attributes(vec![
            ("action", "swap_input"),
            ("input", &quote_asset_amount.to_string()),
            ("output", &base_asset_amount.to_string()),
        ])
    )
}

// Function should only be called by the margin engine
pub fn swap_output(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    direction: Direction,
    base_asset_amount: Uint128,
) -> Result<Response, ContractError> {
    let state: State = read_state(deps.storage)?;

    let quote_asset_amount = get_output_price_with_reserves(
        &state,
        &direction,
        base_asset_amount
    )?;

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


    Ok(Response::new()
        .add_attributes(vec![
            ("action", "swap_output"),
            ("input", &base_asset_amount.to_string()),
            ("output", &quote_asset_amount.to_string()),
        ])
    )
}

pub fn get_input_price_with_reserves(
    state: &State,
    direction: &Direction,
    quote_asset_amount: Uint128,
) -> StdResult<Uint128> {
    if quote_asset_amount == Uint128::zero() {
        Uint128::zero();
    }

    // k = x * y (divided by decimal places)
    let invariant_k = state.quote_asset_reserve
        .checked_mul(state.base_asset_reserve)?
        .checked_div(state.decimals)?;

    let quote_asset_after: Uint128;
    let base_asset_after: Uint128;

    match direction {
        Direction::AddToAmm => {
            quote_asset_after = state.quote_asset_reserve
                .checked_add(quote_asset_amount)?;

        }
        Direction::RemoveFromAmm => {
            quote_asset_after = state.quote_asset_reserve
                .checked_sub(quote_asset_amount)?;
        }
    }
    
    base_asset_after = invariant_k
        .checked_mul(state.decimals)?
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
    state: &State,
    direction: &Direction,
    base_asset_amount: Uint128,
) -> StdResult<Uint128> {
    if base_asset_amount == Uint128::zero() {
        Uint128::zero();
    }
    
    let invariant_k = state.quote_asset_reserve 
        .checked_mul(state.base_asset_reserve)?
        .checked_div(state.decimals)?;

    let quote_asset_after: Uint128;
    let base_asset_after: Uint128;

    match direction {
        Direction::AddToAmm => {
            base_asset_after = state.base_asset_reserve
                .checked_add(base_asset_amount)?;
        }
        Direction::RemoveFromAmm => {
            base_asset_after = state.base_asset_reserve
                .checked_sub(base_asset_amount)?;
        }
    }

    quote_asset_after = invariant_k
        .checked_mul(state.decimals)?
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
    direction: Direction,
    quote_asset_amount: Uint128,
    base_asset_amount: Uint128,
) -> StdResult<Response> {
    let state: State = read_state(storage)?;
    let mut update_state = state.clone();
    
    match direction {
        Direction::AddToAmm => {
            update_state.quote_asset_reserve = update_state.quote_asset_reserve
                .checked_add(quote_asset_amount)?;
            update_state.base_asset_reserve = state.base_asset_reserve
                .checked_sub(base_asset_amount)?;
        }
        Direction::RemoveFromAmm => {
            update_state.base_asset_reserve = update_state.base_asset_reserve
                .checked_add(base_asset_amount)?;
            update_state.quote_asset_reserve = state.quote_asset_reserve
                .checked_sub(quote_asset_amount)?;
        }
    }

    println!("Quote Asset Reserve: {}", update_state.quote_asset_reserve);
    println!("Base Asset Reserve: {}", update_state.base_asset_reserve);

    store_state(storage, &update_state)?;

    Ok(Response::new().add_attributes(vec![("action", "update_reserve")]))
}

/// Does the modulus (%) operator on Uin128.
/// However it follows the design of the perpertual protocol decimals
/// https://github.com/perpetual-protocol/perpetual-protocol/blob/release/v2.1.x/src/utils/Decimal.sol
fn modulo(
    a: Uint128,
    b: Uint128,
) -> Uint128 {
    // TODO the decimals are currently hardcoded to 9dp, this needs to change in the future but without
    // needing to pass the entire world to this function, i.e. access to storage
    let a_decimals = a.checked_mul(Uint128::from(1_000_000_000u128)).unwrap();
    let integral = a_decimals / b;
    return a_decimals - (b*integral)
}
