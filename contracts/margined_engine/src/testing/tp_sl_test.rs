use std::{
    ops::{Add, Sub},
    str::FromStr,
};

use cosmwasm_std::{StdError, Uint128};

use margined_perp::{margined_engine::Side, margined_vamm::Direction};
use margined_utils::{
    cw_multi_test::Executor,
    testing::{to_decimals, SimpleScenario},
};

use crate::{state::TmpReserveInfo, testing::new_simple_scenario, utils::simulate_spot_price};

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
fn test_check_tp_sl_price() {
    /*
    // buy cases
    [2, 1, 0, "0", "0", true], // spot > profit
    [1, 1, 0, "0", "0", true], // spot = profit
    [1, 2, 0, "3", "0", true], // profit - spot < tpSpread
    [1, 2, 0, "1", "0", true], // profit - spot < tpSpread
    [1, 2, 2, "0", "0", true], // loss > spot
    [1, 2, 1, "0", "0", true], // loss = spot
    [2, 3, 1, "0", "2", true], // loss > 0 && spot - loss < slSpread
    [2, 3, 1, "0", "1", true], // loss > 0 && spot - loss = slSpread
    [10, 20, 0, "5", "1", false], // failed every case with stop loss = 0
    [10, 20, 1, "5", "5", false], // failed every case with stop loss > 0 but spot - loss > slSpread
    [20000000, 20000000, 10000000, "5000", "5000", true],
     */

    /*
    // sell cases
    [1, 2, 0, "0", "0", true], // profit > spot
    [1, 1, 0, "0", "0", true], // spot = profit
    [2, 1, 0, "3", "0", true], // spot - profit < tpSpread
    [2, 1, 0, "1", "0", true], // spot - profit = tpSpread
    [2, 2, 1, "0", "0", true], // spot > loss
    [1, 2, 1, "0", "0", true], // loss = spot
    [1, 3, 2, "0", "2", true], // loss > 0 && loss - spot < slSpread
    [1, 3, 2, "0", "1", true], // loss > 0 && loss - spot = slSpread
    [20, 10, 30, "5", "1", false], // failed every case with stop loss > 0
     */

    // TODO: write test cases for check_tp_sl_price. Should be clean
}

