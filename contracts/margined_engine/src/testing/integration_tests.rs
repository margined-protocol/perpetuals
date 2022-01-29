use cw20::{Cw20Coin, Cw20Contract, Cw20ExecuteMsg, Cw20ReceiveMsg};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{Addr, Binary, to_binary, coins, Empty, from_binary, Uint128};
use margined_perp::margined_engine::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg, Side,
};
use margined_perp::margined_vamm::{
    InstantiateMsg as VammInstantiateMsg,
    QueryMsg as VammQueryMsg,
    StateResponse as VammStateResponse,
};
use crate::testing::setup;

#[test]
fn test_initialization() {
    setup::setup();
}


#[test]
// receive cw20 tokens and release upon approval
fn test_open_position() {
    let mut env = setup::setup();

    // set up cw20 helpers
    let usdc = Cw20Contract(env.usdc.addr.clone());

    // ensure our balances
    let owner_balance = usdc.balance(&env.router, env.owner.clone()).unwrap();
    assert_eq!(owner_balance, Uint128::new(5000));
    let alice_balance = usdc.balance(&env.router, env.alice.clone()).unwrap();
    assert_eq!(alice_balance, Uint128::new(1200));

    // verify the engine owner
    let config: ConfigResponse = env.router
        .wrap()
        .query_wasm_smart(&env.engine.addr, &QueryMsg::Config {})
        .unwrap();
    assert_eq!(
        config,
        ConfigResponse {
            owner: env.owner.clone(),
            eligible_collateral: env.usdc.addr.clone(),
        }
    );

    // transfer funds from alice to owner
    let send_msg = Cw20ExecuteMsg::Transfer {
        recipient: env.alice.to_string(),
        amount: Uint128::new(500),
    };
    let _res = env.router
        .execute_contract(env.owner.clone(), env.usdc.addr.clone(), &send_msg, &[]);
    let owner_balance = usdc.balance(&env.router, env.owner.clone()).unwrap();
    assert_eq!(owner_balance, Uint128::new(4500));
    let alice_balance = usdc.balance(&env.router, env.alice.clone()).unwrap();
    assert_eq!(alice_balance, Uint128::new(1700));

    let hook_msg = Cw20HookMsg::OpenPosition {
        vamm: env.vamm.addr.to_string(),
        side: Side::BUY,
        quote_asset_amount: Uint128::from(100u128),
        leverage: Uint128::from(100u128),
    };

    let message: Binary = to_binary(&hook_msg).unwrap();

    let send_msg = Cw20ExecuteMsg::Send {
        contract: env.engine.addr.to_string(),
        amount: Uint128::from(100u128),
        msg: message,
    };
    let res = env.router.execute_contract(
        env.alice.clone(),
        env.usdc.addr.clone(),
        &send_msg,
        &[]
    );

    let owner_balance = usdc.balance(&env.router, env.owner.clone()).unwrap();
    assert_eq!(owner_balance, Uint128::new(4500));
    let alice_balance = usdc.balance(&env.router, env.alice.clone()).unwrap();
    assert_eq!(alice_balance, Uint128::new(1600));
    let engine_balance = usdc.balance(&env.router, env.engine.addr.clone()).unwrap();
    assert_eq!(engine_balance, Uint128::new(100));

    // verify the engine owner
    let config: VammStateResponse = env.router
        .wrap()
        .query_wasm_smart(&env.vamm.addr, &VammQueryMsg::State {})
        .unwrap();
    assert_eq!(
        config,
        VammStateResponse {
            quote_asset_reserve: Uint128::from(100u128),
            base_asset_reserve: Uint128::from(10_000u128),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: Uint128::from(10_000_000_000u128),
        }
    );

    // assert_eq!(1, 2);
}
