use crate::contract::{execute, instantiate, query};
use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{from_binary, Env, OwnedDeps, Uint128};
use margined_perp::margined_vamm::{Direction, ExecuteMsg, InstantiateMsg, QueryMsg};
use margined_utils::scenarios::to_decimals;

pub struct TestingEnv {
    pub deps: OwnedDeps<MockStorage, MockApi, MockQuerier>,
    pub env: Env,
}

fn setup() -> TestingEnv {
    let mut env = mock_env();
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1_000),
        base_asset_reserve: to_decimals(100),
        funding_period: 3_600_u64,
        toll_ratio: Uint128::from(10_000_000u128),   // 0.01
        spread_ratio: Uint128::from(10_000_000u128), // 0.01
        margin_engine: Some("addr0000".to_string()),
        pricefeed: "oracle".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    let msg = ExecuteMsg::SetOpen { open: true };
    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    env.block.time = env.block.time.plus_seconds(14);
    env.block.height += 1;

    for i in 0..30 {
        if i % 3 == 0 {
            let swap_msg = ExecuteMsg::SwapInput {
                direction: Direction::RemoveFromAmm,
                quote_asset_amount: to_decimals(100),
            };

            let info = mock_info("addr0000", &[]);
            execute(deps.as_mut(), env.clone(), info, swap_msg).unwrap();
        } else {
            let swap_msg = ExecuteMsg::SwapInput {
                direction: Direction::AddToAmm,
                quote_asset_amount: to_decimals(50),
            };

            let info = mock_info("addr0000", &[]);
            execute(deps.as_mut(), env.clone(), info, swap_msg).unwrap();
        }
        env.block.time = env.block.time.plus_seconds(14);
        env.block.height += 1;
    }

    TestingEnv { deps, env }
}

#[test]
fn test_get_twap_price() {
    let app = setup();

    let res = query(
        app.deps.as_ref(),
        app.env,
        QueryMsg::TwapPrice { interval: 210 },
    )
    .unwrap();
    let twap: Uint128 = from_binary(&res).unwrap();
    assert_eq!(twap, Uint128::from(9_041_666_665u128));
}

#[test]
fn test_no_change_in_snapshot() {
    let mut app = setup();

    // the timestamp of latest snapshot is now, the latest snapshot wont have any effect
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals(100),
    };

    let info = mock_info("addr0000", &[]);
    execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();

    let res = query(
        app.deps.as_ref(),
        app.env,
        QueryMsg::TwapPrice { interval: 210 },
    )
    .unwrap();
    let twap: Uint128 = from_binary(&res).unwrap();
    assert_eq!(twap, Uint128::from(9_041_666_665u128));
}

#[test]
fn test_interval_greater_than_snapshots_have() {
    let app = setup();

    let res = query(
        app.deps.as_ref(),
        app.env,
        QueryMsg::TwapPrice { interval: 900 },
    )
    .unwrap();
    let twap: Uint128 = from_binary(&res).unwrap();
    assert_eq!(twap, Uint128::from(9_072_580_644u128));
}

#[test]
fn test_interval_less_than_latest_snapshots() {
    let mut app = setup();

    // the timestamp of latest snapshot is now, the latest snapshot wont have any effect
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals(100),
    };

    let info = mock_info("addr0000", &[]);
    execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();
    app.env.block.time = app.env.block.time.plus_seconds(300);

    let res = query(
        app.deps.as_ref(),
        app.env,
        QueryMsg::TwapPrice { interval: 210 },
    )
    .unwrap();
    let twap: Uint128 = from_binary(&res).unwrap();
    assert_eq!(twap, Uint128::from(8_099_999_998u128));
}

#[test]
fn test_zero_interval() {
    let app = setup();

    let res = query(
        app.deps.as_ref(),
        app.env.clone(),
        QueryMsg::TwapPrice { interval: 0 },
    )
    .unwrap();
    let twap: Uint128 = from_binary(&res).unwrap();

    let res = query(app.deps.as_ref(), app.env, QueryMsg::SpotPrice {}).unwrap();
    let spot: Uint128 = from_binary(&res).unwrap();
    assert_eq!(twap, spot);
}