#[test]
fn test_simulate_spot_price() {
    let reserve_amount = Uint128::from(100u128);
    let decimals = Uint128::from(1u128);
    let base_asset_amount = Uint128::from(10u128);
    // normal case add to vamm, should reverse to remove from vamm, should increase base and reduce quote
    let mut tmp_reserve_info = TmpReserveInfo {
        base_asset_reserve: reserve_amount.clone(),
        quote_asset_reserve: reserve_amount.clone(),
    };
    simulate_spot_price(
        &mut tmp_reserve_info,
        decimals,
        base_asset_amount,
        Direction::AddToAmm,
    )
    .unwrap();

    // direction AddToVamm, since we are closing => reverse it to remove from vamm. base +=, quote -=
    assert_eq!(
        tmp_reserve_info.base_asset_reserve,
        reserve_amount.add(base_asset_amount)
    );
    // 100 * 100 / (100 + 10 base asset amount) = 90.9 => quote amount after = 100 - 90.9 ~ 91
    assert_eq!(tmp_reserve_info.quote_asset_reserve, Uint128::from(91u128));

    // reset tmp reserve info
    tmp_reserve_info = TmpReserveInfo {
        base_asset_reserve: reserve_amount.clone(),
        quote_asset_reserve: reserve_amount.clone(),
    };
    simulate_spot_price(
        &mut tmp_reserve_info,
        decimals,
        base_asset_amount,
        Direction::RemoveFromAmm,
    )
    .unwrap();

    // 100 * 100 / (100 - 10) = 111.1 => difference is 11.1 rounded to 12 => new quote reserve is 100 + 12 = 112
    assert_eq!(
        tmp_reserve_info.quote_asset_reserve,
        reserve_amount.add(Uint128::from(12u128))
    );
    assert_eq!(
        tmp_reserve_info.base_asset_reserve,
        reserve_amount.sub(base_asset_amount)
    );
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
        .update_tp_sl(vamm.addr().to_string(), 1, None, None)
        .unwrap();

    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Both take profit and stop loss are not set".to_string()
    );

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
            to_decimals(15u64),
            Some(to_decimals(10u64)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let mut tp_sl_status = engine
        .get_tp_sl_status(&router.wrap(), vamm.addr().to_string(), Side::Buy, true, 10)
        .unwrap();

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
            to_decimals(20u64),
            Some(to_decimals(10u64)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(15_875_999_999u128));
    println!("[LOG] [2] spot price: {:?}", price);

    tp_sl_status = engine
        .get_tp_sl_status(&router.wrap(), vamm.addr().to_string(), Side::Buy, true, 10)
        .unwrap();
    println!("tp_sl_status: {:?}", tp_sl_status);
    assert_eq!(tp_sl_status.is_tpsl, true);

    // take profit trigger
    let msg = engine
        .trigger_tp_sl(vamm.addr().to_string(), Side::Buy, true, 10)
        .unwrap();
    let ret = router.execute(alice.clone(), msg).unwrap();
    println!("take profit tx: {:?}", ret);

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

    let mut tp_sl_status = engine
        .get_tp_sl_status(
            &router.wrap(),
            vamm.addr().to_string(),
            Side::Buy,
            false,
            10,
        )
        .unwrap();
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

    tp_sl_status = engine
        .get_tp_sl_status(
            &router.wrap(),
            vamm.addr().to_string(),
            Side::Buy,
            false,
            10,
        )
        .unwrap();
    println!("tp_sl_status: {:?}", tp_sl_status);
    assert_eq!(tp_sl_status.is_tpsl, true);

    // stop loss trigger
    let msg = engine
        .trigger_tp_sl(vamm.addr().to_string(), Side::Buy, false, 10)
        .unwrap();
    let ret = router.execute(alice.clone(), msg).unwrap();
    println!("stop loss tx: {:?}", ret);

    price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(8_056_874_407u128));
    println!("[LOG] [2] spot price: {:?}", price);

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

#[test]
fn test_multi_takeprofit_long() {
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

    let mut bob_balance = usdc.balance(&router.wrap(), bob.clone()).unwrap();
    assert_eq!(bob_balance, Uint128::from(5_000_000_000_000u128));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(6u64),
            to_decimals(10u64),
            to_decimals(15u64),
            Some(to_decimals(10u64)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let mut tp_sl_status = engine
        .get_tp_sl_status(&router.wrap(), vamm.addr().to_string(), Side::Buy, true, 10)
        .unwrap();

    println!("tp_sl_status: {:?}", tp_sl_status);
    assert_eq!(tp_sl_status.is_tpsl, false);

    let alice_balance_after_open = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(
        alice_balance_after_open,
        Uint128::from(4_994_000_000_000u128)
    );

    println!("alice balance after: {:?}", alice_balance_after_open);

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
            to_decimals(20u64),
            Some(to_decimals(10u64)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(15_875_999_999u128));
    println!("[LOG] [2] spot price: {:?}", price);

    // Price increase to 15,875
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(1u64),
            to_decimals(2u64),
            Uint128::from(15_926_400_000u128),
            Some(Uint128::from(10_000_000_000u128)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(15_926_439_999u128));
    println!("[LOG] [3] spot price: {:?}", price);

    let bob_balance_after_open = usdc.balance(&router.wrap(), bob.clone()).unwrap();
    assert_eq!(bob_balance_after_open, Uint128::from(4_979_000_000_000u128));

    tp_sl_status = engine
        .get_tp_sl_status(&router.wrap(), vamm.addr().to_string(), Side::Buy, true, 10)
        .unwrap();
    println!("tp_sl_status: {:?}", tp_sl_status);
    assert_eq!(tp_sl_status.is_tpsl, true);

    // take profit trigger
    let msg = engine
        .trigger_tp_sl(vamm.addr().to_string(), Side::Buy, true, 10)
        .unwrap();
    let ret = router.execute(alice.clone(), msg).unwrap();
    println!("take profit tx: {:?}", ret);

    price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(13_832_543_768u128));
    println!("[LOG] [3] spot price: {:?}", price);

    alice_balance = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    println!("alice balance after: {:?}", alice_balance);

    bob_balance = usdc.balance(&router.wrap(), bob.clone()).unwrap();
    println!("bob balance after: {:?}", bob_balance);

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

    let err = engine
        .position(&router.wrap(), vamm.addr().to_string(), 3)
        .unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "Querier contract error: margined_perp::margined_engine::Position not found"
                .to_string()
        },
        err
    );

    assert_eq!(ret.events[1].attributes[1].value, "trigger_take_profit");

    // take profit for position 1 and position 3
    assert_eq!(ret.events[3].attributes[7].value, "3");
    assert_eq!(ret.events[9].attributes[7].value, "1");

    assert_eq!(ret.events[5].attributes[8].key, "withdraw_amount");
    assert_eq!(
        alice_balance,
        alice_balance_after_open
            .checked_add(Uint128::from_str(&ret.events[11].attributes[8].value).unwrap())
            .unwrap()
    );
    assert_eq!(
        bob_balance,
        bob_balance_after_open
            .checked_add(Uint128::from_str(&ret.events[5].attributes[8].value).unwrap())
            .unwrap()
    );
}

