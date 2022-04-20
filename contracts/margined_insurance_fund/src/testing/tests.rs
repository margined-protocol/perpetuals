use crate::contract::{execute, instantiate, query};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Addr};
use margined_perp::margined_insurance_fund::{
    AllVammResponse, AllVammStatusResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg,
    VammResponse, VammStatusResponse,
};

const BENEFICIARY: &str = "beneficiary";

#[test]
fn test_instantiation() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {};
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    let info = mock_info("addr0000", &[]);
    assert_eq!(
        config,
        ConfigResponse {
            beneficiary: Addr::unchecked("".to_string()),
            owner: info.sender
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
        beneficiary: Some(BENEFICIARY.to_string()),
    };

    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        config,
        ConfigResponse {
            beneficiary: Addr::unchecked(BENEFICIARY.to_string()),

            owner: Addr::unchecked("addr0001".to_string()),
        }
    );
}
#[test]
fn test_query_vamm() {
    //instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {};
    let info = mock_info("addr0000", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //add an vAMM
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr1 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //check for the added vAMM
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::IsVamm {
            vamm: "addr0001".to_string(),
        },
    )
    .unwrap();

    let res: VammResponse = from_binary(&res).unwrap();
    let is_vamm = res.is_vamm;

    assert_eq!(is_vamm, true);
}

#[test]
fn test_query_all_vamm() {
    //instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {};
    let info = mock_info("addr0000", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //check to see that there are no vAMMs
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAllVamm { limit: None }).unwrap();

    let res: AllVammResponse = from_binary(&res).unwrap();
    let empty: Vec<Addr> = vec![];

    assert_eq!(res.vamm_list, empty);

    //add an vAMM
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr1 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //add another vAMM
    let addr2 = "addr0002".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr2 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //check for the added vAMMs
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAllVamm { limit: None }).unwrap();

    let res: AllVammResponse = from_binary(&res).unwrap();
    let list = res.vamm_list;

    assert_eq!(
        list,
        vec![
            Addr::unchecked("addr0001".to_string()),
            Addr::unchecked("addr0002".to_string())
        ]
    );
}

#[test]
fn test_add_vamm() {
    //instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {};
    let info = mock_info("addr0000", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //query the vAMM we want to add
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::IsVamm {
            vamm: "addr0001".to_string(),
        },
    )
    .unwrap();

    let res: VammResponse = from_binary(&res).unwrap();
    let is_vamm = res.is_vamm;

    assert_eq!(is_vamm, false);

    //add an vAMM
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr1 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //check for the added vAMM
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::IsVamm {
            vamm: "addr0001".to_string(),
        },
    )
    .unwrap();

    let res: VammResponse = from_binary(&res).unwrap();
    let is_vamm = res.is_vamm;

    assert_eq!(is_vamm, true);
}

#[test]
fn test_add_second_vamm() {
    // this tests for adding a second vAMM, to ensure the 'push' match arm of save_vamm is used

    //instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {};
    let info = mock_info("addr0000", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //add first vAMM
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr1 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //add second vAMM
    let addr2 = "addr0002".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr2 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //check for the second added vAMM
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::IsVamm {
            vamm: "addr0002".to_string(),
        },
    )
    .unwrap();

    let res: VammResponse = from_binary(&res).unwrap();
    let is_vamm = res.is_vamm;

    assert_eq!(is_vamm, true);
}

#[test]
fn test_remove_vamm() {
    //instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {};
    let info = mock_info("addr0000", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //add an vAMM
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr1 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //check to see that there is one vAMM
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::IsVamm {
            vamm: "addr0001".to_string(),
        },
    )
    .unwrap();

    let res: VammResponse = from_binary(&res).unwrap();
    let is_vamm = res.is_vamm;

    assert_eq!(is_vamm, true);

    //remove an AMM
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::RemoveVamm { vamm: addr1 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //check that there are zero AMMs
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::IsVamm {
            vamm: "addr0001".to_string(),
        },
    )
    .unwrap();

    let res: VammResponse = from_binary(&res).unwrap();
    let is_vamm = res.is_vamm;

    assert_eq!(is_vamm, false);
}

