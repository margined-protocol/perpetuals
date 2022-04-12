// use crate::testing::setup::{self, to_decimals};
use cosmwasm_std::Uint128;
use cw20::Cw20ExecuteMsg;
use cw_multi_test::Executor;
use margined_perp::margined_engine::Side;
use margined_utils::scenarios::{to_decimals, SimpleScenario};

#[test]
fn test_liquidator_can_open_position_and_liquidate_in_next_block() {
    let mut env = SimpleScenario::new();

    // set the latest price
    let price: Uint128 = Uint128::from(10_000_000_000u128);
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
        .set_maintenance_margin_ratio(Uint128::from(100_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_partial_liquidation_margin_ratio(Uint128::from(250_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_liquidation_fee(Uint128::from(25_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // reduce the allowance
    env.router
        .execute_contract(
            env.alice.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // reduce the allowance
    env.router
        .execute_contract(
            env.bob.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // mint funds for carol
    env.router
        .execute_contract(
            env.owner.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::Mint {
                recipient: env.carol.to_string(),
                amount: to_decimals(1000u64),
            },
            &[],
        )
        .unwrap();

    // set allowance for carol
    env.router
        .execute_contract(
            env.carol.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000u64),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::BUY,
            to_decimals(20u64),
            to_decimals(5u64),
            Uint128::from(9_090_000_000u128),
            vec![],
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
            to_decimals(20u64),
            to_decimals(5u64),
            Uint128::zero(),
            vec![],
            // Uint128::from(7_570_000_000u128),
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
            to_decimals(20u64),
            to_decimals(5u64),
            Uint128::zero(),
            vec![],
            // Uint128::from(7_580_000_000u128),
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
            to_decimals(20u64),
            to_decimals(5u64),
            to_decimals(0u64),
            vec![],
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
            to_decimals(0u64),
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
    let mut env = SimpleScenario::new();

    // set the latest price
    let price: Uint128 = Uint128::from(10_000_000_000u128);
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
        .set_maintenance_margin_ratio(Uint128::from(100_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_partial_liquidation_margin_ratio(Uint128::from(250_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_liquidation_fee(Uint128::from(25_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // reduce the allowance
    env.router
        .execute_contract(
            env.alice.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // reduce the allowance
    env.router
        .execute_contract(
            env.bob.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // mint funds for carol
    env.router
        .execute_contract(
            env.owner.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::Mint {
                recipient: env.carol.to_string(),
                amount: to_decimals(1000u64),
            },
            &[],
        )
        .unwrap();

    // set allowance for carol
    env.router
        .execute_contract(
            env.carol.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000u64),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::BUY,
            to_decimals(20u64),
            to_decimals(5u64),
            Uint128::from(9_090_000_000u128),
            vec![],
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
            to_decimals(20u64),
            to_decimals(5u64),
            Uint128::from(7_570_000_000u128),
            vec![],
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
            to_decimals(20u64),
            to_decimals(5u64),
            Uint128::from(7_580_000_000u128),
            vec![],
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
            to_decimals(20u64),
            to_decimals(5u64),
            to_decimals(0u64),
            vec![],
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
            to_decimals(0u64),
        )
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let msg = env
        .engine
        .close_position(env.vamm.addr().to_string(), to_decimals(0u64))
        .unwrap();
    let response = env.router.execute(env.carol.clone(), msg).unwrap_err();
    assert_eq!(
        response.to_string(),
        "Generic error: Only one action allowed".to_string()
    );
}

#[test]
fn test_can_open_position_long_and_liquidate_but_cannot_do_anything_more_in_same_block() {
    let mut env = SimpleScenario::new();

    // set the latest price
    let price: Uint128 = Uint128::from(10_000_000_000u128);
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
        .set_maintenance_margin_ratio(Uint128::from(100_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_partial_liquidation_margin_ratio(Uint128::from(250_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_liquidation_fee(Uint128::from(25_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // reduce the allowance
    env.router
        .execute_contract(
            env.alice.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // reduce the allowance
    env.router
        .execute_contract(
            env.bob.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // mint funds for carol
    env.router
        .execute_contract(
            env.owner.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::Mint {
                recipient: env.carol.to_string(),
                amount: to_decimals(1000u64),
            },
            &[],
        )
        .unwrap();

    // set allowance for carol
    env.router
        .execute_contract(
            env.carol.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000u64),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::SELL,
            to_decimals(20u64),
            to_decimals(5u64),
            Uint128::zero(),
            vec![],
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
            to_decimals(20u64),
            to_decimals(5u64),
            Uint128::zero(),
            vec![],
        )
        .unwrap();
    env.router.execute(env.alice.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .close_position(env.vamm.addr().to_string(), to_decimals(0u64))
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
            to_decimals(20u64),
            to_decimals(5u64),
            to_decimals(0u64),
            vec![],
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
            to_decimals(0u64),
        )
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let msg = env
        .engine
        .close_position(env.vamm.addr().to_string(), to_decimals(0u64))
        .unwrap();
    let response = env.router.execute(env.carol.clone(), msg).unwrap_err();
    assert_eq!(
        response.to_string(),
        "Generic error: Only one action allowed".to_string()
    );
}

#[test]
fn test_can_open_position_and_liquidate_but_cannot_do_anything_more_in_same_block() {
    let mut env = SimpleScenario::new();

    // set the latest price
    let price: Uint128 = Uint128::from(10_000_000_000u128);
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
        .set_maintenance_margin_ratio(Uint128::from(100_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_partial_liquidation_margin_ratio(Uint128::from(250_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_liquidation_fee(Uint128::from(25_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // reduce the allowance
    env.router
        .execute_contract(
            env.alice.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // reduce the allowance
    env.router
        .execute_contract(
            env.bob.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // mint funds for carol
    env.router
        .execute_contract(
            env.owner.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::Mint {
                recipient: env.carol.to_string(),
                amount: to_decimals(1000u64),
            },
            &[],
        )
        .unwrap();

    // set allowance for carol
    env.router
        .execute_contract(
            env.carol.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000u64),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::BUY,
            to_decimals(20u64),
            to_decimals(5u64),
            Uint128::from(9_090_000_000u128),
            vec![],
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
            to_decimals(20u64),
            to_decimals(5u64),
            Uint128::from(7_570_000_000u128),
            vec![],
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
            to_decimals(20u64),
            to_decimals(5u64),
            Uint128::from(7_580_000_000u128),
            vec![],
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
            to_decimals(10u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
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
            to_decimals(0u64),
        )
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let msg = env
        .engine
        .close_position(env.vamm.addr().to_string(), to_decimals(0u64))
        .unwrap();
    let response = env.router.execute(env.carol.clone(), msg).unwrap_err();
    assert_eq!(
        response.to_string(),
        "Generic error: Only one action allowed".to_string()
    );
}

#[test]
fn test_can_open_position_same_side_and_liquidate_but_cannot_do_anything_more_in_same_block() {
    let mut env = SimpleScenario::new();

    // set the latest price
    let price: Uint128 = Uint128::from(10_000_000_000u128);
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
        .set_maintenance_margin_ratio(Uint128::from(100_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_partial_liquidation_margin_ratio(Uint128::from(250_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_liquidation_fee(Uint128::from(25_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // reduce the allowance
    env.router
        .execute_contract(
            env.alice.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // reduce the allowance
    env.router
        .execute_contract(
            env.bob.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // mint funds for carol
    env.router
        .execute_contract(
            env.owner.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::Mint {
                recipient: env.carol.to_string(),
                amount: to_decimals(1000u64),
            },
            &[],
        )
        .unwrap();

    // set allowance for carol
    env.router
        .execute_contract(
            env.carol.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000u64),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::SELL,
            to_decimals(20u64),
            to_decimals(5u64),
            Uint128::zero(),
            vec![],
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
            to_decimals(20u64),
            to_decimals(5u64),
            Uint128::zero(),
            vec![],
        )
        .unwrap();
    env.router.execute(env.alice.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .close_position(env.vamm.addr().to_string(), to_decimals(0u64))
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
            to_decimals(10u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
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
            to_decimals(0u64),
        )
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let msg = env
        .engine
        .close_position(env.vamm.addr().to_string(), to_decimals(0u64))
        .unwrap();
    let response = env.router.execute(env.carol.clone(), msg).unwrap_err();
    assert_eq!(
        response.to_string(),
        "Generic error: Only one action allowed".to_string()
    );
}
