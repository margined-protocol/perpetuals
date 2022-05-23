use crate::contract::{execute, instantiate, query};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{
    from_binary, to_binary, Addr, BankMsg, Coin, CosmosMsg, Empty, StdError, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use cw_multi_test::Executor;
use margined_common::asset::AssetInfo;
use margined_perp::margined_fee_pool::{
    AllTokenResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, TokenLengthResponse,
    TokenResponse,
};
use margined_utils::scenarios::{NativeTokenScenario, SimpleScenario};

#[test]
fn test_instantiation() {
    let mut deps = mock_dependencies();
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
    let mut deps = mock_dependencies();
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
    let owner = config.owner;

    assert_eq!(owner, Addr::unchecked("addr0001".to_string()),);
}
#[test]
fn test_query_token() {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {};
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
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {};
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
        token: "uusd".to_string(),
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
            AssetInfo::Token {
                contract_addr: Addr::unchecked("token1".to_string())
            },
            AssetInfo::NativeToken {
                denom: "uusd".to_string()
            },
        ]
    );
}

#[test]
fn test_add_token() {
    // instantiate contract here
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {};
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
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {};
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
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {};
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
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {};
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
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {};
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
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {};
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
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {};
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

    // add three tokens
    for n in 1..4 {
        let info = mock_info("owner", &[]);
        let msg = ExecuteMsg::AddToken {
            token: tokens[n - 1].clone(),
        };

        execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    }

    // try to add a fourth token
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
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {};
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
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {};
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
/////////////////////////
/// Integration Tests ///
/////////////////////////

#[test]
fn test_send_native_token() {
    // Using the native token, we only work to 6dp

    let NativeTokenScenario {
        mut router,
        owner,
        bank,
        bob,
        fee_pool,
        ..
    } = NativeTokenScenario::new();

    // give funds to the fee pool contract
    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: fee_pool.addr().to_string(),
        amount: vec![Coin::new(5_000u128 * 10u128.pow(6), "uusd")],
    });
    router.execute(bank.clone(), msg).unwrap();

    // add the token so we can send funds with it
    let msg = fee_pool.add_token("uusd".to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // query balance of bob
    let balance = router.wrap().query_balance(&bob, "uusd").unwrap().amount;
    assert_eq!(balance, Uint128::from(5000u128 * 10u128.pow(6)));

    // query balance of contract
    let balance = router
        .wrap()
        .query_balance(&fee_pool.addr(), "uusd")
        .unwrap()
        .amount;
    assert_eq!(balance, Uint128::from(5000u128 * 10u128.pow(6)));

    // send token
    let msg = fee_pool
        .send_token(
            "uusd".to_string(),
            Uint128::from(1000u128 * 10u128.pow(6)),
            bob.clone().to_string(),
        )
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // query new balance of intended recipient
    let balance = router.wrap().query_balance(&bob, "uusd").unwrap().amount;
    assert_eq!(balance, Uint128::from(6000u128 * 10u128.pow(6)));

    // Query new contract balance
    let balance = router
        .wrap()
        .query_balance(&fee_pool.addr(), "uusd")
        .unwrap()
        .amount;
    assert_eq!(balance, Uint128::from(4000u128 * 10u128.pow(6)));

    /////////////////////////
    // Not supported token //
    /////////////////////////

    let NativeTokenScenario {
        mut router,
        owner,
        bank,
        bob,
        fee_pool,
        ..
    } = NativeTokenScenario::new();

    // give funds to the fee pool contract
    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: fee_pool.addr().to_string(),
        amount: vec![Coin::new(5_000u128 * 10u128.pow(6), "uusd")],
    });
    router.execute(bank.clone(), msg).unwrap();

    // try to send token - note this fails because we have not added the token to the token list, so it is not accepted/supported yet
    let msg = fee_pool
        .send_token(
            "uusd".to_string(),
            Uint128::from(1000u128 * 10u128.pow(6)),
            bob.clone().to_string(),
        )
        .unwrap();
    let res = router.execute(owner.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "This token is not supported".to_string(),
        },
        res.downcast().unwrap()
    );

    ////////////////////////
    // Not enough balance //
    ////////////////////////

    let NativeTokenScenario {
        mut router,
        owner,
        bank,
        bob,
        fee_pool,
        ..
    } = NativeTokenScenario::new();

    // give funds to the fee pool contract
    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: fee_pool.addr().to_string(),
        amount: vec![Coin::new(1_000u128 * 10u128.pow(6), "uusd")],
    });
    router.execute(bank.clone(), msg).unwrap();

    // add the token so we can send funds with it
    let msg = fee_pool.add_token("uusd".to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // query balance of bob
    let balance = router.wrap().query_balance(&bob, "uusd").unwrap().amount;
    assert_eq!(balance, Uint128::from(5000u128 * 10u128.pow(6)));

    // query balance of contract
    let balance = router
        .wrap()
        .query_balance(&fee_pool.addr(), "uusd")
        .unwrap()
        .amount;
    assert_eq!(balance, Uint128::from(1000u128 * 10u128.pow(6)));

    // send token
    let msg = fee_pool
        .send_token(
            "uusd".to_string(),
            Uint128::from(2000u128 * 10u128.pow(6)),
            bob.clone().to_string(),
        )
        .unwrap();
    let res = router.execute(owner.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "Insufficient funds".to_string(),
        },
        res.downcast().unwrap()
    );
    // query new balance of intended recipient
    let balance = router.wrap().query_balance(&bob, "uusd").unwrap().amount;
    assert_eq!(balance, Uint128::from(5000u128 * 10u128.pow(6)));

    // Query new contract balance
    let balance = router
        .wrap()
        .query_balance(&fee_pool.addr(), "uusd")
        .unwrap()
        .amount;
    assert_eq!(balance, Uint128::from(1000u128 * 10u128.pow(6)));
}