#[test]
fn test_multi_stoploss_long() {
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

    let mut bob_balance = usdc.balance(&router.wrap(), bob.clone()).unwrap();
    assert_eq!(bob_balance, Uint128::from(5_000_000_000_000u128));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(10u64),
            to_decimals(3u64),
            to_decimals(20u64),
            Some(to_decimals(9u64)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(10u64),
            to_decimals(3u64),
            to_decimals(20u64),
            Some(to_decimals(9u64)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let mut tp_sl_status = engine
        .get_tp_sl_status(
            &router.wrap(),
            vamm.addr().to_string(),
            Side::Buy,
            false,
            10,
        )
        .unwrap();
    println!("tp_sl_status: {:?}", tp_sl_status);
    assert_eq!(tp_sl_status.is_tpsl, false);

    let alice_balance_after_open = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(
        alice_balance_after_open,
        Uint128::from(4_990_000_000_000u128)
    );

    // take_profit and stop_loss is not set
    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position.take_profit, to_decimals(20));
    assert_eq!(position.stop_loss, Some(to_decimals(9)));

    let mut price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(11_235_999_999u128));
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

    let bob_balance_after_open = usdc.balance(&router.wrap(), bob.clone()).unwrap();
    assert_eq!(bob_balance_after_open, Uint128::from(4_976_000_000_000u128));

    price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(8_463_999_999u128));
    println!("[LOG] [2] spot price: {:?}", price);

    tp_sl_status = engine
        .get_tp_sl_status(
            &router.wrap(),
            vamm.addr().to_string(),
            Side::Buy,
            false,
            10,
        )
        .unwrap();
    println!("tp_sl_status: {:?}", tp_sl_status);
    assert_eq!(tp_sl_status.is_tpsl, true);

    // stop loss trigger
    let msg = engine
        .trigger_tp_sl(vamm.addr().to_string(), Side::Buy, false, 10)
        .unwrap();
    let ret = router.execute(alice.clone(), msg).unwrap();
    println!("stop loss tx: {:?}", ret);

    price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(7_646_837_543u128));
    println!("[LOG] [2] spot price: {:?}", price);

    alice_balance = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    println!("alice balance after: {:?}", alice_balance);

    bob_balance = usdc.balance(&router.wrap(), bob.clone()).unwrap();
    println!("bob balance after: {:?}", bob_balance);

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

    let err = engine
        .position(&router.wrap(), vamm.addr().to_string(), 2)
        .unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "Querier contract error: margined_perp::margined_engine::Position not found"
                .to_string()
        },
        err
    );

    // stop loss for position 1 and position 2
    assert_eq!(ret.events[3].attributes[7].value, "1");
    assert_eq!(ret.events[9].attributes[7].value, "2");

    assert_eq!(ret.events[1].attributes[1].value, "trigger_stop_loss");
    assert_eq!(ret.events[5].attributes[8].key, "withdraw_amount");
    assert_eq!(
        alice_balance,
        alice_balance_after_open
            .checked_add(Uint128::from_str(&ret.events[5].attributes[8].value).unwrap())
            .unwrap()
    );
    assert_eq!(
        bob_balance,
        bob_balance_after_open
            .checked_add(Uint128::from_str(&ret.events[11].attributes[8].value).unwrap())
            .unwrap()
    );
}

