use cosmwasm_std::{to_binary, Deps, QueryRequest, StdResult, Uint128, WasmQuery};

use margined_perp::margined_engine::{ConfigResponse, QueryMsg as EngineQueryMsg};
use margined_perp::margined_vamm::{
    ConfigResponse as VammConfigResponse, QueryMsg as VammQueryMsg, StateResponse,
};

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

// this function queries the vamm with given address and returns the decimals from the config
pub fn query_vamm_decimals(deps: &Deps, vamm_addr: String) -> StdResult<Uint128> {
    let result = deps
        .querier
        .query::<VammConfigResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: vamm_addr,
            msg: to_binary(&VammQueryMsg::Config {})?,
        }))?
        .decimals;
    Ok(result)
}

// this function queries the engine with given address and returns the decimals from the config
pub fn query_engine_decimals(deps: &Deps, vamm_addr: String) -> StdResult<Uint128> {
    let result = deps
        .querier
        .query::<ConfigResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: vamm_addr,
            msg: to_binary(&EngineQueryMsg::Config {})?,
        }))?
        .decimals;
    Ok(result)
}
