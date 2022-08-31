use cosmwasm_std::{Empty, Uint128};
use cw_multi_test::Executor;
use margined_common::integer::Integer;
use margined_perp::margined_engine::Side;
use margined_utils::scenarios::{to_decimals, SimpleScenario};

pub const NEXT_FUNDING_PERIOD_DELTA: u64 = 86_400u64;

#[test]
fn test_add_margin() {
    let SimpleScenario {
        mut router,
        alice,
        engine,
        vamm,
        usdc,
        ..
    } = SimpleScenario::new();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = usdc
        .balance::<_, _, Empty>(&router, engine.addr().clone())
        .unwrap();
    assert_eq!(engine_balance, to_decimals(60u64));

    let msg = engine
        .deposit_margin(vamm.addr().to_string(), to_decimals(80u64), vec![])
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = usdc
        .balance::<_, _, Empty>(&router, engine.addr().clone())
        .unwrap();
    assert_eq!(engine_balance, to_decimals(140u64));

    let alice_position = engine
        .get_position_with_funding_payment(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(alice_position.margin, to_decimals(140u64),);
    assert_eq!(
        engine
            .get_balance_with_funding_payment(&router, alice.to_string())
            .unwrap(),
        to_decimals(140u64),
    );
}

#[test]
fn test_add_margin_insufficent_balance() {
    let SimpleScenario {
        mut router,
        alice,
        engine,
        vamm,
        usdc,
        ..
    } = SimpleScenario::new();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = usdc
        .balance::<_, _, Empty>(&router, engine.addr().clone())
        .unwrap();
    assert_eq!(engine_balance, to_decimals(60u64));

    let msg = engine
        .deposit_margin(vamm.addr().to_string(), to_decimals(5001u64), vec![])
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: transfer failure - reply (id 9)"
    );
}

