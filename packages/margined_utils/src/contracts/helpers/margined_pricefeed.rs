use cosmwasm_schema::cw_serde;
use margined_perp::margined_pricefeed::{ConfigResponse, ExecuteMsg, QueryMsg};

use cosmwasm_std::{Addr, CosmosMsg, QuerierWrapper, StdResult, Uint128};

use margined_common::messages::wasm_execute;

/// PricefeedController is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[cw_serde]
pub struct PricefeedController(pub Addr);

impl PricefeedController {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn append_price(
        &self,
        key: String,
        price: Uint128,
        timestamp: u64,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::AppendPrice {
            key,
            price,
            timestamp,
        };
        wasm_execute(&self.0, &msg, vec![])
    }

    pub fn append_multiple_price(
        &self,
        key: String,
        prices: Vec<Uint128>,
        timestamps: Vec<u64>,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::AppendMultiplePrice {
            key,
            prices,
            timestamps,
        };
        wasm_execute(&self.0, &msg, vec![])
    }

    /// get margined pricefeed configuration
    pub fn config(&self, querier: &QuerierWrapper) -> StdResult<ConfigResponse> {
        let msg = QueryMsg::Config {};

        querier.query_wasm_smart(&self.0, &msg)
    }

    /// get price
    pub fn get_price(&self, querier: &QuerierWrapper, key: String) -> StdResult<Uint128> {
        let msg = QueryMsg::GetPrice { key };

        querier.query_wasm_smart(&self.0, &msg)
    }

    /// get previous price
    pub fn get_previous_price(
        &self,
        querier: &QuerierWrapper,
        key: String,
        num_round_back: u64,
    ) -> StdResult<Uint128> {
        let msg = QueryMsg::GetPreviousPrice {
            key,
            num_round_back,
        };

        querier.query_wasm_smart(&self.0, &msg)
    }

    /// get twap price
    pub fn twap_price(
        &self,
        querier: &QuerierWrapper,
        key: String,
        interval: u64,
    ) -> StdResult<Uint128> {
        let msg = QueryMsg::GetTwapPrice { key, interval };

        querier.query_wasm_smart(&self.0, &msg)
    }
}
