use cosmwasm_std::{StdError, Uint128};
use margined_perp::margined_vamm::CalcFeeResponse;
use margined_utils::{
    cw_multi_test::Executor,
    testing::{to_decimals, SimpleScenario},
};

use crate::testing::new_simple_scenario;

#[test]
fn test_calc_fee() {
    let SimpleScenario {
        mut router,
        vamm,
        owner,
        ..
    } = new_simple_scenario();

    let msg = vamm.set_toll_ratio(Uint128::from(10_000_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm
        .set_spread_ratio(Uint128::from(10_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let result = vamm.calc_fee(&router.wrap(), to_decimals(10)).unwrap();

    assert_eq!(
        result,
        CalcFeeResponse {
            toll_fee: Uint128::from(100_000_000u128),
            spread_fee: Uint128::from(100_000_000u128),
        }
    );
}

#[test]
fn test_set_diff_fee_ratio() {
    let SimpleScenario {
        mut router,
        owner,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = vamm
        .update_config(
            None,
            None,
            Some(Uint128::from(100_000_000u128)),
            Some(Uint128::from(50_000_000u128)),
            None,
            None,
            None,
            None,
            None,
            None
        )
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();
    let result = vamm.calc_fee(&router.wrap(), to_decimals(100)).unwrap();

    assert_eq!(
        result,
        CalcFeeResponse {
            toll_fee: to_decimals(10),
            spread_fee: to_decimals(5),
        }
    );
}

#[test]
fn test_set_fee_ratio_zero() {
    let SimpleScenario {
        mut router,
        owner,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = vamm
        .update_config(
            None,
            None,
            Some(Uint128::zero()),
            Some(Uint128::from(50_000_000u128)),
            None,
            None,
            None,
            None,
            None,
            None
        )
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let result = vamm.calc_fee(&router.wrap(), to_decimals(100)).unwrap();
    assert_eq!(
        result,
        CalcFeeResponse {
            toll_fee: to_decimals(0),
            spread_fee: to_decimals(5),
        }
    );
}

#[test]
fn test_calc_fee_input_zero() {
    let SimpleScenario { router, vamm, .. } = new_simple_scenario();

    let result = vamm.calc_fee(&router.wrap(), to_decimals(0)).unwrap();
    assert_eq!(
        result,
        CalcFeeResponse {
            toll_fee: to_decimals(0),
            spread_fee: to_decimals(0),
        }
    );
}

#[test]
fn test_update_not_owner() {
    let SimpleScenario {
        mut router,
        alice,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = vamm
        .update_config(
            None,
            None,
            Some(Uint128::zero()),
            Some(Uint128::from(50_000_000u128)),
            None,
            None,
            None,
            None,
            None,
            None
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "unauthorized".to_string(),
        },
        err.downcast().unwrap()
    );
}
