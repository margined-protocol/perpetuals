use cosmwasm_std::{Addr, Coin, StdError, Uint128};
use margined_perp::margined_engine::{PositionTpSlResponse, Side};
use margined_utils::{
    cw_multi_test::Executor,
    testing::{test_tube::TestTubeScenario, to_decimals, SimpleScenario},
};
use osmosis_test_tube::{Module, OraichainTestApp, Wasm};
use test_tube::{
    cosmrs::proto::cosmwasm::wasm::v1::MsgExecuteContractResponse, Account, Runner, SigningAccount,
};

#[test]
fn test_takeprofit() {
    let TestTubeScenario {
        mut router,
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
    println!("[LOG] [0] spot price: {:?}", price);

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

    println!("tp_sl_status: {:?}", tp_sl_status);
    // assert_eq!(tp_sl_status.is_tpsl, false);

    // let alice_balance_after_open = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    // assert_eq!(
    //     alice_balance_after_open,
    //     Uint128::from(4_994_000_000_000u128)
    // );

    // // take_profit and stop_loss is not set
    // let position = engine
    //     .position(&router.wrap(), vamm.addr().to_string(), 1)
    //     .unwrap();
    // assert_eq!(position.take_profit, to_decimals(15));
    // assert_eq!(position.stop_loss, Some(to_decimals(10)));

    // let mut price = vamm.spot_price(&router.wrap()).unwrap();
    // assert_eq!(price, Uint128::from(11_235_999_999u128));
    // println!("[LOG] [1] spot price: {:?}", price);

    // // Price increase to 15,875
    // let msg = engine
    //     .open_position(
    //         vamm.addr().to_string(),
    //         Side::Buy,
    //         to_decimals(20u64),
    //         to_decimals(10u64),
    //         to_decimals(20u64),
    //         Some(to_decimals(10u64)),
    //         to_decimals(0u64),
    //         vec![],
    //     )
    //     .unwrap();
    // router.execute(bob.clone(), msg).unwrap();

    // price = vamm.spot_price(&router.wrap()).unwrap();
    // assert_eq!(price, Uint128::from(15_875_999_999u128));
    // println!("[LOG] [2] spot price: {:?}", price);

    // tp_sl_status = engine
    //     .get_tp_sl_status(&router.wrap(), vamm.addr().to_string(), Side::Buy, true, 10)
    //     .unwrap();
    // println!("tp_sl_status: {:?}", tp_sl_status);
    // assert_eq!(tp_sl_status.is_tpsl, true);

    // // take profit trigger
    // let msg = engine
    //     .trigger_tp_sl(vamm.addr().to_string(), Side::Buy, true, 10)
    //     .unwrap();
    // let ret = router.execute(alice.clone(), msg).unwrap();
    // println!("take profit tx: {:?}", ret);

    // alice_balance = usdc.balance(&router.wrap(), alice.clone()).unwrap();

    // let err = engine
    //     .position(&router.wrap(), vamm.addr().to_string(), 1)
    //     .unwrap_err();
    // assert_eq!(
    //     StdError::GenericErr {
    //         msg: "Querier contract error: margined_perp::margined_engine::Position not found"
    //             .to_string()
    //     },
    //     err
    // );

    // assert_eq!(ret.events[1].attributes[1].value, "trigger_take_profit");
    // assert_eq!(ret.events[5].attributes[8].key, "withdraw_amount");
    // assert_eq!(
    //     alice_balance,
    //     alice_balance_after_open
    //         .checked_add(Uint128::from_str(&ret.events[5].attributes[8].value).unwrap())
    //         .unwrap()
    // );
}
