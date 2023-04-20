use cosmwasm_schema::cw_serde;
use margined_perp::margined_insurance_fund::{
    AllVammResponse, AllVammStatusResponse, ConfigResponse, ExecuteMsg, QueryMsg, VammResponse,
    VammStatusResponse,
};

use cosmwasm_std::{
    to_binary, Addr, Coin, CosmosMsg, QuerierWrapper, StdResult, WasmMsg, WasmQuery,
};

/// InsuranceFundController is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[cw_serde]
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
    pub fn update_owner(&self, owner: String) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::UpdateOwner { owner };
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
    pub fn config(&self, querier: &QuerierWrapper) -> StdResult<ConfigResponse> {
        let msg = QueryMsg::Config {};
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        querier.query(&query)
    }

    /// get vamm status
    pub fn vamm_status(
        &self,
        vamm: String,
        querier: &QuerierWrapper,
    ) -> StdResult<VammStatusResponse> {
        let msg = QueryMsg::GetVammStatus { vamm };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        querier.query(&query)
    }

    /// get all the vamms status'
    pub fn all_vamm_status(
        &self,
        limit: Option<u32>,
        querier: &QuerierWrapper,
    ) -> StdResult<AllVammStatusResponse> {
        let msg = QueryMsg::GetAllVammStatus { limit };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        querier.query(&query)
    }

    /// get a list of all the vamms
    pub fn all_vamms(
        &self,
        limit: Option<u32>,
        querier: &QuerierWrapper,
    ) -> StdResult<AllVammResponse> {
        let msg = QueryMsg::GetAllVamm { limit };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        querier.query(&query)
    }

    /// query if the given vamm is actually stored
    pub fn is_vamm(&self, vamm: String, querier: &QuerierWrapper) -> StdResult<VammResponse> {
        let msg = QueryMsg::IsVamm { vamm };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        querier.query(&query)
    }
}
