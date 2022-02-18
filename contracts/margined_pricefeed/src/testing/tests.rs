use crate::{
    contract::{execute, instantiate, query},
    state::PriceData,
};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Uint128};
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
        }
    );
}
