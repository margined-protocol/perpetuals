use std::str::FromStr;

use cosmwasm_std::{StdError, Uint128};

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
            to_decimals(18),
            Some(to_decimals(9)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // take_profit and stop_loss is not set
    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position.take_profit, to_decimals(18));
    assert_eq!(position.stop_loss, Some(to_decimals(9)));

    let msg = engine
        .update_tp_sl(
            vamm.addr().to_string(),
            1,
            Some(to_decimals(26)),
            Some(to_decimals(14)),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // take_profit = 2 and stop_loss = 1
    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position.take_profit, to_decimals(26));
    assert_eq!(position.stop_loss, Some(to_decimals(14)));

    let msg = engine
        .close_position(vamm.addr().to_string(), 1, to_decimals(0))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_takeprofit() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        usdc,
        engine,
        vamm,
        ..
    } = new_simple_scenario();
    let price = vamm.spot_price(&router.wrap()).unwrap();
    println!("[LOG] [0] spot price: {:?}", price);

    let mut alice_balance = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(alice_balance, Uint128::from(5_000_000_000_000u128));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(6u64),
            to_decimals(10u64),
            Uint128::from(15_000_000_000u128),
            Some(Uint128::from(10_000_000_000u128)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
    
    let mut tp_sl_status = engine.get_tp_sl_status(&router.wrap(), vamm.addr().to_string(), 1).unwrap();
    println!("tp_sl_status: {:?}", tp_sl_status);
    assert_eq!(tp_sl_status.is_tpsl, false);

    let alice_balance_after_open = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(
        alice_balance_after_open,
        Uint128::from(4_994_000_000_000u128)
    );

    // take_profit and stop_loss is not set
    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position.take_profit, to_decimals(15));
    assert_eq!(position.stop_loss, Some(to_decimals(10)));

    let mut price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(11_235_999_999u128));
    println!("[LOG] [1] spot price: {:?}", price);

    // Price increase to 15,875
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(20u64),
            to_decimals(10u64),
            Uint128::from(20_000_000_000u128),
            Some(Uint128::from(10_000_000_000u128)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(15_875_999_999u128));
    println!("[LOG] [2] spot price: {:?}", price);

    tp_sl_status = engine.get_tp_sl_status(&router.wrap(), vamm.addr().to_string(), 1).unwrap();
    println!("tp_sl_status: {:?}", tp_sl_status);
    assert_eq!(tp_sl_status.is_tpsl, true);

    // take profit trigger
    let msg = engine
        .trigger_tp_sl(vamm.addr().to_string(), 1, to_decimals(0u64))
        .unwrap();
    let ret = router.execute(alice.clone(), msg).unwrap();

    alice_balance = usdc.balance(&router.wrap(), alice.clone()).unwrap();

    let err = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "Querier contract error: margined_perp::margined_engine::Position not found"
                .to_string()
        },
        err
    );

    assert_eq!(ret.events[1].attributes[1].value, "trigger_take_profit");
    assert_eq!(ret.events[5].attributes[8].key, "withdraw_amount");
    assert_eq!(
        alice_balance,
        alice_balance_after_open
            .checked_add(Uint128::from_str(&ret.events[5].attributes[8].value).unwrap())
            .unwrap()
    );
}

#[test]
fn test_stoploss() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        usdc,
        engine,
        vamm,
        ..
    } = new_simple_scenario();
    let price = vamm.spot_price(&router.wrap()).unwrap();
    println!("[LOG] [0] spot price: {:?}", price);

    let mut alice_balance = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(alice_balance, Uint128::from(5_000_000_000_000u128));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(3u64),
            to_decimals(20u64),
            Some(to_decimals(11u64)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
    
    let mut tp_sl_status = engine.get_tp_sl_status(&router.wrap(), vamm.addr().to_string(), 1).unwrap();
    println!("tp_sl_status: {:?}", tp_sl_status);
    assert_eq!(tp_sl_status.is_tpsl, false);

    let alice_balance_after_open = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(
        alice_balance_after_open,
        Uint128::from(4_940_000_000_000u128)
    );

    // take_profit and stop_loss is not set
    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position.take_profit, to_decimals(20));
    assert_eq!(position.stop_loss, Some(to_decimals(11)));

    let mut price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(13_923_999_999u128));
    println!("[LOG] [1] spot price: {:?}", price);

    // Price decrease to 24,087
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(14u64),
            to_decimals(10u64),
            to_decimals(5u64),
            Some(to_decimals(40u64)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(10_815_999_999u128));
    println!("[LOG] [2] spot price: {:?}", price);

    tp_sl_status = engine.get_tp_sl_status(&router.wrap(), vamm.addr().to_string(), 1).unwrap();
    println!("tp_sl_status: {:?}", tp_sl_status);
    assert_eq!(tp_sl_status.is_tpsl, true);

    // stop loss trigger
    let msg = engine
        .trigger_tp_sl(vamm.addr().to_string(), 1, to_decimals(0u64))
        .unwrap();
    let ret = router.execute(alice.clone(), msg).unwrap();

    alice_balance = usdc.balance(&router.wrap(), alice.clone()).unwrap();

    let err = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "Querier contract error: margined_perp::margined_engine::Position not found"
                .to_string()
        },
        err
    );

    assert_eq!(ret.events[1].attributes[1].value, "trigger_stop_loss");
    assert_eq!(ret.events[5].attributes[8].key, "withdraw_amount");
    assert_eq!(
        alice_balance,
        alice_balance_after_open
            .checked_add(Uint128::from_str(&ret.events[5].attributes[8].value).unwrap())
            .unwrap()
    );
}