#[test]
fn test_send_cw20_token() {
    // using the cw20 token, we work to 9dp

    let SimpleScenario {
        mut router,
        owner,
        bob,
        fee_pool,
        usdc,
        ..
    } = SimpleScenario::new();

    // give funds to the fee pool contract for the test
    let msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: usdc.addr().to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Mint {
            recipient: fee_pool.addr().clone().to_string(),
            amount: Uint128::from(5000u128 * 10u128.pow(9)),
        })
        .unwrap(),
    });
    router.execute(owner.clone(), msg).unwrap();

    // add the token so we can send funds with it
    let msg = fee_pool.add_token(usdc.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // query balance of intended recipient (say, bob)
    let balance = usdc.balance::<_, _, Empty>(&router, bob.clone()).unwrap();
    assert_eq!(balance, Uint128::from(5000u128 * 10u128.pow(9)));

    // query balance of contract
    let balance = usdc
        .balance::<_, _, Empty>(&router, fee_pool.addr())
        .unwrap();
    assert_eq!(balance, Uint128::from(5000u128 * 10u128.pow(9)));

    // send token to bob
    let msg = fee_pool
        .send_token(
            usdc.addr().to_string(),
            Uint128::from(1000u128 * 10u128.pow(9)),
            bob.clone().to_string(),
        )
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // query new balance of bob
    let balance = usdc.balance::<_, _, Empty>(&router, bob).unwrap();
    assert_eq!(balance, Uint128::from(6000u128 * 10u128.pow(9)));

    // Query new contract balance
    let balance = usdc
        .balance::<_, _, Empty>(&router, fee_pool.addr())
        .unwrap();
    assert_eq!(balance, Uint128::from(4000u128 * 10u128.pow(9)));

    /////////////////////////
    // Not supported token //
    /////////////////////////

    let SimpleScenario {
        mut router,
        owner,
        bob,
        fee_pool,
        usdc,
        ..
    } = SimpleScenario::new();

    // give funds to the fee pool contract
    let msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: usdc.addr().to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Mint {
            recipient: fee_pool.addr().clone().to_string(),
            amount: Uint128::from(5000u128 * 10u128.pow(9)),
        })
        .unwrap(),
    });
    router.execute(owner.clone(), msg).unwrap();

    // try to send token - note this fails because we have not added the token to the token list, so it is not accepted/supported yet
    let msg = fee_pool
        .send_token(
            usdc.addr().to_string(),
            Uint128::from(1000u128 * 10u128.pow(9)),
            bob.clone().to_string(),
        )
        .unwrap();
    let res = router.execute(owner.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "This token is not supported".to_string(),
        },
        res.downcast().unwrap()
    );

    ////////////////////////
    // Not enough balance //
    ////////////////////////

    let SimpleScenario {
        mut router,
        owner,
        bob,
        fee_pool,
        usdc,
        ..
    } = SimpleScenario::new();

    // give funds to the fee pool contract for the test
    let msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: usdc.addr().to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Mint {
            recipient: fee_pool.addr().clone().to_string(),
            amount: Uint128::from(1000u128 * 10u128.pow(9)),
        })
        .unwrap(),
    });
    router.execute(owner.clone(), msg).unwrap();

    // add the token so we can send funds with it
    let msg = fee_pool.add_token(usdc.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // query balance of intended recipient (say, bob)
    let balance = usdc.balance::<_, _, Empty>(&router, bob.clone()).unwrap();
    assert_eq!(balance, Uint128::from(5000u128 * 10u128.pow(9)));

    // query balance of contract
    let balance = usdc
        .balance::<_, _, Empty>(&router, fee_pool.addr())
        .unwrap();
    assert_eq!(balance, Uint128::from(1000u128 * 10u128.pow(9)));

    // send token to bob
    let msg = fee_pool
        .send_token(
            usdc.addr().to_string(),
            Uint128::from(2000u128 * 10u128.pow(9)),
            bob.clone().to_string(),
        )
        .unwrap();
    let res = router.execute(owner.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "Insufficient funds".to_string(),
        },
        res.downcast().unwrap()
    );

    // query new balance of bob
    let balance = usdc.balance::<_, _, Empty>(&router, bob).unwrap();
    assert_eq!(balance, Uint128::from(5000u128 * 10u128.pow(9)));

    // Query new contract balance
    let balance = usdc
        .balance::<_, _, Empty>(&router, fee_pool.addr())
        .unwrap();
    assert_eq!(balance, Uint128::from(1000u128 * 10u128.pow(9)));
}

