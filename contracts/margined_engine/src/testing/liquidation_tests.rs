// use crate::testing::setup::{self, to_decimals};
use cosmwasm_std::Uint128;
use cw20::Cw20ExecuteMsg;
use cw_multi_test::Executor;
use margined_common::integer::Integer;
use margined_perp::margined_engine::Side;
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

    // // reduce the allowance
    // router
    //     .execute_contract(
    //         alice.clone(),
    //         usdc.addr().clone(),
    //         &Cw20ExecuteMsg::DecreaseAllowance {
    //             spender: engine.addr().to_string(),
    //             amount: to_decimals(1980),
    //             expires: None,
    //         },
    //         &[],
    //     )
    //     .unwrap();

    // // reduce the allowance
    // router
    //     .execute_contract(
    //         bob.clone(),
    //         usdc.addr().clone(),
    //         &Cw20ExecuteMsg::DecreaseAllowance {
    //             spender: engine.addr().to_string(),
    //             amount: to_decimals(1980),
    //             expires: None,
    //         },
    //         &[],
    //     )
    //     .unwrap();

    // let msg = engine
    //     .open_position(
    //         vamm.addr().to_string(),
    //         Side::SELL,
    //         to_decimals(20u64),
    //         to_decimals(10u64),
    //         to_decimals(0u64),
    //     )
    //     .unwrap();
    // router.execute(alice.clone(), msg).unwrap();

    // let msg = engine
    //     .open_position(
    //         vamm.addr().to_string(),
    //         Side::SELL,
    //         to_decimals(20u64),
    //         to_decimals(10u64),
    //         to_decimals(0u64),
    //     )
    //     .unwrap();
    // router.execute(bob.clone(), msg).unwrap();

    // // alice close position, pnl = 200 -105.88 ~= 94.12
    // // receive pnl + margin = 114.12
    // let msg = engine
    //     .close_position(vamm.addr().to_string(), to_decimals(0u64))
    //     .unwrap();
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
    // assert_eq!(margin_ratio, Integer::new_negative(252_000_000u128));

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
