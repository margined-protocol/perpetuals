use cosmwasm_std::{StdError, Uint128};
use margined_perp::margined_engine::Side;
use margined_utils::{
    cw_multi_test::Executor,
    testing::{to_decimals, SimpleScenario},
};

use crate::testing::new_simple_scenario;

#[test]
fn test_initialization() {
    let SimpleScenario {
        router,
        owner,
        alice,
        bob,
        usdc,
        engine,
        ..
    } = new_simple_scenario();

    // verfiy the balances
    let owner_balance = usdc.balance(&router.wrap(), owner.clone()).unwrap();
    assert_eq!(owner_balance, Uint128::zero());
    let alice_balance = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(alice_balance, Uint128::new(5_000_000_000_000));
    let bob_balance = usdc.balance(&router.wrap(), bob.clone()).unwrap();
    assert_eq!(bob_balance, Uint128::new(5_000_000_000_000));
    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, Uint128::zero());
}

#[test]
fn test_vamm_leverage() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    // initial_margin_ratio = 50_000_000
    // decimal = 1_000_000_000
    // Maximum leveragev = decimal / initial_margin_ratio = 20
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(21u64),
            to_decimals(50),
            Some(to_decimals(9)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();

    assert_eq!(
        StdError::GenericErr {
            msg: "Position is undercollateralized".to_string()
        },
        err.downcast().unwrap()
    );

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(20u64),
            to_decimals(50),
            Some(to_decimals(9)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // take_profit and stop_loss is set
    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position.margin, to_decimals(60));
    assert_eq!(position.take_profit, to_decimals(50));
    assert_eq!(position.stop_loss, Some(to_decimals(9)));
    
    // Set new configuration
    // initial_margin_ratio = 100_000_000
    // decimal = 1_000_000_000
    // Maximum leveragev = decimal / initial_margin_ratio = 10
    let msg = vamm
        .update_config(
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(Uint128::from(100_000_000u128)),
        )
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(60u64),
            to_decimals(20u64),
            to_decimals(70),
            Some(to_decimals(9)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();

    assert_eq!(
        StdError::GenericErr {
            msg: "Position is undercollateralized".to_string()
        },
        err.downcast().unwrap()
    );
}
