use crate::{
    contract::{execute, instantiate, query},
    state::PriceData,
};
use cosmwasm_std::{from_binary, Addr, Uint128};
use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info},
    Timestamp,
};
use margined_perp::margined_pricefeed::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};

#[test]
fn test_instantiation() {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        decimals: 6u8,
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
            owner: info.sender,
            decimals: Uint128::from(1_000_000_u128),
        }
    );
}

#[test]
fn test_update_config() {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        decimals: 6u8,
        oracle_hub_contract: "oracle_hub0000".to_string(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Update the config
    let msg = ExecuteMsg::UpdateConfig {
        owner: Some("addr0001".to_string()),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        config,
        ConfigResponse {
            owner: Addr::unchecked("addr0001".to_string()),
            decimals: Uint128::from(1_000_000u128),
        }
    );
}

#[test]
fn test_set_and_get_price() {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        decimals: 6u8,
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
            owner: info.sender,
            decimals: Uint128::from(1_000_000u128),
        }
    );

    // Set some market data
    let msg = ExecuteMsg::AppendPrice {
        key: "ETHUSD".to_string(),
        price: Uint128::from(500_000_000u128), // 0.5 I think
        timestamp: 1_000_000,                  // 0.5 I think
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
            timestamp: Timestamp::from_seconds(1_000_000),
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
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        decimals: 6u8,
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
            owner: info.sender,
            decimals: Uint128::from(1_000_000_u128),
        }
    );

    let prices = vec![
        Uint128::from(500_000_000u128),
        Uint128::from(600_000_000u128),
        Uint128::from(700_000_000u128),
    ];

    let timestamps = vec![1_000_000, 1_000_000_001, 1_000_000_002];

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
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        decimals: 6u8,
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
            owner: info.sender,
            decimals: Uint128::from(1_000_000_u128),
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
        1_000_000,
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

#[test]
fn test_get_twap_price() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let msg = InstantiateMsg {
        decimals: 6u8,
        oracle_hub_contract: "oracle_hub0000".to_string(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    let res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    let info = mock_info("addr0000", &[]);
    assert_eq!(
        config,
        ConfigResponse {
            owner: info.sender,
            decimals: Uint128::from(1_000_000u128),
        }
    );

    let prices = vec![
        Uint128::from(400_000_000u128),
        Uint128::from(405_000_000u128),
        Uint128::from(410_000_000u128),
    ];

    let timestamps: Vec<u64> = vec![
        env.block.time.seconds(),
        env.block.time.seconds() + 15,
        env.block.time.seconds() + 30,
    ];

    // Set some market data
    let msg = ExecuteMsg::AppendMultiplePrice {
        key: "ETHUSD".to_string(),
        prices,
        timestamps,
    };

    env.block.time = env.block.time.plus_seconds(45);

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // twap Price
    let res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::GetTwapPrice {
            key: "ETHUSD".to_string(),
            interval: 45,
        },
    )
    .unwrap();

    let twap: Uint128 = from_binary(&res).unwrap();
    assert_eq!(twap, Uint128::from(405_000_000u128));

    // asking interval more than aggregator has
    let res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::GetTwapPrice {
            key: "ETHUSD".to_string(),
            interval: 46,
        },
    )
    .unwrap();

    let twap: Uint128 = from_binary(&res).unwrap();
    assert_eq!(twap, Uint128::from(405_000_000u128));

    // asking interval less than aggregator has
    let res = query(
        deps.as_ref(),
        env,
        QueryMsg::GetTwapPrice {
            key: "ETHUSD".to_string(),
            interval: 44,
        },
    )
    .unwrap();

    let twap: Uint128 = from_binary(&res).unwrap();
    assert_eq!(twap, Uint128::from(405_113_636u128));
}