#[test]
fn test_input_twap_get_twap_price() {
    let mut app = setup();
    // annoying second loop that I haven't tidied
    for i in 0..34 {
        if i % 3 == 0 {
            let swap_msg = ExecuteMsg::SwapInput {
                direction: Direction::RemoveFromAmm,
                quote_asset_amount: to_decimals(100),
            };

            let info = mock_info("addr0000", &[]);
            execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();
        } else {
            let swap_msg = ExecuteMsg::SwapInput {
                direction: Direction::AddToAmm,
                quote_asset_amount: to_decimals(50),
            };

            let info = mock_info("addr0000", &[]);
            execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();
        }

        app.env.block.time = app.env.block.time.plus_seconds(14);
        app.env.block.height += 1;
    }

    let res = query(
        app.deps.as_ref(),
        app.env.clone(),
        QueryMsg::InputTwap {
            direction: Direction::AddToAmm,
            amount: to_decimals(10u64),
        },
    )
    .unwrap();
    let twap: Uint128 = from_binary(&res).unwrap();
    assert_eq!(twap, Uint128::from(1_103_873_668u128));
}

#[test]
fn test_input_twap_if_snapshot_is_now_no_effect() {
    let mut app = setup();
    // annoying second loop that I haven't tidied
    for i in 0..34 {
        if i % 3 == 0 {
            let swap_msg = ExecuteMsg::SwapInput {
                direction: Direction::RemoveFromAmm,
                quote_asset_amount: to_decimals(100),
            };

            let info = mock_info("addr0000", &[]);
            execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();
        } else {
            let swap_msg = ExecuteMsg::SwapInput {
                direction: Direction::AddToAmm,
                quote_asset_amount: to_decimals(50),
            };

            let info = mock_info("addr0000", &[]);
            execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();
        }

        app.env.block.time = app.env.block.time.plus_seconds(14);
        app.env.block.height += 1;
    }

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals(100),
    };

    let info = mock_info("addr0000", &[]);
    execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();

    let res = query(
        app.deps.as_ref(),
        app.env.clone(),
        QueryMsg::InputTwap {
            direction: Direction::AddToAmm,
            amount: to_decimals(10u64),
        },
    )
    .unwrap();
    let twap: Uint128 = from_binary(&res).unwrap();
    assert_eq!(twap, Uint128::from(1_103_873_668u128));
}

#[test]
fn test_input_twap_snapshot_less_than_15_mins() {
    let app = setup();

    let res = query(
        app.deps.as_ref(),
        app.env.clone(),
        QueryMsg::InputTwap {
            direction: Direction::AddToAmm,
            amount: to_decimals(10u64),
        },
    )
    .unwrap();
    let twap: Uint128 = from_binary(&res).unwrap();
    assert_eq!(twap, Uint128::from(1_098_903_664u128));
}

#[test]
fn test_input_twap_input_zero() {
    let app = setup();

    let res = query(
        app.deps.as_ref(),
        app.env.clone(),
        QueryMsg::InputTwap {
            direction: Direction::AddToAmm,
            amount: to_decimals(0u64),
        },
    )
    .unwrap();
    let twap: Uint128 = from_binary(&res).unwrap();
    assert_eq!(twap, Uint128::zero());
}

#[test]
fn test_output_twap_get_twap_price() {
    let mut app = setup();
    // annoying second loop that I haven't tidied
    for i in 0..34 {
        if i % 3 == 0 {
            let swap_msg = ExecuteMsg::SwapInput {
                direction: Direction::RemoveFromAmm,
                quote_asset_amount: to_decimals(100),
            };

            let info = mock_info("addr0000", &[]);
            execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();
        } else {
            let swap_msg = ExecuteMsg::SwapInput {
                direction: Direction::AddToAmm,
                quote_asset_amount: to_decimals(50),
            };

            let info = mock_info("addr0000", &[]);
            execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();
        }

        app.env.block.time = app.env.block.time.plus_seconds(14);
        app.env.block.height += 1;
    }

    let res = query(
        app.deps.as_ref(),
        app.env.clone(),
        QueryMsg::OutputTwap {
            direction: Direction::AddToAmm,
            amount: to_decimals(10u64),
        },
    )
    .unwrap();
    let twap: Uint128 = from_binary(&res).unwrap();
    assert_eq!(twap, Uint128::from(82_456_099_249u128));
}

#[test]
fn test_output_twap_snapshot_less_than_15_mins() {
    let app = setup();

    let res = query(
        app.deps.as_ref(),
        app.env.clone(),
        QueryMsg::OutputTwap {
            direction: Direction::AddToAmm,
            amount: to_decimals(10u64),
        },
    )
    .unwrap();
    let twap: Uint128 = from_binary(&res).unwrap();
    assert_eq!(twap, Uint128::from(82_816_779_971u128));
}

#[test]
fn test_output_twap_input_zero() {
    let app = setup();

    let res = query(
        app.deps.as_ref(),
        app.env.clone(),
        QueryMsg::OutputTwap {
            direction: Direction::AddToAmm,
            amount: to_decimals(0u64),
        },
    )
    .unwrap();
    let twap: Uint128 = from_binary(&res).unwrap();
    assert_eq!(twap, Uint128::zero());
}
