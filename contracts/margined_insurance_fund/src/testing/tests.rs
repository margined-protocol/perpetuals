use crate::{
    contract::{execute, instantiate, query},
};
use cosmwasm_std::{from_binary, Addr};
use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info},
};
use margined_perp::margined_insurance_fund::{ConfigResponse, AmmResponse, ExecuteMsg, InstantiateMsg, QueryMsg};

#[test]
fn test_instantiation() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
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
        }
    );
}

#[test]
fn test_update_config() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {};
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
        }
    );
}
#[test]
fn query_amm() {
    //instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {};
    let info = mock_info("addr0000", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //add an AMM
    let addr1 = Addr::unchecked("addr0001".to_string());

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddAMM { amm: addr1 };

    execute(deps.as_mut(), mock_env(), info, msg);

    //check for the added AMM
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAMM { amm: Addr::unchecked("addr0001".to_string()) }).unwrap();
    let amm: AmmResponse = from_binary(&res).unwrap();

}

#[test]
fn query_all_amm(){
    //instantiate contract here

    //check to see that there are no AMMs

    //add an AMM

    //check for the added AMM
}

#[test]
fn add_amm() {
    //instantiate contract here

    //check to see that there are no AMMs

    //add an AMM

    //check for the added AMM
}

#[test]
fn remove_amm(){
    //instantiate contract here

    //add an AMM
    
    //check to see that there is one AMM

    //remove an AMM

    //check that there are zero AMMs
}
