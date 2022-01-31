// Contains queries for external contracts
use cosmwasm_std::{
    to_binary, Decimal, Deps, Fraction, QueryRequest, StdResult, Uint128, WasmQuery,
};

use margined_perp::margined_vamm::{
    QueryMsg, StateResponse
};

// returns the state of the request vamm
// can be used to calculate the input and outputs
pub fn query_vamm_state(deps: &Deps, vamm_address: String) -> StdResult<StateResponse> {
    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: vamm_address,
        msg: to_binary(&QueryMsg::State {})?,
    }))
}
