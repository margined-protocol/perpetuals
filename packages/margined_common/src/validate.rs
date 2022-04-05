use cosmwasm_std::{Response, StdError, StdResult, Uint128};

// TODO, probably we should use decimal256 for ratios but not committed to that yet
pub fn validate_ratio(value: Uint128, decimals: Uint128) -> StdResult<Response> {
    // check that the value is smaller than number of decimals
    if value > decimals {
        return Err(StdError::generic_err("invalid ratio"));
    }

    Ok(Response::new())
}
