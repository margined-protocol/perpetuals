use std::str::FromStr;

use cosmwasm_std::{Attribute, Uint128};
use margined_perp::margined_engine::{Position, PositionTpSlResponse, Side};
use margined_utils::testing::{test_tube::TestTubeScenario, to_decimals};
use osmosis_test_tube::{Module, Wasm};
use test_tube::{cosmrs::proto::cosmwasm::wasm::v1::MsgExecuteContractResponse, Account, Runner};

#[test]
fn test_takeprofit() {
    let TestTubeScenario {
        router,
        accounts,
        usdc,
        engine,
        vamm,
        ..
    } = TestTubeScenario::default();
    let (alice, bob) = (&accounts[1], &accounts[2]);
    let wasm = Wasm::new(&router);
    let price: Uint128 = wasm
        .query(
            vamm.0.as_str(),
            &margined_perp::margined_vamm::QueryMsg::SpotPrice {},
        )
        .unwrap();

    let alice_balance = wasm
        .query::<_, cw20::BalanceResponse>(
            usdc.0.as_str(),
            &cw20_base::msg::QueryMsg::Balance {
                address: alice.address(),
            },
        )
        .unwrap()
        .balance;

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

    let res = router
        .execute_cosmos_msgs::<MsgExecuteContractResponse>(&[msg], alice)
        .unwrap();

    println!("res : {:?}", res.gas_info);

    let tp_sl_status: PositionTpSlResponse = wasm
        .query(
            engine.0.as_str(),
            &margined_perp::margined_engine::QueryMsg::PositionIsTpSl {
                vamm: vamm.0.to_string(),
                side: Side::Buy,
                take_profit: true,
                limit: 10,
            },
        )
        .unwrap();

    assert_eq!(tp_sl_status.is_tpsl, false);

    let alice_balance_after_open = wasm
        .query::<_, cw20::BalanceResponse>(
            usdc.0.as_str(),
            &cw20_base::msg::QueryMsg::Balance {
                address: alice.address(),
            },
        )
        .unwrap()
        .balance;

    assert_eq!(
        alice_balance_after_open,
        Uint128::from(4_994_000_000_000u128)
    );

    let price: Uint128 = wasm
        .query(
            vamm.0.as_str(),
            &margined_perp::margined_vamm::QueryMsg::SpotPrice {},
        )
        .unwrap();
    assert_eq!(price, Uint128::from(11_235_999_999u128));

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
    router
        .execute_cosmos_msgs::<MsgExecuteContractResponse>(&[msg], bob)
        .unwrap();

    let price: Uint128 = wasm
        .query(
            vamm.0.as_str(),
            &margined_perp::margined_vamm::QueryMsg::SpotPrice {},
        )
        .unwrap();
    assert_eq!(price, Uint128::from(15_875_999_999u128));

    let tp_sl_status: PositionTpSlResponse = wasm
        .query(
            engine.0.as_str(),
            &margined_perp::margined_engine::QueryMsg::PositionIsTpSl {
                vamm: vamm.0.to_string(),
                side: Side::Buy,
                take_profit: true,
                limit: 10,
            },
        )
        .unwrap();
    assert_eq!(tp_sl_status.is_tpsl, true);

    // take profit trigger
    let msg = engine
        .trigger_tp_sl(vamm.addr().to_string(), Side::Buy, true, 10)
        .unwrap();
    let ret = router
        .execute_cosmos_msgs::<MsgExecuteContractResponse>(&[msg], alice)
        .unwrap();
    println!("take profit tx: {:?}", ret.gas_info);

    let alice_balance = wasm
        .query::<_, cw20::BalanceResponse>(
            usdc.0.as_str(),
            &cw20_base::msg::QueryMsg::Balance {
                address: alice.address(),
            },
        )
        .unwrap()
        .balance;

    let err = wasm
        .query::<_, Position>(
            engine.0.as_str(),
            &margined_perp::margined_engine::QueryMsg::Position {
                vamm: vamm.0.to_string(),
                position_id: 1,
            },
        )
        .unwrap_err();

    assert_eq!(
        "query error: margined_perp::margined_engine::Position not found: query wasm contract failed",
        err.to_string()
    );

    let attrs: Vec<Attribute> = ret.events.into_iter().flat_map(|e| e.attributes).collect();
    let withdraw_amount_attr = attrs.iter().find(|a| a.key == "withdraw_amount").unwrap();

    assert_eq!(
        alice_balance,
        alice_balance_after_open
            .checked_add(Uint128::from_str(&withdraw_amount_attr.value).unwrap())
            .unwrap()
    );
}

