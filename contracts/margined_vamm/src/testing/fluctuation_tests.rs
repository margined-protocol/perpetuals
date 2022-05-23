use crate::contract::{execute, instantiate};
use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{Env, OwnedDeps, Uint128};
use margined_perp::margined_vamm::{Direction, ExecuteMsg, InstantiateMsg};
use margined_utils::scenarios::{parse_event, to_decimals};

pub struct TestingEnv {
    pub deps: OwnedDeps<MockStorage, MockApi, MockQuerier>,
    pub env: Env,
}

fn setup() -> TestingEnv {
    let mut env = mock_env();
    let mut deps = mock_dependencies();

    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1_000),
        base_asset_reserve: to_decimals(100),
        funding_period: 3_600_u64,
        toll_ratio: Uint128::from(10_000_000u128),   // 0.01
        spread_ratio: Uint128::from(10_000_000u128), // 0.01
        fluctuation_limit_ratio: Uint128::from(50_000_000u128), // 0.05
        margin_engine: Some("addr0000".to_string()),
        pricefeed: "oracle".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    let msg = ExecuteMsg::SetOpen { open: true };
    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    env.block.time = env.block.time.plus_seconds(14);
    env.block.height += 1;

    TestingEnv { deps, env }
}

#[test]
fn test_swap_input_price_goes_up_within_fluctuation_limit() {
    let mut app = setup();

    // fluctuation is 5%, price is between 9.5 ~ 10.5
    // BUY 24, reserve will be 1024 : 97.66, price is 1024 / 97.66 = 10.49
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(24),
        base_asset_limit: to_decimals(0u64),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    let result = execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();
    assert_eq!(parse_event(&result, "action"), "swap_input");
}

#[test]
fn test_swap_input_price_goes_down_within_fluctuation_limit() {
    let mut app = setup();

    // fluctuation is 5%, price is between 9.5 ~ 10.5
    // SELL 25, reserve will be 975 : 102.56, price is 975 / 102.56 = 9.51
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals(25),
        base_asset_limit: to_decimals(0u64),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    let result = execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();
    assert_eq!(parse_event(&result, "action"), "swap_input");
}

#[test]
fn test_swap_input_price_goes_down_then_up_and_down_within_fluctuation_limit() {
    let mut app = setup();

    // fluctuation is 5%, price is between 9.5 ~ 10.5
    // SELL 25, reserve will be 975 : 102.56, price is 975 / 102.56 = 9.51
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals(25),
        base_asset_limit: to_decimals(0u64),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    let _res = execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();

    // BUY 49, reserve will be 1024 : 97.66, price is 1024 / 97.66 = 10.49
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(49),
        base_asset_limit: to_decimals(0u64),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    let _res = execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();

    // SELL 49, reserve will be 975 : 102.56, price is 975 / 102.56 = 9.51
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals(49),
        base_asset_limit: to_decimals(0u64),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    let result = execute(app.deps.as_mut(), app.env, info, swap_msg).unwrap();
    assert_eq!(parse_event(&result, "action"), "swap_input");
}

#[test]
fn test_swap_input_price_goes_can_go_over_fluctuation_limit_once() {
    let mut app = setup();

    // fluctuation is 5%, price is between 9.5 ~ 10.5
    // BUY 25, reserve will be 1025 : 97.56, price is 1025 / 97.56 = 10.50625
    // but _canOverFluctuationLimit is true so it's ok to skip the check
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(25),
        base_asset_limit: to_decimals(0u64),
        can_go_over_fluctuation: true,
    };

    let info = mock_info("addr0000", &[]);
    let result = execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();
    assert_eq!(parse_event(&result, "action"), "swap_input");
}

#[test]
fn test_swap_output_price_goes_up_within_fluctuation_limit() {
    let mut app = setup();

    // fluctuation is 5%, price is between 9.5 ~ 10.5
    // BUY 2.4 base, reserve will be 1024.6 : 97.6, price is 1024.6 / 97.6 = 10.5
    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::AddToAmm,
        base_asset_amount: Uint128::from(2_400_000u64),
        quote_asset_limit: Uint128::zero(),
    };

    let info = mock_info("addr0000", &[]);
    let result = execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();
    assert_eq!(parse_event(&result, "action"), "swap_output");
}

