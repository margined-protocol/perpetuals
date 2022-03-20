// use crate::testing::setup::{self, to_decimals};
use cosmwasm_std::Uint128;
use cw20::Cw20ExecuteMsg;
use cw_multi_test::Executor;
use margined_perp::margined_engine::Side;
use margined_utils::scenarios::SimpleScenario;

pub const DECIMAL_MULTIPLIER: Uint128 = Uint128::new(1_000_000_000);

// takes in a Uint128 and multiplies by the decimals just to make tests more legible
pub fn to_decimals(input: u64) -> Uint128 {
    Uint128::from(input) * DECIMAL_MULTIPLIER
}

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
            Side::SELL,
            to_decimals(20u64),
            to_decimals(10u64),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::SELL,
            to_decimals(20u64),
            to_decimals(10u64),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // alice close position, pnl = 200 -105.88 ~= 94.12
    // receive pnl + margin = 114.12
    let msg = engine.close_position(vamm.addr().to_string()).unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance = usdc.balance(&router, alice.clone()).unwrap();
    assert_eq!(alice_balance, Uint128::from(5_094_117_647_059u128));

    // keeper liquidate bob's under collateral position, bob's positionValue is -294.11
    // bob's pnl = 200 - 294.11 ~= -94.12
    // bob loss all his margin (20) and there's 74.12 badDebt
    // which is already prepaid by insurance fund when alice close the position
    let margin_ratio = engine
        .get_margin_ratio(&router, vamm.addr().to_string(), bob.to_string())
        .unwrap();
    assert_eq!(margin_ratio.ratio, Uint128::from(252_000_000u128));
    assert_eq!(margin_ratio.polarity, false);

    // bob close his under collateral position, positionValue is -294.11
    // bob's pnl = 200 - 294.11 ~= -94.12
    // bob loss all his margin (20) with additional 74.12 badDebt
    // which is already prepaid by insurance fund when alice close the position before
    // clearing house doesn't need to ask insurance fund for covering the bad debt
    let msg = engine
        .liquidate(vamm.addr().to_string(), bob.to_string())
        .unwrap();
    router.execute(carol.clone(), msg).unwrap();

    let carol_balance = usdc.balance(&router, carol.clone()).unwrap();
    assert_eq!(carol_balance, Uint128::from(5_007_352_941_176u128));

    let engine_balance = usdc.balance(&router, engine.addr().clone()).unwrap();
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
            Side::BUY,
            to_decimals(300u64),
            to_decimals(2u64),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::SELL,
            to_decimals(500u64),
            to_decimals(1u64),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // unrealizedPnl: -278.77
    // positionNotional: 600 - 278.77 = 321.23
    // remainMargin: 300 - 278.77 = 21.23
    // liquidationFee: 321.23 * 5% = 16.06
    // margin ratio: = (margin + unrealizedPnl) / positionNotional = 21.23 / 321.23 = 6.608971765%
    let msg = engine
        .liquidate(vamm.addr().to_string(), alice.to_string())
        .unwrap();
    let res = router.execute(carol.clone(), msg).unwrap_err();
    assert_eq!(
        res.to_string(),
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
            Side::BUY,
            to_decimals(150u64),
            to_decimals(4u64),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::SELL,
            to_decimals(500u64),
            to_decimals(1u64),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // alice's margin ratio = (margin + unrealizedPnl) / openNotional = (150 + (-278.77)) / 600 = -21.46%
    let msg = engine
        .liquidate(vamm.addr().to_string(), alice.to_string())
        .unwrap();
    let response = router.execute(carol.clone(), msg).unwrap();
    assert_eq!(
        response.events[4].attributes[2].value,
        Uint128::from(8_030_973_451u128).to_string()
    ); // liquidation fee
    assert_eq!(
        response.events[4].attributes[3].value,
        Uint128::from(278_761_061_950u128).to_string()
    ); // pnl (unsigned)
}

// #[test]
// fn test_alice_long_position_underwater_oracle_price_activated_doesnt_get_liquidated() {
//     let SimpleScenario {
//         mut router,
//         alice,
//         bob,
//         carol,
//         engine,
//         usdc,
//         vamm,
//         ..
//     } = SimpleScenario::new();

//     router
//         .execute_contract(
//             alice.clone(),
//             usdc.addr().clone(),
//             &Cw20ExecuteMsg::DecreaseAllowance {
//                 spender: engine.addr().to_string(),
//                 amount: to_decimals(1850),
//                 expires: None,
//             },
//             &[],
//         )
//         .unwrap();

//     router
//         .execute_contract(
//             bob.clone(),
//             usdc.addr().clone(),
//             &Cw20ExecuteMsg::DecreaseAllowance {
//                 spender: engine.addr().to_string(),
//                 amount: to_decimals(1500),
//                 expires: None,
//             },
//             &[],
//         )
//         .unwrap();

//     let msg = engine
//         .open_position(
//             vamm.addr().to_string(),
//             Side::BUY,
//             to_decimals(150u64),
//             to_decimals(4u64),
//         )
//         .unwrap();
//     router.execute(alice.clone(), msg).unwrap();

//     let spot_price = vamm.spot_price(&router).unwrap();
//     println!("{}", spot_price);
//     assert_eq!(1, 2);

//     // let msg = engine
//     //     .open_position(
//     //         vamm.addr().to_string(),
//     //         Side::SELL,
//     //         to_decimals(500u64),
//     //         to_decimals(1u64),
//     //     )
//     //     .unwrap();
//     // router.execute(bob.clone(), msg).unwrap();

//     // // alice's margin ratio = (margin + unrealizedPnl) / openNotional = (150 + (-278.77)) / 600 = -21.46%
//     // let msg = engine
//     //     .liquidate(vamm.addr().to_string(), alice.to_string())
//     //     .unwrap();
//     // let response = router.execute(carol.clone(), msg).unwrap();
//     // assert_eq!(
//     //     response.events[4].attributes[2].value,
//     //     Uint128::from(8_030_973_451u128).to_string()
//     // ); // liquidation fee
//     // assert_eq!(
//     //     response.events[4].attributes[3].value,
//     //     Uint128::from(278_761_061_950u128).to_string()
//     // ); // pnl (unsigned)
// }
