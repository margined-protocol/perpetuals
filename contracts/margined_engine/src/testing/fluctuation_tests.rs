// use crate::testing::setup::{self, to_decimals};
use cosmwasm_std::Uint128;
use cw20::Cw20ExecuteMsg;
use cw_multi_test::Executor;
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
        .update_config(
            None,
            None,
            None,
            None,
            None,
            Some(Uint128::from(200_000_000u128)), // 0.2
            None,
            None,
            None,
        )
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
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = vamm
        .update_config(
            None,
            None,
            None,
            None,
            None,
            Some(Uint128::from(78_000_000u128)), // 0.078
            None,
            None,
            None,
        )
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
        )
        .unwrap();
    let result = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: reply (id 2) error \"Generic error: price is over fluctuation limit\""
    )
}
