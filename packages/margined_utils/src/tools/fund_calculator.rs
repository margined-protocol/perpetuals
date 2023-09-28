use cosmwasm_std::{Addr, Coin, QuerierWrapper, StdResult, Uint128};
use margined_common::asset::NATIVE_DENOM;
use margined_common::integer::Integer;

use crate::contracts::helpers::VammController;

//////////////////////////////////////////////////////////////////////////
/// This is a tool for calculating the total funds needed when we want
/// to send a transaction using the native token, as the total amount  
/// needs to be known beforehand. This function need to know the       
/// existing position of the trader, and the vamm (to pull fees) which
/// they want to open a position on.                                   
//////////////////////////////////////////////////////////////////////////

// note that this tool calculates if someone is owed margin, but does not deduct the amount from the fees owned

pub const SIX_D_P: Uint128 = Uint128::new(1_000_000u128); // this is 6d.p.

pub fn calculate_funds_needed(
    querier: &QuerierWrapper,
    quote_asset_amount: Uint128,
    leverage: Uint128,
    vamm: Addr,
) -> StdResult<Vec<Coin>> {
    let new_notional = quote_asset_amount
        .checked_mul(leverage)?
        .checked_div(SIX_D_P)?;
    let vamm_controller = VammController(vamm.clone());

    // pull the fees for the vamm that the position will be taken on; note that this will be shifted however many digits
    let fee_amount = vamm_controller.calc_fee(querier, new_notional)?.toll_fee;
    let margin_owed = Integer::new_positive(quote_asset_amount);
    let funds_owed = if margin_owed.is_positive() {
        margin_owed.value
    } else {
        fee_amount
    };
    if funds_owed.is_zero() {
        Ok(vec![])
    } else {
        Ok(vec![Coin::new(funds_owed.u128(), NATIVE_DENOM)])
    }
}
