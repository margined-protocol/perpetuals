// Contains queries for external contracts,
use cosmwasm_std::{to_binary, Deps, QueryRequest, StdResult, Uint128, WasmQuery};

use margined_perp::{
    margined_insurance_fund::{AllVammResponse, QueryMsg as InsuranceFundQueryMsg, VammResponse},
    margined_vamm::{CalcFeeResponse, ConfigResponse, Direction, QueryMsg, StateResponse},
};

// returns the config of the request vamm
pub fn query_vamm_config(deps: &Deps, address: String) -> StdResult<ConfigResponse> {
    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: address,
        msg: to_binary(&QueryMsg::Config {})?,
    }))
}

// returns the state of the request vamm
// can be used to calculate the input and outputs
pub fn query_vamm_state(deps: &Deps, address: String) -> StdResult<StateResponse> {
    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: address,
        msg: to_binary(&QueryMsg::State {})?,
    }))
}

// returns the state of the request vamm
// can be used to calculate the input and outputs
pub fn query_vamm_output_amount(
    deps: &Deps,
    address: String,
    direction: Direction,
    amount: Uint128,
) -> StdResult<Uint128> {
    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: address,
        msg: to_binary(&QueryMsg::OutputAmount { direction, amount })?,
    }))
}

// returns the state of the request vamm
// can be used to calculate the input and outputs
pub fn query_vamm_output_twap(
    deps: &Deps,
    address: String,
    direction: Direction,
    amount: Uint128,
) -> StdResult<Uint128> {
    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: address,
        msg: to_binary(&QueryMsg::OutputTwap { direction, amount })?,
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

// returns bool if vamm is over spread limit
pub fn query_vamm_over_spread_limit(deps: &Deps, address: String) -> StdResult<bool> {
    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: address,
        msg: to_binary(&QueryMsg::IsOverSpreadLimit {})?,
    }))
}

// returns pricefeed price of underlying in vamm
pub fn query_vamm_underlying_price(deps: &Deps, address: String) -> StdResult<Uint128> {
    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: address,
        msg: to_binary(&QueryMsg::UnderlyingPrice {})?,
    }))
}

// returns true if vamm has been registered with the insurance contract
pub fn query_insurance_is_vamm(
    deps: &Deps,
    insurance: String,
    vamm: String,
) -> StdResult<VammResponse> {
    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: insurance,
        msg: to_binary(&InsuranceFundQueryMsg::IsVamm { vamm })?,
    }))
}

// returns all vamm registered in the insurance contract
pub fn query_insurance_all_vamm(
    deps: &Deps,
    insurance: String,
    limit: Option<u32>,
) -> StdResult<AllVammResponse> {
    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: insurance,
        msg: to_binary(&InsuranceFundQueryMsg::GetAllVamm { limit })?,
    }))
}

// returns bool is swap is over fluctuation limit
pub fn query_is_over_fluctuation_limit(
    deps: &Deps,
    vamm: String,
    direction: Direction,
    base_asset_amount: Uint128,
) -> StdResult<bool> {
    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: vamm,
        msg: to_binary(&QueryMsg::IsOverFluctuationLimit {
            direction,
            base_asset_amount,
        })?,
    }))
}
