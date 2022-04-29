use crate::contract::{execute, instantiate, query};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Addr};
use margined_perp::margined_fee_pool::{
    AllTokenResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, TokenLengthResponse,
    TokenResponse,
};

const FUNDS: &str = "fake_fund_address";

#[test]
fn test_instantiation() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        funds: FUNDS.to_string(),
    };
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
    let msg = InstantiateMsg {
        funds: FUNDS.to_string(),
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
    let owner = config.owner;

    assert_eq!(owner, Addr::unchecked("addr0001".to_string()),);
}
#[test]
fn test_query_token() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        funds: FUNDS.to_string(),
    };
    let info = mock_info("owner", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // add token to tokenlist here
    let info = mock_info("owner", &[]);
    let msg = ExecuteMsg::AddToken {
        token: "token1".to_string(),
    };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // query if the token has been added
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::IsToken {
            token: "token1".to_string(),
        },
    )
    .unwrap();

    let res: TokenResponse = from_binary(&res).unwrap();
    let is_token = res.is_token;

    assert_eq!(is_token, true);
}

#[test]
fn test_query_all_token() {
    // instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        funds: FUNDS.to_string(),
    };
    let info = mock_info("owner", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // check to see that there are no tokens listed
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetTokenList { limit: None },
    )
    .unwrap_err();

    assert_eq!(res.to_string(), "Generic error: No tokens are stored");

    // add a token
    let info = mock_info("owner", &[]);
    let msg = ExecuteMsg::AddToken {
        token: "token1".to_string(),
    };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // add another token
    let info = mock_info("owner", &[]);
    let msg = ExecuteMsg::AddToken {
        token: "token2".to_string(),
    };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // check for the added tokens
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetTokenList { limit: None },
    )
    .unwrap();

    let res: AllTokenResponse = from_binary(&res).unwrap();
    let list = res.token_list;

    assert_eq!(
        list,
        vec![
            Addr::unchecked("token1".to_string()),
            Addr::unchecked("token2".to_string())
        ]
    );
}

#[test]
fn test_add_token() {
    // instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        funds: FUNDS.to_string(),
    };
    let info = mock_info("owner", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // query the token we want to add
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::IsToken {
            token: "token1".to_string(),
        },
    )
    .unwrap();

    let res: TokenResponse = from_binary(&res).unwrap();
    let is_token = res.is_token;

    assert_eq!(is_token, false);

    // add a token
    let info = mock_info("owner", &[]);
    let msg = ExecuteMsg::AddToken {
        token: "token1".to_string(),
    };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // check for the added token
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::IsToken {
            token: "token1".to_string(),
        },
    )
    .unwrap();

    let res: TokenResponse = from_binary(&res).unwrap();
    let is_token = res.is_token;

    assert_eq!(is_token, true);
}

#[test]
fn test_add_token_twice() {
    // instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        funds: FUNDS.to_string(),
    };
    let info = mock_info("owner", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // add a token
    let info = mock_info("owner", &[]);
    let msg = ExecuteMsg::AddToken {
        token: "token1".to_string(),
    };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // try to add the same token here
    let info = mock_info("owner", &[]);
    let msg = ExecuteMsg::AddToken {
        token: "token1".to_string(),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    assert_eq!(
        res.to_string(),
        "Generic error: This token is already added"
    );
}

#[test]
fn test_add_second_token() {
    // instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        funds: FUNDS.to_string(),
    };
    let info = mock_info("owner", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // add first token to tokenlist here
    let info = mock_info("owner", &[]);
    let msg = ExecuteMsg::AddToken {
        token: "token1".to_string(),
    };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // add second token to tokenlist here
    let info = mock_info("owner", &[]);
    let msg = ExecuteMsg::AddToken {
        token: "token2".to_string(),
    };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // check for the second added token
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::IsToken {
            token: "token2".to_string(),
        },
    )
    .unwrap();

    let res: TokenResponse = from_binary(&res).unwrap();
    let is_token = res.is_token;

    assert_eq!(is_token, true);
}

#[test]
fn test_remove_token() {
    // instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        funds: FUNDS.to_string(),
    };
    let info = mock_info("owner", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // add first token to tokenlist here
    let info = mock_info("owner", &[]);
    let msg = ExecuteMsg::AddToken {
        token: "token1".to_string(),
    };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // check to see that there is one token
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::IsToken {
            token: "token1".to_string(),
        },
    )
    .unwrap();

    let res: TokenResponse = from_binary(&res).unwrap();
    let is_token = res.is_token;

    assert_eq!(is_token, true);

    // remove the first token
    let info = mock_info("owner", &[]);
    let msg = ExecuteMsg::RemoveToken {
        token: "token1".to_string(),
    };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // check that the first token is not there
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::IsToken {
            token: "token1".to_string(),
        },
    )
    .unwrap();

    let res: TokenResponse = from_binary(&res).unwrap();
    let is_token = res.is_token;

    assert_eq!(is_token, false);
}

#[test]
fn test_remove_when_no_tokens() {
    // instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        funds: FUNDS.to_string(),
    };
    let info = mock_info("owner", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // check to see that there is no token
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::IsToken {
            token: "token1".to_string(),
        },
    )
    .unwrap();

    let res: TokenResponse = from_binary(&res).unwrap();
    let is_token = res.is_token;

    assert_eq!(is_token, false);

    // try to remove the first token
    let info = mock_info("owner", &[]);
    let msg = ExecuteMsg::RemoveToken {
        token: "token1".to_string(),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    assert_eq!(res.to_string(), "Generic error: No tokens are stored")
}

