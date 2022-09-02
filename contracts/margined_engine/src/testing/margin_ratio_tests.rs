use cosmwasm_std::Uint128;
use cw_multi_test::Executor;
use margined_common::integer::Integer;
use margined_perp::margined_engine::Side;
use margined_utils::scenarios::{to_decimals, SimpleScenario};

pub const NEXT_FUNDING_PERIOD_DELTA: u64 = 86_400u64;

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
            Side::Buy,
            to_decimals(25u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // expect to be 0.1
    let margin_ratio = engine
        .get_margin_ratio(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(margin_ratio, Integer::new_positive(100_000_000u128));
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
            Side::Buy,
            to_decimals(25u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_positive(20_000_000_000u128));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(15u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let position = engine
        .position(&router, vamm.addr().to_string(), bob.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_negative(10_909_090_910u128));

    // expect to be -0.13429752
    let margin_ratio = engine
        .get_margin_ratio(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(margin_ratio, Integer::new_negative(134_297_520u128));
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
            Side::Sell,
            to_decimals(25u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(15u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // expect to be 0.287037037
    let margin_ratio = engine
        .get_margin_ratio(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(margin_ratio, Integer::new_negative(287_037_037u128));
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
            Side::Buy,
            to_decimals(25u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
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
            Side::Sell,
            to_decimals(15u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
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
    assert_eq!(margin_ratio, Integer::new_positive(96_890_936u128));
}

#[test]
fn test_verify_margin_ratio_funding_payment_positive() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        pricefeed,
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
            Side::Buy,
            to_decimals(25u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let price: Uint128 = Uint128::from(15_500_000_000u128);
    let timestamp: u64 = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // move to the next funding time
    router.update_block(|block| {
        block.time = block.time.plus_seconds(NEXT_FUNDING_PERIOD_DELTA);
        block.height += 1;
    });

    let msg = engine.pay_funding(vamm.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let premium_fraction = engine
        .get_latest_cumulative_premium_fraction(&router, vamm.addr().to_string())
        .unwrap();
    assert_eq!(premium_fraction, Integer::new_positive(125_000_000u128));

    // marginRatio = (margin + funding payment + unrealized Pnl) / positionNotional
    // funding payment: 20 * -12.5% = -2.5
    // position notional: 250
    // margin ratio: (25 - 2.5) / 250 = 0.09
    let margin_ratio = engine
        .get_margin_ratio(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(margin_ratio, Integer::new_positive(90_000_000u128));
}

#[test]
fn test_verify_margin_ratio_funding_payment_negative() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        pricefeed,
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
            Side::Buy,
            to_decimals(25u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let price: Uint128 = Uint128::from(15_700_000_000u128);
    let timestamp: u64 = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // move to the next funding time
    router.update_block(|block| {
        block.time = block.time.plus_seconds(NEXT_FUNDING_PERIOD_DELTA);
        block.height += 1;
    });

    let msg = engine.pay_funding(vamm.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let premium_fraction = engine
        .get_latest_cumulative_premium_fraction(&router, vamm.addr().to_string())
        .unwrap();
    assert_eq!(premium_fraction, Integer::new_negative(75_000_000u128));

    // marginRatio = (margin + funding payment + unrealized Pnl) / openNotional
    // funding payment: 20 * 7.5% = 1.5
    // position notional: 250
    // margin ratio: (25 + 1.5) / 250 =  0.106
    let margin_ratio = engine
        .get_margin_ratio(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(margin_ratio, Integer::new_positive(106_000_000u128));
}

#[test]
fn test_verify_margin_ratio_with_pnl_funding_payment_positive() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        pricefeed,
        ..
    } = SimpleScenario::new();

    // moves block forward 1 and 15 secs timestamp
    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    // price: 1250 / 80 = 15.625
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(25u64),
            to_decimals(10u64),
            to_decimals(20u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // price: 800 / 125 = 6.4
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(45u64),
            to_decimals(10u64),
            to_decimals(45u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // given the underlying twap price: 6.3
    let price: Uint128 = Uint128::from(6_300_000_000u128);
    let timestamp: u64 = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // move to the next funding time
    router.update_block(|block| {
        block.time = block.time.plus_seconds(NEXT_FUNDING_PERIOD_DELTA);
        block.height += 1;
    });

    let msg = engine.pay_funding(vamm.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let premium_fraction = engine
        .get_latest_cumulative_premium_fraction(&router, vamm.addr().to_string())
        .unwrap();
    assert_eq!(premium_fraction, Integer::new_positive(100_000_000u128));

    // marginRatio = (margin + funding payment + unrealized Pnl) / positionNotional
    // funding payment: 20 (position size) * -10% = -2
    // (800 - x) * (125 + 20) = 1000 * 100
    // position notional / x : 800 - 689.6551724138 = 110.3448275862
    // unrealized Pnl: 250 - 110.3448275862 = 139.6551724138
    // margin ratio: (25 - 2 - 139.6551724138) / 110.3448275862 = -1.0571875
    let margin_ratio = engine
        .get_margin_ratio(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(margin_ratio, Integer::new_negative(1_057_187_500u128));

    // funding payment (bob receives): 45 * 10% = 4.5
    // margin ratio: (45 + 4.5) / 450 = 0.11
    let margin_ratio = engine
        .get_margin_ratio(&router, vamm.addr().to_string(), bob.to_string())
        .unwrap();
    assert_eq!(margin_ratio, Integer::new_positive(110_000_000u128));
}

#[test]
fn test_verify_margin_ratio_with_pnl_funding_payment_negative() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        pricefeed,
        ..
    } = SimpleScenario::new();

    // moves block forward 1 and 15 secs timestamp
    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    // price: 1250 / 80 = 15.625
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(25u64),
            to_decimals(10u64),
            to_decimals(20u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // price: 800 / 125 = 6.4
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(45u64),
            to_decimals(10u64),
            to_decimals(45u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // given the underlying twap price: 6.5
    let price: Uint128 = Uint128::from(6_500_000_000u128);
    let timestamp: u64 = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // move to the next funding time
    router.update_block(|block| {
        block.time = block.time.plus_seconds(NEXT_FUNDING_PERIOD_DELTA);
        block.height += 1;
    });

    let msg = engine.pay_funding(vamm.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let premium_fraction = engine
        .get_latest_cumulative_premium_fraction(&router, vamm.addr().to_string())
        .unwrap();
    assert_eq!(premium_fraction, Integer::new_negative(100_000_000u128));

    // funding payment (alice receives): 20 (position size) * 10% = 2
    // (800 - x) * (125 + 20) = 1000 * 100
    // position notional / x : 800 - 689.6551724138 = 110.3448275862
    // unrealized Pnl: 250 - 110.3448275862 = 139.6551724138
    // margin ratio: (25 + 2 - 139.6551724138) / 110.3448275862 = -1.0209375
    let margin_ratio = engine
        .get_margin_ratio(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(margin_ratio, Integer::new_negative(1_020_937_500u128));

    // funding payment: 45 (position size) * -10% = -4.5
    // margin ratio: (45 - 4.5) / 450 = 0.09
    let margin_ratio = engine
        .get_margin_ratio(&router, vamm.addr().to_string(), bob.to_string())
        .unwrap();
    assert_eq!(margin_ratio, Integer::new_positive(90_000_000u128));
}
