use crate::contract::{instantiate, execute, query};
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
use crate::testing::setup::{
    DECIMAL_MULTIPLIER, to_decimals,
};

#[test]
fn test_instantiation() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
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
            decimals: DECIMAL_MULTIPLIER,
        }
    );
}

#[test]
fn test_update_config() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
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
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1000),
        base_asset_reserve: to_decimals(100),
        funding_period: 3_600 as u64,
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(600),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals(1_600),
            base_asset_reserve: Uint128::from(62_500_000_000u128),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: DECIMAL_MULTIPLIER,
        }
    );
}

#[test]
fn test_swap_input_short() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1000),
        base_asset_reserve: to_decimals(100),
        funding_period: 3_600 as u64,
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals(600),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals(400),
            base_asset_reserve: to_decimals(250),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: DECIMAL_MULTIPLIER,
        }
    );
}

#[test]
fn test_swap_output_short() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1000),
        base_asset_reserve: to_decimals(100),
        funding_period: 3_600 as u64,
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::AddToAmm,
        base_asset_amount: to_decimals(150),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals(400),
            base_asset_reserve: to_decimals(250),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: DECIMAL_MULTIPLIER,
        }
    );
}

#[test]
fn test_swap_output_long() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1000),
        base_asset_reserve: to_decimals(100),
        funding_period: 3_600 as u64,
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::RemoveFromAmm,
        base_asset_amount: to_decimals(50),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals(2_000),
            base_asset_reserve: to_decimals(50),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: DECIMAL_MULTIPLIER,
        }
    );
}

#[test]
fn test_swap_input_short_long() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1000),
        base_asset_reserve: to_decimals(100),
        funding_period: 3_600 as u64,
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals(480),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals(520),
            base_asset_reserve: Uint128::from(192_307_692_308u128),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: DECIMAL_MULTIPLIER,
        }
    );

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(960),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals(1_480),
            base_asset_reserve: Uint128::from(67_567_567_568u128),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: DECIMAL_MULTIPLIER,
        }
    );
}

#[test]
fn test_swap_input_short_long_long() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1000),
        base_asset_reserve: to_decimals(100),
        funding_period: 3_600 as u64,
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals(200),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals(800),
            base_asset_reserve: to_decimals(125),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: DECIMAL_MULTIPLIER,
        }
    );

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(100),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals(900),
            base_asset_reserve: Uint128::from(111_111_111_112u128),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: DECIMAL_MULTIPLIER,
        }
    );

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(200),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();


    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals(1100),
            base_asset_reserve: Uint128::from(90_909_090_910u128),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: DECIMAL_MULTIPLIER,
        }
    );
}

#[test]
fn test_swap_input_short_long_short() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1000),
        base_asset_reserve: to_decimals(100),
        funding_period: 3_600 as u64,
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals(200),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals(800),
            base_asset_reserve: to_decimals(125),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: DECIMAL_MULTIPLIER,
        }
    );

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(450),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals(1250),
            base_asset_reserve: to_decimals(80),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: DECIMAL_MULTIPLIER,
        }
    );

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals(250),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();


    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals(1000),
            base_asset_reserve: to_decimals(100),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: DECIMAL_MULTIPLIER,
        }
    );
}

#[test]
fn test_swap_input_long_integration_example() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: Uint128::from(1_000_000_000_000u128),
        base_asset_reserve: Uint128::from(100_000_000_000u128),
        funding_period: 3_600 as u64,
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: Uint128::from(600_000_000_000u128), // this is swapping 60 at 10x leverage
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: Uint128::from(1_600_000_000_000u128),
            base_asset_reserve: Uint128::from(62_500_000_000u128),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: Uint128::from(1_000_000_000u128),
        }
    );
}

#[test]
fn test_swap_input_long_short_integration_example() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: Uint128::from(1_000_000_000_000u128),
        base_asset_reserve: Uint128::from(100_000_000_000u128),
        funding_period: 3_600 as u64,
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: Uint128::from(600_000_000_000u128), // this is swapping 60 at 10x leverage
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: Uint128::from(1_600_000_000_000u128),
            base_asset_reserve: Uint128::from(62_500_000_000u128),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: Uint128::from(1_000_000_000u128),
        }
    );

    // Swap in ETH
    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::AddToAmm,
        base_asset_amount: Uint128::from(37_500_000_000u128), // this is swapping 60 at 10x leverage
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: Uint128::from(1_000_000_000_000u128),
            base_asset_reserve: Uint128::from(100_000_000_000u128),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: Uint128::from(1_000_000_000u128),
        }
    );
}

#[test]
fn test_swap_input_twice_short_long() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1000),
        base_asset_reserve: to_decimals(100),
        funding_period: 3_600 as u64,
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals(10),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(10),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals(1_000),
            base_asset_reserve: Uint128::from(100_000_000_001u128),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: DECIMAL_MULTIPLIER,
        }
    );
}

#[test]
fn test_swap_input_twice_long_short() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1000),
        base_asset_reserve: to_decimals(100),
        funding_period: 3_600 as u64,
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(10),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals(10),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: to_decimals(1_000),
            base_asset_reserve: Uint128::from(100_000_000_001u128),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: DECIMAL_MULTIPLIER,
        }
    );
}

#[test]
fn test_swap_output_twice_short_long() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1000),
        base_asset_reserve: to_decimals(100),
        funding_period: 3_600 as u64,
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::RemoveFromAmm,
        base_asset_amount: to_decimals(10),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::AddToAmm,
        base_asset_amount: to_decimals(10),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: Uint128::from(1_000_000_000_001u128),
            base_asset_reserve: to_decimals(100),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: DECIMAL_MULTIPLIER,
        }
    );
}

#[test]
fn test_swap_output_twice_long_short() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1000),
        base_asset_reserve: to_decimals(100),
        funding_period: 3_600 as u64,
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::AddToAmm,
        base_asset_amount: to_decimals(10),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::RemoveFromAmm,
        base_asset_amount: to_decimals(10),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: Uint128::from(1_000_000_000_001u128),
            base_asset_reserve: to_decimals(100),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: DECIMAL_MULTIPLIER,
        }
    );
}