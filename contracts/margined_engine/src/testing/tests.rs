use crate::{
    contract::{instantiate, execute, query},
};
// use crate::error::ContractError;
use cw20::{Cw20Coin, Cw20Contract, Cw20ExecuteMsg, Cw20ReceiveMsg};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{Addr, Binary, to_binary, coins, Empty, from_binary, Uint128};
use margined_perp::margined_engine::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg, Side,
};
use margined_perp::margined_vamm::{
    InstantiateMsg as VammInstantiateMsg,
    QueryMsg as VammQueryMsg,
    StateResponse as VammStateResponse,
};
use margined_vamm;

const ALICE: &str = "alice_address";

struct ContractInfo {
    addr: Addr,
    id: u64,
}

struct TestingEnv {
    router: App,
    owner: Addr,
    alice: Addr,
    bob: Addr,
    usdc: ContractInfo,
    vamm: ContractInfo,
    engine: ContractInfo,
}

fn mock_app() -> App {
    AppBuilder::new().build()
}

fn contract_cw20() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );
    Box::new(contract)
}

fn contract_vamm() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(
        margined_vamm::contract::execute,
        margined_vamm::contract::instantiate,
        margined_vamm::contract::query,
    );
    Box::new(contract)
}

fn contract_engine() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(
        execute,
        instantiate,
        query,
    );
    Box::new(contract)
}

fn setup() -> TestingEnv {
    let mut router = mock_app();

    let owner = Addr::unchecked("owner");
    let alice = Addr::unchecked("alice");
    let bob = Addr::unchecked("bob");

    let usdc_id = router.store_code(contract_cw20());
    let engine_id = router.store_code(contract_engine());
    let vamm_id = router.store_code(contract_vamm());

    let usdc_addr = router.instantiate_contract(
        usdc_id,
        owner.clone(),
        & cw20_base::msg::InstantiateMsg {
            name: "USDC".to_string(),
            symbol: "USDC".to_string(),
            decimals: 2,
            initial_balances: vec![
                Cw20Coin {
                    address: owner.to_string(),
                    amount: Uint128::new(5000),
                },
                Cw20Coin {
                    address: ALICE.to_string(),
                    amount: Uint128::new(1200),
                }
            ],
            mint: None,
            marketing: None,
        },
        &[],
        "cw20",
        None
    ).unwrap();

    let vamm_addr = router.instantiate_contract(
        vamm_id,
        owner.clone(),
        &VammInstantiateMsg {
            decimals: 10u8,
            quote_asset: "ETH".to_string(),
            base_asset: "USD".to_string(),
            quote_asset_reserve: Uint128::from(100u128),
            base_asset_reserve: Uint128::from(10_000u128),
            funding_period: 3_600 as u64,
        },
        &[],
        "vamm",
        None
    ).unwrap();

    // set up margined engine contract    
    let engine_addr = router
        .instantiate_contract(
            engine_id,
            owner.clone(),
            &InstantiateMsg {
                decimals: 10u8,
                eligible_collateral: usdc_addr.to_string(),
                initial_margin_ratio: Uint128::from(100u128), 
                maintenance_margin_ratio: Uint128::from(100u128), 
                liquidation_fee: Uint128::from(100u128),
            },
            &[],
            "engine",
            None,
        )
        .unwrap();

    TestingEnv {
        router,
        owner,
        alice,
        bob,
        usdc: ContractInfo {
            addr: usdc_addr,
            id: usdc_id,
        },
        vamm: ContractInfo {
            addr: vamm_addr,
            id: vamm_id,
        },
        engine: ContractInfo {
            addr: engine_addr,
            id: engine_id,
        },
    }
}

#[test]
fn test_initialization() {
    setup();
}


