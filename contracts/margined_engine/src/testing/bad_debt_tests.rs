use cosmwasm_std::StdError;
use cw20::Cw20ExecuteMsg;
use cw_multi_test::Executor;
use margined_perp::margined_engine::Side;
use margined_utils::scenarios::{to_decimals, SimpleScenario};

#[test]
fn test_cannot_increase_position_when_bad_debt() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
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
                amount: to_decimals(1940),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // alice open small long
    // position size: 7.40740741
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(10u64),
            to_decimals(8u64),
            to_decimals(4u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // bob drop spot price
    for _ in 0..5 {
        let msg = engine
            .open_position(
                vamm.addr().to_string(),
                Side::Sell,
                to_decimals(10u64),
                to_decimals(10u64),
                to_decimals(0u64),
                vec![],
            )
            .unwrap();
        router.execute(bob.clone(), msg).unwrap();
    }

    router.update_block(|block| {
        block.time = block.time.plus_seconds(1);
        block.height += 1;
    });

    // increase position should fail since margin is not enough
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
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "Position is undercollateralized".to_string()
        },
        err.downcast().unwrap()
    );

    // pump spot price
    let msg = engine
        .close_position(vamm.addr().to_string(), to_decimals(0u64))
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // increase position should succeed since the position no longer has bad debt
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
}

#[test]
fn test_cannot_reduce_position_when_bad_debt() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
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
                amount: to_decimals(1940),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // alice open small long
    // position size: 7.40740741
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(10u64),
            to_decimals(8u64),
            to_decimals(4u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // bob drop spot price
    for _ in 0..5 {
        let msg = engine
            .open_position(
                vamm.addr().to_string(),
                Side::Sell,
                to_decimals(10u64),
                to_decimals(10u64),
                to_decimals(0u64),
                vec![],
            )
            .unwrap();
        router.execute(bob.clone(), msg).unwrap();
    }

    router.update_block(|block| {
        block.time = block.time.plus_seconds(1);
        block.height += 1;
    });

    // increase position should fail since margin is not enough
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(1u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "Position is undercollateralized".to_string()
        },
        err.downcast().unwrap()
    );

    // pump spot price
    let msg = engine
        .close_position(vamm.addr().to_string(), to_decimals(0u64))
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // increase position should succeed since the position no longer has bad debt
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(1u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_cannot_close_position_when_bad_debt() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
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
                amount: to_decimals(1940),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // alice open small long
    // position size: 7.40740741
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(10u64),
            to_decimals(8u64),
            to_decimals(4u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // bob drop spot price
    for _ in 0..5 {
        let msg = engine
            .open_position(
                vamm.addr().to_string(),
                Side::Sell,
                to_decimals(10u64),
                to_decimals(10u64),
                to_decimals(0u64),
                vec![],
            )
            .unwrap();
        router.execute(bob.clone(), msg).unwrap();
    }

    router.update_block(|block| {
        block.time = block.time.plus_seconds(1);
        block.height += 1;
    });

    // close position should fail since bad debt
    // open notional = 80
    // estimated realized PnL (partial close) = 7.4 * 3.36 - 80 = -55.136
    // estimated remaining margin = 10 + (-55.136) = -45.136
    // real bad debt: 46.10795455
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

    // pump spot price
    let msg = engine
        .close_position(vamm.addr().to_string(), to_decimals(0u64))
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let msg = engine
        .close_position(vamm.addr().to_string(), to_decimals(1u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}
