use cosmwasm_std::{StdResult, Uint128};
use margined_perp::margined_vamm::Direction;

pub fn get_input_price_with_reserves(
    direction: &Direction,
    quote_asset_amount: Uint128,
    quote_asset_reserve: Uint128,
    base_asset_reserve: Uint128,
) -> StdResult<Uint128> {
    if quote_asset_amount == Uint128::zero() {
        return Ok(Uint128::zero());
    }

    // k = x * y
    let invariant_k = quote_asset_reserve
        .checked_mul(base_asset_reserve)?;

    let quote_asset_after = match direction {
        Direction::AddToAmm => quote_asset_reserve.checked_add(quote_asset_amount)?,
        Direction::RemoveFromAmm => quote_asset_reserve.checked_sub(quote_asset_amount)?,
    };

    let base_asset_after = invariant_k.checked_div(quote_asset_after)?;
    let mut base_asset_bought = base_asset_after.abs_diff(base_asset_reserve);

    // follows the design of the perpetual protocol decimals
    // https://github.com/perpetual-protocol/perpetual-protocol/blob/release/v2.1.x/src/utils/Decimal.sol
    let remainder = invariant_k.checked_rem(quote_asset_after)?;
    if remainder != Uint128::zero() {
        if *direction == Direction::AddToAmm {
            base_asset_bought = base_asset_bought.checked_sub(1u128.into())?;
        } else {
            base_asset_bought = base_asset_bought.checked_add(1u128.into())?;
        }
    }

    Ok(base_asset_bought)
}

pub fn get_output_price_with_reserves(
    direction: &Direction,
    base_asset_amount: Uint128,
    quote_asset_reserve: Uint128,
    base_asset_reserve: Uint128,
) -> StdResult<Uint128> {
    if base_asset_amount == Uint128::zero() {
        return Ok(Uint128::zero());
    }
    let invariant_k = quote_asset_reserve
        .checked_mul(base_asset_reserve)?;

    let base_asset_after = match direction {
        Direction::AddToAmm => base_asset_reserve.checked_add(base_asset_amount)?,
        Direction::RemoveFromAmm => base_asset_reserve.checked_sub(base_asset_amount)?,
    };

    let quote_asset_after = invariant_k.checked_div(base_asset_after)?;
    let mut quote_asset_sold = quote_asset_after.abs_diff(quote_asset_reserve);
    // follows the design of the perpetual protocol decimals
    // https://github.com/perpetual-protocol/perpetual-protocol/blob/release/v2.1.x/src/utils/Decimal.sol
    let remainder = invariant_k.checked_rem(base_asset_after)?;
    if remainder != Uint128::zero() {
        if *direction == Direction::AddToAmm {
            quote_asset_sold = quote_asset_sold.checked_sub(1u128.into())?;
        } else {
            quote_asset_sold = quote_asset_sold.checked_add(1u128.into())?;
        }
    }
    Ok(quote_asset_sold)
}
