use cosmwasm_std::Uint128;
use cw20::Cw20ExecuteMsg;
use cw_multi_test::Executor;
use margined_perp::margined_engine::Side;
use margined_utils::scenarios::{to_decimals, SimpleScenario};

#[test]
fn test_increase_with_increase_position() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        usdc,
        ..
    } = SimpleScenario::new();

    let msg = vamm
        .set_open_interest_notional_cap(Uint128::from(600_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // reduce the allowance
    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1400),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(600u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let open_interest_notional = engine.state(&router).unwrap().open_interest_notional;
    assert_eq!(open_interest_notional, to_decimals(600u64));
}

#[test]
fn test_reduce_when_position_is_reduced() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        usdc,
        ..
    } = SimpleScenario::new();

    let msg = vamm
        .set_open_interest_notional_cap(Uint128::from(600_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // reduce the allowance
    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1400),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(600u64),
            to_decimals(1u64),
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
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let open_interest_notional = engine.state(&router).unwrap().open_interest_notional;
    assert_eq!(open_interest_notional, to_decimals(300u64));
}

#[test]
fn test_reduce_when_close_position() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        usdc,
        ..
    } = SimpleScenario::new();

    let msg = vamm
        .set_open_interest_notional_cap(Uint128::from(600_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // reduce the allowance
    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1400),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(400u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .close_position(vamm.addr().to_string(), to_decimals(0u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let open_interest_notional = engine.state(&router).unwrap().open_interest_notional;
    // this is near zero due to some rounding errors
    assert!(open_interest_notional < to_decimals(10u64));
}

#[test]
fn test_increase_when_traders_open_positions_in_diff_directions() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        owner,
        engine,
        vamm,
        usdc,
        ..
    } = SimpleScenario::new();

    let msg = vamm
        .set_open_interest_notional_cap(Uint128::from(600_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // reduce the allowance
    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1700),
                expires: None,
            },
            &[],
        )
        .unwrap();

    router
        .execute_contract(
            bob.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1700),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(300u64),
            to_decimals(1u64),
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
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let open_interest_notional = engine.state(&router).unwrap().open_interest_notional;
    assert_eq!(open_interest_notional, to_decimals(600u64));
}

#[test]
fn test_increase_when_traders_open_larger_positions_in_reverse_directions() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        usdc,
        ..
    } = SimpleScenario::new();

    let msg = vamm
        .set_open_interest_notional_cap(Uint128::from(600_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // reduce the allowance
    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1400),
                expires: None,
            },
            &[],
        )
        .unwrap();

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
            Side::Sell,
            to_decimals(450u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let open_interest_notional = engine.state(&router).unwrap().open_interest_notional;
    // this is near zero due to some rounding errors
    assert_eq!(open_interest_notional, to_decimals(200u64));
}

#[test]
fn test_zero_when_everyone_closes_positions() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        owner,
        engine,
        vamm,
        usdc,
        ..
    } = SimpleScenario::new();

    let msg = vamm
        .set_open_interest_notional_cap(Uint128::from(600_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // reduce the allowance
    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1400),
                expires: None,
            },
            &[],
        )
        .unwrap();

    router
        .execute_contract(
            bob.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1400),
                expires: None,
            },
            &[],
        )
        .unwrap();

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
            Side::Sell,
            to_decimals(250u64),
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

    let msg = engine
        .close_position(vamm.addr().to_string(), to_decimals(0u64))
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let open_interest_notional = engine.state(&router).unwrap().open_interest_notional;
    // this is near zero due to some rounding errors
    assert!(open_interest_notional < to_decimals(10u64));
}

#[test]
fn test_zero_when_everyone_closes_positions_one_position_is_bankrupt() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        owner,
        engine,
        vamm,
        usdc,
        ..
    } = SimpleScenario::new();

    let msg = vamm
        .set_open_interest_notional_cap(Uint128::from(600_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // reduce the allowance
    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1400),
                expires: None,
            },
            &[],
        )
        .unwrap();

    router
        .execute_contract(
            bob.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1400),
                expires: None,
            },
            &[],
        )
        .unwrap();

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
    router.execute(alice.clone(), msg).unwrap();

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
    router.execute(bob.clone(), msg).unwrap();

    let msg = engine
        .liquidate(
            vamm.addr().to_string(),
            alice.to_string(),
            to_decimals(0u64),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = engine
        .close_position(vamm.addr().to_string(), to_decimals(0u64))
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let open_interest_notional = engine.state(&router).unwrap().open_interest_notional;
    // this is near zero due to some rounding errors
    assert!(open_interest_notional < to_decimals(10u64));
}

#[test]
fn test_stop_trading_if_over_open_interest_notional_cap() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        owner,
        engine,
        vamm,
        usdc,
        ..
    } = SimpleScenario::new();

    let msg = vamm
        .set_open_interest_notional_cap(Uint128::from(600_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // reduce the allowance
    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1400),
                expires: None,
            },
            &[],
        )
        .unwrap();

    router
        .execute_contract(
            bob.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1400),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(600u64),
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
            to_decimals(1u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    let result = router.execute(bob.clone(), msg).unwrap_err();
    assert_eq!(result.to_string(), "Generic error: over limit");
}