#[test]
fn test_vamm_off() {
    //instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {};
    let info = mock_info("addr0000", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //add vamm (remember it is default added as 'on')
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr1 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //turn vamm off
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::SwitchVammStatus { vamm: addr1, status: false };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //query vamm status
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetVammStatus {
            vamm: "addr0001".to_string(),
        },
    )
    .unwrap();

    let res: VammStatusResponse = from_binary(&res).unwrap();
    let status = res.vamm_status;

    assert_eq!(status, false);
}

#[test]
fn try_vamm_off_and_on() {
    //instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {};
    let info = mock_info("addr0000", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //try to switch vamm off
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::SwitchVammStatus { vamm: addr1, status: false };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    assert_eq!(
        res.to_string(),
        "Generic error: No vAMM stored"
    );

    //try to switch vamm on
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::SwitchVammStatus { vamm: addr1, status: true };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    assert_eq!(
        res.to_string(),
        "Generic error: No vAMM stored"
    );
}

#[test]
fn test_vamm_on() {
    //instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {};
    let info = mock_info("addr0000", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //add vamm
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr1 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //turn vamm off
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::SwitchVammStatus { vamm: addr1, status: false };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //query vamm status
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetVammStatus {
            vamm: "addr0001".to_string(),
        },
    )
    .unwrap();

    let res: VammStatusResponse = from_binary(&res).unwrap();
    let status = res.vamm_status;

    assert_eq!(status, false);

    //turn vamm on
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::SwitchVammStatus { vamm: addr1, status: true };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //query vamm status
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetVammStatus {
            vamm: "addr0001".to_string(),
        },
    )
    .unwrap();

    let res: VammStatusResponse = from_binary(&res).unwrap();
    let status = res.vamm_status;

    assert_eq!(status, true);
}

#[test]
fn test_off_vamm_off_again() {
    //instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {};
    let info = mock_info("addr0000", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //add vamm (remember it is default added as 'on')
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr1 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //turn vamm on
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::SwitchVammStatus { vamm: addr1, status: false };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //turn vamm off again
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::SwitchVammStatus { vamm: addr1, status: false };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    assert_eq!(res.to_string(), "Generic error: This vAMM is already off");
}

#[test]
fn test_on_vamm_on_again() {
    //instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {};
    let info = mock_info("addr0000", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //add vamm (remember it is default added as 'on')
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr1 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //turn vamm on again
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::SwitchVammStatus { vamm: addr1, status: true };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    assert_eq!(res.to_string(), "Generic error: This vAMM is already on");
}

#[test]
fn test_vamm_shutdown() {
    //instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {};
    let info = mock_info("addr0000", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //add vamm
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr1 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //add second vamm
    let addr2 = "addr0002".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr2 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //query all vamms' status
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAllVammStatus { limit: None }).unwrap();

    let res: AllVammStatusResponse = from_binary(&res).unwrap();
    let vamms_status = res.vamm_list_status;

    assert_eq!(
        vamms_status,
        vec![
            (Addr::unchecked("addr0001".to_string()), true),
            (Addr::unchecked("addr0002".to_string()), true)
        ]
    );

    //shutdown all vamms
    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::ShutdownAllVamm { limit: None };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //query all vamms' status
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAllVammStatus { limit: None }).unwrap();

    let res: AllVammStatusResponse = from_binary(&res).unwrap();
    let vamms_status = res.vamm_list_status;

    assert_eq!(
        vamms_status,
        vec![
            (Addr::unchecked("addr0001".to_string()), false),
            (Addr::unchecked("addr0002".to_string()), false)
        ]
    );
}

#[test]
fn test_query_vamm_status() {
    //instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {};
    let info = mock_info("addr0000", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //add vamm
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr1 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //query vamm status
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetVammStatus {
            vamm: "addr0001".to_string(),
        },
    )
    .unwrap();

    let res: VammStatusResponse = from_binary(&res).unwrap();
    let status = res.vamm_status;

    assert_eq!(status, true);

    //switch vamm off
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::SwitchVammStatus { vamm: addr1, status: false };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //query vamm status
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::GetVammStatus {
            vamm: "addr0001".to_string(),
        },
    )
    .unwrap();

    let res: VammStatusResponse = from_binary(&res).unwrap();
    let status = res.vamm_status;

    assert_eq!(status, false);
}

