use cosmwasm_std::{BankMsg, Coin, CosmosMsg, StdError, Uint128};
use cw_multi_test::Executor;
use margined_common::integer::Integer;
use margined_perp::margined_engine::Side;
use margined_utils::scenarios::NativeTokenScenario;

pub const NEXT_FUNDING_PERIOD_DELTA: u64 = 86_400u64;

#[test]
fn test_add_margin() {
    let NativeTokenScenario {
        mut router,
        alice,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::zero(),
            vec![Coin::new(60_000_000u128, "uwasm")],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = router
        .wrap()
        .query_balance(&engine.addr(), "uwasm")
        .unwrap()
        .amount;
    assert_eq!(engine_balance, Uint128::from(60_000_000u64));

    let msg = engine
        .deposit_margin(
            vamm.addr().to_string(),
            Uint128::from(80_000_000u64),
            vec![Coin::new(80_000_000u128, "uwasm")],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = router
        .wrap()
        .query_balance(&engine.addr(), "uwasm")
        .unwrap()
        .amount;
    assert_eq!(engine_balance, Uint128::from(140_000_000u64));

    let alice_position = engine
        .get_position_with_funding_payment(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(alice_position.margin, Uint128::from(140_000_000u64));
    assert_eq!(
        engine
            .get_balance_with_funding_payment(&router, alice.to_string())
            .unwrap(),
        Uint128::from(140_000_000u64),
    );
}

#[test]
fn test_force_error_add_incorrect_margin() {
    let NativeTokenScenario {
        mut router,
        alice,
        bank,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // give alice a balance of uwasm
    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: alice.to_string(),
        amount: vec![Coin::new(5_000u128 * 10u128.pow(6), "uwasm")],
    });
    router.execute(bank.clone(), msg).unwrap();

    // give alice a balance of ucosmos
    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: alice.to_string(),
        amount: vec![Coin::new(5_000u128 * 10u128.pow(6), "ucosmos")],
    });
    router.execute(bank.clone(), msg).unwrap();

    let msg = engine
        .deposit_margin(
            vamm.addr().to_string(),
            Uint128::from(85_000_000u64),
            vec![Coin::new(85_000_000u128, "ucosmos")],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "Must send reserve token 'uwasm'".to_string(),
        },
        err.downcast().unwrap()
    );
}

