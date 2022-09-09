use cosmwasm_std::{Empty, StdError, Uint128};
use cw20::Cw20ExecuteMsg;
use cw_multi_test::Executor;
use margined_common::integer::Integer;
use margined_perp::margined_engine::{PnlCalcOption, Position, Side};
use margined_utils::scenarios::{to_decimals, SimpleScenario};

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
    } = SimpleScenario::new();

    // verfiy the balances
    let owner_balance = usdc.balance::<_, _, Empty>(&router, owner.clone()).unwrap();
    assert_eq!(owner_balance, Uint128::zero());
    let alice_balance = usdc.balance::<_, _, Empty>(&router, alice.clone()).unwrap();
    assert_eq!(alice_balance, Uint128::new(5_000_000_000_000));
    let bob_balance = usdc.balance::<_, _, Empty>(&router, bob.clone()).unwrap();
    assert_eq!(bob_balance, Uint128::new(5_000_000_000_000));
    let engine_balance = usdc
        .balance::<_, _, Empty>(&router, engine.addr().clone())
        .unwrap();
    assert_eq!(engine_balance, Uint128::zero());
}

#[test]
fn test_force_error_open_position_zero_leverage_or_fractional_leverage() {
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
            to_decimals(60u64),
            to_decimals(0u64),
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

    // expect to be 60
    let margin = engine
        .get_balance_with_funding_payment(&router, alice.to_string())
        .unwrap();
    assert_eq!(margin, to_decimals(60));

    // personal position should be 37.5
    let positions: Vec<Position> = engine
        .get_all_positions(&router, alice.to_string())
        .unwrap();
    assert_eq!(positions[0].size, Integer::new_positive(37_500_000_000u128));
    assert_eq!(positions[0].margin, to_decimals(60u64));

    // clearing house token balance should be 60
    let engine_balance = usdc
        .balance::<_, _, Empty>(&router, engine.addr().clone())
        .unwrap();
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

    // expect to be 60
    let margin = engine
        .get_balance_with_funding_payment(&router, alice.to_string())
        .unwrap();
    assert_eq!(margin, to_decimals(60));

    // personal position should be 37.5
    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_positive(37_500_000_000u128));
    assert_eq!(position.margin, to_decimals(60u64));

    // clearing house token balance should be 60
    let engine_balance = usdc
        .balance::<_, _, Empty>(&router, engine.addr().clone())
        .unwrap();
    assert_eq!(engine_balance, to_decimals(60));
}

#[test]
fn test_open_position_two_longs() {
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
            to_decimals(60u64),
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
            to_decimals(60u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // expect to be 120
    let margin = engine
        .get_balance_with_funding_payment(&router, alice.to_string())
        .unwrap();
    assert_eq!(margin, to_decimals(120));

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_positive(54_545_454_545u128));
    assert_eq!(position.margin, to_decimals(120));
}

#[test]
fn test_open_position_two_shorts() {
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
            Side::Sell,
            to_decimals(40u64),
            to_decimals(5u64),
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
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // personal balance with funding payment
    let margin = engine
        .get_balance_with_funding_payment(&router, alice.to_string())
        .unwrap();
    assert_eq!(margin, to_decimals(80));

    // retrieve the vamm state
    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_negative(66_666_666_667u128));
    assert_eq!(position.margin, to_decimals(80));
}

#[test]
fn test_open_position_equal_size_opposite_side() {
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
            to_decimals(60u64),
            to_decimals(10u64),
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
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // personal balance with funding payment
    let margin = engine
        .get_balance_with_funding_payment(&router, alice.to_string())
        .unwrap();
    assert_eq!(margin, Uint128::zero());

    // retrieve the vamm state
    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::zero());
    assert_eq!(position.margin, Uint128::zero());
}

#[test]
fn test_open_position_one_long_two_shorts() {
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
            to_decimals(60u64),
            to_decimals(10u64),
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
            to_decimals(5u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // retrieve the vamm state
    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_positive(33_333_333_333u128));
    assert_eq!(position.margin, to_decimals(60));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(50u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // personal balance with funding payment
    let margin = engine
        .get_balance_with_funding_payment(&router, alice.to_string())
        .unwrap();
    assert_eq!(margin, Uint128::zero());

    // retrieve the vamm state
    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::zero());
    assert_eq!(position.margin, Uint128::zero());
}

