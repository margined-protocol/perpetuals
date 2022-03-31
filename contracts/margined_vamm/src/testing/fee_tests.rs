use cosmwasm_std::Uint128;
use cw_multi_test::Executor;
use margined_perp::margined_vamm::CalcFeeResponse;
use margined_utils::scenarios::{to_decimals, VammScenario};

#[test]
fn test_calc_fee() {
    let VammScenario { router, vamm, .. } = VammScenario::new();

    let result = vamm.calc_fee(&router, to_decimals(10)).unwrap();

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
    let VammScenario {
        mut router,
        owner,
        vamm,
        ..
    } = VammScenario::new();

    let msg = vamm
        .update_config(
            None,
            None,
            None,
            Some(Uint128::from(100_000_000u128)),
            Some(Uint128::from(50_000_000u128)),
            None,
            None,
            None,
            None,
        )
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();
    let result = vamm.calc_fee(&router, to_decimals(100)).unwrap();

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
    let VammScenario {
        mut router,
        owner,
        vamm,
        ..
    } = VammScenario::new();

    let msg = vamm
        .update_config(
            None,
            None,
            None,
            Some(Uint128::zero()),
            Some(Uint128::from(50_000_000u128)),
            None,
            None,
            None,
            None,
        )
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let result = vamm.calc_fee(&router, to_decimals(100)).unwrap();
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
    let VammScenario { router, vamm, .. } = VammScenario::new();

    let result = vamm.calc_fee(&router, to_decimals(0)).unwrap();
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
    let VammScenario {
        mut router,
        alice,
        vamm,
        ..
    } = VammScenario::new();

    let msg = vamm
        .update_config(
            None,
            None,
            None,
            Some(Uint128::zero()),
            Some(Uint128::from(50_000_000u128)),
            None,
            None,
            None,
            None,
        )
        .unwrap();
    let result = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: unauthorized".to_string()
    );
}