#[test]
fn test_stoploss() {
    let TestTubeScenario {
        router,
        accounts,
        usdc,
        engine,
        vamm,
        ..
    } = TestTubeScenario::default();
    let (alice, bob) = (&accounts[1], &accounts[2]);
    let wasm = Wasm::new(&router);
    let price: Uint128 = wasm
        .query(
            vamm.0.as_str(),
            &margined_perp::margined_vamm::QueryMsg::SpotPrice {},
        )
        .unwrap();

    let alice_balance = wasm
        .query::<_, cw20::BalanceResponse>(
            usdc.0.as_str(),
            &cw20_base::msg::QueryMsg::Balance {
                address: alice.address(),
            },
        )
        .unwrap()
        .balance;

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
    let res = router
        .execute_cosmos_msgs::<MsgExecuteContractResponse>(&[msg], alice)
        .unwrap();

    println!("res : {:?}", res.gas_info);

    let tp_sl_status: PositionTpSlResponse = wasm
        .query(
            engine.0.as_str(),
            &margined_perp::margined_engine::QueryMsg::PositionIsTpSl {
                vamm: vamm.0.to_string(),
                side: Side::Buy,
                take_profit: false,
                limit: 10,
            },
        )
        .unwrap();
    assert_eq!(tp_sl_status.is_tpsl, false);

    let alice_balance_after_open = wasm
        .query::<_, cw20::BalanceResponse>(
            usdc.0.as_str(),
            &cw20_base::msg::QueryMsg::Balance {
                address: alice.address(),
            },
        )
        .unwrap()
        .balance;

    let price: Uint128 = wasm
        .query(
            vamm.0.as_str(),
            &margined_perp::margined_vamm::QueryMsg::SpotPrice {},
        )
        .unwrap();
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
    let res = router
        .execute_cosmos_msgs::<MsgExecuteContractResponse>(&[msg], bob)
        .unwrap();
    println!("res : {:?}", res.gas_info);

    let price: Uint128 = wasm
        .query(
            vamm.0.as_str(),
            &margined_perp::margined_vamm::QueryMsg::SpotPrice {},
        )
        .unwrap();
    assert_eq!(price, Uint128::from(10_815_999_999u128));

    let tp_sl_status: PositionTpSlResponse = wasm
        .query(
            engine.0.as_str(),
            &margined_perp::margined_engine::QueryMsg::PositionIsTpSl {
                vamm: vamm.0.to_string(),
                side: Side::Buy,
                take_profit: false,
                limit: 10,
            },
        )
        .unwrap();
    assert_eq!(tp_sl_status.is_tpsl, true);

    // stop loss trigger
    let msg = engine
        .trigger_tp_sl(vamm.addr().to_string(), Side::Buy, false, 10)
        .unwrap();
    let ret = router
        .execute_cosmos_msgs::<MsgExecuteContractResponse>(&[msg], alice)
        .unwrap();
    println!("stop loss tx: {:?}", ret.gas_info);

    let price: Uint128 = wasm
        .query(
            vamm.0.as_str(),
            &margined_perp::margined_vamm::QueryMsg::SpotPrice {},
        )
        .unwrap();
    assert_eq!(price, Uint128::from(8_056_874_407u128));

    let alice_balance = wasm
        .query::<_, cw20::BalanceResponse>(
            usdc.0.as_str(),
            &cw20_base::msg::QueryMsg::Balance {
                address: alice.address(),
            },
        )
        .unwrap()
        .balance;

        let err = wasm
        .query::<_, Position>(
            engine.0.as_str(),
            &margined_perp::margined_engine::QueryMsg::Position {
                vamm: vamm.0.to_string(),
                position_id: 1,
            },
        )
        .unwrap_err();
    assert_eq!(
        "query error: margined_perp::margined_engine::Position not found: query wasm contract failed",
        err.to_string()
    );

    let attrs: Vec<Attribute> = ret.events.into_iter().flat_map(|e| e.attributes).collect();
    let withdraw_amount_attr = attrs.iter().find(|a| a.key == "withdraw_amount").unwrap();

    assert_eq!(
        alice_balance,
        alice_balance_after_open
            .checked_add(Uint128::from_str(&withdraw_amount_attr.value).unwrap())
            .unwrap()
    );
}