#[test]
fn test_open_position_short_and_two_longs() {
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
            Side::Sell,
            to_decimals(40u64),
            to_decimals(5u64),
            to_decimals(25u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_negative(25_000_000_000u128));
    assert_eq!(position.margin, to_decimals(40));
    assert_eq!(position.notional, to_decimals(200));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(20u64),
            to_decimals(5u64),
            Uint128::from(13_800_000_000u128),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(8u64));

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();

    assert_eq!(position.size, Integer::new_negative(11_111_111_112u128));
    assert_eq!(position.notional, to_decimals(100));
    assert_eq!(position.margin, Uint128::from(40_000_000_000u128));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(10u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // retrieve the vamm state
    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_negative(1_u128));
    assert_eq!(position.margin, Uint128::from(39_999_999_993u128));
}

#[test]
fn test_open_position_short_long_short() {
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
            Side::Sell,
            to_decimals(20u64),
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
            to_decimals(150u64),
            to_decimals(3u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_positive(to_decimals(20u64)));
    assert_eq!(position.margin, Uint128::new(83_333_333_333));

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

    // retrieve the vamm state
    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::zero());
    assert_eq!(position.margin, Uint128::zero());
}

#[test]
fn test_open_position_long_short_long() {
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

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(150u64),
            to_decimals(3u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_negative(to_decimals(25u64)));
    assert_eq!(position.margin, Uint128::new(66_666_666_666));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(20u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // retrieve the vamm state
    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::zero());
    assert_eq!(position.margin, Uint128::zero());
}

