use crate::contract::{execute, instantiate, query};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Addr, StdError};
use margined_perp::margined_insurance_fund::{
    AmmResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg,
};

#[test]
fn test_instantiation() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {};
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    let info = mock_info("addr0000", &[]);
    assert_eq!(config, ConfigResponse { owner: info.sender });
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
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddAmm { amm: addr1 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //check for the added AMM
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetAmm {
            amm: "addr0001".to_string(),
        },
    )
    .unwrap();

    let res: AmmResponse = from_binary(&res).unwrap();
    let addr1 = "addr0001".to_string();

    assert_eq!(res.amm.to_string(), addr1);
}

#[test]
fn query_all_amm() {
    //instantiate contract here

    //check to see that there are no AMMs

    //add an AMM

    //check for the added AMM
}

#[test]
fn add_amm() {
    //instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {};
    let info = mock_info("addr0000", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //query the AMM we want to add
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetAmm {
            amm: "addr0001".to_string(),
        },
    );

    let e_no_amm = Err(StdError::NotFound {
        kind: "margined_insurance_fund::state::VammList".to_string(),
    });
    assert_eq!(res, e_no_amm);

    //add an AMM
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddAmm { amm: addr1 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //check for the added AMM
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetAmm {
            amm: "addr0001".to_string(),
        },
    )
    .unwrap();

    let res: AmmResponse = from_binary(&res).unwrap();
    let addr1 = "addr0001".to_string();

    assert_eq!(res.amm.to_string(), addr1);
}

#[test]
fn add_two_amms() {
    //instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {};
    let info = mock_info("addr0000", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //add first AMM
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddAmm { amm: addr1 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //add second AMM
    let addr2 = "addr0002".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddAmm { amm: addr2 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //check for the second added AMM
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetAmm {
            amm: "addr0002".to_string(),
        },
    )
    .unwrap();

    let res: AmmResponse = from_binary(&res).unwrap();
    let addr2 = "addr0002".to_string();

    assert_eq!(res.amm.to_string(), addr2);
}

#[test]
fn remove_amm() {
    //instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {};
    let info = mock_info("addr0000", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //add an AMM
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddAmm { amm: addr1 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //check to see that there is one AMM
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetAmm {
            amm: "addr0001".to_string(),
        },
    )
    .unwrap();

    let res: AmmResponse = from_binary(&res).unwrap();
    let addr1 = "addr0001".to_string();

    assert_eq!(res.amm.to_string(), addr1);

    //remove an AMM
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::RemoveAmm { amm: addr1 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //check that there are zero AMMs
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetAmm {
            amm: "addr0001".to_string(),
        },
    );

    let e_no_amm = Err(StdError::NotFound {
        kind: "AMM".to_string(),
    });

    assert_eq!(res, e_no_amm);
}