#[test]
fn test_multi_takeprofit_short() {
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

    let mut bob_balance = usdc.balance(&router.wrap(), bob.clone()).unwrap();
    assert_eq!(bob_balance, Uint128::from(5_000_000_000_000u128));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(1u64),
            to_decimals(5u64),
            to_decimals(6u64),
            Some(to_decimals(14u64)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let mut tp_sl_status = engine
        .get_tp_sl_status(
            &router.wrap(),
            vamm.addr().to_string(),
            Side::Sell,
            true,
            10,
        )
        .unwrap();

    println!("tp_sl_status: {:?}", tp_sl_status);
    assert_eq!(tp_sl_status.is_tpsl, false);

    let alice_balance_after_open = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(
        alice_balance_after_open,
        Uint128::from(4_999_000_000_000u128)
    );

    println!("alice balance after: {:?}", alice_balance_after_open);

    let mut price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(9_900_249_999u128));
    println!("[LOG] [1] spot price: {:?}", price);

    // Price increase to 15,875
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(27u64),
            to_decimals(10u64),
            to_decimals(1u64),
            Some(to_decimals(14u64)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(5_256_249_999u128));
    println!("[LOG] [2] spot price: {:?}", price);

    // Price decrease to 15,875
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(3u64),
            to_decimals(5u64),
            Uint128::from(5_100_000_000u128),
            Some(to_decimals(14u64)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(5_040_999_999u128));
    println!("[LOG] [3] spot price: {:?}", price);

    let bob_balance_after_open = usdc.balance(&router.wrap(), bob.clone()).unwrap();
    assert_eq!(bob_balance_after_open, Uint128::from(4_970_000_000_000u128));

    tp_sl_status = engine
        .get_tp_sl_status(
            &router.wrap(),
            vamm.addr().to_string(),
            Side::Sell,
            true,
            10,
        )
        .unwrap();
    println!("tp_sl_status: {:?}", tp_sl_status);
    assert_eq!(tp_sl_status.is_tpsl, true);

    // take profit trigger
    let msg = engine
        .trigger_tp_sl(vamm.addr().to_string(), Side::Sell, true, 10)
        .unwrap();
    let ret = router.execute(alice.clone(), msg).unwrap();
    println!("take profit tx: {:?}", ret);

    price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(5_294_759_629u128));
    println!("[LOG] [3] spot price: {:?}", price);

    alice_balance = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    println!("alice balance after: {:?}", alice_balance);

    bob_balance = usdc.balance(&router.wrap(), bob.clone()).unwrap();
    println!("bob balance after: {:?}", bob_balance);

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

    let err = engine
        .position(&router.wrap(), vamm.addr().to_string(), 3)
        .unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "Querier contract error: margined_perp::margined_engine::Position not found"
                .to_string()
        },
        err
    );

    assert_eq!(ret.events[1].attributes[1].value, "trigger_take_profit");

    // take profit for position 1 and position 3
    assert_eq!(ret.events[3].attributes[7].value, "1");
    assert_eq!(ret.events[9].attributes[7].value, "3");

    assert_eq!(ret.events[5].attributes[8].key, "withdraw_amount");
    assert_eq!(
        alice_balance,
        alice_balance_after_open
            .checked_add(Uint128::from_str(&ret.events[5].attributes[8].value).unwrap())
            .unwrap()
    );
    assert_eq!(
        bob_balance,
        bob_balance_after_open
            .checked_add(Uint128::from_str(&ret.events[11].attributes[8].value).unwrap())
            .unwrap()
    );
}

