use cw20::{Cw20Contract, Cw20ExecuteMsg};
use cw_multi_test::{Executor};
use cosmwasm_std::{Binary, to_binary, Uint128};
use margined_perp::margined_engine::{
    ConfigResponse, Cw20HookMsg, QueryMsg, Side, ExecuteMsg,
    PositionResponse,
};
use margined_perp::margined_vamm::{
    QueryMsg as VammQueryMsg,
    StateResponse as VammStateResponse,
};
use crate::testing::setup::{
    self, DECIMAL_MULTIPLIER,
    to_decimals, increase_allowance,
};

#[test]
fn test_initialization() {
    let env = setup::setup();

    // set up cw20 helpers
    let usdc = Cw20Contract(env.usdc.addr.clone());

    // verfiy the balances
    let owner_balance = usdc.balance(&env.router, env.owner.clone()).unwrap();
    assert_eq!(owner_balance, Uint128::zero());
    let alice_balance = usdc.balance(&env.router, env.alice.clone()).unwrap();
    assert_eq!(alice_balance, Uint128::new(5_000_000_000_000));
    let bob_balance = usdc.balance(&env.router, env.bob.clone()).unwrap();
    assert_eq!(bob_balance, Uint128::new(5_000_000_000_000));
    let engine_balance = usdc.balance(&env.router, env.engine.addr.clone()).unwrap();
    assert_eq!(engine_balance, Uint128::zero());
}

#[test]
fn test_open_position_long() {
    let mut env = setup::setup();

    let msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: env.engine.addr.to_string(),
        amount: to_decimals(2000),
        expires: None,
    };

    env.router.execute_contract(
        env.alice.clone(),
        env.usdc.addr.clone(),
        &msg,
        &[]
    ).unwrap();

    // set up cw20 helpers
    let usdc = Cw20Contract(env.usdc.addr.clone());

    // verify the engine owner
    let _config: ConfigResponse = env.router
        .wrap()
        .query_wasm_smart(&env.engine.addr, &QueryMsg::Config {})
        .unwrap();

    let msg = ExecuteMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::BUY,
        quote_asset_amount: to_decimals(60u64),
        leverage: to_decimals(10u64),
    };

    let _res = env.router.execute_contract(
        env.alice.clone(),
        env.engine.addr.clone(),
        &msg,
        &[]
    ).unwrap();

    // expect t be 60
    let margin = env.router
    .wrap()
    .query_wasm_smart(&env.engine.addr, &QueryMsg::TraderBalance {
        trader: env.alice.to_string(),
    })
    .unwrap();
    assert_eq!(to_decimals(60), margin);

    // personal position should be 37.5
    let position: PositionResponse = env.router
        .wrap()
        .query_wasm_smart(&env.engine.addr, &QueryMsg::Position {
            vamm: env.vamm.addr.to_string(),
            trader: env.alice.to_string(),
        })
        .unwrap();
    assert_eq!(Uint128::new(37500_000_000), position.size);
    assert_eq!(to_decimals(60u64), position.margin);

    // clearing house token balance should be 60
    let engine_balance = usdc.balance(&env.router, env.engine.addr.clone()).unwrap();
    assert_eq!(engine_balance, to_decimals(60));

}

#[test]
fn test_open_position_two_longs() {
    let mut env = setup::setup();

    // set up cw20 helpers
    let usdc = Cw20Contract(env.usdc.addr.clone());

    // verify the engine owner
    let _config: ConfigResponse = env.router
        .wrap()
        .query_wasm_smart(&env.engine.addr, &QueryMsg::Config {})
        .unwrap();

    let send_msg = Cw20ExecuteMsg::Send {
        contract: env.engine.addr.to_string(),
        amount: to_decimals(60u64),
        msg: to_binary(&Cw20HookMsg::OpenPosition {
            vamm: env.vamm.addr.to_string(),
            side: Side::BUY,
            leverage: to_decimals(10u64),
        }).unwrap(),
    };

    let _res = env.router.execute_contract(
        env.alice.clone(),
        env.usdc.addr.clone(),
        &send_msg,
        &[]
    ).unwrap();

    let send_msg = Cw20ExecuteMsg::Send {
        contract: env.engine.addr.to_string(),
        amount: to_decimals(60u64),
        msg: to_binary(&Cw20HookMsg::OpenPosition {
            vamm: env.vamm.addr.to_string(),
            side: Side::BUY,
            leverage: to_decimals(10u64),
        }).unwrap(),
    };

    let _res = env.router.execute_contract(
        env.alice.clone(),
        env.usdc.addr.clone(),
        &send_msg,
        &[]
    ).unwrap();

    let owner_balance = usdc.balance(&env.router, env.owner.clone()).unwrap();
    assert_eq!(owner_balance, Uint128::zero());
    let alice_balance = usdc.balance(&env.router, env.alice.clone()).unwrap();
    assert_eq!(alice_balance, Uint128::new(4880) * DECIMAL_MULTIPLIER);
    let engine_balance = usdc.balance(&env.router, env.engine.addr.clone()).unwrap();
    assert_eq!(engine_balance, Uint128::new(120) * DECIMAL_MULTIPLIER);

    // retrieve the vamm state
    let position: PositionResponse = env.router
        .wrap()
        .query_wasm_smart(&env.engine.addr, &QueryMsg::Position {
            vamm: env.vamm.addr.to_string(),
            trader: env.alice.to_string(),
        })
        .unwrap();
    assert_eq!(Uint128::new(54_545_454_546), position.size);
    assert_eq!(to_decimals(120), position.margin);

}

