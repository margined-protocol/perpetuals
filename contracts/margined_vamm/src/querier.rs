// Contains queries for external contracts,
use cosmwasm_std::{to_binary, Deps, DepsMut, QueryRequest, StdResult, Uint128, WasmQuery};

use margined_perp::margined_pricefeed::QueryMsg;

use crate::state::{read_config, Config};

// returns the underlying price provided by an oracle
pub fn query_underlying_price(deps: &DepsMut) -> StdResult<Uint128> {
    let config: Config = read_config(deps.storage)?;
    let key: String = config.base_asset;

    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: config.pricefeed.to_string(),
        msg: to_binary(&QueryMsg::GetPrice { key })?,
    }))
}

// returns the underlying twap price provided by an oracle
pub fn query_underlying_twap_price(deps: &Deps, interval: u64) -> StdResult<Uint128> {
    let config: Config = read_config(deps.storage)?;
    let key: String = config.base_asset;

    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: config.pricefeed.to_string(),
        msg: to_binary(&QueryMsg::GetTwapPrice { key, interval })?,
    }))
}
