use cosmwasm_bignumber::Decimal256;
use std::str::FromStr;

// takes in a Uint128 and multiplies by the decimals just to make tests more legible
pub fn to_decimals(input: &str) -> Decimal256 {
    return Decimal256::from_str(input).unwrap();
}