#[test]
fn test_add_margin_no_open_position() {
    let SimpleScenario {
        mut router,
        alice,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    let msg = engine
        .deposit_margin(vamm.addr().to_string(), to_decimals(80u64), vec![])
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_remove_margin() {
    let SimpleScenario {
        mut router,
        alice,
        engine,
        vamm,
        usdc,
        ..
    } = SimpleScenario::new();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = usdc
        .balance::<_, _, Empty>(&router, engine.addr().clone())
        .unwrap();
    assert_eq!(engine_balance, to_decimals(60u64));

    let free_collateral = engine
        .get_free_collateral(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(free_collateral, Integer::new_positive(30_000_000_000u128));

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), to_decimals(20u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = usdc
        .balance::<_, _, Empty>(&router, engine.addr().clone())
        .unwrap();
    assert_eq!(engine_balance, to_decimals(40u64));

    let alice_position = engine
        .get_position_with_funding_payment(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(alice_position.margin, to_decimals(40u64),);
    assert_eq!(
        engine
            .get_balance_with_funding_payment(&router, alice.to_string())
            .unwrap(),
        to_decimals(40u64),
    );
}

#[test]
fn test_remove_margin_after_paying_funding() {
    let SimpleScenario {
        mut router,
        alice,
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
            Side::Buy,
            to_decimals(60u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = usdc
        .balance::<_, _, Empty>(&router, engine.addr().clone())
        .unwrap();
    assert_eq!(engine_balance, to_decimals(60u64));

    let price: Uint128 = Uint128::from(25_500_000_000u128);
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

    // funding payment is -3.75
    let msg = engine.pay_funding(vamm.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), to_decimals(20u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = usdc
        .balance::<_, _, Empty>(&router, engine.addr().clone())
        .unwrap();
    assert_eq!(engine_balance, Uint128::from(36_250_000_000u128));

    let alice_position = engine
        .get_position_with_funding_payment(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(alice_position.margin, Uint128::from(36_250_000_000u128),);
    assert_eq!(
        engine
            .get_balance_with_funding_payment(&router, alice.to_string())
            .unwrap(),
        Uint128::from(36_250_000_000u128),
    );
}

#[test]
fn test_remove_margin_insufficient_margin() {
    let SimpleScenario {
        mut router,
        alice,
        engine,
        vamm,
        usdc,
        ..
    } = SimpleScenario::new();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = usdc
        .balance::<_, _, Empty>(&router, engine.addr().clone())
        .unwrap();
    assert_eq!(engine_balance, to_decimals(60u64));

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), to_decimals(61u64))
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Insufficient margin"
    );
}

#[test]
fn test_remove_margin_incorrect_ratio_four_percent() {
    let SimpleScenario {
        mut router,
        alice,
        engine,
        vamm,
        usdc,
        ..
    } = SimpleScenario::new();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = usdc
        .balance::<_, _, Empty>(&router, engine.addr().clone())
        .unwrap();
    assert_eq!(engine_balance, to_decimals(60u64));

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), to_decimals(36u64))
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Insufficient collateral"
    );
}

#[test]
fn test_remove_margin_unrealized_pnl_long_position_with_profit_using_spot_price() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    // reserve 1000 : 100
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(5u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
    // reserve 1300 : 76.92, price = 16.9

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(5u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();
    // reserve 1600 : 62.5, price = 25.6

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(45_010_000_000u128))
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Insufficient collateral"
    );

    let free_collateral = engine
        .get_free_collateral(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(free_collateral, Integer::new_positive(45_000_000_000u128));

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(45_000_000_000u128))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_remove_margin_unrealized_pnl_long_position_with_loss_using_spot_price() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    // reserve 1000 : 100
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(5u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
    // reserve 1300 : 76.92, price = 16.9

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(10u64),
            to_decimals(5u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();
    // reserve 1250 : 80 price = 15.625

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(24_900_000_000u128))
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Insufficient collateral"
    );

    let free_collateral = engine
        .get_free_collateral(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(free_collateral, Integer::new_positive(24_850_746_257u128));

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(24_850_746_257u128))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}
#[test]
fn test_remove_margin_unrealized_pnl_short_position_with_profit_using_spot_price() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    // reserve 1000 : 100
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(5u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
    // reserve 900 : 111.11, price = 8.1

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(5u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();
    // reserve 800 : 125, price = 6.4

    // margin: 20
    // positionSize: -11.11
    // positionNotional: 78.04
    // unrealizedPnl: 100 - 78.04 = 21.96
    // min(margin + funding, margin + funding + unrealized PnL) - position value * 5%
    // min(20, 20 + 21.96) - 78.04 * 0.05 = 16.098
    // can not remove margin > 16.098
    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(16_500_000_000u128))
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Insufficient collateral"
    );

    let free_collateral = engine
        .get_free_collateral(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(free_collateral, Integer::new_positive(16_097_560_976u128));

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(16_097_560_976u128))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_remove_margin_unrealized_pnl_short_position_with_loss_using_spot_price() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    // reserve 1000 : 100
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(5u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(10u64),
            to_decimals(5u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();
    // reserve 800 : 125, price = 6.4

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(2_500_000_000u128))
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Insufficient collateral"
    );

    let free_collateral = engine
        .get_free_collateral(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(free_collateral, Integer::new_positive(2_282_608_687u128));

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(2_282_608_687u128))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_remove_margin_unrealized_pnl_long_position_with_profit_using_twap_price() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    // reserve 1000 : 100
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(5u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
    // reserve 1300 : 76.92, price = 16.9

    router.update_block(|block| {
        block.time = block.time.plus_seconds(450);
        block.height += 1;
    });

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(5u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();
    // reserve 1600 : 62.5, price = 25.6

    router.update_block(|block| {
        block.time = block.time.plus_seconds(450);
        block.height += 1;
    });

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(45_010_000_000u128))
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Insufficient collateral"
    );

    let free_collateral = engine
        .get_free_collateral(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(free_collateral, Integer::new_positive(45_000_000_000u128));

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(45_000_000_000u128))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_remove_margin_unrealized_pnl_long_position_with_loss_using_twap_price() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    // reserve 1000 : 100
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(5u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
    // reserve 1300 : 76.92, price = 16.9

    router.update_block(|block| {
        block.time = block.time.plus_seconds(450);
        block.height += 1;
    });

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(10u64),
            to_decimals(5u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();
    // reserve 1250 : 80 price = 15.625

    router.update_block(|block| {
        block.time = block.time.plus_seconds(450);
        block.height += 1;
    });

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(34_930_000_000u128))
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Insufficient collateral"
    );

    let free_collateral = engine
        .get_free_collateral(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(free_collateral, Integer::new_positive(34_925_373_122u128));

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(34_925_373_122u128))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_remove_margin_unrealized_pnl_short_position_with_profit_using_twap_price() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    // reserve 1000 : 100
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(5u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
    // reserve 900 : 111.11, price = 8.1

    router.update_block(|block| {
        block.time = block.time.plus_seconds(450);
        block.height += 1;
    });

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(5u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();
    // reserve 800 : 125, price = 6.4

    router.update_block(|block| {
        block.time = block.time.plus_seconds(450);
        block.height += 1;
    });

    // margin: 20
    // positionSize: -11.11
    // positionNotional: 78.04
    // unrealizedPnl: 100 - 78.04 = 21.96
    // min(margin + funding, margin + funding + unrealized PnL) - position value * 5%
    // min(20, 20 + 21.96) - 78.04 * 0.05 = 16.098
    // can not remove margin > 16.098
    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(15_600_000_000u128))
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Insufficient collateral"
    );

    let free_collateral = engine
        .get_free_collateral(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(free_collateral, Integer::new_positive(15_548_780_488u128));

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(15_500_000_000u128))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_remove_margin_unrealized_pnl_short_position_with_loss_using_twap_price() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    // reserve 1000 : 100
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(5u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(450);
        block.height += 1;
    });

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(10u64),
            to_decimals(5u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();
    // reserve 800 : 125, price = 6.4

    router.update_block(|block| {
        block.time = block.time.plus_seconds(450);
        block.height += 1;
    });

    // margin: 20
    // positionSize: -11.11
    // positionNotional: (112.1 + 100) / 2 = 106.05
    // unrealizedPnl: 100 - 106.05 = -6.05
    // min(margin + funding, margin + funding + unrealized PnL) - position value * 5%
    // min(20, 20 + (-6.05)) - 106.05 * 0.05 = 8.6475
    // can not remove margin > 8.6475
    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(8_700_000_000u128))
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Insufficient collateral"
    );

    let free_collateral = engine
        .get_free_collateral(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(free_collateral, Integer::new_positive(8_641_304_340u128));

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(8_600_000_000u128))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}
