use cosmwasm_std::{Coin, Uint128};
use cw_multi_test::Executor;
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
            Side::BUY,
            Uint128::from(60_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::zero(),
            vec![Coin::new(60_000_000u128, "uusd")],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = router
        .wrap()
        .query_balance(&engine.addr(), "uusd")
        .unwrap()
        .amount;
    assert_eq!(engine_balance, Uint128::from(60_000_000u64));

    let msg = engine
        .deposit_margin(
            vamm.addr().to_string(),
            Uint128::from(80_000_000u64),
            vec![Coin::new(80_000_000u128, "uusd")],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = router
        .wrap()
        .query_balance(&engine.addr(), "uusd")
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
        engine,
        vamm,
        ..
    } = NativeTokenScenario::new();

    // give alice a balance of UST and LUNA
    router
        .init_bank_balance(
            &alice,
            vec![
                Coin::new(5_000u128 * 10u128.pow(6), "luna"),
                Coin::new(5_000u128 * 10u128.pow(6), "uusd"),
            ],
        )
        .unwrap();

    let msg = engine
        .deposit_margin(
            vamm.addr().to_string(),
            Uint128::from(85_000_000u64),
            vec![Coin::new(85_000_000u128, "luna")],
        )
        .unwrap();
    let res = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        res.to_string(),
        "Generic error: Native token balance mismatch between the argument and the transferred"
            .to_string()
    );
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
            Side::BUY,
            Uint128::from(60_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::zero(),
            vec![Coin::new(60_000_000u128, "uusd")],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = router
        .wrap()
        .query_balance(&engine.addr(), "uusd")
        .unwrap()
        .amount;
    assert_eq!(engine_balance, Uint128::from(60_000_000u64));

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(20_000_000u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = router
        .wrap()
        .query_balance(&engine.addr(), "uusd")
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
            Side::BUY,
            Uint128::from(60_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(0_000_000u64),
            vec![Coin::new(60_000_000u128, "uusd")],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = router
        .wrap()
        .query_balance(&engine.addr(), "uusd")
        .unwrap()
        .amount;
    assert_eq!(engine_balance, Uint128::from(60_000_000u64));

    let price: Uint128 = Uint128::from(25_500_000u128);
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
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(20_000_000u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = router
        .wrap()
        .query_balance(&engine.addr(), "uusd")
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
            Side::BUY,
            Uint128::from(60_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(0_000_000u64),
            vec![Coin::new(60_000_000u128, "uusd")],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = router
        .wrap()
        .query_balance(&engine.addr(), "uusd")
        .unwrap()
        .amount;
    assert_eq!(engine_balance, Uint128::from(60_000_000u64));

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(61_000_000u64))
        .unwrap();
    let result = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(result.to_string(), "Generic error: Insufficient margin");
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
            Side::BUY,
            Uint128::from(60_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(0_000_000u64),
            vec![Coin::new(60_000_000u128, "uusd")],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = router
        .wrap()
        .query_balance(&engine.addr(), "uusd")
        .unwrap()
        .amount;
    assert_eq!(engine_balance, Uint128::from(60_000_000u64));

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), Uint128::from(36_000_000u64))
        .unwrap();
    let result = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: Position is undercollateralized"
    );
}
