use crate::contract::{instantiate, execute, query};
// use crate::error::ContractError;
use cosmwasm_bignumber::{Uint256};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{
    Addr,
    from_binary,
    Uint128,
};
use margined_perp::margined_vamm::{
    ConfigResponse,
    ExecuteMsg,
    InstantiateMsg,
    QueryMsg,
    StateResponse,
};

// fn mock_env_with_block_time(time: u64) -> Env {
//     let env = mock_env();
//     // register time
//     Env {
//         block: BlockInfo {
//             height: 1,
//             time: Timestamp::from_seconds(time),
//             chain_id: "columbus".to_string(),
//         },
//         ..env
//     }
// }

#[test]
fn test_instantiation() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 10u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: Uint256::from(100u128),
        base_asset_reserve: Uint256::from(10_000u128),
        funding_period: Uint128::from(3_600u128),
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
            decimals: 10u8,
            quote_asset: "ETH".to_string(),
            base_asset: "USD".to_string(),
        }
    );

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let state: StateResponse = from_binary(&res).unwrap();
    assert_eq!(
        state,
        StateResponse {
            quote_asset_reserve: Uint256::from(100u128),
            base_asset_reserve: Uint256::from(10_000u128),
            funding_rate: Uint256::zero(),
            funding_period: Uint128::from(3_600u128),
        }
    );
}

#[test]
fn test_update_config() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        decimals: 10u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: Uint256::from(100u128),
        base_asset_reserve: Uint256::from(10_000u128),
        funding_period: Uint128::from(3_600u128),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Update the config
    let mut deps = mock_dependencies(&[]);
    let msg = ExecuteMsg::UpdateConfig {
        owner: "addr0001".to_string(),
        decimals: 18u8,
    };

    let info = mock_info("addr0000", &[]);
    // let err = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();
    // println!("{:?}", err);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    // let info = mock_info("addr0000", &[]);
    assert_eq!(
        config,
        ConfigResponse {
            owner: Addr::unchecked("addr0001".to_string()),
            decimals: 18u8,
            quote_asset: "ETH".to_string(),
            base_asset: "USD".to_string(),
        }
    );
}

// #[test]
// fn test_swap_input() {
//     let mut deps = mock_dependencies(&[]);
//     let msg = InstantiateMsg {
//         decimals: 10u8,
//         quote_asset: "ETH/USD".to_string(),
//         base_asset: "USD".to_string(),
//         quote_asset_reserve: Uint256::from(100u128),
//         base_asset_reserve: Uint256::from(10_000u128),
//         funding_period: Uint128::from(3_600u128),
//     };
//     let info = mock_info("addr0000", &[]);
//     instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

//     let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
//     let config: ConfigResponse = from_binary(&res).unwrap();
//     let info = mock_info("addr0000", &[]);
//     assert_eq!(
//         config,
//         ConfigResponse {
//             owner: info.sender.clone(),
//             decimals: 10u8,
//             quote_asset: "ETH/USD".to_string(),
//             base_asset: "USD".to_string(),
//         }
//     );

//     let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
//     let state: StateResponse = from_binary(&res).unwrap();
//     assert_eq!(
//         state,
//         StateResponse {
//             quote_asset_reserve: Uint256::from(100u128),
//             base_asset_reserve: Uint256::from(10_000u128),
//             funding_rate: Uint256::zero(),
//             funding_period: Uint128::from(3_600u128),
//         }
//     );

//     // Swap in USD
//     let mut deps = mock_dependencies(&[]);
//     let swap_msg = ExecuteMsg::SwapInput {
//         direction: Direction::LONG,
//         quote_asset_amount: Uint256::from(200u128),
//     };

//     let info = mock_info("addr0000", &[]);
//     execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
//     assert_eq!(1, 32)
//     // let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
//     // let state: StateResponse = from_binary(&res).unwrap();
//     // assert_eq!(
//     //     state,
//     //     StateResponse {
//     //         quote_asset_reserve: Uint256::from(100u128),
//     //         base_asset_reserve: Uint256::from(10_200u128),
//     //         funding_rate: Uint256::zero(),
//     //         funding_period: Uint128::from(3_600u128),
//     //     }
//     // );

// }