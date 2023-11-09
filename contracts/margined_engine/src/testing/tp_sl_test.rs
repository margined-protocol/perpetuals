use std::{
    ops::{Add, Sub},
    str::FromStr,
};

use cosmwasm_std::{StdError, Uint128};

use margined_perp::{margined_engine::Side, margined_vamm::Direction};
use margined_utils::{
    cw_multi_test::Executor,
    testing::{to_decimals, SimpleScenario}, tools::price_swap::get_output_price_with_reserves,
};

use crate::{
    state::TmpReserveInfo,
    testing::new_simple_scenario,
    utils::{calculate_tp_sl_spread, check_tp_sl_price, update_reserve},
};

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
fn test_calculate_tp_spread_sl_spread() {
    let value = Uint128::from(5u128);
    let (tp_spread, sl_spread) =
        calculate_tp_sl_spread(Uint128::from(2u128), value, value, Uint128::from(2u128)).unwrap();
    assert_eq!(tp_spread, value);
    assert_eq!(sl_spread, value);
}

#[test]
fn test_check_tp_sl_price() {
    // LONG SIDE - TAKE PROFIT
    // spot_price > take_profit
    let mut action = check_tp_sl_price(
        Uint128::from(2_000_000u64),
        Uint128::from(1_000_000u64),
        Uint128::zero(),
        Uint128::zero(),
        Uint128::zero(),
        &Side::Buy,
    )
    .unwrap();
    assert_eq!(action, "trigger_take_profit");

    // take_profit - spot_price <= tp_spread
    action = check_tp_sl_price(
        Uint128::from(2_000_000u64),
        Uint128::from(2_010_000u64),
        Uint128::zero(),
        Uint128::from(10_000u64),
        Uint128::zero(),
        &Side::Buy,
    )
    .unwrap();
    assert_eq!(action, "trigger_take_profit");

    // take_profit - spot_price > tp_spread
    action = check_tp_sl_price(
        Uint128::from(2_000_000u64),
        Uint128::from(2_020_000u64),
        Uint128::zero(),
        Uint128::from(10_000u64),
        Uint128::zero(),
        &Side::Buy,
    )
    .unwrap();
    assert_eq!(action, "");

    // LONG SIDE - STOP LOSS
    // stop_loss > spot_price
    action = check_tp_sl_price(
        Uint128::from(1_000_000u64),
        Uint128::from(3_000_000u64),
        Uint128::from(2_500_000u64),
        Uint128::zero(),
        Uint128::zero(),
        &Side::Buy,
    )
    .unwrap();
    assert_eq!(action, "trigger_stop_loss");

    // spot_price - stop_loss <= tp_spread
    action = check_tp_sl_price(
        Uint128::from(1_000_000u64),
        Uint128::from(3_000_000u64),
        Uint128::from(990_000u64),
        Uint128::zero(),
        Uint128::from(10_000u64),
        &Side::Buy,
    )
    .unwrap();
    assert_eq!(action, "trigger_stop_loss");

    // spot_price - stop_loss > tp_spread
    action = check_tp_sl_price(
        Uint128::from(2_000_000u64),
        Uint128::from(3_000_000u64),
        Uint128::from(1_980_000u64),
        Uint128::zero(),
        Uint128::from(10_000u64),
        &Side::Buy,
    )
    .unwrap();
    assert_eq!(action, "");

    // SHORT SIDE - TAKE PROFIT
    // take_profit > spot_price
    let mut action = check_tp_sl_price(
        Uint128::from(2_000_000u64),
        Uint128::from(3_000_000u64),
        Uint128::zero(),
        Uint128::zero(),
        Uint128::zero(),
        &Side::Sell,
    )
    .unwrap();
    assert_eq!(action, "trigger_take_profit");

    // spot_price - take_profit <= tp_spread
    action = check_tp_sl_price(
        Uint128::from(2_000_000u64),
        Uint128::from(1_990_000u64),
        Uint128::zero(),
        Uint128::from(10_000u64),
        Uint128::zero(),
        &Side::Sell,
    )
    .unwrap();
    assert_eq!(action, "trigger_take_profit");

    // spot_price - take_profit > tp_spread
    action = check_tp_sl_price(
        Uint128::from(2_000_000u64),
        Uint128::from(1_980_000u64),
        Uint128::zero(),
        Uint128::from(10_000u64),
        Uint128::zero(),
        &Side::Sell,
    )
    .unwrap();
    assert_eq!(action, "");

    // SHORT SIDE - STOP LOSS
    // stop_loss > spot_price
    action = check_tp_sl_price(
        Uint128::from(3_000_000u64),
        Uint128::from(1_000_000u64),
        Uint128::from(2_500_000u64),
        Uint128::zero(),
        Uint128::zero(),
        &Side::Sell,
    )
    .unwrap();
    assert_eq!(action, "trigger_stop_loss");

    // stop_loss - spot_price <= tp_spread
    action = check_tp_sl_price(
        Uint128::from(3_000_000u64),
        Uint128::from(1_000_000u64),
        Uint128::from(3_010_000u64),
        Uint128::zero(),
        Uint128::from(10_000u64),
        &Side::Sell,
    )
    .unwrap();
    assert_eq!(action, "trigger_stop_loss");

    // stop_loss - spot_price > tp_spread
    action = check_tp_sl_price(
        Uint128::from(3_000_000u64),
        Uint128::from(1_000_000u64),
        Uint128::from(3_020_000u64),
        Uint128::zero(),
        Uint128::from(10_000u64),
        &Side::Sell,
    )
    .unwrap();
    assert_eq!(action, "");
}