#[test]
fn test_all_vamm_status() {
    //instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {};
    let info = mock_info("addr0000", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //query all vamms' status (there aren't any yet)
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAllVammStatus { limit: None }).unwrap();

    let res: AllVammStatusResponse = from_binary(&res).unwrap();
    let vamms_status = res.vamm_list_status;

    assert_eq!(vamms_status, vec![]);

    //add vamm
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr1 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //add another vamm
    let addr2 = "addr0002".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr2 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //query all vamms' status
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAllVammStatus { limit: None }).unwrap();

    let res: AllVammStatusResponse = from_binary(&res).unwrap();
    let vamms_status = res.vamm_list_status;

    assert_eq!(
        vamms_status,
        vec![
            (Addr::unchecked("addr0001".to_string()), true),
            (Addr::unchecked("addr0002".to_string()), true)
        ]
    );

    //switch first vamm off
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::SwitchVammStatus { vamm: addr1, status: false };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //query all vamms' status
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAllVammStatus { limit: None }).unwrap();

    let res: AllVammStatusResponse = from_binary(&res).unwrap();
    let vamms_status = res.vamm_list_status;

    assert_eq!(
        vamms_status,
        vec![
            (Addr::unchecked("addr0001".to_string()), false),
            (Addr::unchecked("addr0002".to_string()), true)
        ]
    );
}

