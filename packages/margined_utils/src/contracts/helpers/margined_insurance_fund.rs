use margined_perp::margined_insurance_fund::{
    AllVammResponse, AllVammStatusResponse, ConfigResponse, ExecuteMsg, QueryMsg, VammResponse,
    VammStatusResponse,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    to_binary, Addr, Coin, CosmosMsg, Empty, Querier, QuerierWrapper, StdResult, WasmMsg, WasmQuery,
};

/// InsuranceFundController is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InsuranceFundController(pub Addr);

impl InsuranceFundController {
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

    /////////////////////////
    ///  Execute Messages ///
    /////////////////////////

    #[allow(clippy::too_many_arguments)]
    pub fn update_config(
        &self,
        owner: Option<String>,
        beneficiary: Option<String>,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::UpdateConfig { owner, beneficiary };
        self.call(msg, vec![])
    }

    pub fn add_vamm(&self, vamm: String) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::AddVamm { vamm };
        self.call(msg, vec![])
    }

    pub fn remove_vamm(&self, vamm: String) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::RemoveVamm { vamm };
        self.call(msg, vec![])
    }

    pub fn shutdown_vamms(&self) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::ShutdownVamms {};
        self.call(msg, vec![])
    }

    //////////////////////
    /// Query Messages ///
    //////////////////////

    /// get margin insurance fund configuration
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

    /// get vamm status
    pub fn vamm_status<Q: Querier>(
        &self,
        vamm: String,
        querier: &Q,
    ) -> StdResult<VammStatusResponse> {
        let msg = QueryMsg::GetVammStatus { vamm };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        let res: VammStatusResponse = QuerierWrapper::<Empty>::new(querier).query(&query)?;
        Ok(res)
    }

    /// get all the vamms status'
    pub fn all_vamm_status<Q: Querier>(
        &self,
        limit: Option<u32>,
        querier: &Q,
    ) -> StdResult<AllVammStatusResponse> {
        let msg = QueryMsg::GetAllVammStatus { limit };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        let res: AllVammStatusResponse = QuerierWrapper::<Empty>::new(querier).query(&query)?;
        Ok(res)
    }

    /// get a list of all the vamms
    pub fn all_vamms<Q: Querier>(
        &self,
        limit: Option<u32>,
        querier: &Q,
    ) -> StdResult<AllVammResponse> {
        let msg = QueryMsg::GetAllVamm { limit };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        let res: AllVammResponse = QuerierWrapper::<Empty>::new(querier).query(&query)?;
        Ok(res)
    }

    /// query if the given vamm is actually stored
    pub fn is_vamm<Q: Querier>(&self, vamm: String, querier: &Q) -> StdResult<VammResponse> {
        let msg = QueryMsg::IsVamm { vamm };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        let res: VammResponse = QuerierWrapper::<Empty>::new(querier).query(&query)?;
        Ok(res)
    }
}