#[test]
fn test_open_position_two_shorts() {
    let mut env = setup::setup();

    // set up cw20 helpers
    let usdc = Cw20Contract(env.usdc.addr.clone());

    // verify the engine owner
    let _config: ConfigResponse = env.router
        .wrap()
        .query_wasm_smart(&env.engine.addr, &QueryMsg::Config {})
        .unwrap();

    let send_msg = Cw20ExecuteMsg::Send {
        contract: env.engine.addr.to_string(),
        amount: to_decimals(40u64),
        msg: to_binary(&Cw20HookMsg::OpenPosition {
            vamm: env.vamm.addr.to_string(),
            side: Side::SELL,
            leverage: to_decimals(5u64),
        }).unwrap(),
    };

    let _res = env.router.execute_contract(
        env.alice.clone(),
        env.usdc.addr.clone(),
        &send_msg,
        &[]
    ).unwrap();

    let send_msg = Cw20ExecuteMsg::Send {
        contract: env.engine.addr.to_string(),
        amount: to_decimals(40u64),
        msg: to_binary(&Cw20HookMsg::OpenPosition {
            vamm: env.vamm.addr.to_string(),
            side: Side::SELL,
            leverage: to_decimals(5u64),
        }).unwrap(),
    };

    let _res = env.router.execute_contract(
        env.alice.clone(),
        env.usdc.addr.clone(),
        &send_msg,
        &[]
    ).unwrap();

    // retrieve the vamm state
    let position: PositionResponse = env.router
        .wrap()
        .query_wasm_smart(&env.engine.addr, &QueryMsg::Position {
            vamm: env.vamm.addr.to_string(),
            trader: env.alice.to_string(),
        })
        .unwrap();
    assert_eq!(Uint128::new(66_666_666_666), position.size);
    assert_eq!(to_decimals(80), position.margin);

}

#[test]
fn test_open_position_equal_size_long_short() {
    let mut env = setup::setup();

    // set up cw20 helpers
    let usdc = Cw20Contract(env.usdc.addr.clone());

    // verify the engine owner
    let _config: ConfigResponse = env.router
        .wrap()
        .query_wasm_smart(&env.engine.addr, &QueryMsg::Config {})
        .unwrap();

    let send_msg = Cw20ExecuteMsg::Send {
        contract: env.engine.addr.to_string(),
        amount: to_decimals(60u64),
        msg: to_binary(&Cw20HookMsg::OpenPosition {
            vamm: env.vamm.addr.to_string(),
            side: Side::BUY,
            leverage: to_decimals(10u64),
        }).unwrap(),
    };

    let _res = env.router.execute_contract(
        env.alice.clone(),
        env.usdc.addr.clone(),
        &send_msg,
        &[]
    ).unwrap();

    let send_msg = Cw20ExecuteMsg::Send {
        contract: env.engine.addr.to_string(),
        amount: to_decimals(300u64),
        msg: to_binary(&Cw20HookMsg::OpenPosition {
            vamm: env.vamm.addr.to_string(),
            side: Side::SELL,
            leverage: to_decimals(2u64),
        }).unwrap(),
    };

    let _res = env.router.execute_contract(
        env.alice.clone(),
        env.usdc.addr.clone(),
        &send_msg,
        &[]
    ).unwrap();

    // retrieve the vamm state
    let position: PositionResponse = env.router
        .wrap()
        .query_wasm_smart(&env.engine.addr, &QueryMsg::Position {
            vamm: env.vamm.addr.to_string(),
            trader: env.alice.to_string(),
        })
        .unwrap();
    assert_eq!(Uint128::zero(), position.size);
    assert_eq!(Uint128::zero(), position.margin);

}

