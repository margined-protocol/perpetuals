use cosmwasm_bignumber::{Decimal256, Uint256};
use bigint::U256;

pub const DECIMAL_MULTIPLIER: U256 = U256([1_000_000_000u64, 0, 0, 0]);

// takes in a Uint128 and multiplies by the decimals just to make tests more legible
pub fn to_decimals(input: u64) -> Decimal256 {
    return Decimal256::from_uint256(Uint256::from(input)) *
     Decimal256::from_uint256(Uint256(DECIMAL_MULTIPLIER));
}
