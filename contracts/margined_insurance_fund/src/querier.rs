use cosmwasm_std::{to_binary, Deps, QueryRequest, StdResult, WasmQuery};

use margined_perp::margined_vamm::{QueryMsg as VammQueryMsg, StateResponse};

// this function queries the vamm with given address to find if it is open
pub fn query_vamm_open(deps: &Deps, vamm_addr: String) -> StdResult<bool> {
    let status = deps
        .querier
        .query::<StateResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: vamm_addr,
            msg: to_binary(&VammQueryMsg::State {})?,
        }))?
        .open;
    Ok(status)
}