#[test]
fn test_open_position_one_long_two_shorts() {
    let mut env = setup::setup();

    // set up cw20 helpers
    let usdc = Cw20Contract(env.usdc.addr.clone());

    // verify the engine owner
    let _config: ConfigResponse = env.router
        .wrap()
        .query_wasm_smart(&env.engine.addr, &QueryMsg::Config {})
        .unwrap();

    let send_msg = Cw20ExecuteMsg::Send {
        contract: env.engine.addr.to_string(),
        amount: to_decimals(60u64),
        msg: to_binary(&Cw20HookMsg::OpenPosition {
            vamm: env.vamm.addr.to_string(),
            side: Side::BUY,
            leverage: to_decimals(10u64),
        }).unwrap(),
    };

    let _res = env.router.execute_contract(
        env.alice.clone(),
        env.usdc.addr.clone(),
        &send_msg,
        &[]
    ).unwrap();

    let send_msg = Cw20ExecuteMsg::Send {
        contract: env.engine.addr.to_string(),
        amount: to_decimals(20u64),
        msg: to_binary(&Cw20HookMsg::OpenPosition {
            vamm: env.vamm.addr.to_string(),
            side: Side::SELL,
            leverage: to_decimals(5u64),
        }).unwrap(),
    };

    let _res = env.router.execute_contract(
        env.alice.clone(),
        env.usdc.addr.clone(),
        &send_msg,
        &[]
    ).unwrap();

    // retrieve the vamm state
    let position: PositionResponse = env.router
        .wrap()
        .query_wasm_smart(&env.engine.addr, &QueryMsg::Position {
            vamm: env.vamm.addr.to_string(),
            trader: env.alice.to_string(),
        })
        .unwrap();
    assert_eq!(Uint128::new(33_333_333_334), position.size);
    assert_eq!(to_decimals(60), position.margin);

    let send_msg = Cw20ExecuteMsg::Send {
        contract: env.engine.addr.to_string(),
        amount: to_decimals(50u64),
        msg: to_binary(&Cw20HookMsg::OpenPosition {
            vamm: env.vamm.addr.to_string(),
            side: Side::SELL,
            leverage: to_decimals(10u64),
        }).unwrap(),
    };

    // TODO THERE IS A SMALL ROUNDING ERROR / PROBLEM
    // let _res = env.router.execute_contract(
    //     env.alice.clone(),
    //     env.usdc.addr.clone(),
    //     &send_msg,
    //     &[]
    // ).unwrap();

    // // retrieve the vamm state
    // let position: PositionResponse = env.router
    //     .wrap()
    //     .query_wasm_smart(&env.engine.addr, &QueryMsg::Position {
    //         vamm: env.vamm.addr.to_string(),
    //         trader: env.alice.to_string(),
    //     })
    //     .unwrap();
    // assert_eq!(Uint128::zero(), position.size);
    // assert_eq!(Uint128::zero(), position.margin);
    
}

#[test]
fn test_open_position_short_and_two_longs() {
    let mut env = setup::setup();

    // set up cw20 helpers
    let usdc = Cw20Contract(env.usdc.addr.clone());

    // verify the engine owner
    let _config: ConfigResponse = env.router
        .wrap()
        .query_wasm_smart(&env.engine.addr, &QueryMsg::Config {})
        .unwrap();

    let send_msg = Cw20ExecuteMsg::Send {
        contract: env.engine.addr.to_string(),
        amount: to_decimals(40u64),
        msg: to_binary(&Cw20HookMsg::OpenPosition {
            vamm: env.vamm.addr.to_string(),
            side: Side::SELL,
            leverage: to_decimals(5u64),
        }).unwrap(),
    };

    let _res = env.router.execute_contract(
        env.alice.clone(),
        env.usdc.addr.clone(),
        &send_msg,
        &[]
    ).unwrap();

    let send_msg = Cw20ExecuteMsg::Send {
        contract: env.engine.addr.to_string(),
        amount: to_decimals(20u64),
        msg: to_binary(&Cw20HookMsg::OpenPosition {
            vamm: env.vamm.addr.to_string(),
            side: Side::BUY,
            leverage: to_decimals(5u64),
        }).unwrap(),
    };

    let _res = env.router.execute_contract(
        env.alice.clone(),
        env.usdc.addr.clone(),
        &send_msg,
        &[]
    ).unwrap();

        // retrieve the vamm state
    let position: PositionResponse = env.router
        .wrap()
        .query_wasm_smart(&env.engine.addr, &QueryMsg::Position {
            vamm: env.vamm.addr.to_string(),
            trader: env.alice.to_string(),
        })
        .unwrap();
    assert_eq!(Uint128::new(11_111_111_111), position.size);
    assert_eq!(to_decimals(40), position.margin);

    let send_msg = Cw20ExecuteMsg::Send {
        contract: env.engine.addr.to_string(),
        amount: to_decimals(20u64),
        msg: to_binary(&Cw20HookMsg::OpenPosition {
            vamm: env.vamm.addr.to_string(),
            side: Side::BUY,
            leverage: to_decimals(5u64),
        }).unwrap(),
    };

    let _res = env.router.execute_contract(
        env.alice.clone(),
        env.usdc.addr.clone(),
        &send_msg,
        &[]
    ).unwrap();


    // retrieve the vamm state
    let position: PositionResponse = env.router
        .wrap()
        .query_wasm_smart(&env.engine.addr, &QueryMsg::Position {
            vamm: env.vamm.addr.to_string(),
            trader: env.alice.to_string(),
        })
        .unwrap();
    assert_eq!(Uint128::new(1), position.size);
    assert_eq!(Uint128::new(1), position.margin);

}
