use crate::contract::{instantiate, query};
// use crate::error::ContractError;
use cosmwasm_bignumber::{Uint256};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{
    from_binary,
    Uint128,
};
use margined_perp::margined_vamm::{ConfigResponse, StateResponse, InstantiateMsg, QueryMsg};

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
        base_asset_reserve: Uint256::from(100u128),
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
            base_asset_reserve: Uint256::from(100u128),
            funding_rate: Uint256::zero(),
            funding_period: Uint128::from(3_600u128),
        }
    );
}

// #[test]
// fn test_instantiation_state() {
//     let mut deps = mock_dependencies(&[]);
//     let msg = InstantiateMsg {
//         decimals: 10u8,
//         quote_asset: "ETH".to_string(),
//         base_asset: "USD".to_string(),
//         quote_asset_reserve: Uint256::from(100u128),
//         base_asset_reserve: Uint256::from(100u128),
//         funding_period: 0,
//     };
//     let info = mock_info("addr0000", &[]);
//     instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

//     let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
//     println!("{:?}", res);
//     let state: StateResponse = from_binary(&res).unwrap();
//     // let info = mock_info("addr0000", &[]);
//     assert_eq!(
//         state,
//         StateResponse {
//             quote_asset_reserve: Uint256::from(100u128),
//             base_asset_reserve: Uint256::from(100u128),
//             funding_rate: Uint256::zero(),
//             // funding_period: 0,
//         }
//     );
// }