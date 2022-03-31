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
        pricefeed,
        ..
    } = SimpleScenario::new();

    // set the latest price
    let price: Uint128 = Uint128::from(10_000_000_000u128);
    let timestamp: u64 = router.block_info().time.seconds();

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(900);
        block.height += 1;
    });

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
        .liquidate(
            vamm.addr().to_string(),
            alice.to_string(),
            to_decimals(0u64),
        )
        .unwrap();
    router.execute(carol.clone(), msg).unwrap();

    let position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.margin, Uint128::from(19_274_981_657u128));
    assert_eq!(position.size, Integer::new_positive(15_000_000_000u128));

    // this is todo need to add funding into the get margin ratio
    let margin_ratio = engine
        .get_margin_ratio(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(margin_ratio, Integer::new_positive(43_713_253u128));

    let carol_balance = usdc.balance(&router, carol.clone()).unwrap();
    assert_eq!(carol_balance, Uint128::from(855_695_509u128));

    let insurance_balance = usdc.balance(&router, insurance.clone()).unwrap();
    assert_eq!(insurance_balance, Uint128::from(5_000_855_695_509u128));
}

#[test]
fn test_partially_liquidate_long_position_with_quote_asset_limit() {
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
        pricefeed,
        ..
    } = SimpleScenario::new();

    // set the latest price
    let price: Uint128 = Uint128::from(10_000_000_000u128);
    let timestamp: u64 = router.block_info().time.seconds();

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(900);
        block.height += 1;
    });

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

    // partially liquidate 25%
    // liquidated positionNotional: getOutputPrice(20 (original position) * 0.25) = 68.455
    // if quoteAssetAmountLimit == 273.85 > 68.455 * 4 = 273.82, quote asset gets is less than expected, thus tx reverts
    let msg = engine
        .liquidate(
            vamm.addr().to_string(),
            alice.to_string(),
            Uint128::from(273_850_000_000u64),
        )
        .unwrap();
    let result = router.execute(carol.clone(), msg).unwrap_err();
    assert_eq!(result.to_string(), "Generic error: reply (id 6) error \"Generic error: Less than minimum quote asset amount limit\"");

    // if quoteAssetAmountLimit == 273.8 < 68.455 * 4 = 273.82, quote asset gets is more than expected
    let msg = engine
    .liquidate(
        vamm.addr().to_string(),
        alice.to_string(),
        Uint128::from(273_800_000_000u64),
    )
    .unwrap();
    router.execute(carol.clone(), msg).unwrap();
}

#[test]
fn test_partially_liquidate_short_position() {
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
        pricefeed,
        ..
    } = SimpleScenario::new();

    // set the latest price
    let price: Uint128 = Uint128::from(10_000_000_000u128);
    let timestamp: u64 = router.block_info().time.seconds();

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(900);
        block.height += 1;
    });

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

    // when alice create a 20 margin * 10x position to get 25 short position
    // AMM after: 800 : 125
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::SELL,
            to_decimals(20u64),
            to_decimals(10u64),
            to_decimals(0u64),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    // when bob create a 19.67213115 margin * 1x position to get 3 long position
    // AMM after: 819.6721311 : 122
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::BUY,
            Uint128::from(19_672_131_150u128),
            to_decimals(1u64),
            to_decimals(0u64),
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let msg = engine
        .liquidate(
            vamm.addr().to_string(),
            alice.to_string(),
            to_decimals(0u64),
        )
        .unwrap();
    router.execute(carol.clone(), msg).unwrap();

    let position = engine
        .position(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(position.margin, Uint128::from(16_079_605_165u128));
    assert_eq!(position.size, Integer::new_negative(18_750_000_000u128));

    let margin_ratio = engine
        .get_margin_ratio(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(margin_ratio, Integer::new_positive(45_736_327u128));

    let carol_balance = usdc.balance(&router, carol.clone()).unwrap();
    assert_eq!(carol_balance, Uint128::from(553_234_429u128));

    let insurance_balance = usdc.balance(&router, insurance.clone()).unwrap();
    assert_eq!(insurance_balance, Uint128::from(5_000_553_234_429u128));
}
