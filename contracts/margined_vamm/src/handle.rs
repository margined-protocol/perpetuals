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


    Ok(Response::new().add_attributes(vec![("action", "swap")]))
}

// Function should only be called by the margin engine
pub fn swap_output(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    direction: Direction,
    quote_asset_amount: Uint128,
) -> Result<Response, ContractError> {
    let state: State = read_state(deps.storage)?;

    let base_asset_amount = get_output_price_with_reserves(
        &state,
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
        Direction::LONG => {
            quote_asset_after = state.quote_asset_reserve
                + quote_asset_amount;

        }
        Direction::SHORT => {
            quote_asset_after = state.quote_asset_reserve
                - quote_asset_amount;
        }
    }
    
    base_asset_after = invariant_k
        .checked_mul(state.decimals)?
        .checked_div(quote_asset_after)?;

    let base_asset_bought = if base_asset_after > state.base_asset_reserve {
        base_asset_after - state.base_asset_reserve
    } else {
        state.base_asset_reserve - base_asset_after
    };

    Ok(base_asset_bought)
}

fn get_output_price_with_reserves(
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
        Direction::LONG => {
            base_asset_after = state.base_asset_reserve
                + base_asset_amount;

        }
        Direction::SHORT => {
            base_asset_after = state.base_asset_reserve
                - base_asset_amount;
        }
    }

    quote_asset_after = invariant_k
        .checked_mul(state.decimals)?
        .checked_div(base_asset_after)?;

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
    quote_asset_amount: Uint128,
    base_asset_amount: Uint128,
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

#[test]
fn test_get_input_price() {
    let state = State {
        quote_asset_reserve: Uint128::from(1_000_000_000u128), // 1000
        base_asset_reserve: Uint128::from(100_000_000u128), // 100
        funding_rate: Uint128::from(1_000u128),
        funding_period: 3_600 as u64,
        decimals: Uint128::from(1_000_000u128), // equivalent to 6dp
    };

    // amount = 100(quote asset reserved) - (100 * 1000) / (1000 + 50) = 4.7619...
    // price = 50 / 4.7619 = 10.499
    let quote_asset_amount = Uint128::from(50_000_000u128);
    let result = get_input_price_with_reserves(
        &state,
        &Direction::LONG,
        quote_asset_amount
    ).unwrap();
    assert_eq!(result, Uint128::from(4761905u128));

    // amount = (100 * 1000) / (1000 - 50) - 100(quote asset reserved) = 5.2631578947368
    // price = 50 / 5.263 = 9.5
    let quote_asset_amount = Uint128::from(50_000_000u128);
    let result = get_input_price_with_reserves(
        &state,
        &Direction::SHORT,
        quote_asset_amount
    ).unwrap();
    assert_eq!(result, Uint128::from(5263157u128));

    // amount = 1000(base asset reversed) - (100 * 1000) / (100 + 5) = 47.619047619047619048
    // price = 47.619 / 5 = 9.52
    let quote_asset_amount = Uint128::from(5_000_000u128);
    let result = get_output_price_with_reserves(
        &state,
        &Direction::LONG,
        quote_asset_amount
    ).unwrap();
    assert_eq!(result, Uint128::from(47619048u128));

    // a dividable number should not plus 1 at mantissa
    let quote_asset_amount = Uint128::from(25_000_000u128);
    let result = get_output_price_with_reserves(
        &state,
        &Direction::LONG,
        quote_asset_amount
    ).unwrap();
    assert_eq!(result, Uint128::from(200_000_000u128));

    // amount = (100 * 1000) / (100 - 5) - 1000(base asset reversed) = 52.631578947368
    // price = 52.631 / 5 = 10.52
    let quote_asset_amount = Uint128::from(5_000_000u128);
    let result = get_output_price_with_reserves(
        &state,
        &Direction::SHORT,
        quote_asset_amount
    ).unwrap();
    assert_eq!(result, Uint128::from(52631578u128));

    // divisable output
    let quote_asset_amount = Uint128::from(37_500_000u128);
    let result = get_output_price_with_reserves(
        &state,
        &Direction::SHORT,
        quote_asset_amount
    ).unwrap();
    assert_eq!(result, Uint128::from(600_000_000u128));
}