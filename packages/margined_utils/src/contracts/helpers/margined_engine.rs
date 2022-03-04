use margined_perp::margined_engine::{
    ExecuteMsg, Side,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    to_binary, Addr, Coin, CosmosMsg, StdResult, Uint128,
    WasmMsg,
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
    ) -> StdResult<CosmosMsg<TerraMsgWrapper>> {
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
    ) -> StdResult<CosmosMsg<TerraMsgWrapper>> {
        let msg = ExecuteMsg::OpenPosition {
            vamm,
            side,
            quote_asset_amount,
            leverage,
        };
        self.call(msg, vec![])
    }
}