#[test]
fn test_simulate_close_price() {
    let reserve_amount = Uint128::from(1_000_000_000u64);
    let decimals = Uint128::from(1_000_000u128);
    let base_asset_amount = Uint128::from(100_000u128);

    // normal case add to vamm, should reverse to remove from vamm, should increase base and reduce quote
    let mut tmp_reserve_info = TmpReserveInfo {
        base_asset_reserve: reserve_amount.clone(),
        quote_asset_reserve: reserve_amount.clone(),
    };

    let quote_asset_amount = get_output_price_with_reserves(
        decimals,
        &Direction::AddToAmm,
        base_asset_amount,
        tmp_reserve_info.quote_asset_reserve,
        tmp_reserve_info.base_asset_reserve,
    ).unwrap();

    let close_price = quote_asset_amount
        .checked_mul(decimals).unwrap()
        .checked_div(base_asset_amount).unwrap();
    assert_eq!(close_price, Uint128::from(999_900u64));

    let _ = update_reserve(
        &mut tmp_reserve_info,
        quote_asset_amount,
        base_asset_amount,
        &Direction::AddToAmm,
    );
    
    // direction AddToVamm, since we are closing => reverse it to remove from vamm. base +=, quote -=
    assert_eq!(
        tmp_reserve_info.base_asset_reserve,
        reserve_amount.add(base_asset_amount)
    );

    // quote_asset_reserve = (1_000_000_000 * 1_000_000_000) / (1_000_000_000 + 100_000) = 999_900_010 = 1_000_000_000 - 99_990
    assert_eq!(
        tmp_reserve_info.quote_asset_reserve,
        reserve_amount.sub(Uint128::from(99_990u128))
    );

    // reset tmp reserve info
    tmp_reserve_info = TmpReserveInfo {
        base_asset_reserve: reserve_amount.clone(),
        quote_asset_reserve: reserve_amount.clone(),
    };
    let quote_asset_amount = get_output_price_with_reserves(
        decimals,
        &Direction::RemoveFromAmm,
        base_asset_amount,
        tmp_reserve_info.quote_asset_reserve,
        tmp_reserve_info.base_asset_reserve,
    ).unwrap();

    let close_price = quote_asset_amount
        .checked_mul(decimals).unwrap()
        .checked_div(base_asset_amount).unwrap();
    assert_eq!(close_price, Uint128::from(1_000_110u64));

    let _ = update_reserve(
        &mut tmp_reserve_info,
        quote_asset_amount,
        base_asset_amount,
        &Direction::RemoveFromAmm,
    );

    // 1_000_000_000 * 1_000_000_000 / (1_000_000_000 - 100_000) = 1_000_100_011 => new quote reserve is 1_000_000_000 + 100_011 = 1_000_100_011
    assert_eq!(
        tmp_reserve_info.quote_asset_reserve,
        reserve_amount.add(Uint128::from(100_011u128))
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

    // take_profit and stop_loss is set
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
    let mut alice_balance = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(alice_balance, Uint128::from(5_000_000_000_000u128));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(6u64),
            to_decimals(10u64),
            to_decimals(11u64),
            Some(to_decimals(5u64)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let mut tp_sl_status = engine
        .get_tp_sl_status(&router.wrap(), vamm.addr().to_string(), Side::Buy, true, 10)
        .unwrap();
    assert_eq!(tp_sl_status.is_tpsl, false);

    let alice_balance_after_open = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(
        alice_balance_after_open,
        Uint128::from(4_994_000_000_000u128)
    );

    let mut price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(11_235_999_999u128));

    // Price increase to 15,875
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(22u64),
            to_decimals(10u64),
            to_decimals(20u64),
            Some(to_decimals(10u64)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(16_383_999_999u128));

    tp_sl_status = engine
        .get_tp_sl_status(&router.wrap(), vamm.addr().to_string(), Side::Buy, true, 10)
        .unwrap();
    assert_eq!(tp_sl_status.is_tpsl, true);

    // take profit trigger
    let msg = engine
        .trigger_tp_sl(vamm.addr().to_string(), 1, true)
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
    println!("ret: {:?}", ret);
    assert_eq!(ret.events[1].attributes[1].value, "trigger_take_profit");
    assert_eq!(ret.events[5].attributes[10].key, "withdraw_amount");
    assert_eq!(
        alice_balance,
        alice_balance_after_open
            .checked_add(Uint128::from_str(&ret.events[5].attributes[10].value).unwrap())
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
    assert_eq!(tp_sl_status.is_tpsl, false);

    let alice_balance_after_open = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(
        alice_balance_after_open,
        Uint128::from(4_940_000_000_000u128)
    );

    let mut price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(13_923_999_999u128));

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

    tp_sl_status = engine
        .get_tp_sl_status(
            &router.wrap(),
            vamm.addr().to_string(),
            Side::Buy,
            false,
            10,
        )
        .unwrap();
    assert_eq!(tp_sl_status.is_tpsl, true);

    // stop loss trigger
    let msg = engine
        .trigger_tp_sl(vamm.addr().to_string(), 1, false)
        .unwrap();
    let ret = router.execute(alice.clone(), msg).unwrap();

    price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(8_056_874_407u128));

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
    assert_eq!(ret.events[5].attributes[10].key, "withdraw_amount");
    assert_eq!(
        alice_balance,
        alice_balance_after_open
            .checked_add(Uint128::from_str(&ret.events[5].attributes[10].value).unwrap())
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
            to_decimals(11u64),
            Some(to_decimals(5u64)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let mut tp_sl_status = engine
        .get_tp_sl_status(&router.wrap(), vamm.addr().to_string(), Side::Buy, true, 10)
        .unwrap();
    assert_eq!(tp_sl_status.is_tpsl, false);

    let alice_balance_after_open = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(
        alice_balance_after_open,
        Uint128::from(4_994_000_000_000u128)
    );

    let mut price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(11_235_999_999u128));

    // Price increase to 12,543
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(20u64),
            to_decimals(3u64),
            to_decimals(20u64),
            Some(to_decimals(10u64)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(12_543_999_999u128));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(1u64),
            to_decimals(2u64),
            Uint128::from(12_600_000_000u128),
            Some(Uint128::from(10_000_000_000u128)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // Price increase to 17,476
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(40u64),
            to_decimals(5u64),
            to_decimals(20u64),
            Some(to_decimals(10u64)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();
    

    price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(17_476_839_999u128));

    let bob_balance_after_open = usdc.balance(&router.wrap(), bob.clone()).unwrap();
    assert_eq!(bob_balance_after_open, Uint128::from(4_939_000_000_000u128));

    tp_sl_status = engine
        .get_tp_sl_status(&router.wrap(), vamm.addr().to_string(), Side::Buy, true, 10)
        .unwrap();
    assert_eq!(tp_sl_status.is_tpsl, true);

    // take profit trigger
    let msg = engine
        .trigger_multiple_tp_sl(vamm.addr().to_string(), Side::Buy, true, 10)
        .unwrap();
    let ret = router.execute(alice.clone(), msg).unwrap();

    price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(15_069_004_499u128));

    alice_balance = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    bob_balance = usdc.balance(&router.wrap(), bob.clone()).unwrap();

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

    assert_eq!(ret.events[5].attributes[10].key, "withdraw_amount");
    assert_eq!(
        alice_balance,
        alice_balance_after_open
            .checked_add(Uint128::from_str(&ret.events[11].attributes[10].value).unwrap())
            .unwrap()
    );
    assert_eq!(
        bob_balance,
        bob_balance_after_open
            .checked_add(Uint128::from_str(&ret.events[5].attributes[10].value).unwrap())
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
    assert_eq!(tp_sl_status.is_tpsl, false);

    let alice_balance_after_open = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(
        alice_balance_after_open,
        Uint128::from(4_990_000_000_000u128)
    );

    let mut price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(11_235_999_999u128));

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

    tp_sl_status = engine
        .get_tp_sl_status(
            &router.wrap(),
            vamm.addr().to_string(),
            Side::Buy,
            false,
            10,
        )
        .unwrap();
    assert_eq!(tp_sl_status.is_tpsl, true);

    // stop loss trigger
    let msg = engine
        .trigger_multiple_tp_sl(vamm.addr().to_string(), Side::Buy, false, 10)
        .unwrap();
    let ret = router.execute(alice.clone(), msg).unwrap();

    price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(7_646_837_543u128));

    alice_balance = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    bob_balance = usdc.balance(&router.wrap(), bob.clone()).unwrap();

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
    assert_eq!(ret.events[9].attributes[7].value, "2");
    assert_eq!(ret.events[3].attributes[7].value, "1");

    assert_eq!(ret.events[1].attributes[1].value, "trigger_stop_loss");
    assert_eq!(ret.events[5].attributes[10].key, "withdraw_amount");
    assert_eq!(
        alice_balance,
        alice_balance_after_open
            .checked_add(Uint128::from_str(&ret.events[5].attributes[10].value).unwrap())
            .unwrap()
    );
    assert_eq!(
        bob_balance,
        bob_balance_after_open
            .checked_add(Uint128::from_str(&ret.events[11].attributes[10].value).unwrap())
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
            to_decimals(5u64),
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
    assert_eq!(tp_sl_status.is_tpsl, false);

    let alice_balance_after_open = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(
        alice_balance_after_open,
        Uint128::from(4_999_000_000_000u128)
    );
    let mut price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(9_900_249_999u128));

    // Price decrease to 9,603
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
    assert_eq!(price, Uint128::from(9_603_999_999u128));

    // Price increase to 15,875
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(200u64),
            to_decimals(2u64),
            to_decimals(1u64),
            Some(to_decimals(14u64)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(3_363_999_999u128));

    let bob_balance_after_open = usdc.balance(&router.wrap(), bob.clone()).unwrap();
    assert_eq!(bob_balance_after_open, Uint128::from(4_797_000_000_000u128));

    tp_sl_status = engine
        .get_tp_sl_status(
            &router.wrap(),
            vamm.addr().to_string(),
            Side::Sell,
            true,
            10,
        )
        .unwrap();
    assert_eq!(tp_sl_status.is_tpsl, true);

    // take profit trigger
    let msg = engine
        .trigger_multiple_tp_sl(vamm.addr().to_string(), Side::Sell, true, 10)
        .unwrap();
    let ret = router.execute(alice.clone(), msg).unwrap();

    price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(3_445_074_174u128));

    alice_balance = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    bob_balance = usdc.balance(&router.wrap(), bob.clone()).unwrap();

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

    assert_eq!(ret.events[1].attributes[1].value, "trigger_take_profit");

    // take profit for position 1 and position 3
    assert_eq!(ret.events[3].attributes[7].value, "2");
    assert_eq!(ret.events[9].attributes[7].value, "1");

    assert_eq!(ret.events[5].attributes[10].key, "withdraw_amount");
    assert_eq!(
        alice_balance,
        alice_balance_after_open
            .checked_add(Uint128::from_str(&ret.events[11].attributes[10].value).unwrap())
            .unwrap()
    );
    assert_eq!(
        bob_balance,
        bob_balance_after_open
            .checked_add(Uint128::from_str(&ret.events[5].attributes[10].value).unwrap())
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
    assert_eq!(tp_sl_status.is_tpsl, false);

    let alice_balance_after_open = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(
        alice_balance_after_open,
        Uint128::from(4_996_000_000_000u128)
    );

    let mut price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(9_525_759_999u128));

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

    tp_sl_status = engine
        .get_tp_sl_status(
            &router.wrap(),
            vamm.addr().to_string(),
            Side::Sell,
            false,
            10,
        )
        .unwrap();
    assert_eq!(tp_sl_status.is_tpsl, true);

    // stop loss trigger
    let msg = engine
        .trigger_multiple_tp_sl(vamm.addr().to_string(), Side::Sell, false, 10)
        .unwrap();
    let ret = router.execute(alice.clone(), msg).unwrap();

    price = vamm.spot_price(&router.wrap()).unwrap();
    assert_eq!(price, Uint128::from(11_708_202_218u128));

    alice_balance = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    bob_balance = usdc.balance(&router.wrap(), bob.clone()).unwrap();

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
    assert_eq!(ret.events[5].attributes[10].key, "withdraw_amount");
    assert_eq!(
        alice_balance,
        alice_balance_after_open
            .checked_add(Uint128::from_str(&ret.events[5].attributes[10].value).unwrap())
            .unwrap()
    );
    assert_eq!(
        bob_balance,
        bob_balance_after_open
            .checked_add(Uint128::from_str(&ret.events[11].attributes[10].value).unwrap())
            .unwrap()
    );
}
