use cosmwasm_std::Uint128;


use margined_perp::margined_engine::Side;
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
    
    // take_profit and stop_loss is not set
    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position.take_profit, to_decimals(10));
    assert_eq!(position.stop_loss, Some(to_decimals(0)));

    let msg = engine
        .update_tp_sl(
            vamm.addr().to_string(),
            1,
            Some(to_decimals(2)),
            Some(to_decimals(1)),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // take_profit = 2 and stop_loss = 1
    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position.take_profit, to_decimals(2));
    assert_eq!(position.stop_loss, Some(to_decimals(1)));

    let msg = engine
        .close_position(vamm.addr().to_string(), 1, to_decimals(0)).unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_trigger_tp_sl() {
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
            Uint128::from(27_000_000_000u128),
            Some(Uint128::from(24_000_000_000u128)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
    
    // take_profit and stop_loss is not set
    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position.take_profit, to_decimals(27));
    assert_eq!(position.stop_loss, Some(to_decimals(24)));
    
    let price = vamm.spot_price(&router.wrap()).unwrap();
    println!("[LOG] [1] spot price: {:?}", price);

    // Price decrease to 24,087
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(6u64),
            to_decimals(8u64),
            Uint128::from(6_000_000_000u128),
            Some(Uint128::from(10_000_000_000u128)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();
    
    let price = vamm.spot_price(&router.wrap()).unwrap();
    println!("[LOG] [2] spot price: {:?}", price);

    // Stop loss trigger
    let msg = engine
        .trigger_tp_sl(
            vamm.addr().to_string(),
            1,
            to_decimals(0u64),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
    
    let price = vamm.spot_price(&router.wrap()).unwrap();
    println!("[LOG] [3] spot price: {:?}", price);

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(21u64),
            to_decimals(10u64),
            Uint128::from(27_000_000_000u128),
            Some(Uint128::from(24_000_000_000u128)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
    
    let price = vamm.spot_price(&router.wrap()).unwrap();
    println!("[LOG] [4] spot price: {:?}", price);

    // Take profit trigger
    let msg = engine
        .trigger_tp_sl(
            vamm.addr().to_string(),
            2,
            to_decimals(0u64),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

}