#[test]
// receive cw20 tokens and release upon approval
fn test_open_position() {
    let mut router = mock_app();

    // set personal balance
    let owner = Addr::unchecked("owner");
    let alice_address = Addr::unchecked(ALICE);
    let cw20_id = router.store_code(contract_cw20());
    let engine_id = router.store_code(contract_engine());
    let vamm_id = router.store_code(contract_vamm());

    let cw20_init_msg = cw20_base::msg::InstantiateMsg {
        name: "USDC".to_string(),
        symbol: "USDC".to_string(),
        decimals: 2,
        initial_balances: vec![
            Cw20Coin {
                address: owner.to_string(),
                amount: Uint128::new(5000),
            },
            Cw20Coin {
                address: ALICE.to_string(),
                amount: Uint128::new(1200),
            }
        ],
        mint: None,
        marketing: None,
    };

    let usdc_addr = router.instantiate_contract(
        cw20_id,
        owner.clone(),
        &cw20_init_msg,
        &[],
        "cw20",
        None
    ).unwrap();

    let vamm_init_msg = VammInstantiateMsg {
        decimals: 10u8,
        quote_asset: "ETH".to_string(),
        base_asset: "USD".to_string(),
        quote_asset_reserve: Uint128::from(100u128),
        base_asset_reserve: Uint128::from(10_000u128),
        funding_period: 3_600 as u64,
    };

    let vamm_addr = router.instantiate_contract(
        vamm_id,
        owner.clone(),
        &vamm_init_msg,
        &[],
        "vamm",
        None
    ).unwrap();

    // set up margined engine contract    
    let engine_addr = router
        .instantiate_contract(
            engine_id,
            owner.clone(),
            &InstantiateMsg {
                decimals: 10u8,
                eligible_collateral: usdc_addr.to_string(),
                initial_margin_ratio: Uint128::from(100u128), 
                maintenance_margin_ratio: Uint128::from(100u128), 
                liquidation_fee: Uint128::from(100u128),
            },
            &[],
            "engine",
            None,
        )
        .unwrap();

    // they are different
    assert_ne!(usdc_addr, engine_addr);

    // set up cw20 helpers
    let usdc = Cw20Contract(usdc_addr.clone());

    // ensure our balances
    let owner_balance = usdc.balance(&router, owner.clone()).unwrap();
    assert_eq!(owner_balance, Uint128::new(5000));
    let alice_balance = usdc.balance(&router, ALICE.clone()).unwrap();
    assert_eq!(alice_balance, Uint128::new(1200));

    // verify the engine owner
    let config: ConfigResponse = router
        .wrap()
        .query_wasm_smart(&engine_addr, &QueryMsg::Config {})
        .unwrap();
    assert_eq!(
        config,
        ConfigResponse {
            owner: owner.clone(),
            eligible_collateral: usdc_addr.clone(),
        }
    );

    // transfer funds from alice to owner
    let send_msg = Cw20ExecuteMsg::Transfer {
        recipient: ALICE.to_string(),
        amount: Uint128::new(500),
    };
    let _res = router
        .execute_contract(owner.clone(), usdc_addr.clone(), &send_msg, &[]);
    let owner_balance = usdc.balance(&router, owner.clone()).unwrap();
    assert_eq!(owner_balance, Uint128::new(4500));
    let alice_balance = usdc.balance(&router, ALICE.clone()).unwrap();
    assert_eq!(alice_balance, Uint128::new(1700));

    let hook_msg = Cw20HookMsg::OpenPosition {
        vamm: vamm_addr.to_string(),
        side: Side::BUY,
        quote_asset_amount: Uint128::from(100u128),
        leverage: Uint128::from(100u128),
    };

    let message: Binary = to_binary(&hook_msg).unwrap();

    let send_msg = Cw20ExecuteMsg::Send {
        contract: engine_addr.to_string(),
        amount: Uint128::from(100u128),
        msg: message,
    };
    let res = router.execute_contract(
        alice_address.clone(),
        usdc_addr.clone(),
        &send_msg,
        &[]
    );

    let owner_balance = usdc.balance(&router, owner.clone()).unwrap();
    assert_eq!(owner_balance, Uint128::new(4500));
    let alice_balance = usdc.balance(&router, ALICE.clone()).unwrap();
    assert_eq!(alice_balance, Uint128::new(1600));
    let engine_balance = usdc.balance(&router, engine_addr.clone()).unwrap();
    assert_eq!(engine_balance, Uint128::new(100));

    // verify the engine owner
    let config: VammStateResponse = router
        .wrap()
        .query_wasm_smart(&vamm_addr, &VammQueryMsg::State {})
        .unwrap();
    assert_eq!(
        config,
        VammStateResponse {
            quote_asset_reserve: Uint128::from(100u128),
            base_asset_reserve: Uint128::from(10_000u128),
            funding_rate: Uint128::zero(),
            funding_period: 3_600 as u64,
            decimals: Uint128::from(10_000_000_000u128),
        }
    );

    assert_eq!(1, 2);
}

