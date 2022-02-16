use crate::{
    handle::{get_input_price_with_reserves, get_output_price_with_reserves},
    state::State,
    testing::setup::{to_decimals, DECIMAL_MULTIPLIER},
};
use cosmwasm_std::Uint128;
use margined_perp::margined_vamm::Direction;

/// Unit tests
#[test]
fn test_get_input_and_output_price() {
    let state = State {
        quote_asset_reserve: to_decimals(1_000),
        base_asset_reserve: to_decimals(100),
        funding_rate: Uint128::from(1_000u128),
        funding_period: 3_600 as u64,
        decimals: DECIMAL_MULTIPLIER,
    };

    // amount = 100(quote asset reserved) - (100 * 1000) / (1000 + 50) = 4.7619...
    // price = 50 / 4.7619 = 10.499
    let quote_asset_amount = to_decimals(50);
    let result =
        get_input_price_with_reserves(&state, &Direction::AddToAmm, quote_asset_amount).unwrap();
    assert_eq!(result, Uint128::from(4761904761u128));

    // amount = (100 * 1000) / (1000 - 50) - 100(quote asset reserved) = 5.2631578947368
    // price = 50 / 5.263 = 9.5
    let quote_asset_amount = to_decimals(50);
    let result =
        get_input_price_with_reserves(&state, &Direction::RemoveFromAmm, quote_asset_amount)
            .unwrap();
    assert_eq!(result, Uint128::from(5263157895u128));

    // amount = 1000(base asset reversed) - (100 * 1000) / (100 + 5) = 47.619047619047619048
    // price = 47.619 / 5 = 9.52
    let quote_asset_amount = to_decimals(5);
    let result =
        get_output_price_with_reserves(&state, &Direction::AddToAmm, quote_asset_amount).unwrap();
    assert_eq!(result, Uint128::from(47619047619u128));

    // a dividable number should not plus 1 at mantissa
    let quote_asset_amount = to_decimals(25);
    let result =
        get_output_price_with_reserves(&state, &Direction::AddToAmm, quote_asset_amount).unwrap();
    assert_eq!(result, to_decimals(200));

    // amount = (100 * 1000) / (100 - 5) - 1000(base asset reversed) = 52.631578947368
    // price = 52.631 / 5 = 10.52
    let quote_asset_amount = to_decimals(5);
    let result =
        get_output_price_with_reserves(&state, &Direction::RemoveFromAmm, quote_asset_amount)
            .unwrap();
    assert_eq!(result, Uint128::from(52631578948u128));

    // divisable output
    let quote_asset_amount = Uint128::from(37_500_000_000u128);
    let result =
        get_output_price_with_reserves(&state, &Direction::RemoveFromAmm, quote_asset_amount)
            .unwrap();
    assert_eq!(result, to_decimals(600));
}
