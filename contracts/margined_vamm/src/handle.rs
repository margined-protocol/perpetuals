use cosmwasm_std::{
    attr, DepsMut, Env, MessageInfo, Response, StdResult, Storage,
};

use cosmwasm_bignumber::{Decimal256, Uint256};
use margined_perp::margined_vamm::Direction;
use crate::{
    error::ContractError,
    state::{
        State, STATE,
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
    let base_asset_amount = getInputPriceWithReserves(
        deps.storage,
        &direction,
        quote_asset_amount
    );

    updateReserve(
        deps.storage,
        direction,
        quote_asset_amount,
        base_asset_amount.unwrap()
    )?;


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
    let base_asset_amount = getOutputPriceWithReserves(
        deps.storage,
        &direction,
        quote_asset_amount
    );

    updateReserve(
        deps.storage,
        direction,
        quote_asset_amount,
        base_asset_amount.unwrap()
    )?;


    Ok(Response::new().add_attributes(vec![("action", "swap")]))
}

fn getInputPriceWithReserves(
    storage: &dyn Storage,
    direction: &Direction,
    quote_asset_amount: Uint256,
) -> StdResult<Uint256> {
    if quote_asset_amount == Uint256::zero() {
        Uint256::zero();
    }
    
    let state: State = STATE.load(storage)?;

    let invariant_k = state.quote_asset_reserve * state.base_asset_reserve;
    let quote_asset_after: Uint256;
    let base_asset_after: Uint256;

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

    base_asset_after = invariant_k / Decimal256::from_uint256(quote_asset_after);
    let base_asset_bought = if base_asset_after > state.base_asset_reserve {
        base_asset_after - state.base_asset_reserve
    } else {
        state.base_asset_reserve - base_asset_after
    };


    Ok(base_asset_bought)
}

fn getOutputPriceWithReserves(
    storage: &dyn Storage,
    direction: &Direction,
    base_asset_amount: Uint256,
) -> StdResult<Uint256> {
    if base_asset_amount == Uint256::zero() {
        Uint256::zero();
    }
    
    let state: State = STATE.load(storage)?;

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

fn updateReserve(
    storage: &mut dyn Storage,
    direction: Direction,
    quote_asset_amount: Uint256,
    base_asset_amount: Uint256,
) -> StdResult<Response> {
    let state: State = STATE.load(storage)?;
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

    STATE.save(storage, &update_state)?;

    Ok(Response::new().add_attributes(vec![("action", "update_reserve")]))
}
