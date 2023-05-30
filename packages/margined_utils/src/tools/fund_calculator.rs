use cosmwasm_std::{Addr, Coin, QuerierWrapper, StdResult, Uint128};
use margined_common::asset::ORAI_DENOM;
use margined_common::integer::Integer;
use margined_perp::margined_engine::{PnlCalcOption, Side};
use margined_perp::margined_vamm::Direction;

use crate::contracts::helpers::{EngineController, VammController};

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
    engine: Addr,
    trader: Addr,
    quote_asset_amount: Uint128,
    leverage: Uint128,
    side: Side,
    vamm: Addr,
) -> StdResult<Vec<Coin>> {
    let new_notional = quote_asset_amount
        .checked_mul(leverage)?
        .checked_div(SIX_D_P)?;

    let vamm_controller = VammController(vamm.clone());

    // pull the fees for the vamm that the position will be taken on; note that this will be shifted however many digits
    let fee_amount: Uint128 = vamm_controller.calc_fee(querier, new_notional)?.toll_fee;

    let engine_controller = EngineController(engine);

    // check if they have an existing position so we can calculate if someone owes margin
    let position = engine_controller
        .position(querier, vamm.to_string(), trader.to_string())
        .unwrap_or_default();

    // Check the unrealised PnL and add it to: (existing position + margin)
    let unrealized_pnl = engine_controller.get_unrealized_pnl(
        querier,
        vamm.to_string(),
        trader.to_string(),
        PnlCalcOption::SpotPrice,
    )?;

    // First we check if they are increasing the position or not
    let is_increase = position.direction == Direction::AddToAmm && side == Side::Buy
        || position.direction == Direction::RemoveFromAmm && side == Side::Sell;

    let margin_owed = match is_increase {
        true => Integer::new_positive(quote_asset_amount),
        false => {
            if new_notional > unrealized_pnl.position_notional {
                Integer::new_positive(
                    (new_notional - unrealized_pnl.position_notional)
                        .checked_mul(SIX_D_P)?
                        .checked_div(leverage)?,
                ) - Integer::new_positive(position.margin)
                    - unrealized_pnl.unrealized_pnl
            } else {
                Integer::zero()
            }
        }
    };

    let funds_owed = if margin_owed.is_positive() {
        fee_amount + margin_owed.value
    } else {
        fee_amount
    };

    if funds_owed.is_zero() {
        Ok(vec![])
    } else {
        Ok(vec![Coin::new(funds_owed.u128(), ORAI_DENOM)])
    }
}
