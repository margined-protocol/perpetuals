// use crate::testing::setup::{self, to_decimals};
use cosmwasm_std::{Uint128, BlockInfo, StdResult};
use cw20::{Cw20Contract, Cw20ExecuteMsg};
use cw_multi_test::Executor;
use margined_perp::margined_engine::{PositionResponse, Side};
use margined_utils::scenarios::SimpleScenario;
use sha3::digest::block_buffer::Block;

pub const DECIMAL_MULTIPLIER: Uint128 = Uint128::new(1_000_000_000);

// takes in a Uint128 and multiplies by the decimals just to make tests more legible
pub fn to_decimals(input: u64) -> Uint128 {
    Uint128::from(input) * DECIMAL_MULTIPLIER
}

pub fn next_block(block: &mut BlockInfo) {
    block.time = block.time.plus_seconds(10);
    block.height += 1;
}

#[test]
fn test_get_margin_ratio() {
    let SimpleScenario {
        mut router,
        alice,
        usdc,
        engine,
        vamm,
        ..
    } = SimpleScenario::new();

    let info = router.block_info();
    println!("INfo: {:?}", info);
    router.update_block(next_block);
    let info = router.block_info();
    println!("INfo: {:?}", info);
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::BUY,
            to_decimals(25u64),
            to_decimals(10u64),
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // expect to be 60
    let margin = engine
        .get_margin_ratio(&router, vamm.addr().to_string(), alice.to_string())
        .unwrap();
    assert_eq!(margin, to_decimals(60));
}
