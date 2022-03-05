use margined_perp::margined_engine::{ExecuteMsg, PositionResponse, QueryMsg, Side};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    to_binary, Addr, Coin, CosmosMsg, Querier, QuerierWrapper, StdResult, Uint128, WasmMsg,
    WasmQuery,
};

use terra_cosmwasm::TerraMsgWrapper;

/// EngineController is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EngineController(pub Addr);

impl EngineController {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(
        &self,
        msg: T,
        funds: Vec<Coin>,
        // ) -> StdResult<CosmosMsg<TerraMsgWrapper>> {
    ) -> StdResult<CosmosMsg> {
        let msg = to_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds,
        }
        .into())
    }

    pub fn open_position(
        &self,
        vamm: String,
        side: Side,
        quote_asset_amount: Uint128,
        leverage: Uint128,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::OpenPosition {
            vamm,
            side,
            quote_asset_amount,
            leverage,
        };
        self.call(msg, vec![])
    }

    /// get traders margin balance
    pub fn trader_balance<Q: Querier>(&self, querier: &Q, trader: String) -> StdResult<Uint128> {
        let msg = QueryMsg::TraderBalance { trader };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        let res: Uint128 = QuerierWrapper::new(querier).query(&query)?;
        Ok(res)
    }

    /// get traders position for a particular vamm
    pub fn position<Q: Querier>(
        &self,
        querier: &Q,
        vamm: String,
        trader: String,
    ) -> StdResult<PositionResponse> {
        let msg = QueryMsg::Position { vamm, trader };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        let res: PositionResponse = QuerierWrapper::new(querier).query(&query)?;
        Ok(res)
    }
}
