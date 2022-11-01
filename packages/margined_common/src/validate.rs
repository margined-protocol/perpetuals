use crate::asset::AssetInfo;
use cosmwasm_std::{Deps, Response, StdError, StdResult, Uint128};

/// Validates that the decimals aren't zero and returns the decimal placeholder accordinglys
pub fn validate_decimal_places(decimal_places: u8) -> StdResult<Uint128> {
    // check that the value less than 6dp
    if decimal_places < 6u8 {
        return Err(StdError::generic_err(
            "Decimal places cannot be less than six",
        ));
    }

    Ok(Uint128::from(10u128.pow(decimal_places as u32)))
}

/// Validates that the value supplied is greater than 1
pub fn validate_non_fraction(value: Uint128, decimals: Uint128) -> StdResult<Response> {
    if value < decimals {
        return Err(StdError::generic_err("Value must be bigger than 1"));
    }

    Ok(Response::new())
}

/// Validates that the ratio is between zero and one
pub fn validate_ratio(value: Uint128, decimals: Uint128) -> StdResult<Response> {
    // check that the value is smaller than number of decimals
    if value > decimals {
        return Err(StdError::generic_err("Invalid ratio"));
    }

    Ok(Response::new())
}

/// Validates that the asset's string name is all alphabetic
pub fn validate_assets(string: String) -> StdResult<Response> {
    // check that the string is all alphabetic
    if !(string.chars().all(|x| x.is_alphabetic())) {
        return Err(StdError::generic_err("Not a valid string"));
    }

    Ok(Response::new())
}

/// Validates that maintenance margin is less than the initial margin
pub fn validate_margin_ratios(
    initial_margin: Uint128,
    maintenance_margin: Uint128,
) -> StdResult<Response> {
    if maintenance_margin > initial_margin {
        return Err(StdError::generic_err(
            "Incorrect initialisation of margin ratios, initial >= maintenance",
        ));
    }

    Ok(Response::new())
}

/// Validates that the address used for collateral is native token or cw token and returns as type AssetInfo
pub fn validate_eligible_collateral(deps: Deps, input: String) -> StdResult<AssetInfo> {
    // verify if the string is any of the native tokens for the deployed network
    let response = match input.as_str() {
        "ujunox" => AssetInfo::NativeToken {
            denom: input.to_string(),
        },
        "uwasm" => AssetInfo::NativeToken {
            denom: input.to_string(),
        },
        _ => {
            // check that the input is a valid address else
            // this should throw
            let valid_addr = deps.api.addr_validate(&input)?;
            AssetInfo::Token {
                contract_addr: valid_addr,
            }
        }
    };

    Ok(response)
}
