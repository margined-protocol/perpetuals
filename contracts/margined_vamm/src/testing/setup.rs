use cosmwasm_std::Uint128;

pub const DECIMAL_MULTIPLIER: Uint128 = Uint128::new(1_000_000_000);

// takes in a Uint128 and multiplies by the decimals just to make tests more legible
pub fn to_decimals(input: u64) -> Uint128 {
    return Uint128::from(input) * DECIMAL_MULTIPLIER;
}
