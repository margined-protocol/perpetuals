use crate::contract::{execute, instantiate, query};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Uint128};
use margined_common::integer::Integer;
use margined_perp::margined_vamm::{
    Direction, ExecuteMsg, InstantiateMsg, QueryMsg, StateResponse,
};
use margined_utils::scenarios::to_decimals;

#[test]
fn test_use_getoutputprice_use_to_swapinput_long() {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1000),
        base_asset_reserve: to_decimals(100),
        funding_period: 3_600_u64,
        toll_ratio: Uint128::zero(),
        spread_ratio: Uint128::zero(),
        fluctuation_limit_ratio: Uint128::from(500_000_000u128), // 0.5
        pricefeed: "oracle".to_string(),
        margin_engine: Some("addr0000".to_string()),
        insurance_fund: Some("insurance_fund".to_string()),
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

    // when trader ask what's the requiredQuoteAsset if trader want to remove 10 baseAsset from amm
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::OutputAmount {
            direction: Direction::RemoveFromAmm,
            amount: to_decimals(10),
        },
    )
    .unwrap();
    let required_quote_asset: Uint128 = from_binary(&res).unwrap();

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: required_quote_asset,
        base_asset_limit: Uint128::zero(),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    let result = execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    assert_eq!(
        result
            .attributes
            .iter()
            .find(|&attr| attr.key == "quote_asset_amount")
            .unwrap()
            .value,
        required_quote_asset.to_string(),
    );
    assert_eq!(
        result
            .attributes
            .iter()
            .find(|&attr| attr.key == "base_asset_amount")
            .unwrap()
            .value,
        to_decimals(10u64).to_string(),
    );
    assert_eq!(
        result
            .attributes
            .iter()
            .find(|&attr| attr.key == "direction")
            .unwrap()
            .value,
        Direction::AddToAmm.to_string(),
    );
}

#[test]
fn test_use_getoutputprice_use_to_swapinput_short() {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1000),
        base_asset_reserve: to_decimals(100),
        funding_period: 3_600_u64,
        toll_ratio: Uint128::zero(),
        spread_ratio: Uint128::zero(),
        fluctuation_limit_ratio: Uint128::from(500_000_000u128), // 0.5
        pricefeed: "oracle".to_string(),
        margin_engine: Some("addr0000".to_string()),
        insurance_fund: Some("insurance_fund".to_string()),
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

    // when trader ask what's the requiredQuoteAsset if trader want to remove 10 baseAsset from amm
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::OutputAmount {
            direction: Direction::AddToAmm,
            amount: to_decimals(10),
        },
    )
    .unwrap();
    let required_quote_asset: Uint128 = from_binary(&res).unwrap();

    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: required_quote_asset,
        base_asset_limit: Uint128::zero(),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    let result = execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    assert_eq!(
        result
            .attributes
            .iter()
            .find(|&attr| attr.key == "quote_asset_amount")
            .unwrap()
            .value,
        required_quote_asset.to_string(),
    );
    assert_eq!(
        result
            .attributes
            .iter()
            .find(|&attr| attr.key == "base_asset_amount")
            .unwrap()
            .value,
        to_decimals(10u64).to_string(),
    );
    assert_eq!(
        result
            .attributes
            .iter()
            .find(|&attr| attr.key == "direction")
            .unwrap()
            .value,
        Direction::RemoveFromAmm.to_string(),
    );
}

#[test]
fn test_use_getinputprice_long_use_to_swapoutput() {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1000),
        base_asset_reserve: to_decimals(100),
        funding_period: 3_600_u64,
        toll_ratio: Uint128::zero(),
        spread_ratio: Uint128::zero(),
        fluctuation_limit_ratio: Uint128::from(500_000_000u128), // 0.5
        pricefeed: "oracle".to_string(),
        margin_engine: Some("addr0000".to_string()),
        insurance_fund: Some("insurance_fund".to_string()),
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

    // when trader ask what's the requiredQuoteAsset if trader want to remove 10 baseAsset from amm
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::InputAmount {
            direction: Direction::AddToAmm,
            amount: to_decimals(10),
        },
    )
    .unwrap();
    let received_base_asset: Uint128 = from_binary(&res).unwrap();

    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::RemoveFromAmm,
        base_asset_amount: received_base_asset,
        quote_asset_limit: Uint128::zero(),
    };

    let info = mock_info("addr0000", &[]);
    let result = execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    assert_eq!(
        result
            .attributes
            .iter()
            .find(|&attr| attr.key == "quote_asset_amount")
            .unwrap()
            .value,
        Uint128::from(9_999_999_991u128).to_string(),
    );
    assert_eq!(
        result
            .attributes
            .iter()
            .find(|&attr| attr.key == "base_asset_amount")
            .unwrap()
            .value,
        received_base_asset.to_string(),
    );
    assert_eq!(
        result
            .attributes
            .iter()
            .find(|&attr| attr.key == "direction")
            .unwrap()
            .value,
        Direction::RemoveFromAmm.to_string(),
    );
}