#[test]
fn test_pnl_zero_no_others_trading() {
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
            to_decimals(250u64),
            to_decimals(1u64),
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
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::zero());
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
    } = SimpleScenario::new();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(50u64),
            to_decimals(2u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // retrieve the vamm state
    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_negative(11_111_111_112u128));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(10u64),
            to_decimals(6u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let state = vamm.state(&router).unwrap();
    assert_eq!(state.quote_asset_reserve, to_decimals(960));
    assert_eq!(state.base_asset_reserve, Uint128::from(104_166_666_668u128));

    let msg = engine
        .close_position(vamm.addr().to_string(), to_decimals(0u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let err = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "Querier contract error: Generic error: No position found".to_string()
        },
        err
    );

    let state = vamm.state(&router).unwrap();
    assert_eq!(
        state.quote_asset_reserve,
        Uint128::from(1_074_626_865_681u128)
    );
    assert_eq!(state.base_asset_reserve, Uint128::from(93_055_555_556u128));

    // alice balance should be 4985.373134319
    let engine_balance = usdc.balance::<_, _, Empty>(&router, alice.clone()).unwrap();
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

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_positive(to_decimals(20)));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(35_080_000_000u128),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let msg = engine
        .close_position(vamm.addr().to_string(), to_decimals(0u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let err = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "Querier contract error: Generic error: No position found".to_string()
        },
        err
    );

    let state = vamm.state(&router).unwrap();
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

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::new_positive(to_decimals(20)));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(250u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // Now Alice's position is {balance: 20, margin: 25}
    // positionValue of 20 quoteAsset is 166.67 now
    // marginRatio = (margin(25) + unrealizedPnl(166.67-250)) / openNotionalSize(250) = -23%
    let msg = engine
        .close_position(vamm.addr().to_string(), to_decimals(0u64))
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
    } = SimpleScenario::new();

    let msg = engine
        .close_position(vamm.addr().to_string(), to_decimals(0u64))
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "Position is zero".to_string()
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
    } = SimpleScenario::new();

    let msg = vamm
        .update_config(
            None,
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
        .close_position(vamm.addr().to_string(), to_decimals(0u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = usdc
        .balance::<_, _, Empty>(&router, engine.addr().clone())
        .unwrap();
    assert_eq!(engine_balance, to_decimals(0u64));

    let insurance_balance = usdc
        .balance::<_, _, Empty>(&router, insurance_fund.addr().clone())
        .unwrap();
    assert_eq!(insurance_balance, to_decimals(5024u64));

    let fee_pool_balance = usdc
        .balance::<_, _, Empty>(&router, fee_pool.addr().clone())
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
    } = SimpleScenario::new();

    let msg = vamm
        .update_config(
            None,
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
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let fee_pool_balance = usdc
        .balance::<_, _, Empty>(&router, fee_pool.addr().clone())
        .unwrap();
    assert_eq!(fee_pool_balance, to_decimals(6u64));

    let msg = vamm.set_toll_ratio(Uint128::from(50_000_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(20u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let fee_pool_balance = usdc
        .balance::<_, _, Empty>(&router, fee_pool.addr().clone())
        .unwrap();
    assert_eq!(fee_pool_balance, to_decimals(16u64));

    let msg = engine
        .close_position(vamm.addr().to_string(), to_decimals(0u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let fee_pool_balance = usdc
        .balance::<_, _, Empty>(&router, fee_pool.addr().clone())
        .unwrap();
    assert_eq!(fee_pool_balance, to_decimals(56u64));
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
    } = SimpleScenario::new();

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
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let position: Position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
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
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
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
    } = SimpleScenario::new();

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
    } = SimpleScenario::new();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(21u64),
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
    } = SimpleScenario::new();

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
        .close_position(vamm.addr().to_string(), to_decimals(0u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance = usdc.balance::<_, _, Empty>(&router, alice.clone()).unwrap();
    assert_eq!(alice_balance, Uint128::from(5_094_117_647_059u128));

    let engine_balance = usdc
        .balance::<_, _, Empty>(&router, engine.addr().clone())
        .unwrap();
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
        .get_margin_ratio(&router, vamm.addr().to_string(), bob.to_string())
        .unwrap();
    assert_eq!(margin_ratio, Integer::new_negative(252_000_000u128));

    let msg = engine
        .close_position(vamm.addr().to_string(), to_decimals(0u64))
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let bob_balance = usdc.balance::<_, _, Empty>(&router, bob.clone()).unwrap();
    assert_eq!(bob_balance, Uint128::from(4_980_000_000_000u128));

    let engine_balance = usdc
        .balance::<_, _, Empty>(&router, engine.addr().clone())
        .unwrap();
    assert_eq!(engine_balance, Uint128::zero());

    let insurance_balance = usdc
        .balance::<_, _, Empty>(&router, insurance_fund.addr().clone())
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
    } = SimpleScenario::new();

    // alice opens a position
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(2u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // alice closes her position
    let msg = engine
        .close_position(vamm.addr().to_string(), to_decimals(0u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // we query alices' position to ensure it was closed
    let err = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "Querier contract error: Generic error: No position found".to_string()
        },
        err
    );

    // we query all of bob's positions (should return an empty array)
    let positions: Vec<Position> = engine.get_all_positions(&router, bob.to_string()).unwrap();
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
    } = SimpleScenario::new();

    // insurance contract have 5000 USDC balance
    let insurance_balance = usdc
        .balance::<_, _, Empty>(&router, insurance_fund.addr().clone())
        .unwrap();
    assert_eq!(insurance_balance, to_decimals(5000u64));

    // alice opens a 60 USDC position
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

    // engine contract now have 60 USDC (0 balance + 60)
    let engine_balance = usdc
        .balance::<_, _, Empty>(&router, engine.addr().clone())
        .unwrap();
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
    let engine_balance = usdc
        .balance::<_, _, Empty>(&router, engine.addr().clone())
        .unwrap();
    assert_eq!(engine_balance, to_decimals(0u64));

    // alice withdraws 20 USDC margin from engine contract while engine contract
    // have 0 balance
    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), to_decimals(20u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // shortfall event occured, insurance funds are used instead
    // insurance contract should have 4980 USDC (5000 balance - 20)
    let insurance_balance = usdc
        .balance::<_, _, Empty>(&router, insurance_fund.addr().clone())
        .unwrap();
    assert_eq!(insurance_balance, to_decimals(4980u64));

    // engine contract's state should reflect bad debt as 20
    let bad_debt = engine.state(&router).unwrap().bad_debt;
    assert_eq!(bad_debt, to_decimals(20u64));
}
