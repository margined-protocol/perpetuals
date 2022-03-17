use cosmwasm_std::Uint128;
use cw20::Cw20ExecuteMsg;
use cw_multi_test::Executor;
use margined_perp::margined_engine::{PositionResponse, Side};
use margined_utils::scenarios::SimpleScenario;

#[test]
fn test_settle_funding_delay_before_buffer_period_ends() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        owner,
        vamm,
        pricefeed,
        ..
    } = SimpleScenario::new();

    let prices = vec![
        Uint128::from(500_000_000u128),
        Uint128::from(600_000_000u128),
        Uint128::from(700_000_000u128),
    ];

    let timestamps: Vec<u64> = vec![1_000_000_000, 1_000_000_001, 1_000_000_002];

    let msg = pricefeed
        .append_multiple_price(
            "ETH".to_string(),
            prices,
            timestamps,
        ).unwrap();
    router.execute(owner.clone(), msg).unwrap();


    let state = vamm
        .state(&router)
        .unwrap();
    let expected_funding_time = router.block_info().time.plus_seconds(3_600u64);
    assert_eq!(state.next_funding_time, expected_funding_time.seconds());

    // moves block forward 1 and 15 secs timestamp
    router.update_block(|block| {
        block.time = block.time.plus_seconds(3_600u64);
        block.height += 1;
    });

    let msg = vamm
        .settle_funding().unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let state = vamm
        .state(&router)
        .unwrap();
    let expected_funding_time = expected_funding_time.plus_seconds(3_600u64);
    assert_eq!(state.next_funding_time, expected_funding_time.seconds());

    assert_eq!(1, 2);
    // let msg = engine
    //     .open_position(
    //         vamm.addr().to_string(),
    //         Side::SELL,
    //         to_decimals(20u64),
    //         to_decimals(10u64),
    //     )
    //     .unwrap();
    // router.execute(bob.clone(), msg).unwrap();

    // // alice close position, pnl = 200 -105.88 ~= 94.12
    // // receive pnl + margin = 114.12
    // let msg = engine.close_position(vamm.addr().to_string()).unwrap();
    // router.execute(alice.clone(), msg).unwrap();

    // let alice_balance = usdc.balance(&router, alice.clone()).unwrap();
    // assert_eq!(alice_balance, Uint128::from(5_094_117_647_059u128));

    // // keeper liquidate bob's under collateral position, bob's positionValue is -294.11
    // // bob's pnl = 200 - 294.11 ~= -94.12
    // // bob loss all his margin (20) and there's 74.12 badDebt
    // // which is already prepaid by insurance fund when alice close the position
    // let margin_ratio = engine
    //     .get_margin_ratio(&router, vamm.addr().to_string(), bob.to_string())
    //     .unwrap();
    // assert_eq!(margin_ratio.ratio, Uint128::from(252_000_000u128));
    // assert_eq!(margin_ratio.polarity, false);

    // // bob close his under collateral position, positionValue is -294.11
    // // bob's pnl = 200 - 294.11 ~= -94.12
    // // bob loss all his margin (20) with additional 74.12 badDebt
    // // which is already prepaid by insurance fund when alice close the position before
    // // clearing house doesn't need to ask insurance fund for covering the bad debt
    // let msg = engine
    //     .liquidate(vamm.addr().to_string(), bob.to_string())
    //     .unwrap();
    // router.execute(carol.clone(), msg).unwrap();

    // let carol_balance = usdc.balance(&router, carol.clone()).unwrap();
    // assert_eq!(carol_balance, Uint128::from(5_007_352_941_176u128));

    // let engine_balance = usdc.balance(&router, engine.addr().clone()).unwrap();
    // assert_eq!(engine_balance, to_decimals(0u64));
}
