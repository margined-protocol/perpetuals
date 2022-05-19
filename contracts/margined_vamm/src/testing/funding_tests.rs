use cosmwasm_std::Uint128;
use cw_multi_test::Executor;
use margined_utils::scenarios::VammScenario;

#[test]
fn test_settle_funding_delay_before_buffer_period_ends() {
    let VammScenario {
        mut router,
        owner,
        vamm,
        pricefeed,
        ..
    } = VammScenario::new();

    let price: Uint128 = Uint128::from(500_000_000u128);
    let timestamp: u64 = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let state = vamm.state(&router).unwrap();
    let original_next_funding_time = state.next_funding_time;
    let expected_funding_time = router.block_info().time.plus_seconds(3_600u64);
    assert_eq!(original_next_funding_time, expected_funding_time.seconds());

    router.update_block(|block| {
        block.time = block.time.plus_seconds(5_400u64);
        block.height += 1;
    });

    let msg = vamm.settle_funding().unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let state = vamm.state(&router).unwrap();
    let expected_funding_time = original_next_funding_time + 3_600u64;
    assert_eq!(state.next_funding_time, expected_funding_time);
}

#[test]
fn test_settle_funding_delay_after_buffer_period_ends_before_next_funding_time() {
    let VammScenario {
        mut router,
        owner,
        vamm,
        pricefeed,
        ..
    } = VammScenario::new();

    let price: Uint128 = Uint128::from(500_000_000u128);
    let timestamp: u64 = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let state = vamm.state(&router).unwrap();
    let original_next_funding_time = state.next_funding_time;
    let settle_funding_time = original_next_funding_time + 1_801u64;
    let expected_funding_time = router.block_info().time.plus_seconds(3_600u64);
    assert_eq!(original_next_funding_time, expected_funding_time.seconds());

    router.update_block(|block| {
        block.time = block.time.plus_seconds(5_401u64);
        block.height += 1;
    });

    let msg = vamm.settle_funding().unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let state = vamm.state(&router).unwrap();
    let expected_funding_time = settle_funding_time + 1_800u64;
    assert_eq!(state.next_funding_time, expected_funding_time);
}

#[test]
fn test_force_error_caller_is_not_couterparty_or_owner() {
    let VammScenario {
        mut router,
        alice,
        owner,
        vamm,
        pricefeed,
        ..
    } = VammScenario::new();

    let price: Uint128 = Uint128::from(500_000_000u128);
    let timestamp: u64 = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm.settle_funding().unwrap();
    let res = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        res.to_string(),
        "Generic error: sender not margin engine".to_string()
    );
}

#[test]
fn test_cant_settle_funding_multiple_times_at_once_even_settle_funding_delay() {
    let VammScenario {
        mut router,
        owner,
        vamm,
        pricefeed,
        ..
    } = VammScenario::new();

    let price: Uint128 = Uint128::from(500_000_000u128);
    let timestamp: u64 = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // moves block forward 1 and 15 secs timestamp
    router.update_block(|block| {
        block.time = block.time.plus_seconds(1_800u64); // funding_period / 2
        block.height += 1;
    });

    let msg = vamm.settle_funding().unwrap();
    let res = router.execute(owner.clone(), msg).unwrap_err();
    assert_eq!(
        res.to_string(),
        "Generic error: settle funding called too early".to_string()
    );
}
