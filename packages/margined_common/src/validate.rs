use crate::asset::AssetInfo;
use cosmwasm_std::{Addr, Api, Deps, Response, StdError, StdResult, Uint128};

/// Validates that the decimals aren't zero and returns the decimal placeholder accordinglys
pub fn validate_decimal_places(decimal_places: u8) -> StdResult<Uint128> {
    // check that the value is not zero
    if decimal_places == 0u8 {
        return Err(StdError::generic_err("Decimal places cannot be zero"));
    }

    Ok(Uint128::from(10u128.pow(decimal_places as u32)))
}

/// Validates that the ratio is between zero and one
pub fn validate_ratio(value: Uint128, decimals: Uint128) -> StdResult<Response> {
    // check that the value is smaller than number of decimals
    if value > decimals {
        return Err(StdError::generic_err("Invalid ratio"));
    }

    Ok(Response::new())
}

/// Verfiy that the address used for collateral is native token or cw token and returns as type AssetInfo
pub fn validate_eligible_collateral(deps: Deps, input: String) -> StdResult<AssetInfo> {
    // verify if the string is any of the native stables for terra
    let response = match input.as_str() {
        "uusd" => AssetInfo::NativeToken {
            denom: input.to_string(),
        },
        "ukrw" => AssetInfo::NativeToken {
            denom: input.to_string(),
        },
        "uluna" => AssetInfo::NativeToken {
            denom: input.to_string(),
        },
        // TODO remove as this is only for testing
        "ujunox" => AssetInfo::NativeToken {
            denom: input.to_string(),
        },
        _ => {
            // check that the input is a valid address else
            // this should throw
            validate_address(deps.api, &input)?;
            AssetInfo::Token {
                contract_addr: deps.api.addr_validate(&input.to_string())?,
            }
        }
    };

    Ok(response)
}

/// Validates an address is correctly formatted and normalized
pub fn validate_address(api: &dyn Api, addr: &str) -> StdResult<Addr> {
    if addr.to_lowercase() != addr {
        return Err(StdError::generic_err(format!(
            "Address {} should be lowercase",
            addr
        )));
    }
    api.addr_validate(addr)
}
