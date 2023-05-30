use cosmwasm_schema::cw_serde;
use margined_perp::margined_insurance_fund::{
    AllVammResponse, AllVammStatusResponse, ConfigResponse, ExecuteMsg, QueryMsg, VammResponse,
    VammStatusResponse,
};

use cosmwasm_std::{Addr, CosmosMsg, QuerierWrapper, StdResult};

use margined_common::messages::wasm_execute;

/// InsuranceFundController is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[cw_serde]
pub struct InsuranceFundController(pub Addr);

impl InsuranceFundController {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    /////////////////////////
    ///  Execute Messages ///
    /////////////////////////

    #[allow(clippy::too_many_arguments)]
    pub fn update_owner(&self, owner: String) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::UpdateOwner { owner };
        wasm_execute(&self.0, &msg, vec![])
    }

    pub fn add_vamm(&self, vamm: String) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::AddVamm { vamm };
        wasm_execute(&self.0, &msg, vec![])
    }

    pub fn remove_vamm(&self, vamm: String) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::RemoveVamm { vamm };
        wasm_execute(&self.0, &msg, vec![])
    }

    pub fn shutdown_vamms(&self) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::ShutdownVamms {};
        wasm_execute(&self.0, &msg, vec![])
    }

    //////////////////////
    /// Query Messages ///
    //////////////////////

    /// get margin insurance fund configuration
    pub fn config(&self, querier: &QuerierWrapper) -> StdResult<ConfigResponse> {
        let msg = QueryMsg::Config {};

        querier.query_wasm_smart(&self.0, &msg)
    }

    /// get vamm status
    pub fn vamm_status(
        &self,
        querier: &QuerierWrapper,
        vamm: String,
    ) -> StdResult<VammStatusResponse> {
        let msg = QueryMsg::GetVammStatus { vamm };

        querier.query_wasm_smart(&self.0, &msg)
    }

    /// get all the vamms status'
    pub fn all_vamm_status(
        &self,
        querier: &QuerierWrapper,
        limit: Option<u32>,
    ) -> StdResult<AllVammStatusResponse> {
        let msg = QueryMsg::GetAllVammStatus { limit };

        querier.query_wasm_smart(&self.0, &msg)
    }

    /// get a list of all the vamms
    pub fn all_vamms(
        &self,
        querier: &QuerierWrapper,
        limit: Option<u32>,
    ) -> StdResult<AllVammResponse> {
        querier.query_wasm_smart(&self.0, &QueryMsg::GetAllVamm { limit })
    }

    /// query if the given vamm is actually stored
    pub fn is_vamm(&self, querier: &QuerierWrapper, vamm: String) -> StdResult<bool> {
        let res: VammResponse = querier.query_wasm_smart(&self.0, &QueryMsg::IsVamm { vamm })?;
        Ok(res.is_vamm)
    }
}
