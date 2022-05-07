use cosmwasm_std::{Addr, Api, Deps, Response, StdError, StdResult, Uint128};
use terraswap::asset::AssetInfo;

// TODO, probably we should use decimal for ratios but not committed to that yet
pub fn validate_ratio(value: Uint128, decimals: Uint128) -> StdResult<Response> {
    // check that the value is smaller than number of decimals
    if value > decimals || value.is_zero() {
        return Err(StdError::generic_err("invalid ratio"));
    }

    Ok(Response::new())
}

// Turns into TerraSwap asset info depending on whether a CW20 or native token is defined
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
        _ => {
            // check that the input is a valid address else
            // this should throw
            validate_address(deps.api, &input)?;
            AssetInfo::Token {
                contract_addr: input.to_string(),
            }
        }
    };

    Ok(response)
}

// Validates an address is correctly formatted and normalized which seems to be a problem
pub fn validate_address(api: &dyn Api, addr: &str) -> StdResult<Addr> {
    if addr.to_lowercase() != addr {
        return Err(StdError::generic_err(format!(
            "Address {} should be lowercase",
            addr
        )));
    }
    api.addr_validate(addr)
}