#[test]
fn test_multi_stoploss_short() {
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

    let mut bob_balance = usdc.balance(&router.wrap(), bob.clone()).unwrap();
    assert_eq!(bob_balance, Uint128::from(5_000_000_000_000u128));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(4u64),
            to_decimals(3u64),
            to_decimals(5u64),
            Some(to_decimals(11u64)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(4u64),
            to_decimals(3u64),
            to_decimals(5u64),
            Some(to_decimals(11u64)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let mut tp_sl_status = engine
        .get_tp_sl_status(
            &router.wrap(),
            vamm.addr().to_string(),
            Side::Sell,
            false,
            10,
        )
        .unwrap();
    println!("tp_sl_status: {:?}", tp_sl_status);
    assert_eq!(tp_sl_status.is_tpsl, false);

    let alice_balance_after_open = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(
        alice_balance_after_open,
        Uint128::from(4_996_000_000_000u128)
    );

    let mut price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(9_525_759_999u128));
    println!("[LOG] [1] spot price: {:?}", price);

    // Price decrease to 24,087
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(13u64),
            to_decimals(6u64),
            to_decimals(40u64),
            Some(to_decimals(5u64)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let bob_balance_after_open = usdc.balance(&router.wrap(), bob.clone()).unwrap();
    assert_eq!(bob_balance_after_open, Uint128::from(4_983_000_000_000u128));

    price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(11_109_159_999u128));
    println!("[LOG] [2] spot price: {:?}", price);

    tp_sl_status = engine
        .get_tp_sl_status(
            &router.wrap(),
            vamm.addr().to_string(),
            Side::Sell,
            false,
            10,
        )
        .unwrap();
    println!("tp_sl_status: {:?}", tp_sl_status);
    assert_eq!(tp_sl_status.is_tpsl, true);

    // stop loss trigger
    let msg = engine
        .trigger_tp_sl(vamm.addr().to_string(), Side::Sell, false, 10)
        .unwrap();
    let ret = router.execute(alice.clone(), msg).unwrap();
    println!("stop loss tx: {:?}", ret);

    price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(11_708_202_218u128));
    println!("[LOG] [2] spot price: {:?}", price);

    alice_balance = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    println!("alice balance after: {:?}", alice_balance);

    bob_balance = usdc.balance(&router.wrap(), bob.clone()).unwrap();
    println!("bob balance after: {:?}", bob_balance);

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

    let err = engine
        .position(&router.wrap(), vamm.addr().to_string(), 2)
        .unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "Querier contract error: margined_perp::margined_engine::Position not found"
                .to_string()
        },
        err
    );

    // stop loss for position 1 and position 2
    assert_eq!(ret.events[9].attributes[7].value, "1");
    assert_eq!(ret.events[3].attributes[7].value, "2");

    assert_eq!(ret.events[1].attributes[1].value, "trigger_stop_loss");
    assert_eq!(ret.events[5].attributes[8].key, "withdraw_amount");
    assert_eq!(
        alice_balance,
        alice_balance_after_open
            .checked_add(Uint128::from_str(&ret.events[11].attributes[8].value).unwrap())
            .unwrap()
    );
    assert_eq!(
        bob_balance,
        bob_balance_after_open
            .checked_add(Uint128::from_str(&ret.events[5].attributes[8].value).unwrap())
            .unwrap()
    );
}