#[test]
fn test_get_twap_variant_price_period() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let msg = InstantiateMsg {
        decimals: 6u8,
        oracle_hub_contract: "oracle_hub0000".to_string(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    let res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    let info = mock_info("addr0000", &[]);
    assert_eq!(
        config,
        ConfigResponse {
            owner: info.sender,
            decimals: Uint128::from(1_000_000_u128),
        }
    );

    let prices = vec![
        Uint128::from(400_000_000u128),
        Uint128::from(405_000_000u128),
        Uint128::from(410_000_000u128),
        Uint128::from(420_000_000u128),
    ];

    let timestamps: Vec<u64> = vec![
        env.block.time.seconds(),
        env.block.time.seconds() + 15,
        env.block.time.seconds() + 30,
        env.block.time.seconds() + 75,
    ];

    // Set some market data
    let msg = ExecuteMsg::AppendMultiplePrice {
        key: "ETHUSD".to_string(),
        prices,
        timestamps,
    };

    env.block.time = env.block.time.plus_seconds(95);

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let res = query(
        deps.as_ref(),
        env,
        QueryMsg::GetTwapPrice {
            key: "ETHUSD".to_string(),
            interval: 95,
        },
    )
    .unwrap();

    let twap: Uint128 = from_binary(&res).unwrap();
    assert_eq!(twap, Uint128::from(409_736_842u128));
}

#[test]
fn test_get_twap_latest_price_update_is_earlier_than_request() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let msg = InstantiateMsg {
        decimals: 6u8,
        oracle_hub_contract: "oracle_hub0000".to_string(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    let res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    let info = mock_info("addr0000", &[]);
    assert_eq!(
        config,
        ConfigResponse {
            owner: info.sender,
            decimals: Uint128::from(1_000_000_u128),
        }
    );

    let prices = vec![
        Uint128::from(400_000_000u128),
        Uint128::from(405_000_000u128),
        Uint128::from(410_000_000u128),
    ];

    let timestamps: Vec<u64> = vec![
        env.block.time.seconds(),
        env.block.time.seconds() + 15,
        env.block.time.seconds() + 30,
    ];

    // Set some market data
    let msg = ExecuteMsg::AppendMultiplePrice {
        key: "ETHUSD".to_string(),
        prices,
        timestamps,
    };

    env.block.time = env.block.time.plus_seconds(100);

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let res = query(
        deps.as_ref(),
        env,
        QueryMsg::GetTwapPrice {
            key: "ETHUSD".to_string(),
            interval: 45,
        },
    )
    .unwrap();

    let twap: Uint128 = from_binary(&res).unwrap();
    assert_eq!(twap, Uint128::from(410_000_000u128));
}

#[test]
fn test_get_twap_no_rounds() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let msg = InstantiateMsg {
        decimals: 6u8,
        oracle_hub_contract: "oracle_hub0000".to_string(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    let res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    let info = mock_info("addr0000", &[]);
    assert_eq!(
        config,
        ConfigResponse {
            owner: info.sender,
            decimals: Uint128::from(1_000_000_u128),
        }
    );

    let res = query(
        deps.as_ref(),
        env,
        QueryMsg::GetTwapPrice {
            key: "ETHUSD".to_string(),
            interval: 45,
        },
    )
    .unwrap_err();
    assert_eq!(res.to_string(), "Generic error: Insufficient history");
}

#[test]
fn test_get_twap_error_zero_interval() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let msg = InstantiateMsg {
        decimals: 6u8,
        oracle_hub_contract: "oracle_hub0000".to_string(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    let res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    let info = mock_info("addr0000", &[]);
    assert_eq!(
        config,
        ConfigResponse {
            owner: info.sender,
            decimals: Uint128::from(1_000_000_u128),
        }
    );

    let prices = vec![
        Uint128::from(400_000_000u128),
        Uint128::from(405_000_000u128),
        Uint128::from(410_000_000u128),
    ];

    let timestamps: Vec<u64> = vec![
        env.block.time.seconds(),
        env.block.time.seconds() + 15,
        env.block.time.seconds() + 30,
    ];

    // Set some market data
    let msg = ExecuteMsg::AppendMultiplePrice {
        key: "ETHUSD".to_string(),
        prices,
        timestamps,
    };

    env.block.time = env.block.time.plus_seconds(30);

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let res = query(
        deps.as_ref(),
        env,
        QueryMsg::GetTwapPrice {
            key: "ETHUSD".to_string(),
            interval: 0,
        },
    );
    assert!(res.is_err());
}
