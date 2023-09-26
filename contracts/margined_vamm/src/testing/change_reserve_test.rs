use crate::contract::{execute, instantiate, query};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Uint128};
use margined_perp::margined_vamm::{
    ExecuteMsg, InstantiateMsg, QueryMsg, StateResponse,
};
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
    };
    let mut info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(state.quote_asset_reserve, to_decimals(100));
    assert_eq!(state.base_asset_reserve, to_decimals(10_000));

    let mut msg = ExecuteMsg::ChangeReserve {
        quote_asset_reserve: to_decimals(100),
        base_asset_reserve: to_decimals(20_000),
    };

    info = mock_info("addr0001", &[]);
    let result = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: unauthorized".to_string()
    );

    msg = ExecuteMsg::ChangeReserve {
        quote_asset_reserve: to_decimals(100),
        base_asset_reserve: to_decimals(0),
    };
    info = mock_info("addr0000", &[]);
    let result = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: Input must be non-zero".to_string()
    );

    msg = ExecuteMsg::ChangeReserve {
        quote_asset_reserve: to_decimals(0),
        base_asset_reserve: to_decimals(100),
    };
    info = mock_info("addr0000", &[]);
    let result = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: Input must be non-zero".to_string()
    );

    msg = ExecuteMsg::ChangeReserve {
        quote_asset_reserve: to_decimals(0),
        base_asset_reserve: to_decimals(0),
    };
    info = mock_info("addr0000", &[]);
    let result = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: Input must be non-zero".to_string()
    );

    msg = ExecuteMsg::ChangeReserve {
        quote_asset_reserve: to_decimals(123),
        base_asset_reserve: to_decimals(456),
    };
    info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();


    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(state.quote_asset_reserve, to_decimals(123));
    assert_eq!(state.base_asset_reserve, to_decimals(456));
}
