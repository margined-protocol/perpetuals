// Contains queries for external contracts,
use cosmwasm_std::{to_binary, Deps, DepsMut, QueryRequest, StdResult, Uint128, WasmQuery};

use margined_perp::margined_vamm::{Direction, QueryMsg, StateResponse, CalcFeeResponse};

// returns the state of the request vamm
// can be used to calculate the input and outputs
pub fn _query_vamm_state(deps: &DepsMut, address: String) -> StdResult<StateResponse> {
    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: address,
        msg: to_binary(&QueryMsg::State {})?,
    }))
}

// returns the state of the request vamm
// can be used to calculate the input and outputs
pub fn query_vamm_output_price(
    deps: &Deps,
    address: String,
    direction: Direction,
    amount: Uint128,
) -> StdResult<Uint128> {
    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: address,
        msg: to_binary(&QueryMsg::OutputPrice { direction, amount })?,
    }))
}


// returns the spread and toll fees
pub fn query_vamm_calc_fee(
    deps: &Deps,
    address: String,
    quote_asset_amount: Uint128,
) -> StdResult<CalcFeeResponse> {
    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: address,
        msg: to_binary(&QueryMsg::CalcFee { quote_asset_amount })?,
    }))
}
