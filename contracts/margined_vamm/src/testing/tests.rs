use crate::contract::{instantiate, query};
// use crate::error::ContractError;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{
    from_binary,
};
use margined_perp::margined_vamm::{ConfigResponse, InstantiateMsg, QueryMsg};

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
    )
}
