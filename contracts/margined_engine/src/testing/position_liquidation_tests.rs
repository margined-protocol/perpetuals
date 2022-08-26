use cosmwasm_std::{Empty, Uint128};
use cw20::Cw20ExecuteMsg;
use cw_multi_test::Executor;
use margined_common::integer::Integer;
use margined_perp::margined_engine::{PnlCalcOption, Side};
use margined_utils::scenarios::{to_decimals, SimpleScenario};

#[test]
fn test_alice_take_profit_from_bob_unrealized_undercollateralized_position_bob_liquidated() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        carol,
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
                amount: to_decimals(1980),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // reduce the allowance
    router
        .execute_contract(
            bob.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1980),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // alice close position, pnl = 200 -105.88 ~= 94.12
    // receive pnl + margin = 114.12
    let msg = engine
        .close_position(vamm.addr().to_string(), to_decimals(0u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance = usdc.balance::<_, _, Empty>(&router, alice.clone()).unwrap();
    assert_eq!(alice_balance, Uint128::from(5_094_117_647_059u128));

    // keeper liquidate bob's under collateral position, bob's positionValue is -294.11
    // bob's pnl = 200 - 294.11 ~= -94.12
    // bob loss all his margin (20) and there's 74.12 badDebt
    // which is already prepaid by insurance fund when alice close the position
    let margin_ratio = engine
        .get_margin_ratio(&router, vamm.addr().to_string(), bob.to_string())
        .unwrap();
    assert_eq!(margin_ratio, Integer::new_negative(252_000_000u128));

    // bob close his under collateral position, positionValue is -294.11
    // bob's pnl = 200 - 294.11 ~= -94.12
    // bob loss all his margin (20) with additional 74.12 badDebt
    // which is already prepaid by insurance fund when alice close the position before
    // clearing house doesn't need to ask insurance fund for covering the bad debt
    let msg = engine
        .liquidate(vamm.addr().to_string(), bob.to_string(), to_decimals(0u64))
        .unwrap();
    router.execute(carol.clone(), msg).unwrap();

    let carol_balance = usdc.balance::<_, _, Empty>(&router, carol.clone()).unwrap();
    assert_eq!(carol_balance, Uint128::from(7_352_941_176u128));

    let engine_balance = usdc
        .balance::<_, _, Empty>(&router, engine.addr().clone())
        .unwrap();
    assert_eq!(engine_balance, to_decimals(0u64));
}

#[test]
fn test_alice_has_enough_margin_cant_get_liquidated() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        carol,
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
                amount: to_decimals(1700),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // reduce the allowance
    router
        .execute_contract(
            bob.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1500),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(300u64),
            to_decimals(2u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(500u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // unrealizedPnl: -278.77
    // positionNotional: 600 - 278.77 = 321.23
    // remainMargin: 300 - 278.77 = 21.23
    // liquidationFee: 321.23 * 5% = 16.06
    // margin ratio: = (margin + unrealizedPnl) / positionNotional = 21.23 / 321.23 = 6.608971765%
    let msg = engine
        .liquidate(
            vamm.addr().to_string(),
            alice.to_string(),
            to_decimals(0u64),
        )
        .unwrap();
    let err = router.execute(carol.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Position is overcollateralized".to_string()
    );
}

#[test]
fn test_alice_gets_liquidated_insufficient_margin_for_liquidation_fee() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        carol,
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
                amount: to_decimals(1850),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // reduce the allowance
    router
        .execute_contract(
            bob.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1500),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(150u64),
            to_decimals(4u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(500u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // alice's margin ratio = (margin + unrealizedPnl) / openNotional = (150 + (-278.77)) / 600 = -21.46%
    let msg = engine
        .liquidate(
            vamm.addr().to_string(),
            alice.to_string(),
            to_decimals(0u64),
        )
        .unwrap();
    let response = router.execute(carol.clone(), msg).unwrap();
    assert_eq!(
        response.events[5].attributes[2].value,
        Uint128::from(8_030_973_451u128).to_string()
    ); // liquidation fee
    assert_eq!(
        response.events[5].attributes[3].value,
        Integer::new_negative(278_761_061_950u64).to_string()
    ); // pnl (unsigned)
}

#[test]
fn test_alice_long_position_underwater_oracle_price_activated_doesnt_get_liquidated() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        bob,
        carol,
        engine,
        usdc,
        vamm,
        pricefeed,
        ..
    } = SimpleScenario::new();

    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1850),
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
                amount: to_decimals(1500),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(150u64),
            to_decimals(4u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // sync amm price to oracle = 25.6
    let price = vamm.spot_price(&router).unwrap();
    let timestamp: u64 = 1_000_000_000;
    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(500u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // alice's margin ratio = (margin + unrealizedPnl) / openNotional = (150 + (-278.77)) / 600 = -21.46%

    // however, oracle price is more than 10% higher than spot ((25.6 - 12.1) / 12.1 = 111.570247%)
    //   price = 25.6
    //   position notional = 25.6 * 37.5 = 960
    //   unrealizedPnl = 960 - 600 = 360
    //   margin ratio = (150 + 360) / 960 = 53.125% (won't liquidate)
    let msg = engine
        .liquidate(
            vamm.addr().to_string(),
            alice.to_string(),
            to_decimals(0u64),
        )
        .unwrap();
    let err = router.execute(carol.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Position is overcollateralized"
    );
}

#[test]
fn test_alice_short_position_underwater_oracle_price_activated_doesnt_get_liquidated() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        bob,
        carol,
        engine,
        usdc,
        vamm,
        pricefeed,
        ..
    } = SimpleScenario::new();

    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1880),
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
                amount: to_decimals(1880),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // sync amm price to oracle = 25.6
    let price = vamm.spot_price(&router).unwrap();
    let timestamp: u64 = 1_000_000_000;
    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(10u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // alice:
    //   positionNotional = 100000 / (111.111111 - 25) - 900 = 261.290324
    //   unrealizedPnl = 200 - 261.290324 = -61.290324
    // alice's margin ratio = (margin + unrealizedPnl) / openNotional = (20 + (-61.290324)) / 261.290324 = -15.802469%

    // however, oracle price is more than 10% lower than spot ((6.4 - 8.1) / 8.1 = -20.987654%)
    //   price = 6.4
    //   position notional = 25 * 6.4 = 160
    //   unrealizedPnl = 200 - 160 = 40
    //   margin ratio = (20 + 40) / 160 = 37.5% (won't liquidate)
    let msg = engine
        .liquidate(
            vamm.addr().to_string(),
            alice.to_string(),
            to_decimals(0u64),
        )
        .unwrap();
    let err = router.execute(carol.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Position is overcollateralized"
    );
}

