use cw_multi_test::Executor;
use margined_perp::margined_engine::Side;
use margined_utils::scenarios::{to_decimals, SimpleScenario};

#[test]
fn test_paused_by_admin() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    let msg = engine.set_pause(true).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::BUY,
            to_decimals(1u64),
            to_decimals(1u64),
            to_decimals(0u64),
        )
        .unwrap();
    let response = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        response.to_string(),
        "Generic error: margin engine is paused".to_string()
    );

    let msg = engine
        .deposit_margin(vamm.addr().to_string(), to_decimals(1u64))
        .unwrap();
    let response = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        response.to_string(),
        "Generic error: margin engine is paused".to_string()
    );

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), to_decimals(1u64))
        .unwrap();
    let response = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        response.to_string(),
        "Generic error: margin engine is paused".to_string()
    );

    let msg = engine
        .close_position(vamm.addr().to_string(), to_decimals(0u64))
        .unwrap();
    let response = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        response.to_string(),
        "Generic error: margin engine is paused".to_string()
    );
}

#[test]
fn test_cant_be_paused_by_non_admin() {
    let SimpleScenario {
        mut router,
        alice,
        engine,
        ..
    } = SimpleScenario::new();

    let msg = engine.set_pause(true).unwrap();
    let response = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        response.to_string(),
        "Generic error: unauthorized".to_string()
    );
}

#[test]
fn test_pause_then_unpause_by_admin() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    let msg = engine.set_pause(true).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine.set_pause(false).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::BUY,
            to_decimals(1u64),
            to_decimals(1u64),
            to_decimals(0u64),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .deposit_margin(vamm.addr().to_string(), to_decimals(1u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), to_decimals(1u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .close_position(vamm.addr().to_string(), to_decimals(0u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_cant_unpause_when_already_unpaused_and_vice_versa() {
    let SimpleScenario {
        mut router,
        owner,
        engine,
        ..
    } = SimpleScenario::new();

    let msg = engine.set_pause(false).unwrap();
    let response = router.execute(owner.clone(), msg).unwrap_err();
    assert_eq!(
        response.to_string(),
        "Generic error: unauthorized".to_string()
    );

    let msg = engine.set_pause(true).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine.set_pause(true).unwrap();
    let response = router.execute(owner.clone(), msg).unwrap_err();
    assert_eq!(
        response.to_string(),
        "Generic error: unauthorized".to_string()
    );
}
