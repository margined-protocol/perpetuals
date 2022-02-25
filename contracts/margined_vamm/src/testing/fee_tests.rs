use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
// use crate::testing::setup::to_decimals;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary};
use cosmwasm_bignumber::{Decimal256};
use margined_perp::margined_vamm::{CalcFeeResponse, ExecuteMsg, InstantiateMsg, QueryMsg};

#[test]
fn test_calc_fee() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: Decimal256::from_ratio(100u64, 1u64),
        base_asset_reserve: Decimal256::from_ratio(10_000u64, 1u64),
        funding_period: 3_600 as u64,
        toll_ratio: Decimal256::from_ratio(1u64, 10u64),   // 0.01
        spread_ratio: Decimal256::from_ratio(1u64, 10u64), // 0.01
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let amount = Decimal256::from_ratio(10u64, 1u64);

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
            toll_fee: Decimal256::from_ratio(1u64, 10u64),
            spread_fee: Decimal256::from_ratio(1u64, 10u64),
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
        quote_asset_reserve: Decimal256::from_ratio(100u64, 1u64),
        base_asset_reserve: Decimal256::from_ratio(10_000u64, 1u64),
        funding_period: 3_600 as u64,
        toll_ratio: Decimal256::from_ratio(1u64, 10u64),   // 0.01
        spread_ratio: Decimal256::from_ratio(1u64, 10u64), // 0.01
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Update the config
    let msg = ExecuteMsg::UpdateConfig {
        owner: None,
        toll_ratio: Some(Decimal256::from_ratio(1u64, 10u64)), // 0.1
        spread_ratio: Some(Decimal256::from_ratio(50u64, 1000u64)), // 0.01
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let amount = Decimal256::from_ratio(100u64, 1u64);

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
            toll_fee: Decimal256::from_ratio(10u64, 1u64),
            spread_fee: Decimal256::from_ratio(5u64, 1u64),
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
        quote_asset_reserve: Decimal256::from_ratio(100u64, 1u64),
        base_asset_reserve: Decimal256::from_ratio(10_000u64, 1u64),
        funding_period: 3_600 as u64,
        toll_ratio: Decimal256::zero(),
        spread_ratio: Decimal256::from_ratio(50u64, 1000u64), // 0.05
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let amount = Decimal256::from_ratio(100u64, 1u64);

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
            toll_fee: Decimal256::zero(),
            spread_fee: Decimal256::from_ratio(5u64, 1u64),
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
        quote_asset_reserve: Decimal256::from_ratio(100u64, 1u64),
        base_asset_reserve: Decimal256::from_ratio(10_000u64, 1u64),
        funding_period: 3_600 as u64,
        toll_ratio: Decimal256::from_ratio(50u64, 1000u64), // 0.05,
        spread_ratio: Decimal256::from_ratio(50u64, 1000u64), // 0.05
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let amount = Decimal256::zero();

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
            toll_fee: Decimal256::zero(),
            spread_fee: Decimal256::zero(),
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
        quote_asset_reserve: Decimal256::from_ratio(100u64, 1u64),
        base_asset_reserve: Decimal256::from_ratio(10_000u64, 1u64),
        funding_period: 3_600 as u64,
        toll_ratio: Decimal256::from_ratio(50u64, 1000u64), // 0.05,
        spread_ratio: Decimal256::from_ratio(50u64, 1000u64), // 0.05
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Update the config
    let msg = ExecuteMsg::UpdateConfig {
        owner: None,
        toll_ratio: Some(Decimal256::from_ratio(1u64, 10u64)), // 0.1
        spread_ratio: Some(Decimal256::from_ratio(50u64, 1000u64)), // 0.01
    };

    let info = mock_info("addr0001", &[]);
    let result = execute(deps.as_mut(), mock_env(), info, msg);
    match result {
        Err(ContractError::Unauthorized {}) => {}
        _ => panic!("DO NOT ENTER HERE"),
    }
}