#[test]
fn test_add_margin_no_open_position() {
    let NativeTokenScenario {
        mut router,
        alice,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    let msg = engine
        .deposit_margin(
            vamm.addr().to_string(),
            Uint128::from(80_000_000u64),
            vec![Coin::new(80_000_000u128, "uwasm")],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_remove_margin() {
    let NativeTokenScenario {
        mut router,
        alice,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::zero(),
            vec![Coin::new(60_000_000u128, "uwasm")],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = router
        .wrap()
        .query_balance(&engine.addr(), "uwasm")
        .unwrap()
        .amount;
    assert_eq!(engine_balance, Uint128::from(60_000_000u64));

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(20_000_000u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = router
        .wrap()
        .query_balance(&engine.addr(), "uwasm")
        .unwrap()
        .amount;
    assert_eq!(engine_balance, Uint128::from(40_000_000u64));

    let alice_position = engine
        .get_position_with_funding_payment(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(alice_position.margin, Uint128::from(40_000_000u64));
    assert_eq!(
        engine
            .get_balance_with_funding_payment(&router, alice.to_string())
            .unwrap(),
        Uint128::from(40_000_000u64),
    );
}

#[test]
fn test_remove_margin_after_paying_funding() {
    let NativeTokenScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        pricefeed,
        ..
    } = NativeTokenScenario::new();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(0_000_000u64),
            vec![Coin::new(60_000_000u128, "uwasm")],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = router
        .wrap()
        .query_balance(&engine.addr(), "uwasm")
        .unwrap()
        .amount;
    assert_eq!(engine_balance, Uint128::from(60_000_000u64));

    let price: Uint128 = Uint128::from(25_500_000u128);
    let timestamp: u64 = 1_000_000;

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
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(20_000_000u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = router
        .wrap()
        .query_balance(&engine.addr(), "uwasm")
        .unwrap()
        .amount;
    assert_eq!(engine_balance, Uint128::from(36_250_000u128));

    let alice_position = engine
        .get_position_with_funding_payment(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(alice_position.margin, Uint128::from(36_250_000u128),);
    assert_eq!(
        engine
            .get_balance_with_funding_payment(&router, alice.to_string())
            .unwrap(),
        Uint128::from(36_250_000u128),
    );
}

#[test]
fn test_remove_margin_insufficient_margin() {
    let NativeTokenScenario {
        mut router,
        alice,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(0_000_000u64),
            vec![Coin::new(60_000_000u128, "uwasm")],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = router
        .wrap()
        .query_balance(&engine.addr(), "uwasm")
        .unwrap()
        .amount;
    assert_eq!(engine_balance, Uint128::from(60_000_000u64));

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(61_000_000u64))
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Insufficient margin"
    );
}

#[test]
fn test_remove_margin_incorrect_ratio_four_percent() {
    let NativeTokenScenario {
        mut router,
        alice,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(0_000_000u64),
            vec![Coin::new(60_000_000u128, "uwasm")],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = router
        .wrap()
        .query_balance(&engine.addr(), "uwasm")
        .unwrap()
        .amount;
    assert_eq!(engine_balance, Uint128::from(60_000_000u64));

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(36_000_000u64))
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Insufficient collateral"
    );
}

#[test]
fn test_remove_margin_unrealized_pnl_long_position_with_profit_using_spot_price() {
    let NativeTokenScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // reserve 1000 : 100
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(0_000_000u64),
            vec![Coin::new(60_000_000u128, "uwasm")],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
    // reserve 1300 : 76.92, price = 16.9

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(0_000_000u64),
            vec![Coin::new(60_000_000u128, "uwasm")],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();
    // reserve 1600 : 62.5, price = 25.6

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(45_010_000u128))
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Insufficient collateral"
    );

    let free_collateral = engine
        .get_free_collateral(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(free_collateral, Integer::new_positive(45_000_000u128));

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(45_000_000u128))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_remove_margin_unrealized_pnl_long_position_with_loss_using_spot_price() {
    let NativeTokenScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // reserve 1000 : 100
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(0u64),
            vec![Coin::new(60_000_000u128, "uwasm")],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
    // reserve 1300 : 76.92, price = 16.9

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(10_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(0u64),
            vec![Coin::new(10_000_000u128, "uwasm")],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();
    // reserve 1250 : 80 price = 15.625

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(24_900_000u128))
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Insufficient collateral"
    );

    let free_collateral = engine
        .get_free_collateral(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(free_collateral, Integer::new_positive(24_850_742u128));

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(24_850_742u128))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}
#[test]
fn test_remove_margin_unrealized_pnl_short_position_with_profit_using_spot_price() {
    let NativeTokenScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // reserve 1000 : 100
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(20_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(0u64),
            vec![Coin::new(20_000_000u128, "uwasm")],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
    // reserve 900 : 111.11, price = 8.1

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(20_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(0u64),
            vec![Coin::new(20_000_000u128, "uwasm")],
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
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(16_500_000u128))
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Insufficient collateral"
    );

    let free_collateral = engine
        .get_free_collateral(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(free_collateral, Integer::new_positive(16_097_561u128));

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(16_097_561u128))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_remove_margin_unrealized_pnl_short_position_with_loss_using_spot_price() {
    let NativeTokenScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // reserve 1000 : 100
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(20_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(0u64),
            vec![Coin::new(20_000_000u128, "uwasm")],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(10_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(0u64),
            vec![Coin::new(10_000_000u128, "uwasm")],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();
    // reserve 800 : 125, price = 6.4

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(2_500_000u128))
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Insufficient collateral"
    );

    let free_collateral = engine
        .get_free_collateral(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(free_collateral, Integer::new_positive(2_282_600u128));

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(2_282_600u128))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_remove_margin_unrealized_pnl_long_position_with_profit_using_twap_price() {
    let NativeTokenScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // reserve 1000 : 100
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(0u64),
            vec![Coin::new(60_000_000u128, "uwasm")],
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
            Uint128::from(60_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(0u64),
            vec![Coin::new(60_000_000u128, "uwasm")],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();
    // reserve 1600 : 62.5, price = 25.6

    router.update_block(|block| {
        block.time = block.time.plus_seconds(450);
        block.height += 1;
    });

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(45_010_000u128))
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Insufficient collateral"
    );

    let free_collateral = engine
        .get_free_collateral(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(free_collateral, Integer::new_positive(45_000_000u128));

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(45_000_000u128))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_remove_margin_unrealized_pnl_long_position_with_loss_using_twap_price() {
    let NativeTokenScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // reserve 1000 : 100
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(0u64),
            vec![Coin::new(60_000_000u128, "uwasm")],
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
            Uint128::from(10_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(0u64),
            vec![Coin::new(10_000_000u128, "uwasm")],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();
    // reserve 1250 : 80 price = 15.625

    router.update_block(|block| {
        block.time = block.time.plus_seconds(450);
        block.height += 1;
    });

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(34_930_000u128))
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Insufficient collateral"
    );

    let free_collateral = engine
        .get_free_collateral(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(free_collateral, Integer::new_positive(34_925_370u128));

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(34_925_370u128))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}
#[test]
fn test_remove_margin_unrealized_pnl_short_position_with_profit_using_twap_price() {
    let NativeTokenScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // reserve 1000 : 100
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(20_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(0u64),
            vec![Coin::new(20_000_000u128, "uwasm")],
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
            Uint128::from(20_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(0u64),
            vec![Coin::new(20_000_000u128, "uwasm")],
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
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(15_600_000u128))
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Insufficient collateral"
    );

    let free_collateral = engine
        .get_free_collateral(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(free_collateral, Integer::new_positive(15_548_781u128));

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(15_500_000u128))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_remove_margin_unrealized_pnl_short_position_with_loss_using_twap_price() {
    let NativeTokenScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // reserve 1000 : 100
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(20_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(0u64),
            vec![Coin::new(20_000_000u128, "uwasm")],
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
            Uint128::from(10_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(0u64),
            vec![Coin::new(10_000_000u128, "uwasm")],
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
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(8_700_000u128))
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Insufficient collateral"
    );

    let free_collateral = engine
        .get_free_collateral(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(free_collateral, Integer::new_positive(8_641_296u128));

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(8_600_000u128))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_native_token_sent() {
    // cw20 token
    let required = Asset {
        info: AssetInfo::Token {
            contract_addr: Addr::unchecked("usdc"),
        },
        amount: Uint128::from(100_000_000u32),
    };

    // message payload
    let coin = Coin {
        denom: "uwasm".to_string(),
        amount: Uint128::from(100_000_000u32),
    };
    let msg_info = MessageInfo {
        sender: Addr::unchecked("alice"),
        funds: vec![coin],
    };

    // try to send cw20 and call assert_sent_native_token_balance
    let err = required
        .assert_sent_native_token_balance(&msg_info)
        .unwrap_err();

    // check error
    assert_eq!(err.to_string(), "Generic error: self is not native token");
}

#[test]
fn test_native_token_missing_denom() {
    // required token
    let required = Asset {
        info: AssetInfo::NativeToken {
            denom: "uwasm".to_string(),
        },
        amount: Uint128::from(100_000_000u32),
    };

    // message payload
    let coin = Coin {
        denom: "ujuno".to_string(),
        amount: Uint128::from(100_000_000u32),
    };
    let msg_info = MessageInfo {
        sender: Addr::unchecked("alice"),
        funds: vec![coin],
    };

    // send a denom that isnt requested in the function
    let err = required
        .assert_sent_native_token_balance(&msg_info)
        .unwrap_err();

    // check error
    assert_eq!(
        err.to_string(),
        "Generic error: Must send reserve token 'uwasm'"
    );
}

#[test]
fn test_native_token_multiple_denoms() {
    //
    // Note that the error tested for here occurs if multiple denoms are sent, regardless of whether the correct one is included
    //

    // required token
    let required = Asset {
        info: AssetInfo::NativeToken {
            denom: "uwasm".to_string(),
        },
        amount: Uint128::from(100_000_000u32),
    };

    // message payload
    let coin1 = Coin {
        denom: "ujuno".to_string(),
        amount: Uint128::from(100_000_000u32),
    };
    let coin2 = Coin {
        denom: "uosmo".to_string(),
        amount: Uint128::from(100_000_000u32),
    };
    let msg_info = MessageInfo {
        sender: Addr::unchecked("alice"),
        funds: vec![coin1, coin2],
    };

    // send a denom that isnt requested in the function
    let err = required
        .assert_sent_native_token_balance(&msg_info)
        .unwrap_err();

    // check error
    assert_eq!(
        err.to_string(),
        "Generic error: Sent more than one denomination"
    );
}

#[test]
fn test_native_token_no_funds() {
    // required token
    let required = Asset {
        info: AssetInfo::NativeToken {
            denom: "uwasm".to_string(),
        },
        amount: Uint128::from(100_000_000u32),
    };

    // message payload
    let coin = Coin {
        denom: "ujuno".to_string(),
        amount: Uint128::from(0u32),
    };
    let msg_info = MessageInfo {
        sender: Addr::unchecked("alice"),
        funds: vec![coin],
    };

    // send a denom that isnt requested in the function
    let err = required
        .assert_sent_native_token_balance(&msg_info)
        .unwrap_err();

    // check error
    assert_eq!(err.to_string(), "Generic error: No funds sent");
}

#[test]
fn test_native_token_different_amounts() {
    // required token
    let required = Asset {
        info: AssetInfo::NativeToken {
            denom: "uwasm".to_string(),
        },
        amount: Uint128::from(100_000_000u32),
    };

    // message payload
    let coin = Coin {
        denom: "uwasm".to_string(),
        amount: Uint128::from(100_000u32),
    };
    let msg_info = MessageInfo {
        sender: Addr::unchecked("alice"),
        funds: vec![coin],
    };

    // send a denom that isnt requested in the function
    let err = required
        .assert_sent_native_token_balance(&msg_info)
        .unwrap_err();

    // check error
    assert_eq!(
        err.to_string(),
        "Generic error: Native token balance mismatch between the argument and the transferred"
    );
}
