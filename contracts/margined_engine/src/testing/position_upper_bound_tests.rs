use cosmwasm_std::{StdError, Uint128};
use cw_multi_test::Executor;
use margined_perp::margined_engine::Side;
use margined_utils::scenarios::{to_decimals, SimpleScenario};

#[test]
fn test_open_long_and_short_under_limit() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    let msg = vamm
        .set_base_asset_holding_cap(Uint128::from(10_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(110u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(50u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_open_two_long_positions_under_limit() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    let msg = vamm
        .set_base_asset_holding_cap(Uint128::from(10_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(55u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(55u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_open_short_and_long_under_limit() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    let msg = vamm
        .set_base_asset_holding_cap(Uint128::from(10_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(90u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(50u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_open_two_short_positions_under_limit() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    let msg = vamm
        .set_base_asset_holding_cap(Uint128::from(10_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(45u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(45u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_change_position_size_cap_and_open_position() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    let msg = vamm
        .set_base_asset_holding_cap(Uint128::from(10_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(25u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .close_position(vamm.addr().to_string(), to_decimals(0u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = vamm
        .set_base_asset_holding_cap(Uint128::from(20_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(16u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_force_error_open_long_position_over_cap() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    let msg = vamm
        .set_base_asset_holding_cap(Uint128::from(10_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(120u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "base asset holding exceeds cap".to_string(),
        },
        err.downcast().unwrap()
    );
}

#[test]
fn test_force_error_open_two_long_positions_over_cap() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    let msg = vamm
        .set_base_asset_holding_cap(Uint128::from(10_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "base asset holding exceeds cap".to_string(),
        },
        err.downcast().unwrap()
    );
}

#[test]
fn test_force_error_open_short_position_over_cap() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    let msg = vamm
        .set_base_asset_holding_cap(Uint128::from(10_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(95u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "base asset holding exceeds cap".to_string(),
        },
        err.downcast().unwrap()
    );
}

#[test]
fn test_force_error_open_two_short_positions_over_cap() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    let msg = vamm
        .set_base_asset_holding_cap(Uint128::from(10_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(45u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(50u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "base asset holding exceeds cap".to_string(),
        },
        err.downcast().unwrap()
    );
}

#[test]
fn test_force_error_open_long_and_reverse_short_over_cap() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    let msg = vamm
        .set_base_asset_holding_cap(Uint128::from(10_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(10u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "base asset holding exceeds cap".to_string(),
        },
        err.downcast().unwrap()
    );
}

#[test]
fn test_force_error_open_short_and_reverse_long_over_cap() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    let msg = vamm
        .set_base_asset_holding_cap(Uint128::from(10_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(9u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(21u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "base asset holding exceeds cap".to_string(),
        },
        err.downcast().unwrap()
    );
}

#[test]
fn test_add_remove_whitelist() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        ..
    } = SimpleScenario::new();

    // add alice to whitelist
    let msg = engine.add_whitelist(alice.to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let whitelist = engine.get_whitelist(&router).unwrap();

    assert_eq!(whitelist, vec![alice.to_string()]);

    // remove alice from whitelist
    let msg = engine.remove_whitelist(alice.to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let whitelist = engine.get_whitelist(&router).unwrap();
    let empty: Vec<String> = Vec::new();

    assert_eq!(whitelist, empty)
}

#[test]
fn test_query_all_whitelist_and_is_whitelist() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        owner,
        engine,
        ..
    } = SimpleScenario::new();

    // add alice to whitelist
    let msg = engine.add_whitelist(alice.to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let whitelist = engine.get_whitelist(&router).unwrap();

    assert_eq!(whitelist, vec![alice.to_string()]);

    // add bob to whitelist, alice already in
    let msg = engine.add_whitelist(bob.to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let whitelist = engine.get_whitelist(&router).unwrap();

    assert_eq!(whitelist, vec![alice.to_string(), bob.to_string()]);

    // test if alice is in whitelist
    let bool = engine.is_whitelist(&router, alice.to_string()).unwrap();

    assert!(bool);
    // test if bob is in whitelist
    let bool = engine.is_whitelist(&router, bob.to_string()).unwrap();

    assert!(bool);
}

#[test]
fn test_whitelist_works() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    // add alice to whitelist
    let msg = engine.add_whitelist(alice.to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm
        .set_base_asset_holding_cap(Uint128::from(10_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(9u64),
            to_decimals(1u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(21u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    let res = router.execute(alice.clone(), msg);

    assert!(res.is_ok())
}
