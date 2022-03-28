use cosmwasm_std::Uint128;
use cw_multi_test::Executor;
use margined_perp::margined_engine::Side;
use margined_utils::scenarios::{to_decimals, SimpleScenario};

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
            to_decimals(0u64),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // expect to be 0.1
    let margin_ratio = engine
        .get_margin_ratio(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(margin_ratio.ratio, Uint128::from(100_000_000u128));
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
            to_decimals(0u64),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::SELL,
            to_decimals(15u64),
            to_decimals(10u64),
            to_decimals(0u64),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // expect to be -0.13429752
    let margin_ratio = engine
        .get_margin_ratio(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(margin_ratio.ratio, Uint128::from(134_297_520u128));
    assert_eq!(margin_ratio.polarity, false);
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
            to_decimals(0u64),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::BUY,
            to_decimals(15u64),
            to_decimals(10u64),
            to_decimals(0u64),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // expect to be 0.287037037
    let margin_ratio = engine
        .get_margin_ratio(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(margin_ratio.ratio, Uint128::from(287_037_037u128));
    assert_eq!(margin_ratio.polarity, false);
}

#[test]
fn test_get_margin_higher_twap() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    // moves block forward 1 and 15 secs timestamp
    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::BUY,
            to_decimals(25u64),
            to_decimals(10u64),
            to_decimals(0u64),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15 * 62);
        block.height += 62;
    });

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::SELL,
            to_decimals(15u64),
            to_decimals(10u64),
            to_decimals(0u64),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // moves block forward 1 and 15 secs timestamp
    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    // expect to be 0.09689093601
    let margin_ratio = engine
        .get_margin_ratio(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(margin_ratio.ratio, Uint128::from(96_890_936u128));
    assert_eq!(margin_ratio.polarity, true);
}
