use crate::{
    contract::{execute, instantiate, query},
    state::PriceData,
};
use cosmwasm_std::{from_binary, Uint128};
use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info},
    Timestamp,
};
use margined_perp::margined_pricefeed::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};

#[test]
fn test_instantiation() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        oracle_hub_contract: "oracle_hub0000".to_string(),
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
            decimals: Uint128::from(1_000_000_000 as u128),
        }
    );
}

// #[test]
// fn test_update_config() {
//     let mut deps = mock_dependencies(&[]);
//     let msg = InstantiateMsg {
//         decimals: 9u8,
//         quote_asset: "ETH".to_string(),
//         base_asset: "USD".to_string(),
//         quote_asset_reserve: Uint128::from(100u128),
//         base_asset_reserve: Uint128::from(10_000u128),
//         funding_period: 3_600 as u64,
//         toll_ratio: Uint128::zero(),
//         spread_ratio: Uint128::zero(),
//     };
//     let info = mock_info("addr0000", &[]);
//     instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

//     // Update the config
//     let msg = ExecuteMsg::UpdateConfig {
//         owner: Some("addr0001".to_string()),
//         toll_ratio: None,
//         spread_ratio: None,
//     };

//     let info = mock_info("addr0000", &[]);
//     execute(deps.as_mut(), mock_env(), info, msg).unwrap();

//     let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
//     let config: ConfigResponse = from_binary(&res).unwrap();
//     assert_eq!(
//         config,
//         ConfigResponse {
//             owner: Addr::unchecked("addr0001".to_string()),
//             quote_asset: "ETH".to_string(),
//             base_asset: "USD".to_string(),
//             toll_ratio: Uint128::zero(),
//             spread_ratio: Uint128::zero(),
//             decimals: DECIMAL_MULTIPLIER,
//         }
//     );
// }

#[test]
fn test_set_and_get_price() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        oracle_hub_contract: "oracle_hub0000".to_string(),
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
            decimals: Uint128::from(1_000_000_000 as u128),
        }
    );

    // Set some market data
    let msg = ExecuteMsg::AppendPrice {
        key: "ETHUSD".to_string(),
        price: Uint128::from(500_000_000u128), // 0.5 I think
        timestamp: 1_000_000_000,              // 0.5 I think
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetPrice {
            key: "ETHUSD".to_string(),
        },
    )
    .unwrap();
    let price: PriceData = from_binary(&res).unwrap();
    assert_eq!(
        price,
        PriceData {
            round_id: Uint128::from(1u64),
            price: Uint128::from(500_000_000u128),
            timestamp: Timestamp::from_seconds(1_000_000_000),
        }
    );

    // Set some market data
    let msg = ExecuteMsg::AppendPrice {
        key: "ETHUSD".to_string(),
        price: Uint128::from(600_000_000u128), // 0.5 I think
        timestamp: 1_000_000_001,              // 0.5 I think
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetPrice {
            key: "ETHUSD".to_string(),
        },
    )
    .unwrap();
    let price: PriceData = from_binary(&res).unwrap();
    assert_eq!(
        price,
        PriceData {
            round_id: Uint128::from(2u64),
            price: Uint128::from(600_000_000u128),
            timestamp: Timestamp::from_seconds(1_000_000_001),
        }
    );
}

#[test]
fn test_set_multiple_price() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        oracle_hub_contract: "oracle_hub0000".to_string(),
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
            decimals: Uint128::from(1_000_000_000 as u128),
        }
    );

    let prices = vec![
        Uint128::from(500_000_000u128),
        Uint128::from(600_000_000u128),
        Uint128::from(700_000_000u128),
    ];

    let timestamps = vec![1_000_000_000, 1_000_000_001, 1_000_000_002];

    // Set some market data
    let msg = ExecuteMsg::AppendMultiplePrice {
        key: "ETHUSD".to_string(),
        prices,
        timestamps,
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetPrice {
            key: "ETHUSD".to_string(),
        },
    )
    .unwrap();
    let price: PriceData = from_binary(&res).unwrap();
    assert_eq!(
        price,
        PriceData {
            round_id: Uint128::from(3u64),
            price: Uint128::from(700_000_000u128),
            timestamp: Timestamp::from_seconds(1_000_000_002),
        }
    );
}

#[test]
fn test_get_previous_price() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 9u8,
        oracle_hub_contract: "oracle_hub0000".to_string(),
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
            decimals: Uint128::from(1_000_000_000 as u128),
        }
    );

    let prices = vec![
        Uint128::from(500_000_000u128),
        Uint128::from(600_000_000u128),
        Uint128::from(700_000_000u128),
        Uint128::from(600_000_000u128),
        Uint128::from(670_000_000u128),
        Uint128::from(710_000_000u128),
    ];

    let timestamps = vec![
        1_000_000_000,
        1_000_000_001,
        1_000_000_002,
        1_000_000_003,
        1_000_000_004,
        1_000_000_005,
    ];

    // Set some market data
    let msg = ExecuteMsg::AppendMultiplePrice {
        key: "ETHUSD".to_string(),
        prices,
        timestamps,
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetPreviousPrice {
            key: "ETHUSD".to_string(),
            num_round_back: Uint128::from(3u128),
        },
    )
    .unwrap();

    let price: PriceData = from_binary(&res).unwrap();
    assert_eq!(
        price,
        PriceData {
            round_id: Uint128::from(3u64),
            price: Uint128::from(700_000_000u128),
            timestamp: Timestamp::from_seconds(1_000_000_002),
        }
    );

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetPreviousPrice {
            key: "ETHUSD".to_string(),
            num_round_back: Uint128::from(7u128),
        },
    );
    assert!(res.is_err());
}
