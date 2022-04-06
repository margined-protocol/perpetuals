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
            base_asset_holding_cap: Uint128::zero(),
            open_interest_notional_cap: Uint128::zero(),
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
        base_asset_holding_cap: None,
        open_interest_notional_cap: None,
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
            base_asset_holding_cap: Uint128::zero(),
            open_interest_notional_cap: Uint128::zero(),
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
fn test_update_config_fail() {
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
        base_asset_holding_cap: None,
        open_interest_notional_cap: None,
        toll_ratio: None,
        spread_ratio: None,
        fluctuation_limit_ratio: Some(Uint128::MAX),
        margin_engine: None,
        pricefeed: None,
        spot_price_twap_interval: None,
    };

    let info = mock_info("addr0000", &[]);
    let result = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: invalid ratio".to_string()
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

#[test]
fn test_swap_output_short_and_indivisable() {
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

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::OutputPrice {
            direction: Direction::AddToAmm,
            amount: to_decimals(5),
        },
    )
    .unwrap();
    let amount: Uint128 = from_binary(&res).unwrap();

    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::AddToAmm,
        base_asset_amount: to_decimals(5),
        quote_asset_limit: Uint128::zero(),
    };
    let info = mock_info("addr0000", &[]);
    let result = execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    assert_eq!(
        result
            .attributes
            .iter()
            .find(|&attr| attr.key == "action")
            .unwrap()
            .value,
        "swap_output"
    );
    assert_eq!(
        result
            .attributes
            .iter()
            .find(|&attr| attr.key == "quote_asset_amount")
            .unwrap()
            .value,
        amount.to_string()
    );
    assert_eq!(
        result
            .attributes
            .iter()
            .find(|&attr| attr.key == "base_asset_amount")
            .unwrap()
            .value,
        to_decimals(5u64).to_string()
    );
}

#[test]
fn test_swap_output_long_and_indivisable() {
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

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::OutputPrice {
            direction: Direction::RemoveFromAmm,
            amount: to_decimals(5),
        },
    )
    .unwrap();
    let amount: Uint128 = from_binary(&res).unwrap();

    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::RemoveFromAmm,
        base_asset_amount: to_decimals(5),
        quote_asset_limit: Uint128::zero(),
    };
    let info = mock_info("addr0000", &[]);
    let result = execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    assert_eq!(
        result
            .attributes
            .iter()
            .find(|&attr| attr.key == "action")
            .unwrap()
            .value,
        "swap_output"
    );
    assert_eq!(
        result
            .attributes
            .iter()
            .find(|&attr| attr.key == "quote_asset_amount")
            .unwrap()
            .value,
        amount.to_string()
    );
    assert_eq!(
        result
            .attributes
            .iter()
            .find(|&attr| attr.key == "base_asset_amount")
            .unwrap()
            .value,
        to_decimals(5u64).to_string()
    );
}

#[test]
fn test_swap_output_long_short_same_size_should_get_diff_base_asset_amount() {
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

    // quote asset = (1000 * 100 / (100 - 10)) - 1000 = 111.111...2
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::OutputPrice {
            direction: Direction::RemoveFromAmm,
            amount: to_decimals(10),
        },
    )
    .unwrap();
    let amount1: Uint128 = from_binary(&res).unwrap();

    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::RemoveFromAmm,
        base_asset_amount: to_decimals(10),
        quote_asset_limit: Uint128::zero(),
    };
    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state.quote_asset_reserve,
        Uint128::from(1_111_111_111_112u128)
    );
    assert_eq!(state.base_asset_reserve, to_decimals(90));

    // quote asset = 1111.111 - (111.111 * 90 / (90 + 10)) = 111.11...1
    // price will be 1 wei less after traded
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::OutputPrice {
            direction: Direction::AddToAmm,
            amount: to_decimals(10),
        },
    )
    .unwrap();
    let amount2: Uint128 = from_binary(&res).unwrap();
    assert_eq!(amount1, amount2 + Uint128::from(1u64));
}