#[test]
fn test_not_owner() {
    //instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {};
    let info = mock_info("addr0000", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // try to update the config
    let msg = ExecuteMsg::UpdateConfig {
        owner: Some("addr0001".to_string()),
        beneficiary: None,
    };

    let info = mock_info("not_the_owner", &[]);

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    assert_eq!(res.to_string(), "Unauthorized");

    // try to add a vAMM
    let addr1 = "addr0001".to_string();

    let info = mock_info("not_the_owner", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr1 };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    assert_eq!(res.to_string(), "Unauthorized");

    //try to remove a vAMM
    let addr1 = "addr0001".to_string();

    let info = mock_info("not_the_owner", &[]);
    let msg = ExecuteMsg::RemoveVamm { vamm: addr1 };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    assert_eq!(res.to_string(), "Unauthorized");

    //try to switch vamm on
    let addr1 = "addr0001".to_string();

    let info = mock_info("not_the_owner", &[]);
    let msg = ExecuteMsg::SwitchVammStatus { vamm: addr1, status: true };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    assert_eq!(res.to_string(), "Unauthorized");

    //try to switch vamm off
    let addr1 = "addr0001".to_string();

    let info = mock_info("not_the_owner", &[]);
    let msg = ExecuteMsg::SwitchVammStatus { vamm: addr1, status: false };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    assert_eq!(res.to_string(), "Unauthorized");

    //try to shutdown all vamms
    let info = mock_info("not_the_owner", &[]);
    let msg = ExecuteMsg::ShutdownAllVamm { limit: None };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    assert_eq!(res.to_string(), "Unauthorized");
}

#[test]
fn test_pagination() {
    //instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {};
    let info = mock_info("addr0000", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //add vamm
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr1 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //add second vamm
    let addr2 = "addr0002".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr2 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //query all vamms' status
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAllVammStatus { limit: None }).unwrap();

    let res: AllVammStatusResponse = from_binary(&res).unwrap();
    let vamms_status = res.vamm_list_status;

    assert_eq!(
        vamms_status,
        vec![
            (Addr::unchecked("addr0001".to_string()), true),
            (Addr::unchecked("addr0002".to_string()), true),
        ]
    );

    //shutdown only the first vamm (because we give it limit of 1)
    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::ShutdownAllVamm { limit: Some(1u32) };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //query all vamms' status again to see that only the first vamm has changed
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAllVammStatus { limit: None }).unwrap();

    let res: AllVammStatusResponse = from_binary(&res).unwrap();
    let vamms_status = res.vamm_list_status;

    assert_eq!(
        vamms_status,
        vec![
            (Addr::unchecked("addr0001".to_string()), false),
            (Addr::unchecked("addr0002".to_string()), true)
        ]
    );
    
    //query only the first vamm (because we gave it limit of 1)
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAllVammStatus { limit: Some(1u32) }).unwrap();

    let res: AllVammStatusResponse = from_binary(&res).unwrap();
    let vamms_status = res.vamm_list_status;

    assert_eq!(
        vamms_status,
        vec![
            (Addr::unchecked("addr0001".to_string()), false),
        ]
    );
}
#[test]
fn test_pagination_limit() {
    //instantiate contract here
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {};
    let info = mock_info("addr0000", &[]);

    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //add vamm
    let addr1 = "addr0001".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr1 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //add second vamm
    let addr2 = "addr0002".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr2 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //add third vamm
    let addr3 = "addr0003".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr3 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();    

    //add fourth vamm
    let addr4 = "addr0004".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr4 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();
 
    //add fifth vamm
    let addr5 = "addr0005".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr5 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //add sixth vamm
    let addr6 = "addr0006".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr6 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    
    //add seventh vamm
    let addr7 = "addr0007".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr7 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    
    //add eighth vamm
    let addr8 = "addr0008".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr8 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    
    //add ninth vamm
    let addr9 = "addr0009".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr9 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();    

    //add tenth vamm
    let addr10 = "addr0010".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr10 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    
    //add eleventh vamm
    let addr11 = "addr0011".to_string();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::AddVamm { vamm: addr11 };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //query all vamms status
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAllVammStatus { limit: Some(11u32) }).unwrap();

    let res: AllVammStatusResponse = from_binary(&res).unwrap();
    let vamms_status = res.vamm_list_status;

    assert_eq!(
        vamms_status,
        vec![
            (Addr::unchecked("addr0001".to_string()), true),
            (Addr::unchecked("addr0002".to_string()), true),
            (Addr::unchecked("addr0003".to_string()), true),
            (Addr::unchecked("addr0004".to_string()), true),
            (Addr::unchecked("addr0005".to_string()), true),
            (Addr::unchecked("addr0006".to_string()), true),
            (Addr::unchecked("addr0007".to_string()), true),
            (Addr::unchecked("addr0008".to_string()), true),
            (Addr::unchecked("addr0009".to_string()), true),
            (Addr::unchecked("addr0010".to_string()), true),
            (Addr::unchecked("addr0011".to_string()), true),
        ]
    );

    //shutdown the first 10 vamms (base limit of 10)
    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::ShutdownAllVamm { limit: None };

    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //query all vamms' status again to see that only the first ten vamms have changed
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAllVammStatus { limit: Some(11u32) }).unwrap();

    let res: AllVammStatusResponse = from_binary(&res).unwrap();
    let vamms_status = res.vamm_list_status;

    assert_eq!(
        vamms_status,
        vec![
            (Addr::unchecked("addr0001".to_string()), false),
            (Addr::unchecked("addr0002".to_string()), false),
            (Addr::unchecked("addr0003".to_string()), false),
            (Addr::unchecked("addr0004".to_string()), false),
            (Addr::unchecked("addr0005".to_string()), false),
            (Addr::unchecked("addr0006".to_string()), false),
            (Addr::unchecked("addr0007".to_string()), false),
            (Addr::unchecked("addr0008".to_string()), false),
            (Addr::unchecked("addr0009".to_string()), false),
            (Addr::unchecked("addr0010".to_string()), false),
            (Addr::unchecked("addr0011".to_string()), true),
        ]
    );
    
    //query only the first ten vamms (default limit)
    let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAllVammStatus { limit: None }).unwrap();

    let res: AllVammStatusResponse = from_binary(&res).unwrap();
    let vamms_status = res.vamm_list_status;

    assert_eq!(
        vamms_status,
        vec![
            (Addr::unchecked("addr0001".to_string()), false),
            (Addr::unchecked("addr0002".to_string()), false),
            (Addr::unchecked("addr0003".to_string()), false),
            (Addr::unchecked("addr0004".to_string()), false),
            (Addr::unchecked("addr0005".to_string()), false),
            (Addr::unchecked("addr0006".to_string()), false),
            (Addr::unchecked("addr0007".to_string()), false),
            (Addr::unchecked("addr0008".to_string()), false),
            (Addr::unchecked("addr0009".to_string()), false),
            (Addr::unchecked("addr0010".to_string()), false),
        ]
    );
    // should really test the limit of 30 vamms that is currently there

}