use cosmwasm_std::{Coin, Uint128};
use margined_perp::margined_engine::Side;
use margined_utils::scenarios::NativeTokenScenario;
use terra_multi_test::Executor;

#[test]
fn test_liquidator_can_open_position_and_liquidate_in_next_block() {
    let mut env = NativeTokenScenario::new();

    // set the latest price
    let price: Uint128 = Uint128::from(10_000_000u128);
    let timestamp: u64 = env.router.block_info().time.seconds();

    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(900);
        block.height += 1;
    });

    // set the margin ratios
    let msg = env
        .engine
        .set_maintenance_margin_ratio(Uint128::from(100_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_partial_liquidation_margin_ratio(Uint128::from(250_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_liquidation_fee(Uint128::from(25_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // mint funds for carol
    let init_funds = vec![Coin::new(1_000u128 * 10u128.pow(6), "uusd")];
    env.router
        .init_bank_balance(&env.carol, init_funds.clone())
        .unwrap();

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::BUY,
            Uint128::from(20_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(9_090_000u128),
            vec![Coin::new(20_000_000u128, "uusd")],
        )
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::BUY,
            Uint128::from(20_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::zero(),
            vec![Coin::new(20_000_000u128, "uusd")],
        )
        .unwrap();
    env.router.execute(env.alice.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::SELL,
            Uint128::from(20_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::zero(),
            vec![Coin::new(20_000_000u128, "uusd")],
        )
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::SELL,
            Uint128::from(20_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::zero(),
            vec![Coin::new(20_000_000u128, "uusd")],
        )
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let price = env.vamm.spot_price(&env.router).unwrap();
    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .liquidate(
            env.vamm.addr().to_string(),
            env.alice.to_string(),
            Uint128::zero(),
        )
        .unwrap();
    let response = env.router.execute(env.carol.clone(), msg).unwrap();
    assert_eq!(
        response.events[4].attributes[1].value,
        "partial_liquidate_reply".to_string()
    );
}

#[test]
fn test_can_open_position_short_and_liquidate_but_cannot_do_anything_more_in_same_block() {
    let mut env = NativeTokenScenario::new();

    // set the latest price
    let price: Uint128 = Uint128::from(10_000_000u128);
    let timestamp: u64 = env.router.block_info().time.seconds();

    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(900);
        block.height += 1;
    });

    // set the margin ratios
    let msg = env
        .engine
        .set_maintenance_margin_ratio(Uint128::from(100_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_partial_liquidation_margin_ratio(Uint128::from(250_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_liquidation_fee(Uint128::from(25_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // mint funds for carol
    let init_funds = vec![Coin::new(1_000u128 * 10u128.pow(6), "uusd")];
    env.router
        .init_bank_balance(&env.carol, init_funds.clone())
        .unwrap();

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::BUY,
            Uint128::from(20_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(9_090_000u128),
            vec![Coin::new(20_000_000u128, "uusd")],
        )
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::BUY,
            Uint128::from(20_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(7_570_000u128),
            vec![Coin::new(20_000_000u128, "uusd")],
        )
        .unwrap();
    env.router.execute(env.alice.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::SELL,
            Uint128::from(20_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(7_580_000u128),
            vec![Coin::new(20_000_000u128, "uusd")],
        )
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::SELL,
            Uint128::from(20_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::zero(),
            vec![Coin::new(20_000_000u128, "uusd")],
        )
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let price = env.vamm.spot_price(&env.router).unwrap();
    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .liquidate(
            env.vamm.addr().to_string(),
            env.alice.to_string(),
            Uint128::zero(),
        )
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let msg = env
        .engine
        .close_position(env.vamm.addr().to_string(), Uint128::zero())
        .unwrap();
    let response = env.router.execute(env.carol.clone(), msg).unwrap_err();
    assert_eq!(
        response.to_string(),
        "Generic error: Only one action allowed".to_string()
    );
}

#[test]
fn test_can_open_position_long_and_liquidate_but_cannot_do_anything_more_in_same_block() {
    let mut env = NativeTokenScenario::new();

    // set the latest price
    let price: Uint128 = Uint128::from(10_000_000u128);
    let timestamp: u64 = env.router.block_info().time.seconds();

    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(900);
        block.height += 1;
    });

    let msg = env
        .engine
        .set_maintenance_margin_ratio(Uint128::from(100_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_partial_liquidation_margin_ratio(Uint128::from(250_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_liquidation_fee(Uint128::from(25_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // mint funds for carol
    let init_funds = vec![Coin::new(1_000u128 * 10u128.pow(6), "uusd")];
    env.router
        .init_bank_balance(&env.carol, init_funds.clone())
        .unwrap();

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::SELL,
            Uint128::from(20_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::zero(),
            vec![Coin::new(20_000_000u128, "uusd")],
        )
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::SELL,
            Uint128::from(20_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::zero(),
            vec![Coin::new(20_000_000u128, "uusd")],
        )
        .unwrap();
    env.router.execute(env.alice.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .close_position(env.vamm.addr().to_string(), Uint128::zero())
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::BUY,
            Uint128::from(20_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::zero(),
            vec![Coin::new(20_000_000u128, "uusd")],
        )
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let price = env.vamm.spot_price(&env.router).unwrap();
    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .liquidate(
            env.vamm.addr().to_string(),
            env.alice.to_string(),
            Uint128::zero(),
        )
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let msg = env
        .engine
        .close_position(env.vamm.addr().to_string(), Uint128::zero())
        .unwrap();
    let response = env.router.execute(env.carol.clone(), msg).unwrap_err();
    assert_eq!(
        response.to_string(),
        "Generic error: Only one action allowed".to_string()
    );
}

#[test]
fn test_can_open_position_and_liquidate_but_cannot_do_anything_more_in_same_block() {
    let mut env = NativeTokenScenario::new();

    // set the latest price
    let price: Uint128 = Uint128::from(10_000_000u128);
    let timestamp: u64 = env.router.block_info().time.seconds();

    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(900);
        block.height += 1;
    });

    let msg = env
        .engine
        .set_maintenance_margin_ratio(Uint128::from(100_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_partial_liquidation_margin_ratio(Uint128::from(250_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_liquidation_fee(Uint128::from(25_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // mint funds for carol
    let init_funds = vec![Coin::new(1_000u128 * 10u128.pow(6), "uusd")];
    env.router
        .init_bank_balance(&env.carol, init_funds.clone())
        .unwrap();

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::BUY,
            Uint128::from(20_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(9_090_000u128),
            vec![Coin::new(20_000_000u128, "uusd")],
        )
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::BUY,
            Uint128::from(20_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(7_570_000u128),
            vec![Coin::new(20_000_000u128, "uusd")],
        )
        .unwrap();
    env.router.execute(env.alice.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::SELL,
            Uint128::from(20_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::from(7_580_000u128),
            vec![Coin::new(20_000_000u128, "uusd")],
        )
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::BUY,
            Uint128::from(10_000_000u64),
            Uint128::from(1_000_000u64),
            Uint128::from(0u64),
            vec![Coin::new(10_000_000u128, "uusd")],
        )
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let price = env.vamm.spot_price(&env.router).unwrap();
    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .liquidate(
            env.vamm.addr().to_string(),
            env.alice.to_string(),
            Uint128::zero(),
        )
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let msg = env
        .engine
        .close_position(env.vamm.addr().to_string(), Uint128::zero())
        .unwrap();
    let response = env.router.execute(env.carol.clone(), msg).unwrap_err();
    assert_eq!(
        response.to_string(),
        "Generic error: Only one action allowed".to_string()
    );
}

#[test]
fn test_can_open_position_same_side_and_liquidate_but_cannot_do_anything_more_in_same_block() {
    let mut env = NativeTokenScenario::new();

    // set the latest price
    let price: Uint128 = Uint128::from(10_000_000u128);
    let timestamp: u64 = env.router.block_info().time.seconds();

    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(900);
        block.height += 1;
    });

    let msg = env
        .engine
        .set_maintenance_margin_ratio(Uint128::from(100_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_partial_liquidation_margin_ratio(Uint128::from(250_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_liquidation_fee(Uint128::from(25_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // mint funds for carol
    let init_funds = vec![Coin::new(1_000u128 * 10u128.pow(6), "uusd")];
    env.router
        .init_bank_balance(&env.carol, init_funds.clone())
        .unwrap();

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::SELL,
            Uint128::from(20_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::zero(),
            vec![Coin::new(20_000_000u128, "uusd")],
        )
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::SELL,
            Uint128::from(20_000_000u64),
            Uint128::from(5_000_000u64),
            Uint128::zero(),
            vec![Coin::new(20_000_000u128, "uusd")],
        )
        .unwrap();
    env.router.execute(env.alice.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .close_position(env.vamm.addr().to_string(), Uint128::zero())
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::SELL,
            Uint128::from(10_000_000u64),
            Uint128::from(1u64),
            Uint128::zero(),
            vec![Coin::new(10_000_000u128, "uusd")],
        )
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let price = env.vamm.spot_price(&env.router).unwrap();
    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .liquidate(
            env.vamm.addr().to_string(),
            env.alice.to_string(),
            Uint128::zero(),
        )
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let msg = env
        .engine
        .close_position(env.vamm.addr().to_string(), Uint128::zero())
        .unwrap();
    let response = env.router.execute(env.carol.clone(), msg).unwrap_err();
    assert_eq!(
        response.to_string(),
        "Generic error: Only one action allowed".to_string()
    );
}
