use cosmwasm_schema::cw_serde;
use margined_perp::margined_fee_pool::{
    AllTokenResponse, ConfigResponse, ExecuteMsg, QueryMsg, TokenLengthResponse, TokenResponse,
};

use cosmwasm_std::{
    to_binary, Addr, Coin, CosmosMsg, QuerierWrapper, StdResult, Uint128, WasmMsg, WasmQuery,
};

/// FeePoolController is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[cw_serde]
pub struct FeePoolController(pub Addr);

impl FeePoolController {
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

    pub fn add_token(&self, token: String) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::AddToken { token };
        self.call(msg, vec![])
    }

    pub fn remove_token(&self, token: String) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::RemoveToken { token };
        self.call(msg, vec![])
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
        self.call(msg, vec![])
    }

    //////////////////////
    /// Query Messages ///
    //////////////////////

    /// get margin fee pool configuration
    pub fn config(&self, querier: &QuerierWrapper) -> StdResult<ConfigResponse> {
        let msg = QueryMsg::Config {};
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        querier.query(&query)
    }

    /// get the token list length
    pub fn token_list_length(&self, querier: &QuerierWrapper) -> StdResult<TokenLengthResponse> {
        let msg = QueryMsg::GetTokenLength {};
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        querier.query(&query)
    }

    /// get all the tokens in a list
    pub fn all_tokens_list(
        &self,
        limit: Option<u32>,
        querier: &QuerierWrapper,
    ) -> StdResult<AllTokenResponse> {
        let msg = QueryMsg::GetTokenList { limit };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        querier.query(&query)
    }

    /// query if the given token is actually stored
    pub fn is_token(&self, token: String, querier: &QuerierWrapper) -> StdResult<TokenResponse> {
        let msg = QueryMsg::IsToken { token };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        querier.query(&query)
    }
}
