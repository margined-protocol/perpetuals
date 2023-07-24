use cosmwasm_std::{StdError, Uint128};
use cw20::Cw20ExecuteMsg;
use margined_common::integer::Integer;
use margined_perp::margined_engine::{PnlCalcOption, Side, PositionFilter};
use margined_utils::{
    cw_multi_test::Executor,
    testing::{to_decimals, SimpleScenario},
};

use crate::testing::new_simple_scenario;

#[test]
fn test_initialization() {
    let SimpleScenario {
        router,
        owner,
        alice,
        bob,
        usdc,
        engine,
        ..
    } = new_simple_scenario();

    // verfiy the balances
    let owner_balance = usdc.balance(&router.wrap(), owner.clone()).unwrap();
    assert_eq!(owner_balance, Uint128::zero());
    let alice_balance = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(alice_balance, Uint128::new(5_000_000_000_000));
    let bob_balance = usdc.balance(&router.wrap(), bob.clone()).unwrap();
    assert_eq!(bob_balance, Uint128::new(5_000_000_000_000));
    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, Uint128::zero());
}

#[test]
fn test_force_error_open_position_zero() {
    let SimpleScenario {
        mut router,
        alice,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::zero(),
            to_decimals(1u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Input must be non-zero".to_string()
    );
}

#[test]
fn test_force_error_open_position_zero_leverage_or_fractional_leverage() {
    let SimpleScenario {
        mut router,
        alice,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(0u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Input must be non-zero".to_string()
    );

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            Uint128::from(1u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Leverage must be greater than 1".to_string()
    );
}

#[test]
fn test_get_all_positions_open_position_long() {
    let SimpleScenario {
        mut router,
        alice,
        usdc,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(10u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // expect to be 60
    let margin = engine
        .get_balance_with_funding_payment(&router.wrap(), 1)
        .unwrap();
    assert_eq!(margin, to_decimals(60));

    // personal position should be 37.5
    let positions = engine
        .get_all_positions(&router.wrap(), alice.to_string(), None, None, None)
        .unwrap();
    assert_eq!(positions[0].size, Integer::new_positive(37_500_000_000u128));
    assert_eq!(positions[0].margin, to_decimals(60u64));

    // clearing house token balance should be 60
    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, to_decimals(60));
}

#[test]
fn test_open_position_long() {
    let SimpleScenario {
        mut router,
        alice,
        usdc,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(10u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // expect to be 60
    let margin = engine
        .get_balance_with_funding_payment(&router.wrap(), 1)
        .unwrap();
    assert_eq!(margin, to_decimals(60));

    // personal position should be 37.5
    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    println!("position.notional: {:?}", position.notional);
    assert_eq!(position.size, Integer::new_positive(37_500_000_000u128)); //37_500_000_000 // 600_000_000_000
    assert_eq!(position.margin, to_decimals(60u64));

    // clearing house token balance should be 60
    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, to_decimals(60));
}

#[test]
fn test_open_position_two_longs() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(10u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
    
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(33u64),
            to_decimals(5u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(10u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(55u64),
            to_decimals(5u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(42u64),
            to_decimals(7u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(5u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let buy_positions = engine
        .get_positions(&router.wrap(), vamm.addr().to_string(), PositionFilter::None, Some(Side::Buy), None, None, None)
        .unwrap();
    for position in buy_positions {
        println!("buy position: {:?}", position);
    }
    println!("\n\n\n");

    let sell_positions = engine
        .get_positions(&router.wrap(), vamm.addr().to_string(), PositionFilter::None, Some(Side::Sell), None, None, None)
        .unwrap();
    for position in sell_positions {
        println!("sell position: {:?}", position);
    }
    println!("\n\n\n");

    let alice_positions = engine
        .get_positions(&router.wrap(), vamm.addr().to_string(), PositionFilter::Trader(alice.to_string()), None, None, None, None)
        .unwrap();
    for position in alice_positions {
        println!("alice's position: {:?}", position);
    }
    println!("\n\n\n");

    let bob_positions = engine
        .get_positions(&router.wrap(), vamm.addr().to_string(), PositionFilter::Trader(bob.to_string()), None, None, None, None)
        .unwrap();
    for position in bob_positions {
        println!("bob's position: {:?}", position);
    }
    println!("\n\n\n");

    let all_positions = engine
        .get_positions(&router.wrap(), vamm.addr().to_string(), PositionFilter::None, None, None, None, None)
        .unwrap();
    for position in all_positions {
        println!("position: {:?}", position);
    }
    println!("\n\n\n");

    // expect to be 120
    let margin_1 = engine
        .get_balance_with_funding_payment(&router.wrap(), 1)
        .unwrap();
    let margin_2 = engine
        .get_balance_with_funding_payment(&router.wrap(), 2)
        .unwrap();
    assert_eq!(margin_1 + margin_2, to_decimals(93));

    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    let position_2 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 2)
        .unwrap();
    assert_eq!(position_1.size + position_2.size, Integer::new_positive(43_342_776_203u128));
    assert_eq!(position_1.margin + position_2.margin, to_decimals(93));
}

#[test]
fn test_open_position_two_shorts() {
    let SimpleScenario {
        mut router,
        alice,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(40u64),
            to_decimals(5u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(40u64),
            to_decimals(5u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // personal balance with funding payment
    let margin_1 = engine
        .get_balance_with_funding_payment(&router.wrap(), 1)
        .unwrap();
    let margin_2 = engine
        .get_balance_with_funding_payment(&router.wrap(), 2)
        .unwrap();
    assert_eq!(margin_1 + margin_2, to_decimals(80));

    // retrieve the vamm state
    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    let position_2 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 2)
        .unwrap();
    assert_eq!(position_1.size + position_2.size, Integer::new_negative(66_666_666_667u128));
    assert_eq!(position_1.margin + position_2.margin, to_decimals(80));
}

#[test]
fn test_open_position_equal_size_opposite_side() {
    let SimpleScenario {
        mut router,
        alice,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(10u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(300u64),
            to_decimals(2u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // personal balance with funding payment
    let margin_1 = engine
        .get_balance_with_funding_payment(&router.wrap(), 1)
        .unwrap();
    let margin_2 = engine
        .get_balance_with_funding_payment(&router.wrap(), 2)
        .unwrap();
    assert_eq!(margin_1, to_decimals(60));
    assert_eq!(margin_2, to_decimals(300));

    // retrieve the vamm state
    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    let position_2 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 2)
        .unwrap();
    assert_eq!(position_1.size + position_2.size, Integer::zero());
}

#[test]
fn test_open_position_short_and_two_longs() {
    let SimpleScenario {
        mut router,
        alice,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(40u64),
            to_decimals(5u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(25u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position_1.size, Integer::new_negative(25_000_000_000u128));
    assert_eq!(position_1.margin, to_decimals(40));
    assert_eq!(position_1.notional, to_decimals(200));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(20u64),
            to_decimals(5u64),
            to_decimals(10),
            Some(Uint128::zero()),
            Uint128::from(13_800_000_000u128),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            2,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(7u64));

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 2)
        .unwrap();

    assert_eq!(position.size, Integer::new_positive(13_888_888_888u128));
    assert_eq!(position.notional, to_decimals(100));
    assert_eq!(position.margin, Uint128::from(20_000_000_000u128));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(10u64),
            to_decimals(10u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // retrieve the vamm state
    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 3)
        .unwrap();
    assert_eq!(position.size, Integer::new_positive(11_111_111_111u128));
    assert_eq!(position.margin, Uint128::from(10_000_000_000u128));
}

#[test]
fn test_open_position_short_long_short() {
    let SimpleScenario {
        mut router,
        alice,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(10u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(150u64),
            to_decimals(3u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position_1.size, Integer::new_negative(to_decimals(25u64)));
    assert_eq!(position_1.margin, Uint128::new(20_000_000_000));

    let position_2 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 2)
        .unwrap();
    assert_eq!(position_2.size, Integer::new_positive(to_decimals(45u64)));
    assert_eq!(position_2.margin, Uint128::new(150_000_000_000));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(25u64),
            to_decimals(10u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position_3 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 3)
        .unwrap();
    assert_eq!(position_3.size, Integer::new_negative(to_decimals(20u64)));
    assert_eq!(position_3.margin, Uint128::new(25_000_000_000));
}

#[test]
fn test_open_position_long_short_long() {
    let SimpleScenario {
        mut router,
        alice,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(25u64),
            to_decimals(10u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(150u64),
            to_decimals(3u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // position 1
    let position_1 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position_1.size, Integer::new_positive(to_decimals(20u64)));
    assert_eq!(position_1.margin, Uint128::new(25_000_000_000));
    
    // position 2
    let position_2 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 2)
        .unwrap();
    assert_eq!(position_2.size, Integer::new_negative(to_decimals(45u64)));
    assert_eq!(position_2.margin, Uint128::new(150_000_000_000));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(20u64),
            to_decimals(10u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // position 3
    let position_3 = engine
        .position(&router.wrap(), vamm.addr().to_string(), 3)
        .unwrap();
    assert_eq!(position_3.size, Integer::new_positive(to_decimals(25u64)));
    assert_eq!(position_3.margin, Uint128::new(20_000_000_000)); 
}

#[test]
fn test_pnl_zero_no_others_trading() {
    let SimpleScenario {
        mut router,
        alice,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(250u64),
            to_decimals(1u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(750u64),
            to_decimals(1u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let pnl_1= engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();

    let pnl_2 = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            2,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl_1.unrealized_pnl, Integer::new_positive(321428571428u128));
    assert_eq!(pnl_2.unrealized_pnl, Integer::zero());
}

#[test]
fn test_close_safe_position() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        engine,
        usdc,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(50u64),
            to_decimals(2u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // retrieve the vamm state
    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position.size, Integer::new_negative(11_111_111_112u128));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(10u64),
            to_decimals(6u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let state = vamm.state(&router.wrap()).unwrap();
    assert_eq!(state.quote_asset_reserve, to_decimals(960));
    assert_eq!(state.base_asset_reserve, Uint128::from(104_166_666_668u128));

    let msg = engine
        .close_position(vamm.addr().to_string(), 1, to_decimals(0u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let err = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "Querier contract error: margined_perp::margined_engine::Position not found".to_string()
        },
        err
    );

    let state = vamm.state(&router.wrap()).unwrap();
    assert_eq!(
        state.quote_asset_reserve,
        Uint128::from(1_074_626_865_681u128)
    );
    assert_eq!(state.base_asset_reserve, Uint128::from(93_055_555_556u128));

    // alice balance should be 4985.373134319
    let engine_balance = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(engine_balance, Uint128::from(4_985_373_134_319u128));
}

#[test]
fn test_close_position_over_maintenance_margin_ratio() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(25u64),
            to_decimals(10u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position.size, Integer::new_positive(to_decimals(20)));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(35_080_000_000u128),
            to_decimals(1u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let msg = engine
        .close_position(vamm.addr().to_string(), 1, to_decimals(0u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let err = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "Querier contract error: margined_perp::margined_engine::Position not found".to_string()
        },
        err
    );

    let state = vamm.state(&router.wrap()).unwrap();
    assert_eq!(
        state.quote_asset_reserve,
        Uint128::from(977_422_074_621u128)
    );
    assert_eq!(state.base_asset_reserve, Uint128::from(102_309_946_334u128));
}

#[test]
fn test_cannot_close_position_with_bad_debt() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(25u64),
            to_decimals(10u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position.size, Integer::new_positive(to_decimals(20)));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(250u64),
            to_decimals(1u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // Now Alice's position is {balance: 20, margin: 25}
    // positionValue of 20 quoteAsset is 166.67 now
    // marginRatio = (margin(25) + unrealizedPnl(166.67-250)) / openNotionalSize(250) = -23%
    let msg = engine
        .close_position(vamm.addr().to_string(), 1, to_decimals(0u64))
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "Cannot close position - bad debt".to_string()
        },
        err.downcast().unwrap()
    );
}

#[test]
fn test_close_zero_position() {
    let SimpleScenario {
        mut router,
        alice,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = engine
        .close_position(vamm.addr().to_string(), 1, to_decimals(0u64))
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::NotFound {
            kind: "margined_perp::margined_engine::Position".to_string()
        },
        err.downcast().unwrap()
    );
}

#[test]
fn test_openclose_position_to_check_fee_is_charged() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        insurance_fund,
        fee_pool,
        engine,
        vamm,
        usdc,
        ..
    } = new_simple_scenario();

    let msg = vamm
        .update_config(
            None,
            None,
            Some(Uint128::from(10_000_000u128)), // 0.01
            Some(Uint128::from(20_000_000u128)), // 0.01
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(10u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, to_decimals(60u64));

    let msg = engine
        .close_position(vamm.addr().to_string(), 1, to_decimals(0u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, to_decimals(0u64));

    let insurance_balance = usdc
        .balance(&router.wrap(), insurance_fund.addr().clone())
        .unwrap();
    assert_eq!(insurance_balance, to_decimals(5024u64));

    let fee_pool_balance = usdc
        .balance(&router.wrap(), fee_pool.addr().clone())
        .unwrap();
    assert_eq!(fee_pool_balance, to_decimals(12u64));
}

#[test]
fn test_openclose_position_to_check_fee_is_charged_toll_ratio_5_percent() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        fee_pool,
        engine,
        vamm,
        usdc,
        ..
    } = new_simple_scenario();

    let msg = vamm
        .update_config(
            None,
            None,
            Some(Uint128::from(10_000_000u128)), // 0.01
            Some(Uint128::from(20_000_000u128)), // 0.01
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(10u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let fee_pool_balance = usdc
        .balance(&router.wrap(), fee_pool.addr().clone())
        .unwrap();
    assert_eq!(fee_pool_balance, to_decimals(6u64));

    let msg = vamm.set_toll_ratio(Uint128::from(50_000_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(58u64),
            to_decimals(10u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let fee_pool_balance = usdc
        .balance(&router.wrap(), fee_pool.addr().clone())
        .unwrap();
    assert_eq!(fee_pool_balance, to_decimals(35u64));

    let msg = engine
        .close_position(vamm.addr().to_string(), 1, to_decimals(0u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
    let fee_pool_balance = usdc
        .balance(&router.wrap(), fee_pool.addr().clone())
        .unwrap();
    assert_eq!(fee_pool_balance, to_decimals(65u64));
}

#[test]
fn test_pnl_unrealized() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    // Alice long by 25 base token with leverage 10x to get 20 ptoken
    // 25 * 10 = 250 which is x
    // (1000 + 250) * (100 + y) = 1000 * 100
    // so y = -20
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(25u64),
            to_decimals(10u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // Bob's balance in clearingHouse: 2000
    // current equation is:
    // (1250 + x) * (80 + y) = 1000 * 100
    // Bob short by 100 base token with leverage 10x to get -320 ptoken
    // 100 * 10 = 1000 which is x
    // (1250 - 1000) * (80 + y) = 1000 * 100
    // so y = 320
    //
    // and current equation is :
    // (250 + x) * (400 + y) = 1000 * 100
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(100u64),
            to_decimals(10u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position.size, Integer::new_positive(to_decimals(20u64)));

    // calculate Alice's unrealized PNL:
    // Alice has position 20 ptoken, so
    // (250 + x) * (400 + 20) = 1000 * 100
    // x = -11.9047619048
    // alice will get 11.9047619048 if she close position
    // since Alice use 250 to buy
    // 11.9047619048 - 250 = -238.0952380952 which is unrealized PNL.
    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(
        pnl.unrealized_pnl,
        Integer::new_negative(238_095_238_096u64)
    );
}

#[test]
fn test_error_open_position_insufficient_balance() {
    let SimpleScenario {
        mut router,
        alice,
        engine,
        vamm,
        usdc,
        ..
    } = new_simple_scenario();

    // reduce the allowance
    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1900),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(10u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(10u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "transfer failure - reply (id 9)".to_string()
        },
        err.downcast().unwrap()
    );
}

#[test]
fn test_error_open_position_exceed_margin_ratio() {
    let SimpleScenario {
        mut router,
        alice,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(21u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "Position is undercollateralized".to_string()
        },
        err.downcast().unwrap(),
    );
}

#[test]
fn test_alice_take_profit_from_bob_unrealized_undercollateralized_position_bob_closes() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        owner,
        insurance_fund,
        engine,
        usdc,
        vamm,
        ..
    } = new_simple_scenario();

    // avoid actions from exceeding the fluctuation limit
    let msg = vamm
        .set_fluctuation_limit_ratio(Uint128::from(800_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // reduce the allowance
    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1980),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // reduce the allowance
    router
        .execute_contract(
            bob.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1980),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(10u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(10u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(900);
        block.height += 1;
    });

    // alice close position, pnl = 200 -105.88 ~= 94.12
    // receive pnl + margin = 114.12
    let msg = engine
        .close_position(vamm.addr().to_string(), 1, to_decimals(0u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(alice_balance, Uint128::from(5_094_117_647_059u128));

    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, Uint128::zero());

    router.update_block(|block| {
        block.time = block.time.plus_seconds(900);
        block.height += 1;
    });

    // bob close his under collateral position, positionValue is -294.11
    // bob's pnl = 200 - 294.11 ~= -94.12
    // bob loss all his margin (20) with additional 74.12 badDebt
    // which is already prepaid by insurance fund when alice close the position before
    // clearing house doesn't need to ask insurance fund for covering the bad debt
    let margin_ratio = engine
        .get_margin_ratio(&router.wrap(), vamm.addr().to_string(), 2)
        .unwrap();
    assert_eq!(margin_ratio, Integer::new_negative(252_000_000u128));

    let msg = engine
        .close_position(vamm.addr().to_string(),  2, to_decimals(0u64))
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let bob_balance = usdc.balance(&router.wrap(), bob.clone()).unwrap();
    assert_eq!(bob_balance, Uint128::from(4_980_000_000_000u128));

    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, Uint128::zero());

    let insurance_balance = usdc
        .balance(&router.wrap(), insurance_fund.addr().clone())
        .unwrap();
    assert_eq!(insurance_balance, Uint128::from(4_925_882_352_941u128));
}

#[test]
fn test_query_no_user_positions() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    // alice opens a position
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(2u64),
            to_decimals(10u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // alice closes her position
    let msg = engine
        .close_position(vamm.addr().to_string(),  1, to_decimals(0u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // we query alices' position to ensure it was closed
    let err = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "Querier contract error: margined_perp::margined_engine::Position not found".to_string()
        },
        err
    );

    // we query all of bob's positions (should return an empty array)
    let positions = engine
        .get_all_positions(&router.wrap(), bob.to_string(), None, None, None)
        .unwrap();
    assert_eq!(positions, vec![]);
}

#[test]
fn test_bad_debt_recorded() {
    let SimpleScenario {
        mut router,
        alice,
        engine,
        vamm,
        usdc,
        insurance_fund,
        ..
    } = new_simple_scenario();

    // insurance contract have 5000 USDC balance
    let insurance_balance = usdc
        .balance(&router.wrap(), insurance_fund.addr().clone())
        .unwrap();
    assert_eq!(insurance_balance, to_decimals(5000u64));

    // alice opens a 60 USDC position
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(10u64),
            to_decimals(10),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // engine contract now have 60 USDC (0 balance + 60)
    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, to_decimals(60u64));

    // from oak:
    // we try to trigger bad debt by making the engine contract having
    // insufficient balance to repay the user
    // we do this via mocking the engine contract to burn all their USDC
    // this would cause the engine contract unable to repay the user, hence
    // having a positive amount of bad debt
    // burn all usdc from engine balance
    let burn_msg = Cw20ExecuteMsg::Burn {
        amount: engine_balance,
    };

    // wrap into CosmosMsg
    let cosmos_burn_msg = usdc.call(burn_msg).unwrap();

    // execute the burn msg
    router
        .execute(engine.addr().clone(), cosmos_burn_msg)
        .unwrap();

    // engine contract now have 0 balance
    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, to_decimals(0u64));

    // alice withdraws 20 USDC margin from engine contract while engine contract
    // have 0 balance
    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), 1, to_decimals(20u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // shortfall event occured, insurance funds are used instead
    // insurance contract should have 4980 USDC (5000 balance - 20)
    let insurance_balance = usdc
        .balance(&router.wrap(), insurance_fund.addr().clone())
        .unwrap();
    assert_eq!(insurance_balance, to_decimals(4980u64));

    // engine contract's state should reflect bad debt as 20
    let bad_debt = engine.state(&router.wrap()).unwrap().bad_debt;
    assert_eq!(bad_debt, to_decimals(20u64));
}
