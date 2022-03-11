use cosmwasm_std::Uint128;
use cw_multi_test::Executor;
use margined_perp::margined_engine::Side;
use margined_utils::scenarios::SimpleScenario;

pub const DECIMAL_MULTIPLIER: Uint128 = Uint128::new(1_000_000_000);

// takes in a Uint128 and multiplies by the decimals just to make tests more legible
pub fn to_decimals(input: u64) -> Uint128 {
    Uint128::from(input) * DECIMAL_MULTIPLIER
}

// pub fn next_block(block: &mut BlockInfo) {
//     block.time = block.time.plus_seconds(10);
//     block.height += 1;
// }

#[test]
fn test_get_margin_ratio() {
    let SimpleScenario {
        mut router,
        alice,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::BUY,
            to_decimals(25u64),
            to_decimals(10u64),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // expect to be 0.1
    let margin_ratio = engine
        .get_margin_ratio(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(margin_ratio, Uint128::from(100_000_000u128));
}

#[test]
fn test_get_margin_ratio_long() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::BUY,
            to_decimals(25u64),
            to_decimals(10u64),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::SELL,
            to_decimals(15u64),
            to_decimals(10u64),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // expect to be 0.13429752
    // need to show a direction also probably
    let margin_ratio = engine
        .get_margin_ratio(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(margin_ratio, Uint128::from(134_297_520u128));
}

#[test]
fn test_get_margin_ratio_short() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::SELL,
            to_decimals(25u64),
            to_decimals(10u64),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::BUY,
            to_decimals(15u64),
            to_decimals(10u64),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // expect to be 0.287037037
    // need to show a direction also probably
    let margin_ratio = engine
        .get_margin_ratio(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(margin_ratio, Uint128::from(287_037_037u128));
}
