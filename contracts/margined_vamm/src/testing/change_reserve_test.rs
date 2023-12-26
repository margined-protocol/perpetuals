use crate::contract::{execute, instantiate, query};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Uint128};
use margined_perp::margined_vamm::{ExecuteMsg, InstantiateMsg, QueryMsg, StateResponse};
use margined_utils::testing::to_decimals;

#[test]
fn test_change_reserve() {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        decimals: 9u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: to_decimals(100),
        base_asset_reserve: to_decimals(10_000),
        funding_period: 3_600_u64,
        toll_ratio: Uint128::zero(),
        spread_ratio: Uint128::zero(),
        fluctuation_limit_ratio: Uint128::zero(),
        margin_engine: Some("addr0000".to_string()),
        insurance_fund: Some("insurance_fund".to_string()),
        pricefeed: "oracle".to_string(),
        initial_margin_ratio: to_decimals(1),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(state.quote_asset_reserve, to_decimals(100));
    assert_eq!(state.base_asset_reserve, to_decimals(10_000));

    let msg = ExecuteMsg::MigrateLiquidity {
        fluctuation_limit_ratio: None,
        liquidity_multiplier: 500_000_000u128.into(),
    };

    let info = mock_info("addr0000", &[]);
    let _ = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(state.quote_asset_reserve, to_decimals(50));
    assert_eq!(state.base_asset_reserve, to_decimals(5_000));

    let msg = ExecuteMsg::MigrateLiquidity {
        fluctuation_limit_ratio: None,
        liquidity_multiplier: 2_000_000_000u128.into(),
    };
    let info = mock_info("addr0000", &[]);
    let _ = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(state.quote_asset_reserve, to_decimals(100));
    assert_eq!(state.base_asset_reserve, to_decimals(10_000));
}