///////////////////////
/// Permission Test ///
///////////////////////

#[test]
fn test_not_owner() {
    // instantiate contract here
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {};
    let info = mock_info("owner", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // try to update the config
    let msg = ExecuteMsg::UpdateConfig {
        owner: Some("addr0001".to_string()),
    };
    let info = mock_info("not_the_owner", &[]);

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    assert_eq!(res.to_string(), "Generic error: unauthorized");

    // try to add a token
    let info = mock_info("not_the_owner", &[]);
    let msg = ExecuteMsg::AddToken {
        token: "token1".to_string(),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    assert_eq!(res.to_string(), "Generic error: unauthorized");

    // try to remove a token
    let info = mock_info("not_the_owner", &[]);
    let msg = ExecuteMsg::RemoveToken {
        token: "token1".to_string(),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    assert_eq!(res.to_string(), "Generic error: unauthorized");

    // instantiate the other blockchain
    let SimpleScenario {
        mut router,
        bob,
        fee_pool,
        usdc,
        ..
    } = SimpleScenario::new();

    // try to send money
    let msg = fee_pool
        .send_token(
            usdc.addr().to_string(),
            Uint128::from(1000u128 * 10u128.pow(9)),
            bob.clone().to_string(),
        )
        .unwrap();
    let res = router
        .execute(Addr::unchecked("not_the_owner"), msg)
        .unwrap_err();

    assert_eq!(
        StdError::GenericErr {
            msg: "unauthorized".to_string(),
        },
        res.downcast().unwrap()
    );
}
