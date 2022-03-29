// use crate::testing::setup::{self, to_decimals};
use cosmwasm_std::Uint128;
use cw20::Cw20ExecuteMsg;
use cw_multi_test::Executor;
use margined_common::integer::Integer;
use margined_perp::margined_engine::Side;
use margined_utils::scenarios::{to_decimals, SimpleScenario};

#[test]
fn test_partially_liquidate_long_position() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        carol,
        owner,
        insurance,
        engine,
        usdc,
        vamm,
        ..
    } = SimpleScenario::new();

    // set the margin ratios
    let msg = engine
    .update_config(
        None,
        None,
        None,
        None,
        None,
        None,
        Some(Uint128::from(100_000_000u128)),
        Some(Uint128::from(250_000_000u128)),
        Some(Uint128::from(25_000_000u128)),
    )
    .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // reduce the allowance
    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1900),
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
                amount: to_decimals(1900),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // when alice create a 25 margin * 10x position to get 20 long position
    // AMM after: 1250 : 80
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::BUY,
            to_decimals(25u64),
            to_decimals(10u64),
            to_decimals(0u64),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    // when bob create a 45.18072289 margin * 1x position to get 3 short position
    // AMM after: 1204.819277 : 83
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::SELL,
            Uint128::from(45_180_722_890u128),
            to_decimals(1u64),
            to_decimals(0u64),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let msg = engine
        .liquidate(vamm.addr().to_string(), alice.to_string())
        .unwrap();
    router.execute(carol.clone(), msg).unwrap();

    let position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.margin, Uint128::from(19_274_981_657u128));
    assert_eq!(position.size, Integer::new_positive(15_000_000_000u128));

    // this is todo need to add funding into the get margin ratio
    // let margin_ratio = engine
    //     .get_margin_ratio(&router, vamm.addr().to_string(), alice.to_string())
    //     .unwrap();
    // assert_eq!(margin_ratio, Integer::new_positive(43_713_253u128));

    let carol_balance = usdc.balance(&router, carol.clone()).unwrap();
    assert_eq!(carol_balance, Uint128::from(855_695_509u128));

    let insurance_balance = usdc.balance(&router, insurance.clone()).unwrap();
    assert_eq!(insurance_balance, Uint128::from(5_000_855_695_509u128));
}
