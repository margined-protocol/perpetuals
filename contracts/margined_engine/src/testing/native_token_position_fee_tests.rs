use cosmwasm_std::{BankMsg, Coin, CosmosMsg, Uint128};
use margined_common::integer::Integer;
use margined_perp::margined_engine::{PnlCalcOption, Side};
use margined_utils::tools::fund_calculator::calculate_funds_needed;

use margined_utils::{cw_multi_test::Executor, testing::NativeTokenScenario};

use crate::testing::new_native_token_scenario;

// Note: these tests also verify the 10% fees for the amm are functioning

#[test]
fn test_initialization() {
    let NativeTokenScenario {
        router,
        owner,
        alice,
        bob,
        engine,
        ..
    } = new_native_token_scenario();

    // verfiy the balances
    let owner_balance = router.wrap().query_balance(&owner, "orai").unwrap().amount;
    assert_eq!(owner_balance, Uint128::zero());
    let alice_balance = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(alice_balance, Uint128::new(5_000_000_000));
    let bob_balance = router.wrap().query_balance(&bob, "orai").unwrap().amount;
    assert_eq!(bob_balance, Uint128::new(5_000_000_000));
    let engine_balance = router
        .wrap()
        .query_balance(&engine.addr(), "orai")
        .unwrap()
        .amount;
    assert_eq!(engine_balance, Uint128::zero());
}

#[test]
fn test_force_error_open_position_no_token_sent() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000u64),
            Uint128::from(9_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(5_000_000u64),
            vec![],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();

    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: sent funds are insufficient".to_string()
    );
}

#[test]
fn test_ten_percent_fee_open_long_position() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        fee_pool,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice opens long position with 60 margin, 10x leverage
    // (1000 + 600) * (100 + baseAssetDelta) = 100k, baseAssetDelta = -37.5
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000u64),
            Uint128::from(6_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(10_500_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(60_000_000u64),
                Uint128::from(10_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(alice_balance, Uint128::new(4_940_000_000));

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();

    // transferred margin = margin + fee = 60 + (60 * 10 * 10%) = 120
    assert_eq!(position.size, Integer::new_positive(12_587_412u128));
    assert_eq!(position.margin, Uint128::from(24_000_000u64));
    assert_eq!(position.notional, Uint128::from(144_000_000u64));

    let fee_pool_balance = router
        .wrap()
        .query_balance(&fee_pool.addr(), "orai")
        .unwrap()
        .amount;
    assert_eq!(fee_pool_balance, Uint128::from(36_000_000u64));
    let engine_balance = router
        .wrap()
        .query_balance(&engine.addr(), "orai")
        .unwrap()
        .amount;
    assert_eq!(engine_balance, Uint128::from(24_000_000u64));
}

#[test]
fn test_force_error_insufficient_token_long_position() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000u64),
            Uint128::from(8_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(4_500_000u64),
            vec![Coin::new(59_000_000u128, "orai")],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: sent funds are insufficient".to_string()
    );
}

#[test]
fn test_ten_percent_fee_open_short_position() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        fee_pool,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice opens long position with 60 margin, 10x leverage
    // (1000 + 600) * (100 + baseAssetDelta) = 100k, baseAssetDelta = -37.5
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(60_000_000u64),
            Uint128::from(6_000_000u64),
            Uint128::from(2_000_000u64),
            Some(Uint128::from(10_000_000u64)),
            Uint128::from(150_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(60_000_000u64),
                Uint128::from(10_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(alice_balance, Uint128::new(4_940_000_000));

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();

    // transferred margin = margin + fee = 60 + (60 * 10 * 10%) = 120
    assert_eq!(position.size, Integer::new_negative(16_822_430u128));
    assert_eq!(position.margin, Uint128::from(24_000_000u64));
    assert_eq!(position.notional, Uint128::from(144_000_000u64));

    let fee_pool_balance = router
        .wrap()
        .query_balance(&fee_pool.addr(), "orai")
        .unwrap()
        .amount;
    assert_eq!(fee_pool_balance, Uint128::from(36_000_000u64));
    let engine_balance = router
        .wrap()
        .query_balance(&engine.addr(), "orai")
        .unwrap()
        .amount;
    assert_eq!(engine_balance, Uint128::from(24_000_000u64));
}

#[test]
fn test_force_error_insufficient_token_short_position() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice opens long position with 60 margin, 10x leverage
    // (1000 + 600) * (100 + baseAssetDelta) = 100k, baseAssetDelta = -37.5
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(60_000_000u64),
            Uint128::from(8_000_000u64),
            Uint128::from(2_000_000u64),
            Some(Uint128::from(12_000_000u64)),
            Uint128::from(15_000_000u64),
            vec![Coin::new(59_999_999u128, "orai")],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: sent funds are insufficient".to_string()
    );
}

