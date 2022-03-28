use crate::contract::{execute, instantiate, query};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Addr, Uint128};
use margined_common::integer::Integer;
use margined_perp::margined_vamm::{
    ConfigResponse, Direction, ExecuteMsg, InstantiateMsg, QueryMsg, StateResponse,
};
use margined_utils::scenarios::{to_decimals, DECIMAL_MULTIPLIER};

#[test]
fn test_instantiation() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: Uint128::from(100u128),
        base_asset_reserve: Uint128::from(10_000u128),
        funding_period: 3_600_u64,
        toll_ratio: Uint128::zero(),
        spread_ratio: Uint128::zero(),
        fluctuation_limit_ratio: Uint128::zero(),
        margin_engine: Some("addr0000".to_string()),
        pricefeed: "oracle".to_string(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    let info = mock_info("addr0000", &[]);
    assert_eq!(
        config,
        ConfigResponse {
            owner: info.sender,
            quote_asset: "ETH".to_string(),
            base_asset: "USD".to_string(),
            toll_ratio: Uint128::zero(),
            spread_ratio: Uint128::zero(),
            fluctuation_limit_ratio: Uint128::zero(),
            decimals: DECIMAL_MULTIPLIER,
            margin_engine: Addr::unchecked("addr0000".to_string()),
            pricefeed: Addr::unchecked("oracle".to_string()),
            funding_period: 3_600u64,
        }
    );

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            open: false,
            quote_asset_reserve: Uint128::from(100u128),
            base_asset_reserve: Uint128::from(10_000u128),
            total_position_size: Integer::default(),
            funding_rate: Uint128::zero(),
            next_funding_time: 0u64,
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
        funding_period: 3_600_u64,
        toll_ratio: Uint128::zero(),
        spread_ratio: Uint128::zero(),
        fluctuation_limit_ratio: Uint128::zero(),
        pricefeed: "oracle".to_string(),
        margin_engine: Some("addr0000".to_string()),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Update the config
    let msg = ExecuteMsg::UpdateConfig {
        owner: None,
        toll_ratio: None,
        spread_ratio: None,
        fluctuation_limit_ratio: None,
        margin_engine: Some("addr0001".to_string()),
        pricefeed: None,
        spot_price_twap_interval: None,
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        config,
        ConfigResponse {
            owner: Addr::unchecked("addr0000".to_string()),
            quote_asset: "ETH".to_string(),
            base_asset: "USD".to_string(),
            toll_ratio: Uint128::zero(),
            spread_ratio: Uint128::zero(),
            fluctuation_limit_ratio: Uint128::zero(),
            decimals: DECIMAL_MULTIPLIER,
            margin_engine: Addr::unchecked("addr0001".to_string()),
            pricefeed: Addr::unchecked("oracle".to_string()),
            funding_period: 3_600u64,
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
        funding_period: 3_600_u64,
        toll_ratio: Uint128::zero(),
        spread_ratio: Uint128::zero(),
        fluctuation_limit_ratio: Uint128::zero(),
        pricefeed: "oracle".to_string(),
        margin_engine: Some("addr0000".to_string()),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // open amm
    let info = mock_info("addr0000", &[]);
    execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SetOpen { open: true },
    )
    .unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(600),
        base_asset_limit: Uint128::zero(),
        can_go_over_fluctuation: false,
    };
    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            open: true,
            quote_asset_reserve: to_decimals(1_600),
            base_asset_reserve: Uint128::from(62_500_000_000u128),
            total_position_size: Integer::new_positive(37_500_000_000u128),
            funding_rate: Uint128::zero(),
            next_funding_time: 1_571_801_019u64,
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
        funding_period: 3_600_u64,
        toll_ratio: Uint128::zero(),
        spread_ratio: Uint128::zero(),
        fluctuation_limit_ratio: Uint128::zero(),
        pricefeed: "oracle".to_string(),
        margin_engine: Some("addr0000".to_string()),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // open amm
    let info = mock_info("addr0000", &[]);
    execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SetOpen { open: true },
    )
    .unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals(600),
        base_asset_limit: Uint128::zero(),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            open: true,
            quote_asset_reserve: to_decimals(400),
            base_asset_reserve: to_decimals(250),
            total_position_size: Integer::new_negative(to_decimals(150)),
            funding_rate: Uint128::zero(),
            next_funding_time: 1_571_801_019u64,
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
        funding_period: 3_600_u64,
        toll_ratio: Uint128::zero(),
        spread_ratio: Uint128::zero(),
        fluctuation_limit_ratio: Uint128::zero(),
        pricefeed: "oracle".to_string(),
        margin_engine: Some("addr0000".to_string()),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // open amm
    let info = mock_info("addr0000", &[]);
    execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SetOpen { open: true },
    )
    .unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::AddToAmm,
        base_asset_amount: to_decimals(150),
        quote_asset_limit: Uint128::zero(),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            open: true,
            quote_asset_reserve: to_decimals(400),
            base_asset_reserve: to_decimals(250),
            total_position_size: Integer::new_negative(to_decimals(150)),
            funding_rate: Uint128::zero(),
            next_funding_time: 1_571_801_019u64,
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
        funding_period: 3_600_u64,
        toll_ratio: Uint128::zero(),
        spread_ratio: Uint128::zero(),
        fluctuation_limit_ratio: Uint128::zero(),
        pricefeed: "oracle".to_string(),
        margin_engine: Some("addr0000".to_string()),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // open amm
    let info = mock_info("addr0000", &[]);
    execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SetOpen { open: true },
    )
    .unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::RemoveFromAmm,
        base_asset_amount: to_decimals(50),
        quote_asset_limit: Uint128::zero(),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            open: true,
            quote_asset_reserve: to_decimals(2_000),
            base_asset_reserve: to_decimals(50),
            total_position_size: Integer::new_positive(to_decimals(50)),
            funding_rate: Uint128::zero(),
            next_funding_time: 1_571_801_019u64,
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
        funding_period: 3_600_u64,
        toll_ratio: Uint128::zero(),
        spread_ratio: Uint128::zero(),
        fluctuation_limit_ratio: Uint128::zero(),
        pricefeed: "oracle".to_string(),
        margin_engine: Some("addr0000".to_string()),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // open amm
    let info = mock_info("addr0000", &[]);
    execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SetOpen { open: true },
    )
    .unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals(480),
        base_asset_limit: Uint128::zero(),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            open: true,
            quote_asset_reserve: to_decimals(520),
            base_asset_reserve: Uint128::from(192_307_692_308u128),
            total_position_size: Integer::new_negative(92_307_692_308u128),
            funding_rate: Uint128::zero(),
            next_funding_time: 1_571_801_019u64,
        }
    );

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(960),
        base_asset_limit: Uint128::zero(),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            open: true,
            quote_asset_reserve: to_decimals(1_480),
            base_asset_reserve: Uint128::from(67_567_567_568u128),
            total_position_size: Integer::new_positive(32_432_432_432u128),
            funding_rate: Uint128::zero(),
            next_funding_time: 1_571_801_019u64,
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
        funding_period: 3_600_u64,
        toll_ratio: Uint128::zero(),
        spread_ratio: Uint128::zero(),
        fluctuation_limit_ratio: Uint128::zero(),
        pricefeed: "oracle".to_string(),
        margin_engine: Some("addr0000".to_string()),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // open amm
    let info = mock_info("addr0000", &[]);
    execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SetOpen { open: true },
    )
    .unwrap();

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals(200),
        base_asset_limit: Uint128::zero(),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            open: true,
            quote_asset_reserve: to_decimals(800),
            base_asset_reserve: to_decimals(125),
            total_position_size: Integer::new_negative(25_000_000_000u128),
            funding_rate: Uint128::zero(),
            next_funding_time: 1_571_801_019u64,
        }
    );

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(100),
        base_asset_limit: Uint128::zero(),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            open: true,
            quote_asset_reserve: to_decimals(900),
            base_asset_reserve: Uint128::from(111_111_111_112u128),
            total_position_size: Integer::new_negative(11_111_111_112u128),
            funding_rate: Uint128::zero(),
            next_funding_time: 1_571_801_019u64,
        }
    );

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(200),
        base_asset_limit: Uint128::zero(),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            open: true,
            quote_asset_reserve: to_decimals(1100),
            base_asset_reserve: Uint128::from(90_909_090_910u128),
            total_position_size: Integer::new_positive(90_909_090_90u128),
            funding_rate: Uint128::zero(),
            next_funding_time: 1_571_801_019u64,
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
        funding_period: 3_600_u64,
        toll_ratio: Uint128::zero(),
        spread_ratio: Uint128::zero(),
        fluctuation_limit_ratio: Uint128::zero(),
        pricefeed: "oracle".to_string(),
        margin_engine: Some("addr0000".to_string()),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // open amm
    let info = mock_info("addr0000", &[]);
    execute(
        deps.as_mut(),
        mock_env(),
        info,
        ExecuteMsg::SetOpen { open: true },
    )
    .unwrap();

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals(200),
        base_asset_limit: Uint128::zero(),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            open: true,
            quote_asset_reserve: to_decimals(800),
            base_asset_reserve: to_decimals(125),
            total_position_size: Integer::new_negative(25_000_000_000u128),
            funding_rate: Uint128::zero(),
            next_funding_time: 1_571_801_019u64,
        }
    );

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(450),
        base_asset_limit: Uint128::zero(),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            open: true,
            quote_asset_reserve: to_decimals(1250),
            base_asset_reserve: to_decimals(80),
            total_position_size: Integer::new_positive(20_000_000_000u128),
            funding_rate: Uint128::zero(),
            next_funding_time: 1_571_801_019u64,
        }
    );

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals(250),
        base_asset_limit: Uint128::zero(),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            open: true,
            quote_asset_reserve: to_decimals(1000),
            base_asset_reserve: to_decimals(100),
            total_position_size: Integer::default(),
            funding_rate: Uint128::zero(),
            next_funding_time: 1_571_801_019u64,
        }
    );
}
