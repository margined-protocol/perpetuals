use cosmwasm_std::{Deps, Response, StdError, StdResult, Uint128};
use terraswap::asset::AssetInfo;

// TODO, probably we should use decimal256 for ratios but not committed to that yet
pub fn validate_ratio(value: Uint128, decimals: Uint128) -> StdResult<Response> {
    // check that the value is smaller than number of decimals
    if value > decimals {
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
            deps.api.addr_validate(&input)?;
            AssetInfo::Token {
                contract_addr: input.to_string(),
            }
        }
    };

    Ok(response)
}