// #[test]
// fn test_instantiation() {
//     let mut deps = mock_dependencies(&[]);
//     let msg = InstantiateMsg {
//         decimals: 10u8,
//         eligible_collateral: COLLATERAL_TOKEN.to_string(),
//         initial_margin_ratio: Uint128::from(100u128), 
//         maintenance_margin_ratio: Uint128::from(100u128), 
//         liquidation_fee: Uint128::from(100u128),
//     };
//     let info = mock_info(OWNER, &[]);
//     instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

//     let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
//     let config: ConfigResponse = from_binary(&res).unwrap();
//     let info = mock_info(OWNER, &[]);
//     assert_eq!(
//         config,
//         ConfigResponse {
//             owner: info.sender.clone(),
//         }
//     );
// }

// #[test]
// fn test_update_config() {
//     let mut deps = mock_dependencies(&[]);
//     let msg = InstantiateMsg {
//         decimals: 10u8,
//         eligible_collateral: COLLATERAL_TOKEN.to_string(),
//         initial_margin_ratio: Uint128::from(100u128), 
//         maintenance_margin_ratio: Uint128::from(100u128), 
//         liquidation_fee: Uint128::from(100u128),
//     };
//     let info = mock_info(OWNER, &[]);
//     instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

//     // Update the config
//     let msg = ExecuteMsg::UpdateConfig {
//         owner: "addr0001".to_string(),
//     };

//     let info = mock_info(OWNER, &[]);
//     execute(deps.as_mut(), mock_env(), info, msg).unwrap();

//     let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
//     let config: ConfigResponse = from_binary(&res).unwrap();
//     assert_eq!(
//         config,
//         ConfigResponse {
//             owner: Addr::unchecked("addr0001".to_string()),
//         }
//     );

//     // Update should fail
//     let msg = ExecuteMsg::UpdateConfig {
//         owner: OWNER.to_string(),
//     };

//     let info = mock_info(OWNER, &[]);
//     let result = execute(deps.as_mut(), mock_env(), info, msg);
//     assert!(result.is_err());
// }

// #[test]
// fn test_open_position() {
//     let mut deps = mock_dependencies(&coins(1000, COLLATERAL_TOKEN));
//     let msg = InstantiateMsg {
//         decimals: 10u8,
//         eligible_collateral: COLLATERAL_TOKEN.to_string(),
//         initial_margin_ratio: Uint128::from(100u128), 
//         maintenance_margin_ratio: Uint128::from(100u128), 
//         liquidation_fee: Uint128::from(100u128),
//     };
//     let info = mock_info(OWNER, &[]);
//     instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

//     // Swap in USD
//     let open_position_msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
//         sender: ALICE.to_string(),
//         amount: Uint128::new(100),
//         msg: to_binary(&Cw20HookMsg::OpenPosition {
//             vamm: VAMM.to_string(),
//             side: Side::BUY,
//             quote_asset_amount: Uint128::from(100u128),
//             leverage: Uint128::from(100u128),
//         })
//         .unwrap(),
//     });

//     let info = mock_info(COLLATERAL_TOKEN, &[]);
//     let execute_res = execute(
//         deps.as_mut(),
//         mock_env(),
//         info,
//         open_position_msg)
//     .unwrap();
//     println!("{:?}", execute_res);

//     // let amount = deps.querier.query_all_balances(&ALICE.to_string())?;


//     // let info = mock_info(OWNER, &[]);
//     // execute(deps.as_mut(), mock_env(), info, swap_msg).unwrap();
//     // let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
//     // let state: StateResponse = from_binary(&res).unwrap();
//     // assert_eq!(
//     //     state,
//     //     StateResponse {
//     //         quote_asset_reserve: Uint128::from(1_600_000_000u128),
//     //         base_asset_reserve: Uint128::from(62_500_000u128),
//     //         funding_rate: Uint128::zero(),
//     //         funding_period: 3_600 as u64,
//     //         decimals: Uint128::from(10_000_000_000u128),
//     //     }
//     // );
// }
