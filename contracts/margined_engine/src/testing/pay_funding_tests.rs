use cosmwasm_std::{Addr, Uint128};
use cw_multi_test::{App, Executor};
use margined_perp::margined_engine::Side;
use margined_utils::{scenarios::SimpleScenario, contracts::helpers::{EngineController, VammController}};

pub const DECIMAL_MULTIPLIER: Uint128 = Uint128::new(1_000_000_000);

// takes in a Uint128 and multiplies by the decimals just to make tests more legible
pub fn to_decimals(input: u64) -> Uint128 {
    Uint128::from(input) * DECIMAL_MULTIPLIER
}


#[test]
fn test_generate_loss_for_amm_when_funding_rate_is_positive_and_amm_is_long() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        owner,
        engine,
        vamm,
        usdc,
        pricefeed,
        ..
    } = SimpleScenario::new();

    let msg = engine
    .open_position(
        vamm.addr().to_string(),
        Side::BUY,
        to_decimals(300u64),
        to_decimals(2u64),
    )
    .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
    .open_position(
        vamm.addr().to_string(),
        Side::SELL,
        to_decimals(1200u64),
        to_decimals(1u64),
    )
    .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let engine_balance = usdc.balance(&router, engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, Uint128::from(1_500_000_000_000u128));

    let price: Uint128 = Uint128::from(1_590_000_000u128);
    let timestamp: u64 = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // move to the next funding time
    router.update_block(|block| {
        block.time = block.time.plus_seconds(3_600u64);
        block.height += 1;
    });

    let msg = engine
        .pay_funding(vamm.addr().to_string())
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();
}
