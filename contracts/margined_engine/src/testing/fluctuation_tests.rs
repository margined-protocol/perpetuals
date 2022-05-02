// use crate::testing::setup::{self, to_decimals};
use cosmwasm_std::Uint128;
use cw20::Cw20ExecuteMsg;
use terra_multi_test::Executor;
use margined_perp::margined_engine::Side;
use margined_utils::scenarios::{to_decimals, SimpleScenario};

#[test]
fn test_force_error_open_position_exceeds_fluctuation_limit() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        usdc,
        vamm,
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

    let msg = vamm
        .set_fluctuation_limit_ratio(Uint128::from(200_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // alice pays 20 margin * 5x long quote when 9.0909091 base
    // AMM after: 1100 : 90.9090909, price: 12.1000000012
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::BUY,
            to_decimals(20u64),
            to_decimals(5u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    let result = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: reply (id 1) error \"Generic error: price is over fluctuation limit\""
    )
}

#[test]
fn test_force_error_reduce_position_exceeds_fluctuation_limit() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        usdc,
        vamm,
        ..
    } = SimpleScenario::new();

    // reduce the allowance
    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1500),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // alice pays 250 margin * 1x long to get 20 base
    // AMM after: 1250 : 80, price: 15.625
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::BUY,
            to_decimals(250u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = vamm
        .set_fluctuation_limit_ratio(Uint128::from(78_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // AMM after: 1200 : 83.3333333333, price: 14.4
    // price fluctuation: (15.625 - 14.4) / 15.625 = 0.0784
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::SELL,
            to_decimals(50u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    let result = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: reply (id 2) error \"Generic error: price is over fluctuation limit\""
    )
}

#[test]
fn test_close_position_limit_force_error_exceeding_fluctuation_limit_twice_in_same_block() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        owner,
        engine,
        usdc,
        vamm,
        ..
    } = SimpleScenario::new();

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

    router
        .execute_contract(
            bob.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1900),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // when bob create a 20 margin * 5x long position when 9.0909091 quoteAsset = 100
    // AMM after: 1100 : 90.9090909, price: 12.1000000012
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::BUY,
            to_decimals(20u64),
            to_decimals(5u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    // when alice create a 20 margin * 5x long position when 7.5757609 quoteAsset = 100
    // AMM after: 1200 : 83.3333333, price: 14.4000000058
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::BUY,
            to_decimals(20u64),
            to_decimals(5u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });
    let msg = vamm
        .set_fluctuation_limit_ratio(Uint128::from(43_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // after alice closes her position partially, price: 13.767109
    // price fluctuation: (14.4000000058 - 13.767109) / 14.4000000058 = 0.0524
    let msg = engine
        .close_position(vamm.addr().to_string(), to_decimals(0u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = vamm
        .set_fluctuation_limit_ratio(Uint128::from(42_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .close_position(vamm.addr().to_string(), to_decimals(0u64))
        .unwrap();
    let result = router.execute(bob.clone(), msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: reply (id 4) error \"Generic error: price is already over fluctuation limit\""
    )
}

#[test]
fn test_close_position_slippage_limit_originally_long() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        engine,
        usdc,
        vamm,
        ..
    } = SimpleScenario::new();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

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

    router
        .execute_contract(
            bob.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1900),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // when bob create a 20 margin * 5x long position when 9.0909091 quoteAsset = 100
    // AMM after: 1100 : 90.9090909, price: 12.1000000012
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::BUY,
            to_decimals(20u64),
            to_decimals(5u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    // when alice create a 20 margin * 5x long position when 7.5757609 quoteAsset = 100
    // AMM after: 1200 : 83.3333333, price: 14.4000000058
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::BUY,
            to_decimals(20u64),
            to_decimals(5u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    // when bob close his position
    // AMM after: 1081.96721 : 92.4242424
    let msg = engine
        .close_position(vamm.addr().to_string(), to_decimals(118u64))
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let state = vamm.state(&router).unwrap();
    assert_eq!(
        state.quote_asset_reserve,
        Uint128::from(1_081_967_213_128u128)
    );
    assert_eq!(state.base_asset_reserve, Uint128::from(92_424_242_425u128));
}

#[test]
fn test_close_position_slippage_limit_originally_short() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        engine,
        usdc,
        vamm,
        ..
    } = SimpleScenario::new();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

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

    router
        .execute_contract(
            bob.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1900),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // when bob create a 20 margin * 5x short position when 11.1111111111 quoteAsset = 100 DAI
    // AMM after: 900 : 111.1111111111
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::SELL,
            to_decimals(20u64),
            to_decimals(5u64),
            Uint128::from(11_111_111_112u128),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    // when alice create a 20 margin * 5x short position when 13.8888888889 quoteAsset = 100 DAI
    // AMM after: 800 : 125
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::SELL,
            to_decimals(20u64),
            to_decimals(5u64),
            Uint128::from(13_890_000_000u128),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    // when bob close his position
    // AMM after: 878.0487804877 : 113.8888888889
    let msg = engine
        .close_position(vamm.addr().to_string(), to_decimals(79u64))
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let state = vamm.state(&router).unwrap();
    assert_eq!(
        state.quote_asset_reserve,
        Uint128::from(878_048_780_494u128)
    );
    assert_eq!(state.base_asset_reserve, Uint128::from(113_888_888_889u128));
}

#[test]
fn test_force_error_close_position_slippage_limit_originally_long() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        engine,
        usdc,
        vamm,
        ..
    } = SimpleScenario::new();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

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

    router
        .execute_contract(
            bob.clone(),
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
            Side::BUY,
            to_decimals(20u64),
            to_decimals(5u64),
            to_decimals(9u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::BUY,
            to_decimals(20u64),
            to_decimals(5u64),
            Uint128::from(7_500_000_000u128),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = engine
        .close_position(vamm.addr().to_string(), to_decimals(119u64))
        .unwrap();
    let result = router.execute(bob.clone(), msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: reply (id 4) error \"Generic error: Less than minimum quote asset amount limit\""
    )
}

#[test]
fn test_force_error_close_position_slippage_limit_originally_short() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        engine,
        usdc,
        vamm,
        ..
    } = SimpleScenario::new();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

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

    router
        .execute_contract(
            bob.clone(),
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
            Side::SELL,
            to_decimals(20u64),
            to_decimals(5u64),
            Uint128::from(11_111_111_112u128),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::SELL,
            to_decimals(20u64),
            to_decimals(5u64),
            Uint128::from(13_890_000_000u128),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = engine
        .close_position(vamm.addr().to_string(), to_decimals(78u64))
        .unwrap();
    let result = router.execute(bob.clone(), msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: reply (id 4) error \"Generic error: Greater than maximum quote asset amount limit\""
    )
}