#[test]
fn test_use_getinputprice_short_use_to_swapoutput() {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1000),
        base_asset_reserve: to_decimals(100),
        funding_period: 3_600_u64,
        toll_ratio: Uint128::zero(),
        spread_ratio: Uint128::zero(),
        fluctuation_limit_ratio: Uint128::from(500_000_000u128), // 0.5
        pricefeed: "oracle".to_string(),
        margin_engine: Some("addr0000".to_string()),
        insurance_fund: Some("insurance_fund".to_string()),
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

    // when trader ask what's the requiredQuoteAsset if trader want to remove 10 baseAsset from amm
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::InputAmount {
            direction: Direction::RemoveFromAmm,
            amount: to_decimals(10),
        },
    )
    .unwrap();
    let received_base_asset: Uint128 = from_binary(&res).unwrap();

    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::AddToAmm,
        base_asset_amount: received_base_asset,
        quote_asset_limit: Uint128::zero(),
    };

    let info = mock_info("addr0000", &[]);
    let result = execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
    assert_eq!(
        result
            .attributes
            .iter()
            .find(|&attr| attr.key == "quote_asset_amount")
            .unwrap()
            .value,
        Uint128::from(10_000_000_008u128).to_string(),
    );
    assert_eq!(
        result
            .attributes
            .iter()
            .find(|&attr| attr.key == "base_asset_amount")
            .unwrap()
            .value,
        received_base_asset.to_string(),
    );
    assert_eq!(
        result
            .attributes
            .iter()
            .find(|&attr| attr.key == "direction")
            .unwrap()
            .value,
        Direction::AddToAmm.to_string(),
    );
}

#[test]
fn test_swap_input_twice_short_long() {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1000),
        base_asset_reserve: to_decimals(100),
        funding_period: 3_600_u64,
        toll_ratio: Uint128::zero(),
        spread_ratio: Uint128::zero(),
        fluctuation_limit_ratio: Uint128::zero(),
        pricefeed: "oracle".to_string(),
        margin_engine: Some("addr0000".to_string()),
        insurance_fund: Some("insurance_fund".to_string()),
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
        quote_asset_amount: to_decimals(10),
        base_asset_limit: Uint128::zero(),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::AddToAmm,
        quote_asset_amount: to_decimals(10),
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
            quote_asset_reserve: to_decimals(1_000),
            base_asset_reserve: Uint128::from(100_000_000_001u128),
            total_position_size: Integer::new_negative(1u128),
            funding_rate: Integer::zero(),
            next_funding_time: 1_571_801_019u64,
        }
    );
}

#[test]
fn test_swap_input_twice_long_short() {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1000),
        base_asset_reserve: to_decimals(100),
        funding_period: 3_600_u64,
        toll_ratio: Uint128::zero(),
        spread_ratio: Uint128::zero(),
        fluctuation_limit_ratio: Uint128::zero(),
        pricefeed: "oracle".to_string(),
        margin_engine: Some("addr0000".to_string()),
        insurance_fund: Some("insurance_fund".to_string()),
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
        quote_asset_amount: to_decimals(10),
        base_asset_limit: Uint128::zero(),
        can_go_over_fluctuation: false,
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapInput {
        direction: Direction::RemoveFromAmm,
        quote_asset_amount: to_decimals(10),
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
            quote_asset_reserve: to_decimals(1_000),
            base_asset_reserve: Uint128::from(100_000_000_001u128),
            total_position_size: Integer::new_negative(1u128),
            funding_rate: Integer::zero(),
            next_funding_time: 1_571_801_019u64,
        }
    );
}

#[test]
fn test_swap_output_twice_short_long() {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1000),
        base_asset_reserve: to_decimals(100),
        funding_period: 3_600_u64,
        toll_ratio: Uint128::zero(),
        spread_ratio: Uint128::zero(),
        fluctuation_limit_ratio: Uint128::zero(),
        pricefeed: "oracle".to_string(),
        margin_engine: Some("addr0000".to_string()),
        insurance_fund: Some("insurance_fund".to_string()),
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
        base_asset_amount: to_decimals(10),
        quote_asset_limit: Uint128::zero(),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    // Swap in USD
    let swap_msg = ExecuteMsg::SwapOutput {
        direction: Direction::AddToAmm,
        base_asset_amount: to_decimals(10),
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
            quote_asset_reserve: Uint128::from(1_000_000_000_001u128),
            base_asset_reserve: to_decimals(100),
            total_position_size: Integer::default(),
            funding_rate: Integer::zero(),
            next_funding_time: 1_571_801_019u64,
        }
    );
}

#[test]
fn test_swap_output_twice_long_short() {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(1000),
        base_asset_reserve: to_decimals(100),
        funding_period: 3_600_u64,
        toll_ratio: Uint128::zero(),
        spread_ratio: Uint128::zero(),
        fluctuation_limit_ratio: Uint128::zero(),
        pricefeed: "oracle".to_string(),
        margin_engine: Some("addr0000".to_string()),
        insurance_fund: Some("insurance_fund".to_string()),
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
        base_asset_amount: to_decimals(10),
        quote_asset_limit: Uint128::zero(),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();

    // Swap in USD
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
        state,
        StateResponse {
            open: true,
            quote_asset_reserve: Uint128::from(1_000_000_000_001u128),
            base_asset_reserve: to_decimals(100),
            total_position_size: Integer::default(),
            funding_rate: Integer::zero(),
            next_funding_time: 1_571_801_019u64,
        }
    );
}
