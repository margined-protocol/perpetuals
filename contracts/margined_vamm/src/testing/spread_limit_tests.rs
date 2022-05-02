use cosmwasm_std::Uint128;
use margined_utils::scenarios::{to_decimals, VammScenario};
use terra_multi_test::Executor;

#[test]
fn test_will_fail_is_pricefeed_zero() {
    let VammScenario {
        mut router,
        owner,
        vamm,
        pricefeed,
        ..
    } = VammScenario::new();

    let spot_price = vamm.spot_price(&router).unwrap();
    assert_eq!(spot_price, to_decimals(10u64));

    let price: Uint128 = Uint128::from(0u128);
    let timestamp: u64 = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let result = vamm.is_over_spread_limit(&router).unwrap_err();
    assert_eq!(
        result.to_string(),
        "Generic error: Querier contract error: Generic error: underlying price is 0".to_string()
    );
}

#[test]
fn test_is_true_if_greater_than_ten_percent() {
    let VammScenario {
        mut router,
        owner,
        vamm,
        pricefeed,
        ..
    } = VammScenario::new();

    let spot_price = vamm.spot_price(&router).unwrap();
    assert_eq!(spot_price, to_decimals(10u64));

    let price: Uint128 = to_decimals(12u64);
    let timestamp: u64 = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let result = vamm.is_over_spread_limit(&router).unwrap();
    assert_eq!(result, true,);

    let price: Uint128 = to_decimals(8u64);
    let timestamp: u64 = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let result = vamm.is_over_spread_limit(&router).unwrap();
    assert_eq!(result, true,);
}

#[test]
fn test_is_false_if_less_than_ten_percent() {
    let VammScenario {
        mut router,
        owner,
        vamm,
        pricefeed,
        ..
    } = VammScenario::new();

    let spot_price = vamm.spot_price(&router).unwrap();
    assert_eq!(spot_price, to_decimals(10u64));

    let price: Uint128 = Uint128::from(10_500_000_000u128);
    let timestamp: u64 = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let result = vamm.is_over_spread_limit(&router).unwrap();
    assert_eq!(result, false,);

    let price: Uint128 = Uint128::from(9_500_000_000u128);
    let timestamp: u64 = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let result = vamm.is_over_spread_limit(&router).unwrap();
    assert_eq!(result, false,);
}