#[test]
fn test_swap_output_price_goes_down_within_fluctuation_limit() {
    let mut app = setup();

    // fluctuation is 5%, price is between 9.5 ~ 10.5
    // SELL 2.5 base, reserve will be 975.6 : 102.5, price is 975.6 / 102.5 = 9.52
    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::RemoveFromAmm,
        base_asset_amount: Uint128::from(2_500_000u64),
        quote_asset_limit: Uint128::zero(),
    };

    let info = mock_info("addr0000", &[]);
    let result = execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();
    assert_eq!(parse_event(&result, "action"), "swap_output");
}

#[test]
fn test_force_error_swap_input_price_down_over_limit() {
    let mut app = setup();

    // fluctuation is 5%, price is between 9.5 ~ 10.5
    // SELL 26, reserve will be 974 : 102.67, price is 974 / 102.67 = 9.49
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals(26),
        base_asset_limit: to_decimals(0u64),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    let result = execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: price is over fluctuation limit"
    );
}

#[test]
fn test_force_error_swap_input_can_go_over_limit_but_fails_second_time() {
    let mut app = setup();

    // fluctuation is 5%, price is between 9.5 ~ 10.5
    // BUY 25, reserve will be 1025 : 97.56, price is 1025 / 97.56 = 10.50625
    // _canOverFluctuationLimit is true so it's ok to skip the check the first time, while the rest cannot
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(25),
        base_asset_limit: to_decimals(0u64),
        can_go_over_fluctuation: true,
    };

    let info = mock_info("addr0000", &[]);
    let _result = execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(1),
        base_asset_limit: to_decimals(0u64),
        can_go_over_fluctuation: true,
    };

    let info = mock_info("addr0000", &[]);
    let result = execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: price is already over fluctuation limit"
    );
}

#[test]
fn test_force_error_swap_input_short_can_go_over_limit_but_fails_second_time() {
    let mut app = setup();

    // fluctuation is 5%, price is between 9.5 ~ 10.5
    // SELL 30, reserve will be 970 : 103.09, price is 975 / 102.56 = 9.40
    // _canOverFluctuationLimit is true so it's ok to skip the check the first time, while the rest cannot
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals(30),
        base_asset_limit: to_decimals(0u64),
        can_go_over_fluctuation: true,
    };

    let info = mock_info("addr0000", &[]);
    let _result = execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals(1),
        base_asset_limit: to_decimals(0u64),
        can_go_over_fluctuation: true,
    };

    let info = mock_info("addr0000", &[]);
    let result = execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: price is already over fluctuation limit"
    );
}

#[test]
fn test_force_error_swap_output_can_go_over_limit_but_fails_second_time() {
    let mut app = setup();

    // fluctuation is 5%, price is between 9.5 ~ 10.5
    // BUY 2.5 base, reserve will be 1025.6 : 97.5, price is 1025.6 / 97.5 = 10.52
    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::RemoveFromAmm,
        base_asset_amount: Uint128::from(25_000_000_000u64),
        quote_asset_limit: Uint128::zero(),
    };

    let info = mock_info("addr0000", &[]);
    execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();

    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::RemoveFromAmm,
        base_asset_amount: Uint128::from(100_000_000u64),
        quote_asset_limit: Uint128::zero(),
    };

    let info = mock_info("addr0000", &[]);
    let result = execute(app.deps.as_mut(), app.env, info, swap_msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: price is already over fluctuation limit"
    );
}

#[test]
fn test_force_error_swap_output_short_can_go_over_limit_but_fails_second_time() {
    let mut app = setup();

    // fluctuation is 5%, price is between 9.5 ~ 10.5
    // SELL 3 base, reserve will be 970.873 : 103, price is 970.873 / 103 = 9.425
    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::AddToAmm,
        base_asset_amount: Uint128::from(3_000_000_000u64),
        quote_asset_limit: Uint128::zero(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();

    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::AddToAmm,
        base_asset_amount: Uint128::from(3_000_000_000u64),
        quote_asset_limit: Uint128::zero(),
    };

    let info = mock_info("addr0000", &[]);
    let result = execute(app.deps.as_mut(), app.env, info, swap_msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: price is already over fluctuation limit"
    );
}

