use crate::contract::{instantiate, execute, query};
// use crate::error::ContractError;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{Addr, from_binary, Uint128};
use margined_perp::margined_vamm::{
    ConfigResponse,
    ExecuteMsg,
    InstantiateMsg,
    QueryMsg,
    StateResponse,
    Direction,
};

// fn mock_env_with_block_time(time: u64) -> Env {
//     let env = mock_env();
//     // register time
//     Env {
//         block: BlockInfo {
//             height: 1,
//             time: Timestamp::from_seconds(time),
//             chain_id: "columbus".to_string(),
//         },
//         ..env
//     }
// }

#[test]
fn test_instantiation() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 10u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: Uint128::from(100u128),
        base_asset_reserve: Uint128::from(10_000u128),
        funding_period: 3_600 as u64,
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    let info = mock_info("addr0000", &[]);
    assert_eq!(
        config,
        ConfigResponse {
            owner: info.sender.clone(),
            quote_asset: "ETH".to_string(),
            base_asset: "USD".to_string(),
        }
    );

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: Uint128::from(100u128),
            base_asset_reserve: Uint128::from(10_000u128),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: Uint128::from(10_000_000_000u128),
        }
    );
}

#[test]
fn test_update_config() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 10u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: Uint128::from(100u128),
        base_asset_reserve: Uint128::from(10_000u128),
        funding_period: 3_600 as u64,
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Update the config
    let msg = ExecuteMsg::UpdateConfig {
        owner: "addr0001".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        config,
        ConfigResponse {
            owner: Addr::unchecked("addr0001".to_string()),
            quote_asset: "ETH".to_string(),
            base_asset: "USD".to_string(),
        }
    );
}

#[test]
fn test_swap_input_long() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 10u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: Uint128::from(1_000_000_000u128),
        base_asset_reserve: Uint128::from(100_000_000u128),
        funding_period: 3_600 as u64,
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::LONG,
        quote_asset_amount: Uint128::from(600_000_000u128),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: Uint128::from(1_600_000_000u128),
            base_asset_reserve: Uint128::from(62_500_000u128),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: Uint128::from(10_000_000_000u128),
        }
    );
}

#[test]
fn test_swap_input_short() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 10u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: Uint128::from(1_000_000_000u128),
        base_asset_reserve: Uint128::from(100_000_000u128),
        funding_period: 3_600 as u64,
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::SHORT,
        quote_asset_amount: Uint128::from(600_000_000u128),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: Uint128::from(400_000_000u128),
            base_asset_reserve: Uint128::from(250_000_000u128),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: Uint128::from(10_000_000_000u128),
        }
    );
}

#[test]
fn test_swap_output_short() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 10u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: Uint128::from(1_000_000_000u128),
        base_asset_reserve: Uint128::from(100_000_000u128),
        funding_period: 3_600 as u64,
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::LONG,
        base_asset_amount: Uint128::from(150_000_000u128),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: Uint128::from(400_000_000u128),
            base_asset_reserve: Uint128::from(250_000_000u128),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: Uint128::from(10_000_000_000u128),
        }
    );
}

#[test]
fn test_swap_output_long() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 10u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: Uint128::from(1_000_000_000u128),
        base_asset_reserve: Uint128::from(100_000_000u128),
        funding_period: 3_600 as u64,
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::SHORT,
        base_asset_amount: Uint128::from(50_000_000u128),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: Uint128::from(2_000_000_000u128),
            base_asset_reserve: Uint128::from(50_000_000u128),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: Uint128::from(10_000_000_000u128),
        }
    );
}

#[test]
fn test_swap_input_short_long() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 10u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: Uint128::from(1_000_000_000u128),
        base_asset_reserve: Uint128::from(100_000_000u128),
        funding_period: 3_600 as u64,
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::SHORT,
        quote_asset_amount: Uint128::from(480_000_000u128),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::LONG,
        quote_asset_amount: Uint128::from(960_000_000u128),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: Uint128::from(1_480_000_000u128),
            base_asset_reserve: Uint128::from(67_567_560u128),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: Uint128::from(10_000_000_000u128),
        }
    );
}

#[test]
fn test_swap_input_short_long_long() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 10u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: Uint128::from(1_000_000_000u128),
        base_asset_reserve: Uint128::from(100_000_000u128),
        funding_period: 3_600 as u64,
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::SHORT,
        quote_asset_amount: Uint128::from(200_000_000u128),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::LONG,
        quote_asset_amount: Uint128::from(100_000_000u128),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::LONG,
        quote_asset_amount: Uint128::from(200_000_000u128),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();


    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: Uint128::from(1_100_000_000u128),
            base_asset_reserve: Uint128::from(90_909_081u128),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: Uint128::from(10_000_000_000u128),
        }
    );
}

#[test]
fn test_swap_input_short_long_short() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 10u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: Uint128::from(1_000_000_000u128),
        base_asset_reserve: Uint128::from(100_000_000u128),
        funding_period: 3_600 as u64,
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::SHORT,
        quote_asset_amount: Uint128::from(200_000_000u128),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::LONG,
        quote_asset_amount: Uint128::from(450_000_000u128),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::SHORT,
        quote_asset_amount: Uint128::from(250_000_000u128),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();


    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: Uint128::from(1_000_000_000u128),
            base_asset_reserve: Uint128::from(100_000_000u128),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: Uint128::from(10_000_000_000u128),
        }
    );
}

#[test]
fn test_swap_output_short_long() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 10u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: Uint128::from(1_000_000_000u128),
        base_asset_reserve: Uint128::from(100_000_000u128),
        funding_period: 3_600 as u64,
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::SHORT,
        base_asset_amount: Uint128::from(10_000_000u128),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::LONG,
        base_asset_amount: Uint128::from(10_000_000u128),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: Uint128::from(999_999_900u128),
            base_asset_reserve: Uint128::from(100_000_000u128),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: Uint128::from(10_000_000_000u128),
        }
    );
}