use margined_perp::margined_pricefeed::{ConfigResponse, ExecuteMsg, QueryMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    to_binary, Addr, Coin, CosmosMsg, Empty, Querier, QuerierWrapper, StdResult, Uint128, WasmMsg,
    WasmQuery,
};

/// PricefeedController is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PricefeedController(pub Addr);

impl PricefeedController {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T, funds: Vec<Coin>) -> StdResult<CosmosMsg> {
        let msg = to_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds,
        }
        .into())
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
        self.call(msg, vec![])
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
        self.call(msg, vec![])
    }

    /// get margined pricefeed configuration
    pub fn config<Q: Querier>(&self, querier: &Q) -> StdResult<ConfigResponse> {
        let msg = QueryMsg::Config {};
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        let res: ConfigResponse = QuerierWrapper::<Empty>::new(querier).query(&query)?;
        Ok(res)
    }

    /// get price
    pub fn get_price<Q: Querier>(&self, querier: &Q, key: String) -> StdResult<Uint128> {
        let msg = QueryMsg::GetPrice { key };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        let res: Uint128 = QuerierWrapper::<Empty>::new(querier).query(&query)?;
        Ok(res)
    }

    /// get previous price
    pub fn get_previous_price<Q: Querier>(
        &self,
        querier: &Q,
        key: String,
        num_round_back: Uint128,
    ) -> StdResult<Uint128> {
        let msg = QueryMsg::GetPreviousPrice {
            key,
            num_round_back,
        };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        let res: Uint128 = QuerierWrapper::<Empty>::new(querier).query(&query)?;
        Ok(res)
    }

    /// get twap price
    pub fn twap_price<Q: Querier>(
        &self,
        querier: &Q,
        key: String,
        interval: u64,
    ) -> StdResult<Uint128> {
        let msg = QueryMsg::GetTwapPrice { key, interval };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        let res: Uint128 = QuerierWrapper::<Empty>::new(querier).query(&query)?;
        Ok(res)
    }
}
