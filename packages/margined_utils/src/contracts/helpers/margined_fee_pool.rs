use cosmwasm_schema::cw_serde;
use margined_perp::margined_fee_pool::{
    AllTokenResponse, ConfigResponse, ExecuteMsg, QueryMsg, TokenLengthResponse, TokenResponse,
};

use cosmwasm_std::{Addr, CosmosMsg, QuerierWrapper, StdResult, Uint128};

use margined_common::messages::wasm_execute;

/// FeePoolController is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[cw_serde]
pub struct FeePoolController(pub Addr);

impl FeePoolController {
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

    pub fn add_token(&self, token: String) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::AddToken { token };
        wasm_execute(&self.0, &msg, vec![])
    }

    pub fn remove_token(&self, token: String) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::RemoveToken { token };
        wasm_execute(&self.0, &msg, vec![])
    }

    pub fn send_token(
        &self,
        token: String,
        amount: Uint128,
        recipient: String,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::SendToken {
            token,
            amount,
            recipient,
        };
        wasm_execute(&self.0, &msg, vec![])
    }

    //////////////////////
    /// Query Messages ///
    //////////////////////

    /// get margin fee pool configuration
    pub fn config(&self, querier: &QuerierWrapper) -> StdResult<ConfigResponse> {
        let msg = QueryMsg::Config {};

        querier.query_wasm_smart(&self.0, &msg)
    }

    /// get the token list length
    pub fn token_list_length(&self, querier: &QuerierWrapper) -> StdResult<TokenLengthResponse> {
        let msg = QueryMsg::GetTokenLength {};

        querier.query_wasm_smart(&self.0, &msg)
    }

    /// get all the tokens in a list
    pub fn all_tokens_list(
        &self,
        querier: &QuerierWrapper,
        limit: Option<u32>,
    ) -> StdResult<AllTokenResponse> {
        let msg = QueryMsg::GetTokenList { limit };

        querier.query_wasm_smart(&self.0, &msg)
    }

    /// query if the given token is actually stored
    pub fn is_token(&self, querier: &QuerierWrapper, token: String) -> StdResult<TokenResponse> {
        let msg = QueryMsg::IsToken { token };

        querier.query_wasm_smart(&self.0, &msg)
    }
}