#[test]
fn test_can_open_same_side_position_even_thought_long_is_underwater_as_long_over_maintenance_ratio_after(
) {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(25u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(250u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // position size = 20
    // margin = 25
    // positionNotional = 166.67
    // openNotional = 250
    // unrealizedPnl = 166.67 - 250 = -83.33
    // marginRatio = (25 + -83.33) / 250 = -23%
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(100u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // AMM after 1100 : 90.90909
    // positionNotional = 166.67 + 100 = 266.67
    // position size = 20 + 9.09 = 29.09
    // margin = 25 + 100
    let position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.margin, to_decimals(125u64));
    assert_eq!(position.size, Integer::from(29_090_909_090u128));
    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.position_notional, Uint128::from(266_666_666_658u128));
}

#[test]
fn test_can_open_same_side_position_even_thought_short_is_underwater_as_long_over_maintenance_ratio_after(
) {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(88u64),
            Uint128::from(2_841_000_000u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
    let position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.margin, to_decimals(88u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(250u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // position size = 20
    // margin = 88
    // positionNotional = 166.67
    // openNotional ~= 250
    // unrealizedPnl = 166.67 - 250 = -83.33
    // marginRatio = (88 + -83.33) / 250 = 1.868%
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(150u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // AMM after 850 : 117.64705882
    // positionNotional = 166.67 - 150 = 16.67
    // position size = 20 - 17.64705882 = 2.35
    // realizedPnl = -83.33 * (20 - 2.35) / 20 = -73.538725
    // margin = 88 -73.538725 ~= 14.4
    let position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.size, Integer::from(2_353_760_434u128));
    assert_eq!(position.margin, Uint128::from(14_471_986_128u128));
    let pnl = engine
        .get_unrealized_pnl(
            &router,
            vamm.addr().to_string(),
            alice.to_string(),
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.position_notional, Uint128::from(16_672_666_672u128));
}

#[test]
fn test_cannot_open_position_even_thought_long_is_underwater_if_still_under_after_action() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(25u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(250u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(1u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Position is undercollateralized"
    );

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(1u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Position is undercollateralized"
    );
}

#[test]
fn test_cannot_open_position_even_thought_short_is_underwater_if_still_under_after_action() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(250u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(1u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Position is undercollateralized"
    );
}
