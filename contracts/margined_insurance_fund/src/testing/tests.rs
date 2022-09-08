use crate::contract::{execute, instantiate, query};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Addr, StdError};
use cw_multi_test::Executor;
use margined_perp::margined_insurance_fund::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg,
};
use margined_utils::scenarios::ShutdownScenario;

const BENEFICIARY: &str = "beneficiary";

#[test]
fn test_instantiation() {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        beneficiary: BENEFICIARY.to_string(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    let info = mock_info("addr0000", &[]);
    assert_eq!(
        config,
        ConfigResponse {
            engine: Addr::unchecked("".to_string()),
            beneficiary: Addr::unchecked(BENEFICIARY.to_string()),
            owner: info.sender
        }
    );
}

#[test]
fn test_update_config() {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        beneficiary: BENEFICIARY.to_string(),
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Update the config
    let msg = ExecuteMsg::UpdateConfig {
        owner: Some("addr0001".to_string()),
        engine: Some(BENEFICIARY.to_string()),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        config,
        ConfigResponse {
            beneficiary: Addr::unchecked(BENEFICIARY.to_string()),
            engine: Addr::unchecked(BENEFICIARY.to_string()),
            owner: Addr::unchecked("addr0001".to_string()),
        }
    );
}
#[test]
fn test_query_vamm() {
    let ShutdownScenario {
        mut router,
        owner,
        insurance_fund,
        vamm1,
        ..
    } = ShutdownScenario::new();

    // add vamm to vammlist in insurance_fund here
    let msg = insurance_fund.add_vamm(vamm1.addr().to_string()).unwrap();
    router.execute(owner, msg).unwrap();

    // query if the vamm has been added
    let res = insurance_fund
        .is_vamm(vamm1.addr().to_string(), &router)
        .unwrap();
    let is_vamm = res.is_vamm;

    assert_eq!(is_vamm, true);
}

#[test]
fn test_query_all_vamm() {
    let ShutdownScenario {
        mut router,
        owner,
        insurance_fund,
        vamm1,
        vamm2,
        ..
    } = ShutdownScenario::new();

    // check to see that there are no vAMMs
    let res = insurance_fund.all_vamms(None, &router).unwrap_err();

    assert_eq!(
        res.to_string(),
        "Generic error: Querier contract error: Generic error: No vAMMs are stored"
    );

    // add a vAMM
    let msg = insurance_fund.add_vamm(vamm1.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // add another vAMM
    let msg = insurance_fund.add_vamm(vamm2.addr().to_string()).unwrap();
    router.execute(owner, msg).unwrap();

    // check for the added vAMMs
    let res = insurance_fund.all_vamms(None, &router).unwrap();
    let list = res.vamm_list;

    assert_eq!(list, vec![vamm1.addr(), vamm2.addr()]);
}

#[test]
fn test_add_vamm() {
    let ShutdownScenario {
        mut router,
        owner,
        insurance_fund,
        vamm1,
        ..
    } = ShutdownScenario::new();

    // query the vAMM we want to add
    let res = insurance_fund
        .is_vamm(vamm1.addr().to_string(), &router)
        .unwrap();
    let is_vamm = res.is_vamm;

    assert_eq!(is_vamm, false);

    // add vamm to vammlist in insurance_fund here
    let msg = insurance_fund.add_vamm(vamm1.addr().to_string()).unwrap();
    router.execute(owner, msg).unwrap();

    // check for the added vAMM
    let res = insurance_fund
        .is_vamm(vamm1.addr().to_string(), &router)
        .unwrap();
    let is_vamm = res.is_vamm;

    assert_eq!(is_vamm, true);
}

#[test]
fn test_add_vamm_twice() {
    let ShutdownScenario {
        mut router,
        owner,
        insurance_fund,
        vamm1,
        ..
    } = ShutdownScenario::new();

    // add vamm to vammlist in insurance_fund here
    let msg = insurance_fund.add_vamm(vamm1.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // try to add the same vamm here
    let msg = insurance_fund.add_vamm(vamm1.addr().to_string()).unwrap();
    let err = router.execute(owner, msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "This vAMM is already added".to_string(),
        },
        err.downcast().unwrap()
    );
}

#[test]
fn test_add_second_vamm() {
    // this tests for adding a second vAMM, to ensure the 'push' match arm of save_vamm is used
    let ShutdownScenario {
        mut router,
        owner,
        insurance_fund,
        vamm1,
        vamm2,
        ..
    } = ShutdownScenario::new();

    // add first vamm to vammlist in insurance_fund here
    let msg = insurance_fund.add_vamm(vamm1.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // add second vamm to vammlist in insurance_fund here
    let msg = insurance_fund.add_vamm(vamm2.addr().to_string()).unwrap();
    router.execute(owner, msg).unwrap();

    // check for the second added vAMM
    let res = insurance_fund
        .is_vamm(vamm2.addr().to_string(), &router)
        .unwrap();
    let is_vamm = res.is_vamm;

    assert_eq!(is_vamm, true);
}

#[test]
fn test_remove_vamm() {
    let ShutdownScenario {
        mut router,
        owner,
        insurance_fund,
        vamm1,
        ..
    } = ShutdownScenario::new();

    // add first vamm to vammlist in insurance_fund here
    let msg = insurance_fund.add_vamm(vamm1.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // check to see that there is one vAMM
    let res = insurance_fund
        .is_vamm(vamm1.addr().to_string(), &router)
        .unwrap();
    let is_vamm = res.is_vamm;

    assert_eq!(is_vamm, true);

    // remove the first vAMM
    let msg = insurance_fund
        .remove_vamm(vamm1.addr().to_string())
        .unwrap();
    router.execute(owner, msg).unwrap();

    // check that there are zero AMMs
    let res = insurance_fund
        .is_vamm(vamm1.addr().to_string(), &router)
        .unwrap();
    let is_vamm = res.is_vamm;

    assert_eq!(is_vamm, false);
}

#[test]
fn test_remove_no_vamms() {
    let ShutdownScenario {
        mut router,
        owner,
        insurance_fund,
        vamm1,
        ..
    } = ShutdownScenario::new();

    // check to see that there is no vAMM
    let res = insurance_fund
        .is_vamm(vamm1.addr().to_string(), &router)
        .unwrap();
    let is_vamm = res.is_vamm;

    assert_eq!(is_vamm, false);

    // remove the first vAMM
    let msg = insurance_fund
        .remove_vamm(vamm1.addr().to_string())
        .unwrap();
    let err = router.execute(owner, msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "No vAMMs are stored".to_string(),
        },
        err.downcast().unwrap()
    );
}

#[test]
fn test_remove_non_existed_vamm() {
    let ShutdownScenario {
        mut router,
        owner,
        insurance_fund,
        vamm1,
        vamm2,
        ..
    } = ShutdownScenario::new();

    // add first vamm to vammlist in insurance_fund here
    let msg = insurance_fund.add_vamm(vamm1.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // check to see that there is one vAMM
    let res = insurance_fund
        .is_vamm(vamm1.addr().to_string(), &router)
        .unwrap();
    let is_vamm = res.is_vamm;

    assert_eq!(is_vamm, true);

    // remove a vAMM which isn't stored
    let msg = insurance_fund
        .remove_vamm(vamm2.addr().to_string())
        .unwrap();
    let err = router.execute(owner, msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "This vAMM has not been added".to_string(),
        },
        err.downcast().unwrap()
    );
}

#[test]
fn test_off_vamm_off_again() {
    let ShutdownScenario {
        mut router,
        owner,
        insurance_fund,
        vamm1,
        ..
    } = ShutdownScenario::new();

    // add vamm (remember it is default added as 'on')
    let msg = insurance_fund.add_vamm(vamm1.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    //turn vamm off
    let msg = insurance_fund.shutdown_vamms().unwrap();
    router.execute(owner.clone(), msg).unwrap();

    //turn vamm off again (note the unauthorized error comes from state.open == open)
    let msg = insurance_fund.shutdown_vamms().unwrap();
    let err = router.execute(owner, msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "unauthorized".to_string(),
        },
        err.downcast().unwrap()
    );
}

#[test]
fn test_vamm_shutdown() {
    let ShutdownScenario {
        mut router,
        owner,
        insurance_fund,
        vamm1,
        vamm2,
        vamm3,
        ..
    } = ShutdownScenario::new();

    // add vamm
    let msg = insurance_fund.add_vamm(vamm1.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // add second vamm
    let msg = insurance_fund.add_vamm(vamm2.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // add third vamm
    let msg = insurance_fund.add_vamm(vamm3.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // query all vamms' status
    let res = insurance_fund.all_vamm_status(None, &router).unwrap();
    let vamms_status = res.vamm_list_status;

    assert_eq!(
        vamms_status,
        vec![
            (vamm1.addr(), true),
            (vamm2.addr(), true),
            (vamm3.addr(), true)
        ]
    );

    // shutdown all vamms
    let msg = insurance_fund.shutdown_vamms().unwrap();
    router.execute(owner, msg).unwrap();

    // query all vamms' status
    let res = insurance_fund.all_vamm_status(None, &router).unwrap();
    let vamms_status = res.vamm_list_status;

    assert_eq!(
        vamms_status,
        vec![
            (vamm1.addr(), false),
            (vamm2.addr(), false),
            (vamm3.addr(), false)
        ]
    );
}

#[test]
fn test_query_vamm_status() {
    let ShutdownScenario {
        mut router,
        owner,
        insurance_fund,
        vamm1,
        ..
    } = ShutdownScenario::new();

    // add vamm
    let msg = insurance_fund.add_vamm(vamm1.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // query vamm status
    let res = insurance_fund
        .vamm_status(vamm1.addr().to_string(), &router)
        .unwrap();
    let status = res.vamm_status;

    assert_eq!(status, true);

    // shutdown vamm
    let msg = insurance_fund.shutdown_vamms().unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // query vamm status
    let res = insurance_fund
        .vamm_status(vamm1.addr().to_string(), &router)
        .unwrap();
    let status = res.vamm_status;

    assert_eq!(status, false);
}

#[test]
fn test_all_vamm_status() {
    let ShutdownScenario {
        mut router,
        owner,
        insurance_fund,
        vamm1,
        vamm2,
        ..
    } = ShutdownScenario::new();

    // query all vamms' status (there aren't any yet)
    let res = insurance_fund.all_vamm_status(None, &router).unwrap_err();

    assert_eq!(
        res.to_string(),
        "Generic error: Querier contract error: Generic error: No vAMMs are stored"
    );

    // add vamm
    let msg = insurance_fund.add_vamm(vamm1.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // add another vamm
    let msg = insurance_fund.add_vamm(vamm2.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // query all vamms' status
    let res = insurance_fund.all_vamm_status(None, &router).unwrap();
    let vamms_status = res.vamm_list_status;

    assert_eq!(
        vamms_status,
        vec![(vamm1.addr(), true), (vamm2.addr(), true)]
    );

    // switch first vamm off
    let msg = insurance_fund.shutdown_vamms().unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // query all vamms' status
    let res = insurance_fund.all_vamm_status(None, &router).unwrap();
    let vamms_status = res.vamm_list_status;

    assert_eq!(
        vamms_status,
        vec![(vamm1.addr(), false), (vamm2.addr(), false)]
    );
}

#[test]
fn test_pagination() {
    // note that this test is superfluous because DEFAULT_PAGINATION_LIMIT > MAX_PAGINATION_LIMIT (this tests default pagi limit)

    let ShutdownScenario {
        mut router,
        owner,
        insurance_fund,
        vamm1,
        vamm2,
        ..
    } = ShutdownScenario::new();

    // add vamm
    let msg = insurance_fund.add_vamm(vamm1.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    //add second vamm
    let msg = insurance_fund.add_vamm(vamm2.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    //query only the first vamm (because we gave it limit of 1)
    let res = insurance_fund.all_vamm_status(Some(1u32), &router).unwrap();
    let vamms_status = res.vamm_list_status;

    assert_eq!(vamms_status, vec![(vamm1.addr(), true)]);
}
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

#[test]
fn test_vamm_capacity() {
    // for the purpose of this test, VAMM_LIMIT is set to 3 (so four exceeds it!)

    let ShutdownScenario {
        mut router,
        owner,
        insurance_fund,
        vamm1,
        vamm2,
        vamm3,
        vamm4,
        ..
    } = ShutdownScenario::new();

    ///////////////////////////////////////////////////////
    // Test exceeding VAMM_LIMIT by adding a single vAMM //
    ///////////////////////////////////////////////////////

    let vamms: Vec<String> = vec![
        vamm1.addr().to_string(),
        vamm2.addr().to_string(),
        vamm3.addr().to_string(),
        vamm4.addr().to_string(),
    ];

    //add three vamms
    for n in 1..4 {
        let msg = insurance_fund.add_vamm(vamms[n - 1].clone()).unwrap();
        router.execute(owner.clone(), msg).unwrap();
    }

    //try to add a fourth vamm
    let msg = insurance_fund.add_vamm(vamm4.addr().to_string()).unwrap();
    let err = router.execute(owner, msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "The vAMM capacity is already reached".to_string(),
        },
        err.downcast().unwrap()
    );

    ////////////////////////////////////////////
    // Test exceeding VAMM_LIMIT via for loop //
    ////////////////////////////////////////////

    let ShutdownScenario {
        mut router,
        owner,
        insurance_fund,
        vamm1,
        vamm2,
        vamm3,
        vamm4,
        ..
    } = ShutdownScenario::new();

    let vamms: Vec<String> = vec![
        vamm1.addr().to_string(),
        vamm2.addr().to_string(),
        vamm3.addr().to_string(),
        vamm4.addr().to_string(),
    ];

    // add four vamms
    for n in 1..5 {
        let msg = insurance_fund.add_vamm(vamms[n - 1].clone()).unwrap();

        if n == 4 {
            let err = router.execute(owner.clone(), msg).unwrap_err();

            assert_eq!(
                StdError::GenericErr {
                    msg: "The vAMM capacity is already reached".to_string(),
                },
                err.downcast().unwrap()
            );
            break;
        }
        router.execute(owner.clone(), msg).unwrap();
    }
}

#[test]
fn test_not_owner() {
    //instantiate contract here
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        beneficiary: BENEFICIARY.to_string(),
    };
    let info = mock_info("owner", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // try to update the config
    let msg = ExecuteMsg::UpdateConfig {
        owner: Some("addr0001".to_string()),
        engine: None,
    };

    let info = mock_info("not_the_owner", &[]);

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    assert_eq!(res.to_string(), "Generic error: unauthorized");

    // try to add a vAMM
    let addr1 = "addr0001".to_string();

    let info = mock_info("not_the_owner", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr1 };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    assert_eq!(res.to_string(), "Generic error: unauthorized");

    //try to remove a vAMM
    let addr1 = "addr0001".to_string();

    let info = mock_info("not_the_owner", &[]);
    let msg = ExecuteMsg::RemoveVamm { vamm: addr1 };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    assert_eq!(res.to_string(), "Generic error: unauthorized");

    //try to shutdown all vamms
    let info = mock_info("not_the_owner", &[]);
    let msg = ExecuteMsg::ShutdownVamms {};

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    assert_eq!(res.to_string(), "Generic error: unauthorized");
}

#[test]
fn test_incompatible_decimals() {
    let ShutdownScenario {
        mut router,
        owner,
        insurance_fund,
        vamm5,
        ..
    } = ShutdownScenario::new();

    let msg = insurance_fund.add_vamm(vamm5.addr().to_string()).unwrap();
    let err = router.execute(owner, msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "vAMM decimals incompatible with margin engine".to_string(),
        },
        err.downcast().unwrap()
    );
}
