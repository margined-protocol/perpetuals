use cosmwasm_bignumber::{Decimal256};

/// However it follows the design of the perpertual protocol decimals
/// https://github.com/perpetual-protocol/perpetual-protocol/blob/release/v2.1.x/src/utils/Decimal.sol
// pub(crate) fn modulo(a: Uint128, b: Uint128) -> Uint128 {
//     // TODO the decimals are currently hardcoded to 9dp, this needs to change in the future but without
//     // needing to pass the entire world to this function, i.e. access to storage
//     let a_decimals = a.checked_mul(Uint128::from(1_000_000_000u128)).unwrap();
//     let integral = a_decimals / b;
//     a_decimals - (b * integral)
// }

pub(crate) fn modulo(a: Decimal256, b: Decimal256) -> Decimal256 {
    // TODO the decimals are currently hardcoded to 9dp, this needs to change in the future but without
    // needing to pass the entire world to this function, i.e. access to storage
    // let decimal : Uint256 = Uint256::from(1_000_000_000u128);
    // let a_decimals = a;
    let integral = a / b;
    a - (b * integral)
}
