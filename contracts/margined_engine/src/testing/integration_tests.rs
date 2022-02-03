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
use crate::testing::setup::{self, DECIMAL_MULTIPLIER};

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
// receive cw20 tokens and release upon approval
fn test_open_position_long() {
    let mut env = setup::setup();

    // set up cw20 helpers
    let usdc = Cw20Contract(env.usdc.addr.clone());

    // verify the engine owner
    let _config: ConfigResponse = env.router
        .wrap()
        .query_wasm_smart(&env.engine.addr, &QueryMsg::Config {})
        .unwrap();

    let hook_msg = Cw20HookMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::BUY,
        quote_asset_amount: Uint128::from(60u128) * DECIMAL_MULTIPLIER,
        leverage: Uint128::from(10u128),
    };

    let message: Binary = to_binary(&hook_msg).unwrap();

    let send_msg = Cw20ExecuteMsg::Send {
        contract: env.engine.addr.to_string(),
        amount: Uint128::from(60u128) * DECIMAL_MULTIPLIER,
        msg: message,
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
    assert_eq!(alice_balance, Uint128::new(4940) * DECIMAL_MULTIPLIER);
    let bob_balance = usdc.balance(&env.router, env.bob.clone()).unwrap();
    assert_eq!(bob_balance, Uint128::new(5000) * DECIMAL_MULTIPLIER);
    let engine_balance = usdc.balance(&env.router, env.engine.addr.clone()).unwrap();
    assert_eq!(engine_balance, Uint128::new(60) * DECIMAL_MULTIPLIER);

    // retrieve the vamm state
    let position: PositionResponse = env.router
        .wrap()
        .query_wasm_smart(&env.engine.addr, &QueryMsg::Position {
            vamm: env.vamm.addr.to_string(),
            trader: env.alice.to_string(),
        })
        .unwrap();
    assert_eq!(Uint128::new(3750_000_000_000), position.size);

}
