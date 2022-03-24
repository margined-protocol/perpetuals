use crate::contract::{execute, instantiate, query};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Uint128};
use margined_perp::margined_vamm::{CalcFeeResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use margined_utils::scenarios::to_decimals;
#[test]
fn test_calc_fee() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(100),
        base_asset_reserve: to_decimals(10_000),
        funding_period: 3_600_u64,
        toll_ratio: Uint128::from(10_000_000u128),   // 0.01
        spread_ratio: Uint128::from(10_000_000u128), // 0.01
        pricefeed: "oracle".to_string(),
        margin_engine: Some("addr0000".to_string()),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let amount = to_decimals(10);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::CalcFee {
            quote_asset_amount: amount,
        },
    )
    .unwrap();
    let state: CalcFeeResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        CalcFeeResponse {
            toll_fee: Uint128::from(100_000_000u128),
            spread_fee: Uint128::from(100_000_000u128),
        }
    );
}

#[test]
fn test_set_diff_fee_ratio() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(100),
        base_asset_reserve: to_decimals(10_000),
        funding_period: 3_600_u64,
        toll_ratio: Uint128::from(10_000_000u128),   // 0.01
        spread_ratio: Uint128::from(10_000_000u128), // 0.01
        pricefeed: "oracle".to_string(),
        margin_engine: Some("addr0000".to_string()),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Update the config
    let msg = ExecuteMsg::UpdateConfig {
        owner: None,
        toll_ratio: Some(Uint128::from(100_000_000u128)), // 0.1
        spread_ratio: Some(Uint128::from(50_000_000u128)), // 0.01
        margin_engine: None,
        pricefeed: None,
        spot_price_twap_interval: None,
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let amount = to_decimals(100);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::CalcFee {
            quote_asset_amount: amount,
        },
    )
    .unwrap();
    let state: CalcFeeResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        CalcFeeResponse {
            toll_fee: to_decimals(10),
            spread_fee: to_decimals(5),
        }
    );
}

#[test]
fn test_set_fee_ratio_zero() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(100),
        base_asset_reserve: to_decimals(10_000),
        funding_period: 3_600_u64,
        toll_ratio: Uint128::zero(),
        spread_ratio: Uint128::from(50_000_000u128), // 0.05
        pricefeed: "oracle".to_string(),
        margin_engine: Some("addr0000".to_string()),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let amount = to_decimals(100);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::CalcFee {
            quote_asset_amount: amount,
        },
    )
    .unwrap();
    let state: CalcFeeResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        CalcFeeResponse {
            toll_fee: to_decimals(0),
            spread_fee: to_decimals(5),
        }
    );
}

#[test]
fn test_calc_fee_input_zero() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(100),
        base_asset_reserve: to_decimals(10_000),
        funding_period: 3_600_u64,
        toll_ratio: Uint128::from(50_000_000u128), // 0.05,
        spread_ratio: Uint128::from(50_000_000u128), // 0.05
        pricefeed: "oracle".to_string(),
        margin_engine: Some("addr0000".to_string()),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let amount = to_decimals(0);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::CalcFee {
            quote_asset_amount: amount,
        },
    )
    .unwrap();
    let state: CalcFeeResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        CalcFeeResponse {
            toll_fee: to_decimals(0),
            spread_fee: to_decimals(0),
        }
    );
}

#[test]
fn test_update_not_owner() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(100),
        base_asset_reserve: to_decimals(10_000),
        funding_period: 3_600_u64,
        toll_ratio: Uint128::from(50_000_000u128), // 0.05,
        spread_ratio: Uint128::from(50_000_000u128), // 0.05
        pricefeed: "oracle".to_string(),
        margin_engine: Some("addr0000".to_string()),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Update the config
    let msg = ExecuteMsg::UpdateConfig {
        owner: None,
        toll_ratio: Some(Uint128::from(100_000_000u128)), // 0.1
        spread_ratio: Some(Uint128::from(50_000_000u128)), // 0.01
        margin_engine: None,
        pricefeed: None,
        spot_price_twap_interval: None,
    };

    let info = mock_info("addr0001", &[]);
    let result = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: unauthorized".to_string()
    );
}