#[test]
fn test_force_error_swap_output_short_can_go_over_limit_but_fails_larger_price() {
    let mut app = setup();

    // fluctuation is 5%, price is between 9.5 ~ 10.5
    // SELL 3 base, reserve will be 970.873 : 103, price is 970.873 / 103 = 9.425
    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::AddToAmm,
        base_asset_amount: Uint128::from(3_000_000_000u64),
        quote_asset_limit: Uint128::zero(),
    };

    let info = mock_info("addr0000", &[]);
    let _result = execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();

    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::AddToAmm,
        base_asset_amount: Uint128::from(3_000_000_000u64),
        quote_asset_limit: Uint128::zero(),
    };

    let info = mock_info("addr0000", &[]);
    let result = execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: price is already over fluctuation limit"
    );
}

#[test]
fn test_force_error_swap_many_times() {
    let mut app = setup();

    // move forward 1 block
    app.env.block.time = app.env.block.time.plus_seconds(14);
    app.env.block.height += 1;

    // fluctuation is 5%, price is between 9.5 ~ 10.5
    // BUY 10+10+10, reserve will be 1030 : 97.09, price is 1030 / 97.09 = 10.61
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(10),
        base_asset_limit: to_decimals(0u64),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    let _res = execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(10),
        base_asset_limit: to_decimals(0u64),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    let _res = execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(10),
        base_asset_limit: to_decimals(0u64),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    execute(app.deps.as_mut(), app.env, info, swap_msg).unwrap_err();
}

#[test]
fn test_force_error_compare_price_fluctuation_with_previous_blocks() {
    let mut app = setup();

    // BUY 10, reserve will be 1010 : 99.01, price is 1010 / 99.01 = 10.2
    // fluctuation is 5%, price is between 9.69 ~ 10.71
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(10),
        base_asset_limit: to_decimals(0u64),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    let _res = execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();

    // move forward 1 block
    app.env.block.time = app.env.block.time.plus_seconds(14);
    app.env.block.height += 1;

    // SELL 26, reserve will be 984 : 101.63, price is 984 / 101.63 = 9.68
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals(26),
        base_asset_limit: to_decimals(0u64),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    let result = execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: price is over fluctuation limit"
    );

    // BUY 30, reserve will be 1040 : 96.15, price is 1040 / 96.15 = 10.82
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(30),
        base_asset_limit: to_decimals(0u64),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    let result = execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: price is over fluctuation limit"
    );

    // should revert as well if BUY 30 separately
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(10),
        base_asset_limit: to_decimals(0u64),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    let _res = execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(20),
        base_asset_limit: to_decimals(0u64),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    let result = execute(app.deps.as_mut(), app.env, info, swap_msg).unwrap_err();

    assert_eq!(
        result.to_string(),
        "Generic error: price is over fluctuation limit"
    );
}

#[test]
fn test_force_error_value_of_fluctuation_is_same_even_no_trading_for_multiple_blocks() {
    let mut app = setup();

    // BUY 10, reserve will be 1010 : 99.01, price is 1010 / 99.01 = 10.2
    // fluctuation is 5%, price is between 9.69 ~ 10.71
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(10),
        base_asset_limit: to_decimals(0u64),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    let _res = execute(app.deps.as_mut(), app.env.clone(), info, swap_msg).unwrap();

    // move forward 3 blocks
    app.env.block.time = app.env.block.time.plus_seconds(42);
    app.env.block.height += 3;

    // BUY 25, reserve will be 1035 : 96.62, price is 1035 / 96.62 = 10.712
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(25),
        base_asset_limit: to_decimals(0u64),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    let result = execute(app.deps.as_mut(), app.env, info, swap_msg).unwrap_err();

    assert_eq!(
        result.to_string(),
        "Generic error: price is over fluctuation limit"
    );
}
