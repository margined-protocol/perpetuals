use cosmwasm_std::{
    attr, DepsMut, Env, MessageInfo, Response, StdResult, Storage,
};

use cosmwasm_bignumber::{Decimal256, Uint256};
use margined_perp::margined_vamm::Direction;
use crate::{
    error::ContractError,
    state::{
        State, read_state, store_state,
    },
};

// Function should only be called by the margin engine
pub fn swap_input(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    direction: Direction,
    quote_asset_amount: Uint256,
) -> Result<Response, ContractError> {
    let state: State = read_state(deps.storage)?;
    println!("TEST");
    // let base_asset_amount = get_input_price_with_reserves(
    //     deps.storage,
    //     &direction,
    //     quote_asset_amount
    // )?;
    // println!("Base Asset Amount: {:?}", base_asset_amount);

    // update_reserve(
    //     deps.storage,
    //     direction,
    //     quote_asset_amount,
    //     base_asset_amount
    // )?;


    Ok(Response::new().add_attributes(vec![("action", "swap")]))
}

// Function should only be called by the margin engine
pub fn swap_output(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    direction: Direction,
    quote_asset_amount: Uint256,
) -> Result<Response, ContractError> {
    let base_asset_amount = get_output_price_with_reserves(
        deps.storage,
        &direction,
        quote_asset_amount
    );

    update_reserve(
        deps.storage,
        direction,
        quote_asset_amount,
        base_asset_amount.unwrap()
    )?;


    Ok(Response::new().add_attributes(vec![("action", "swap")]))
}

fn get_input_price_with_reserves(
    storage: &mut dyn Storage,
    direction: &Direction,
    quote_asset_amount: Uint256,
) -> StdResult<Uint256> {
    if quote_asset_amount == Uint256::zero() {
        Uint256::zero();
    }
    println!("HERE");
    let state: State = read_state(storage)?;
    println!("HERE");
    let invariant_k = state.quote_asset_reserve * state.base_asset_reserve;
    let quote_asset_after: Uint256;
    let base_asset_after: Uint256;

    println!("HERE");
    match direction {
        Direction::LONG => {
            quote_asset_after = state.quote_asset_reserve
                + quote_asset_amount;

        }
        Direction::SHORT => {
            quote_asset_after = state.quote_asset_reserve
                - quote_asset_amount;
        }
    }
    println!("{:?}", invariant_k);
    base_asset_after = invariant_k / Decimal256::from_uint256(quote_asset_after);
    let base_asset_bought = if base_asset_after > state.base_asset_reserve {
        base_asset_after - state.base_asset_reserve
    } else {
        state.base_asset_reserve - base_asset_after
    };
    println!("{:?}", base_asset_bought);


    Ok(base_asset_bought)
}

fn get_output_price_with_reserves(
    storage: &mut dyn Storage,
    direction: &Direction,
    base_asset_amount: Uint256,
) -> StdResult<Uint256> {
    if base_asset_amount == Uint256::zero() {
        Uint256::zero();
    }
    
    let state: State = read_state(storage)?;

    let invariant_k = state.quote_asset_reserve * state.base_asset_reserve;
    let quote_asset_after: Uint256;
    let base_asset_after: Uint256;

    match direction {
        Direction::LONG => {
            base_asset_after = state.base_asset_reserve
                + base_asset_amount;

        }
        Direction::SHORT => {
            base_asset_after = state.base_asset_reserve
                - base_asset_amount;
        }
    }

    quote_asset_after = invariant_k / Decimal256::from_uint256(base_asset_after);
    let quote_asset_sold = if quote_asset_after > state.quote_asset_reserve {
        quote_asset_after - state.quote_asset_reserve
    } else {
        state.quote_asset_reserve - quote_asset_after
    };


    Ok(quote_asset_sold)
}

fn update_reserve(
    storage: &mut dyn Storage,
    direction: Direction,
    quote_asset_amount: Uint256,
    base_asset_amount: Uint256,
) -> StdResult<Response> {
    let state: State = read_state(storage)?;
    let mut update_state = state.clone();

    match direction {
        Direction::LONG => {
            update_state.quote_asset_reserve += quote_asset_amount;
            update_state.base_asset_reserve = state.base_asset_reserve - base_asset_amount;
        }
        Direction::SHORT => {
            update_state.base_asset_reserve += base_asset_amount;
            update_state.quote_asset_reserve = state.quote_asset_reserve - quote_asset_amount;
        }
    }

    store_state(storage, &update_state)?;

    Ok(Response::new().add_attributes(vec![("action", "update_reserve")]))
}