#[test]
fn test_remove_non_existed_token() {
    // instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        funds: FUNDS.to_string(),
    };
    let info = mock_info("owner", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // add a token
    let info = mock_info("owner", &[]);
    let msg = ExecuteMsg::AddToken {
        token: "token1".to_string(),
    };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // check to see that there is one token
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::IsToken {
            token: "token1".to_string(),
        },
    )
    .unwrap();

    let res: TokenResponse = from_binary(&res).unwrap();
    let is_token = res.is_token;

    assert_eq!(is_token, true);

    // remove a token which isn't stored
    let info = mock_info("owner", &[]);
    let msg = ExecuteMsg::RemoveToken {
        token: "token2".to_string(),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    assert_eq!(
        res.to_string(),
        "Generic error: This token has not been added"
    )
}

#[test]
fn test_token_capacity() {
    // for the purpose of this test, TOKEN_LIMIT is set to 3 (so four exceeds it!)

    // instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        funds: FUNDS.to_string(),
    };
    let info = mock_info("owner", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    ////////////////////////////////////////////////////////
    // Test exceeding TOKEN_LIMIT by adding a single vAMM //
    ////////////////////////////////////////////////////////

    let tokens: Vec<String> = vec![
        "token1".to_string(),
        "token2".to_string(),
        "token3".to_string(),
        "token4".to_string(),
    ];

    //add three tokens
    for n in 1..4 {
        let info = mock_info("owner", &[]);
        let msg = ExecuteMsg::AddToken {
            token: tokens[n - 1].clone(),
        };

        execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    }

    //try to add a fourth token
    let info = mock_info("owner", &[]);
    let msg = ExecuteMsg::AddToken {
        token: "token4".to_string(),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    assert_eq!(
        res.to_string(),
        "Generic error: The token capacity is already reached"
    );

    ////////////////////////////////////////////
    // Test exceeding VAMM_LIMIT via for loop //
    ////////////////////////////////////////////

    // instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        funds: FUNDS.to_string(),
    };
    let info = mock_info("owner", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let tokens: Vec<String> = vec![
        "token1".to_string(),
        "token2".to_string(),
        "token3".to_string(),
        "token4".to_string(),
    ];

    // add four vamms
    for n in 1..5 {
        let info = mock_info("owner", &[]);
        let msg = ExecuteMsg::AddToken {
            token: tokens[n - 1].clone(),
        };

        if n == 4 {
            let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

            assert_eq!(
                res.to_string(),
                "Generic error: The token capacity is already reached"
            );
            break;
        }
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    }
}

#[test]
fn test_token_length() {
    // instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        funds: FUNDS.to_string(),
    };
    let info = mock_info("owner", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // add first token to tokenlist here
    let info = mock_info("owner", &[]);
    let msg = ExecuteMsg::AddToken {
        token: "token1".to_string(),
    };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // add second token to tokenlist here
    let info = mock_info("owner", &[]);
    let msg = ExecuteMsg::AddToken {
        token: "token2".to_string(),
    };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // check for the second added token
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetTokenLength {}).unwrap();

    let res: TokenLengthResponse = from_binary(&res).unwrap();
    let length = res.length;

    assert_eq!(length, 2usize);
}

#[test]
fn test_not_owner() {
    //instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        funds: FUNDS.to_string(),
    };
    let info = mock_info("owner", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // try to update the config
    let msg = ExecuteMsg::UpdateConfig {
        owner: Some("addr0001".to_string()),
    };
    let info = mock_info("not_the_owner", &[]);

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    assert_eq!(res.to_string(), "Unauthorized");

    // try to add a token
    let info = mock_info("not_the_owner", &[]);
    let msg = ExecuteMsg::AddToken {
        token: "token1".to_string(),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    assert_eq!(res.to_string(), "Unauthorized");

    //try to remove a token
    let info = mock_info("not_the_owner", &[]);
    let msg = ExecuteMsg::RemoveToken {
        token: "token1".to_string(),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    assert_eq!(res.to_string(), "Unauthorized");
}

/*
#[test]
fn test_pagination_limit() {
    // for the purpose of this test, VAMM_LIMIT is set to 3 (so four exceeds it!)

    let ShutdownScenario {
        mut router,
        owner,
        insurance_fund,
        vamm1,
        vamm2,
        vamm3,
        ..
    } = ShutdownScenario::new();

    let vamms: Vec<String> = vec![
        vamm1.addr().to_string(),
        vamm2.addr().to_string(),
        vamm3.addr().to_string(),
    ];

    // add three vamms
    for n in 1..4 {
        let msg = insurance_fund.add_vamm(vamms[n - 1].clone()).unwrap();
        router.execute(owner.clone(), msg).unwrap();
    }

    // query all vamms status
    let res = insurance_fund.all_vamm_status(None, &router).unwrap();
    let vamms_status = res.vamm_list_status;

    assert_eq!(
        vamms_status,
        vec![
            (vamm1.addr(), true),
            (vamm2.addr(), true),
            (vamm3.addr(), true),
        ]
    );

    //query only the first two vamms
    let res = insurance_fund.all_vamm_status(Some(2u32), &router).unwrap();
    let vamms_status = res.vamm_list_status;

    assert_eq!(
        vamms_status,
        vec![(vamm1.addr(), true), (vamm2.addr(), true),]
    );
}
*/
