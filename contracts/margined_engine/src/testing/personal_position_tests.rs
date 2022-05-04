use cosmwasm_std::Uint128;
use cw20::Cw20ExecuteMsg;
use margined_common::integer::Integer;
use margined_perp::margined_engine::Side;
use margined_utils::scenarios::{to_decimals, SimpleScenario};
use terra_multi_test::Executor;

pub const NEXT_FUNDING_PERIOD_DELTA: u64 = 86_400u64;

#[test]
fn test_get_personal_position_with_funding_payments() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        usdc,
        pricefeed,
        ..
    } = SimpleScenario::new();

    // reduce the allowance
    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1940),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // given alice takes 10x short position (size: -150) with 60 margin
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(60u64),
            to_decimals(10u64),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // given the underlying twap price is $2.1, and current snapShot price is 400B/250Q = $1.6
    let msg = pricefeed
        .append_price(
            "ETH".to_string(),
            Uint128::from(2_100_000_000u128),
            router.block_info().time.seconds(),
        )
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(NEXT_FUNDING_PERIOD_DELTA);
        block.height += 1;
    });

    // when the new fundingRate is -50% which means underlyingPrice < snapshotPrice
    let msg = engine.pay_funding(vamm.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let premium_fraction = engine
        .get_latest_cumulative_premium_fraction(&router, vamm.addr().to_string())
        .unwrap();
    assert_eq!(
        premium_fraction,
        Integer::new_negative(500_000_000u128), // -0.5
    );

    // then alice need to pay 150 * 50% = $75
    // {size: -150, margin: 300} => {size: -150, margin: 0}
    let alice_position = engine
        .get_position_with_funding_payment(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(alice_position.margin, to_decimals(0u64),);
    assert_eq!(
        alice_position.size,
        Integer::new_negative(to_decimals(150u64))
    );
}
