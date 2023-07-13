use cosmwasm_std::{StdError, Uint128};
use cw20::Cw20ExecuteMsg;
use margined_common::integer::Integer;
use margined_perp::margined_engine::{PnlCalcOption, Side};
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
fn test_change_tp_sl() {
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
            Uint128::zero(),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // expect to be 60
    let margin = engine
        .get_balance_with_funding_payment(&router.wrap(), alice.to_string(), 1)
        .unwrap();
    assert_eq!(margin, to_decimals(60));

    // personal position should be 37.5
    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1, alice.to_string())
        .unwrap();
    println!("position.notional: {:?}", position.notional);
    assert_eq!(position.size, Integer::new_positive(37_500_000_000u128)); //37_500_000_000 // 600_000_000_000
    assert_eq!(position.margin, to_decimals(60u64));

    // clearing house token balance should be 60
    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, to_decimals(60));
}