#[test]
fn test_ten_percent_fee_long_position_price_remains_long_again() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        fee_pool,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice opens long position with 25 margin, 10x leverage
    // (1000 + 250) * (100 + baseAssetDelta) = 100k, baseAssetDelta = -20
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(25_000_000u64),
            Uint128::from(9_000_000u64),
            Uint128::from(15_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(1_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(25_000_000u64),
                Uint128::from(10_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(alice_balance_1, Uint128::new(4_975_000_000));

    // alice opens long position with 175 margin, 2x leverage
    // (1250 + 350) * (80 + baseAssetDelta) = 100k, baseAssetDelta = -17.5
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(175_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(23_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(17_500_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(175_000_000u64),
                Uint128::from(2_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(175_000_000u128)
    );

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();

    // transferred margin = margin + fee = 60 + (60 * 10 * 10%) = 120
    assert_eq!(position.size, Integer::new_positive(2_200_488u128));
    assert_eq!(position.margin, Uint128::from(2_500_000u64));
    assert_eq!(position.notional, Uint128::from(22_500_000u64));

    let fee_pool_balance = router
        .wrap()
        .query_balance(&fee_pool.addr(), "orai")
        .unwrap()
        .amount;
    assert_eq!(fee_pool_balance, Uint128::from(57_500_000u64));
    let engine_balance = router
        .wrap()
        .query_balance(&engine.addr(), "orai")
        .unwrap()
        .amount;
    assert_eq!(engine_balance, Uint128::from(142_500_000u64));
}

#[test]
fn test_ten_percent_fee_long_position_price_up_long_again() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        fee_pool,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice opens long position with 25 margin, 10x leverage
    // (1000 + 250) * (100 + baseAssetDelta) = 100k, baseAssetDelta = -20
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(25_000_000u64),
            Uint128::from(6_000_000u64),
            Uint128::from(22_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(4_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(25_000_000u64),
                Uint128::from(10_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(alice_balance_1, Uint128::new(4_975_000_000));

    // bob opens long position with 35 margin, 10x leverage, price up
    // (1250 + 350) * (80 + baseAssetDelta) = 100k, baseAssetDelta = -17.5
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(35_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(21_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(3_500_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(35_000_000u64),
                Uint128::from(10_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_positive(9_987_482u64));

    // alice opens long position with 175 margin, 2x leverage
    // (1250 + 350) * (80 + baseAssetDelta) = 100k, baseAssetDelta = -17.5
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(200_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(35_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(12_500_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(200_000_000u64),
                Uint128::from(2_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(alice_balance_1 - alice_balance_2, Uint128::new(200_000_000));

    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();

    let position_3 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 3)
        .unwrap();

    // transferred margin = margin + fee = 200 + (200 * 2 * 10%) = 240
    assert_eq!(position_1.size + position_3.size, Integer::new_positive(24_663_246u128));
    assert_eq!(position_1.margin + position_3.margin, Uint128::from(170_000_000u64));
    assert_eq!(position_1.notional + position_3.notional, Uint128::from(380_000_000u64));

    let fee_pool_balance = router
        .wrap()
        .query_balance(&fee_pool.addr(), "orai")
        .unwrap()
        .amount;
    assert_eq!(fee_pool_balance, Uint128::new(72_500_000));
    let engine_balance = router
        .wrap()
        .query_balance(&engine.addr(), "orai")
        .unwrap()
        .amount;
    assert_eq!(engine_balance, Uint128::from(187_500_000u64));
}

#[test]
fn test_ten_percent_fee_long_position_price_down_long_again() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice opens long position with 125 margin, 2x leverage
    // (1000 + 250) * (100 + baseAssetDelta) = 100k, baseAssetDelta = -20
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(125_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(10_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(125_000_000u64),
                Uint128::from(2_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    // bob opens short position with 125 margin, 2x leverage, price down
    // (1250 - 250) * (80 + baseAssetDelta) = 100k, baseAssetDelta = 20
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(125_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(8_000_000u64),
            Some(Uint128::from(15_000_000u64)),
            Uint128::from(20_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(125_000_000u64),
                Uint128::from(2_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // alice's 20 long position worth 166.67 now
    // (1000 + quoteAssetDelta) * (100 + 20) = 100k, quoteAssetDelta = -166.666666666666666666
    // unrealizedPnl = positionValue - cost = 166.666666666666666666 - 250 = -83.333333333333333333
    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(57_142_864u64));

    // alice opens long position with 50 margin, 5x leverage
    // (1000 + 250) * (100 + baseAssetDelta) = 100k, baseAssetDelta = -20
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(50_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(10_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(50_000_000u64),
                Uint128::from(5_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(50_000_000u128)
    );

    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();

    let position_3 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 3)
        .unwrap();

    // transferred margin = margin + fee = 50 + (50 * 5 * 10%) = 75
    assert_eq!(position_1.size + position_3.size, Integer::new_positive(27_777_777u128));
    assert_eq!(position_1.margin + position_3.margin, Uint128::from(125_000_000u64));
    assert_eq!(position_1.notional + position_3.notional, Uint128::from(325_000_000u64));
}

#[test]
fn test_ten_percent_fee_short_position_price_remains_short_again() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice opens short position with 100 margin, 2x leverage
    // (1000 - 200) * (100 + baseAssetDelta) = 100k, baseAssetDelta = 25
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(100_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(2_000_000u64),
            Some(Uint128::from(18_000_000u64)),
            Uint128::from(25_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(100_000_000u64),
                Uint128::from(2_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    // alice opens short position with 50 margin, 8x leverage
    // (800 - 400) * (125 + baseAssetDelta) = 100k, baseAssetDelta = 125
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(50_000_000u64),
            Uint128::from(8_000_000u64),
            Uint128::from(2_000_000u64),
            Some(Uint128::from(10_000_000u64)),
            Uint128::from(125_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(50_000_000u64),
                Uint128::from(8_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(50_000_000u128)
    );

    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();

    let position_2 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 2)
        .unwrap();
    // then transferred margin = margin + fee = 50 + (50 * 8 * 10%) = 90
    assert_eq!(position_1.size + position_2.size, Integer::new_negative(31_578_949u128));
    assert_eq!(position_1.margin + position_2.margin, Uint128::from(90_000_000u64));
    assert_eq!(position_1.notional + position_2.notional, Uint128::from(240_000_000u64));

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_positive(31_358_568u128));
}

#[test]
fn test_ten_percent_fee_short_position_price_down_short_again() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice opens short position with 100 margin, 2x leverage
    // (1000 - 200) * (100 + baseAssetDelta) = 100k, baseAssetDelta = 25
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(100_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(2_000_000u64),
            Some(Uint128::from(12_000_000u64)),
            Uint128::from(25_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(100_000_000u64),
                Uint128::from(2_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    // bob opens short position with 150 margin, 2x leverage, price down
    // (800 - 300) * (125 + baseAssetDelta) = 100k, baseAssetDelta = 75
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(150_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(1_000_000u64),
            Some(Uint128::from(8_000_000u64)),
            Uint128::from(75_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(150_000_000u64),
                Uint128::from(2_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // alice's 25 short position worth 71.43 now
    // (500 + quoteAssetDelta) * (200 - 25) = 100k, quoteAssetDelta = -71.4285714286
    // unrealizedPnl = positionValueWhenBorrowed - positionValueWhenReturned = 200 - 71.4285714286 = 128.5714285714
    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_positive(82_580_641u64));

    // alice opens short position with 100 margin, 3x leverage
    // (500 - 300) * (200 + baseAssetDelta) = 100k, baseAssetDelta = 300
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(100_000_000u64),
            Uint128::from(3_000_000u64),
            Uint128::from(800_000u64),
            Some(Uint128::from(6_000_000u64)),
            Uint128::from(300_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(100_000_000u64),
                Uint128::from(3_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // transferred margin = margin + fee = 100 + (100 * 3 * 10%) = 130
    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(100_000_000u128)
    );

    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();

    let position_3 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 3)
        .unwrap();

    assert_eq!(position_1.size + position_3.size, Integer::new_negative(108_791_211u128));
    assert_eq!(position_1.margin + position_3.margin, Uint128::from(150_000_000u64));
    assert_eq!(position_1.notional + position_3.notional, Uint128::from(370_000_000u64));
}

#[test]
fn test_ten_percent_fee_short_position_price_up_short_again() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice opens short position with 200 margin, 1x leverage
    // (1000 - 200) * (100 + baseAssetDelta) = 100k, baseAssetDelta = 25
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(200_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(2_000_000u64),
            Some(Uint128::from(12_000_000u64)),
            Uint128::from(25_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(200_000_000u64),
                Uint128::from(1_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    // bob opens long position with 200 margin, 1x leverage, price up
    // (800 + 200) * (125 + baseAssetDelta) = 100k, baseAssetDelta = -25
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(200_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(15_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(200_000_000u64),
                Uint128::from(1_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // alice's 25 short position worth 333.33 now
    // (1000 + quoteAssetDelta) * (100 - 25) = 100k, quoteAssetDelta = 333.3333333333
    // unrealizedPnl = positionValueWhenBorrowed - positionValueWhenReturned = 200 - 333.3333333333 = -133.3333333333
    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(101_250_005u64));

    // alice opens short position with 50 margin, 4x leverage
    // (1000 - 200) * (100 + baseAssetDelta) = 100k, baseAssetDelta = 25
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(50_000_000u64),
            Uint128::from(4_000_000u64),
            Uint128::from(2_000_000u64),
            Some(Uint128::from(12_000_000u64)),
            Uint128::from(25_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(50_000_000u64),
                Uint128::from(4_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // then transferred margin = margin + fee = 50 + (50 * 4 * 10%) = 70
    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(50_000_000u128)
    );

    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();

    let position_3 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 3)
        .unwrap();

    assert_eq!(position_1.size + position_3.size, Integer::new_negative(35_587_584u128));
    assert_eq!(position_1.margin + position_3.margin, Uint128::from(210_000_000u64));
    assert_eq!(position_1.notional + position_3.notional, Uint128::from(300_000_000u64));
}

#[test]
fn test_ten_percent_fee_long_position_price_remains_reduce_position() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000u64),
            Uint128::from(7_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(10_500_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(60_000_000u64),
                Uint128::from(10_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(350_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(4_000_000u64),
            Some(Uint128::from(21_000_000u64)),
            Uint128::from(50_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(350_000_000u64),
                Uint128::from(1_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();

    let position_2 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 2)
        .unwrap();
    assert_eq!(position_1.size + position_2.size, Integer::new_negative(23_304_563u128));
    assert_eq!(position_2.notional - position_1.notional, Uint128::from(189_000_000u64));
    assert_eq!(position_1.margin, Uint128::from(18_000_000u64));

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(58_524_192u128));
}

#[test]
fn test_ten_percent_fee_reduce_long_position_zero_fee() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();

    let msg = vamm.set_toll_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000u64),
            Uint128::from(10_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(37_500_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(60_000_000u64),
                Uint128::from(10_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(350_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(10_000_000u64),
            Some(Uint128::from(22_000_000u64)),
            Uint128::from(17_500_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(350_000_000u64),
                Uint128::from(1_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    let position_2 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 2)
        .unwrap();
    assert_eq!(position_1.size + position_2.size, Integer::new_positive(20_000_000u128));
    assert_eq!(position_1.notional - position_2.notional, Uint128::from(250_000_000u64));
    assert_eq!(position_1.margin, Uint128::from(60_000_000u64));

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(201063830u128));
}

#[test]
fn test_ten_percent_fee_short_position_price_remains_reduce_position() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(6_000_000u64),
            Uint128::from(7_000_000u64),
            Uint128::from(7_000_000u64),
            Some(Uint128::from(22_000_000u64)),
            Uint128::from(5_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(6_000_000u64),
                Uint128::from(10_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(40_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(5_500_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(40_000_000u64),
                Uint128::from(5_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    let position_2 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 2)
        .unwrap();

    assert_eq!(position_1.size + position_2.size, Integer::new_positive(8_037_520u128));
    assert_eq!(position_2.notional - position_1.notional, Uint128::from(87_400_000u64));
    assert_eq!(position_2.margin - position_1.margin, Uint128::from(18_200_000u64));

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(2_701_173u128));
}

#[test]
fn test_ten_percent_fee_reduce_long_position_price_up_long_again() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000u64),
            Uint128::from(7_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(10_500_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(60_000_000u64),
                Uint128::from(10_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let price = vamm.spot_price(&router.wrap()).unwrap();
    println!("[LOG] [1] spot price: {:?}", price);

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(400_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(35_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(12_500_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(400_000_000u64),
                Uint128::from(1_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let price = vamm.spot_price(&router.wrap()).unwrap();
    println!("[LOG] [2] spot price: {:?}", price);
    
    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_positive(85_868_001u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(400_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(10_000_000u64),
            Some(Uint128::from(32_000_000u64)),
            Uint128::from(22_500_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(400_000_000u64),
                Uint128::from(1_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();

    let position_3 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 3)
        .unwrap();
    assert_eq!(position_1.size + position_3.size, Integer::new_negative(10_325_144u128));
    assert_eq!(position_3.margin - position_1.margin, Uint128::from(342_000_000u64));
    assert_eq!(position_3.notional - position_1.notional, Uint128::from(234_000_000u64));

    let price = vamm.spot_price(&router.wrap()).unwrap();
    println!("[LOG] [3] spot price: {:?}", price);

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(6u128));
}

#[test]
fn test_ten_percent_fee_reduce_long_position_price_down_long_again() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(500_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(23_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(20_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(500_000_000u64),
                Uint128::from(2_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(400_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(7_000_000u64),
            Some(Uint128::from(34_000_000u64)),
            Uint128::from(20_500_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(400_000_000u64),
                Uint128::from(1_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(238_048_787u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(350_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(10_000_000u64),
            Some(Uint128::from(22_000_000u64)),
            Uint128::from(37_500_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(350_000_000u64),
                Uint128::from(1_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    let position_3 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 3)
        .unwrap();
    assert_eq!(position_1.size + position_3.size, Integer::new_positive(24_999_999u128));
    assert_eq!(position_1.notional - position_3.notional, Uint128::from(485_000_000u64));
    assert_eq!(position_1.margin, Uint128::from(400_000_000u64));

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(425_000_006u64));
}

#[test]
fn test_ten_percent_fee_reduce_short_position_price_up_short_again() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(100_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(7_000_000u64),
            Some(Uint128::from(14_000_000u64)),
            Uint128::from(25_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(100_000_000u64),
                Uint128::from(2_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(50_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(2_350_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(50_000_000u64),
                Uint128::from(1_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(19_432_998u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(150_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(7_640_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(150_000_000u64),
                Uint128::from(1_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();

    let position_3 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 3)
        .unwrap();
    assert_eq!(position_1.size + position_3.size, Integer::new_negative(4_092_486u128));
    assert_eq!(position_1.notional - position_3.notional, Uint128::from(25_000_000u64));
    assert_eq!(position_3.margin - position_1.margin, Uint128::from(55_000_000u64));

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(85_957_455u64));
}

#[test]
fn test_ten_percent_fee_reduce_short_position_price_down_short_again() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(250_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(3_000_000u64),
            Some(Uint128::from(10_000_000u64)),
            Uint128::from(100_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(250_000_000u64),
                Uint128::from(2_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
    
    let price = vamm.spot_price(&router.wrap()).unwrap();
    println!("[LOG] [1] spot price: {:?}", price);

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(100_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(1_000_000u64),
            Some(Uint128::from(5_000_000u64)),
            Uint128::from(50_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(100_000_000u64),
                Uint128::from(1_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();
    
    let price = vamm.spot_price(&router.wrap()).unwrap();
    println!("[LOG] [2] spot price: {:?}", price);

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_positive(137_272_726u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(100_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(10_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(100_000_000u64),
                Uint128::from(1_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();

    let position_3 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 3)
        .unwrap();
    assert_eq!(position_1.size + position_3.size, Integer::new_negative(37_254_903u128));
    assert_eq!(position_1.notional - position_3.notional, Uint128::from(310_000_000u64));
    assert_eq!(position_1.margin - position_3.margin, Uint128::from(110_000_000u64));
    
    let price = vamm.spot_price(&router.wrap()).unwrap();
    println!("[LOG] [3] spot price: {:?}", price);

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_positive(1u128));
}

#[test]
fn test_ten_percent_fee_open_long_price_remains_close_manually() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();
    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(50_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(5_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(50_000_000u64),
                Uint128::from(5_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(250_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(5_000_000u64),
            Some(Uint128::from(22_000_000u64)),
            Uint128::from(30_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(250_000_000u64),
                Uint128::from(1_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(250_000_000u64)
    );
}

#[test]
fn test_ten_percent_fee_open_short_price_remains_close_manually() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(100_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(6_000_000u64),
            Some(Uint128::from(10_000_000u64)),
            Uint128::from(25_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(100_000_000u64),
                Uint128::from(2_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(200_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(5_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(200_000_000u64),
                Uint128::from(1_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(200_000_000u64)
    );
}

#[test]
fn test_ten_percent_fee_open_long_price_up_close_manually() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        bank,
        ..
    } = new_native_token_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // give engine some funds so it has enough collateral to pay profit
    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: engine.addr().to_string(),
        amount: vec![Coin::new(1_000u128 * 10u128.pow(6), "orai")],
    });
    router.execute(bank.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(25_000_000u64),
            Uint128::from(6_000_000u64),
            Uint128::from(30_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(5_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(25_000_000u64),
                Uint128::from(10_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(35_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(30_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(5_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(35_000_000u64),
                Uint128::from(10_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_positive(9_987_482u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            pnl.position_notional,
            Uint128::from(1_000_000u64),
            Uint128::from(5_000_000u64),
            Some(Uint128::from(25_000_000u64)),
            Uint128::from(40_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                pnl.position_notional,
                Uint128::from(1_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(69_987_482u64)
    );
}

#[test]
fn test_ten_percent_fee_open_long_price_down_close_manually() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(500_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(23_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(10_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(500_000_000u64),
                Uint128::from(2_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(400_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(16_000_000u64),
            Some(Uint128::from(34_000_000u64)),
            Uint128::from(30_500_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(400_000_000u64),
                Uint128::from(1_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(238_048_787u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            pnl.position_notional,
            Uint128::from(1_000_000u64),
            Uint128::from(4_000_000u64),
            Some(Uint128::from(16_000_000u64)),
            Uint128::from(50_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                pnl.position_notional,
                Uint128::from(1_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(561_951_213u64)
    );
    println!("alice_balance 1: {:?}", alice_balance_1);
    println!("alice_balance 2: {:?}", alice_balance_2);

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position.size, Integer::new_positive(44_444_444u128));
    assert_eq!(position.notional, Uint128::from(800_000_000u128));
    assert_eq!(position.margin, Uint128::from(400_000_000u128));
}

#[test]
fn test_ten_percent_fee_open_short_price_up_close_manually() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(200_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(7_000_000u64),
            Some(Uint128::from(22_000_000u64)),
            Uint128::from(25_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(200_000_000u64),
                Uint128::from(1_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(50_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(2_350_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(50_000_000u64),
                Uint128::from(1_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(22_740_483u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            pnl.position_notional,
            Uint128::from(1_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(10_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                pnl.position_notional,
                Uint128::from(1_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(202_740_483u64)
    );
}

#[test]
fn test_ten_percent_fee_open_short_price_down_close_manually() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        bank,
        ..
    } = new_native_token_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // give engine some funds so it has enough collateral to pay profit
    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: engine.addr().to_string(),
        amount: vec![Coin::new(1000u128 * 10u128.pow(6), "orai")],
    });
    router.execute(bank.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(250_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(3_000_000u64),
            Some(Uint128::from(11_000_000u64)),
            Uint128::from(100_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(250_000_000u64),
                Uint128::from(2_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(100_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(1_000_000u64),
            Some(Uint128::from(5_000_000u64)),
            Uint128::from(50_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(100_000_000u64),
                Uint128::from(1_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_positive(137_272_726u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            pnl.position_notional,
            Uint128::from(1_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(50_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                pnl.position_notional,
                Uint128::from(1_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(262_727_274u64)
    );
}

#[test]
fn test_ten_percent_fee_open_long_price_remains_close_opening_larger_short() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();
    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(125_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(5_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(125_000_000u64),
                Uint128::from(2_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(45_000_000u64),
            Uint128::from(7_000_000u64),
            Uint128::from(8_000_000u64),
            Some(Uint128::from(22_000_000u64)),
            Uint128::from(55_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(45_000_000u64),
                Uint128::from(10_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(45_000_000u64)
    );

    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();

    let position_2 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 2)
        .unwrap();
    assert_eq!(position_1.size + position_2.size, Integer::new_positive(9_543_192u64));
    assert_eq!(position_1.notional - position_2.notional, Uint128::from(105_500_000u64));
    assert_eq!(position_1.margin, Uint128::from(100_000_000u64));
}

#[test]
fn test_ten_percent_fee_open_short_price_remains_close_opening_larger_long() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();
    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(20_000_000u64),
            Uint128::from(6_000_000u64),
            Uint128::from(6_000_000u64),
            Some(Uint128::from(22_000_000u64)),
            Uint128::from(40_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(20_000_000u64),
                Uint128::from(10_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(90_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(5_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(90_000_000u64),
                Uint128::from(5_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(90_000_000u64)
    );

    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();

    let position_2 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 2)
        .unwrap();
    assert_eq!(position_1.size + position_2.size, Integer::new_positive(15_038_232u64));
    assert_eq!(position_1.margin, Uint128::from(8_000_000u64));

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(26_255_162u128));
}

#[test]
fn test_ten_percent_fee_open_long_price_up_close_opening_larger_short() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        fee_pool,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(25_000_000u64),
            Uint128::from(6_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(5_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(25_000_000u64),
                Uint128::from(10_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(35_000_000u64),
            Uint128::from(6_000_000u64),
            Uint128::from(25_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(5_500_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(35_000_000u64),
                Uint128::from(10_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_positive(9_574_137u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(100_000_000u64),
            Uint128::from(8_000_000u64),
            Uint128::from(1_000_000u64),
            Some(Uint128::from(22_000_000u64)),
            Uint128::from(62_510_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(100_000_000u64),
                Uint128::from(8_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(100_000_000u64)
    );

    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();

    let position_3 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 3)
        .unwrap();
    assert_eq!(position_1.size + position_3.size, Integer::new_negative(8_553_052u64));
    assert_eq!(position_3.notional - position_1.notional, Uint128::from(100_000_000u64));
    assert_eq!(position_1.margin, Uint128::from(10_000_000u64));

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(8_084_648u64));

    let fee_pool_balance = router
        .wrap()
        .query_balance(&fee_pool.addr(), "orai")
        .unwrap()
        .amount;
    assert_eq!(fee_pool_balance, Uint128::from(116_000_000u64));
}

#[test]
fn test_ten_percent_fee_open_long_price_down_close_opening_larger_short() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(125_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(10_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(125_000_000u64),
                Uint128::from(2_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(125_000_000u64),
            Uint128::from(2_000_000u64),
            Uint128::from(6_000_000u64),
            Some(Uint128::from(22_000_000u64)),
            Uint128::from(40_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(125_000_000u64),
                Uint128::from(2_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(57_142_864u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(60_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(1_000_000u64),
            Some(Uint128::from(10_000_000u64)),
            Uint128::from(1_450_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(60_000_000u64),
                Uint128::from(10_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(60_000_000u64)
    );

    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();

    let position_3 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 3)
        .unwrap();
    assert_eq!(position_1.size + position_3.size, Integer::new_negative(980_393u64));
    assert_eq!(position_1.notional - position_3.notional, Uint128::from(50_000_000u64));
    assert_eq!(position_1.margin, Uint128::from(100_000_000u64));

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(94_525_553u64));
}

#[test]
fn test_ten_percent_fee_open_short_price_up_close_opening_larger_long() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(200_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(6_000_000u64),
            Some(Uint128::from(22_000_000u64)),
            Uint128::from(25_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(200_000_000u64),
                Uint128::from(1_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(50_000_000u64),
            Uint128::from(4_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(7_349_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(50_000_000u64),
                Uint128::from(4_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(64_388_449u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000u64),
            Uint128::from(7_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(10_490_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(60_000_000u64),
                Uint128::from(10_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();

    let position_3 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 3)
        .unwrap();
    assert_eq!(position_1.size + position_3.size, Integer::new_negative(9_376_872u64));
    assert_eq!(position_1.notional - position_3.notional, Uint128::from(54_000_000u64));
    assert_eq!(position_1.margin - position_3.margin, Uint128::from(162_000_000u64));

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(145_644_911u64));
}

#[test]
fn test_ten_percent_fee_open_short_price_down_close_opening_larger_long() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();
    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(500_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(4_000_000u64),
            Some(Uint128::from(22_000_000u64)),
            Uint128::from(100_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(500_000_000u64),
                Uint128::from(1_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = router.wrap().query_balance(&alice, "orai").unwrap().amount;

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(100_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(1_000_000u64),
            Some(Uint128::from(10_000_000u64)),
            Uint128::from(50_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(100_000_000u64),
                Uint128::from(1_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_positive(172_390_670u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(60_000_000u64),
            Uint128::from(7_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::from(45_000_000u64),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(60_000_000u64),
                Uint128::from(10_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = router.wrap().query_balance(&alice, "orai").unwrap().amount;
    println!("alice_balance 1: {:?}", alice_balance_1);
    println!("alice_balance 2: {:?}", alice_balance_2);
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(60_000_000u64)
    );

    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();

    let position_3 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 3)
        .unwrap();
    assert_eq!(position_1.size + position_3.size, Integer::new_negative(35_075_342u64));
    assert_eq!(position_1.notional - position_3.notional, Uint128::from(324_000_000u64));
    assert_eq!(position_1.margin + position_3.margin, Uint128::from(468_000_000u64));
}

#[test]
fn test_ten_percent_fee_open_long_price_down_liquidation() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();
    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(5_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::zero(),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(5_000_000u64),
                Uint128::from(10_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(50_000_000u64),
            Uint128::from(6_000_000u64),
            Uint128::from(5_000_000u64),
            Some(Uint128::from(22_000_000u64)),
            Uint128::zero(),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(50_000_000u64),
                Uint128::from(10_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(2_773_155u64));

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position.size, Integer::new_positive(1_234_567u64));
    assert_eq!(position.notional, Uint128::from(12_500_000u64));
    assert_eq!(position.margin, Uint128::from(2_500_000u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(60_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(900_000u64),
            Some(Uint128::from(10_000_000u64)),
            Uint128::zero(),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(60_000_000u64),
                Uint128::from(1_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();

    assert_eq!(
        calculate_funds_needed(
            &router.wrap(),
            Uint128::from(60_000_000u64),
            Uint128::from(1_000_000u64),
            vamm.addr(),
        )
        .unwrap(),
        vec![Coin::new(60_000_000u128, "orai")]
    );

    router.execute(alice.clone(), msg).unwrap();

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 3)
        .unwrap();
    assert_eq!(position.size, Integer::new_negative(7_215_767u64));
    assert_eq!(position.notional, Uint128::from(54_000_000u64));
    assert_eq!(position.margin, Uint128::from(54_000_000u64));
}

#[test]
fn test_ten_percent_fee_open_long_price_down_liquidation_with_positive_margin() {
    let NativeTokenScenario {
        mut router,
        owner,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();
    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(100_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.set_spread_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(10_000_000u64),
            Uint128::from(7_000_000u64),
            Uint128::from(18_000_000u64),
            Some(Uint128::zero()),
            Uint128::zero(),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(10_000_000u64),
                Uint128::from(10_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(10_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(5_000_000u64),
            Some(Uint128::from(22_000_000u64)),
            Uint128::zero(),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(10_000_000u64),
                Uint128::from(5_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(1_005_744u64));

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position.size, Integer::new_positive(2_056_807u64));
    assert_eq!(position.notional, Uint128::from(21_000_000u64));
    assert_eq!(position.margin, Uint128::from(3_000_000u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(1_000_000u64),
            Uint128::from(20_000_001u64),
            Uint128::from(6_000_000u64),
            Some(Uint128::from(15_000_000u64)),
            Uint128::zero(),
            calculate_funds_needed(
                &router.wrap(),
                Uint128::from(1_000_000u64),
                Uint128::from(20_000_000u64),
                vamm.addr(),
            )
            .unwrap(),
        )
        .unwrap();

    assert_eq!(
        calculate_funds_needed(
            &router.wrap(),
            Uint128::from(60_000_000u64),
            Uint128::from(1_000_000u64),
            vamm.addr(),
        )
        .unwrap(),
        vec![Coin::new(60_000_000u128, "orai")]
    );

    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Position is undercollateralized"
    );
}