#[test]
fn test_force_error_swapinput_long_but_less_than_min_base_amount() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1250),
        base_asset_reserve: to_decimals(80),
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

    // long 600 should get 37.5 base asset, and reserves will be 1600:62.5
    // but someone front run it, long 200 before the order 600/37.5
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(600),
        base_asset_limit: Uint128::from(37_500_000_000u128),
        can_go_over_fluctuation: false,
    };
    let info = mock_info("addr0000", &[]);
    let result = execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: Less than minimum base asset amount limit"
    );
}

#[test]
fn test_force_error_swapinput_short_but_more_than_min_base_amount() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(800),
        base_asset_reserve: to_decimals(125),
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

    // long 600 should get 37.5 base asset, and reserves will be 1600:62.5
    // but someone front run it, long 200 before the order 600/37.5
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals(600),
        base_asset_limit: to_decimals(150),
        can_go_over_fluctuation: false,
    };
    let info = mock_info("addr0000", &[]);
    let result = execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: Greater than maximum base asset amount limit"
    );
}

#[test]
fn test_swapoutput_short_slippage_limit() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1250),
        base_asset_reserve: to_decimals(80),
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

    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::AddToAmm,
        base_asset_amount: to_decimals(20),
        quote_asset_limit: to_decimals(100),
    };
    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(state.quote_asset_reserve, to_decimals(1000));
    assert_eq!(state.base_asset_reserve, to_decimals(100));
}

#[test]
fn test_swapoutput_short_at_slippage_limit() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1250),
        base_asset_reserve: to_decimals(80),
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

    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::AddToAmm,
        base_asset_amount: to_decimals(20),
        quote_asset_limit: to_decimals(249),
    };
    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(state.quote_asset_reserve, to_decimals(1000));
    assert_eq!(state.base_asset_reserve, to_decimals(100));
}

#[test]
fn test_swapoutput_short_force_error_min_quote_251() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1250),
        base_asset_reserve: to_decimals(80),
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

    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::AddToAmm,
        base_asset_amount: to_decimals(20),
        quote_asset_limit: to_decimals(400),
    };
    let info = mock_info("addr0000", &[]);
    let result = execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: Less than minimum quote asset amount limit"
    );
}

#[test]
fn test_swapoutput_short_force_error_min_quote_400() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1250),
        base_asset_reserve: to_decimals(80),
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

    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::AddToAmm,
        base_asset_amount: to_decimals(20),
        quote_asset_limit: to_decimals(400),
    };
    let info = mock_info("addr0000", &[]);
    let result = execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: Less than minimum quote asset amount limit"
    );
}

#[test]
fn test_swapoutput_long_slippage_limit() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(800),
        base_asset_reserve: to_decimals(125),
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

    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::RemoveFromAmm,
        base_asset_amount: to_decimals(25),
        quote_asset_limit: to_decimals(400),
    };
    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(state.quote_asset_reserve, to_decimals(1000));
    assert_eq!(state.base_asset_reserve, to_decimals(100));
}

#[test]
fn test_swapoutput_long_at_slippage_limit() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(800),
        base_asset_reserve: to_decimals(125),
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

    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::RemoveFromAmm,
        base_asset_amount: to_decimals(25),
        quote_asset_limit: to_decimals(201),
    };
    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(state.quote_asset_reserve, to_decimals(1000));
    assert_eq!(state.base_asset_reserve, to_decimals(100));
}

#[test]
fn test_swapoutput_long_force_error_min_quote_199() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(800),
        base_asset_reserve: to_decimals(125),
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

    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::RemoveFromAmm,
        base_asset_amount: to_decimals(25),
        quote_asset_limit: to_decimals(199),
    };
    let info = mock_info("addr0000", &[]);
    let result = execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: Greater than maximum quote asset amount limit"
    );
}

#[test]
fn test_swapoutput_long_force_error_min_quote_100() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH/USD".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(800),
        base_asset_reserve: to_decimals(125),
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

    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::RemoveFromAmm,
        base_asset_amount: to_decimals(25),
        quote_asset_limit: to_decimals(100),
    };

    let info = mock_info("addr0000", &[]);
    let result = execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: Greater than maximum quote asset amount limit"
    );
}
