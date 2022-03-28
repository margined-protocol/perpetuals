use margined_perp::margined_engine::{
    ConfigResponse, ExecuteMsg, MarginRatioResponse, PositionResponse, QueryMsg, Side,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    to_binary, Addr, Coin, CosmosMsg, Querier, QuerierWrapper, StdResult, Uint128, WasmMsg,
    WasmQuery,
};

use margined_common::integer::Integer;

/// EngineController is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EngineController(pub Addr);

impl EngineController {
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

    pub fn update_config(&self, owner: String) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::UpdateConfig { owner };
        self.call(msg, vec![])
    }

    pub fn open_position(
        &self,
        vamm: String,
        side: Side,
        quote_asset_amount: Uint128,
        leverage: Uint128,
        base_asset_limit: Uint128,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::OpenPosition {
            vamm,
            side,
            quote_asset_amount,
            leverage,
            base_asset_limit,
        };
        self.call(msg, vec![])
    }

    pub fn close_position(&self, vamm: String, quote_asset_limit: Uint128) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::ClosePosition {
            vamm,
            quote_asset_limit,
        };
        self.call(msg, vec![])
    }

    pub fn liquidate(&self, vamm: String, trader: String) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::Liquidate { vamm, trader };
        self.call(msg, vec![])
    }

    pub fn pay_funding(&self, vamm: String) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::PayFunding { vamm };
        self.call(msg, vec![])
    }

    pub fn deposit_margin(&self, vamm: String, amount: Uint128) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::DepositMargin { vamm, amount };
        self.call(msg, vec![])
    }

    pub fn withdraw_margin(&self, vamm: String, amount: Uint128) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::WithdrawMargin { vamm, amount };
        self.call(msg, vec![])
    }

    /// get margin engine configuration
    pub fn config<Q: Querier>(&self, querier: &Q) -> StdResult<ConfigResponse> {
        let msg = QueryMsg::Config {};
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        let res: ConfigResponse = QuerierWrapper::new(querier).query(&query)?;
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

    /// get unrealized profit and loss for a position
    pub fn unrealized_pnl<Q: Querier>(
        &self,
        querier: &Q,
        vamm: String,
        trader: String,
    ) -> StdResult<Uint128> {
        let msg = QueryMsg::UnrealizedPnl { vamm, trader };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        let res: Uint128 = QuerierWrapper::new(querier).query(&query)?;
        Ok(res)
    }

    /// get margin ratio
    pub fn get_margin_ratio<Q: Querier>(
        &self,
        querier: &Q,
        vamm: String,
        trader: String,
    ) -> StdResult<MarginRatioResponse> {
        let msg = QueryMsg::MarginRatio { vamm, trader };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        let res: MarginRatioResponse = QuerierWrapper::new(querier).query(&query)?;
        Ok(res)
    }

    /// get traders margin balance
    pub fn get_balance_with_funding_payment<Q: Querier>(
        &self,
        querier: &Q,
        trader: String,
    ) -> StdResult<Uint128> {
        let msg = QueryMsg::BalanceWithFundingPayment { trader };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        let res: Uint128 = QuerierWrapper::new(querier).query(&query)?;
        Ok(res)
    }

    /// get personal position with funding payment
    pub fn get_position_with_funding_payment<Q: Querier>(
        &self,
        querier: &Q,
        vamm: String,
        trader: String,
    ) -> StdResult<PositionResponse> {
        let msg = QueryMsg::PositionWithFundingPayment { vamm, trader };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        let res: PositionResponse = QuerierWrapper::new(querier).query(&query)?;
        Ok(res)
    }

    /// get the latest cumulative premium fraction
    pub fn get_latest_cumulative_premium_fraction<Q: Querier>(
        &self,
        querier: &Q,
        vamm: String,
    ) -> StdResult<Integer> {
        let msg = QueryMsg::CumulativePremiumFraction { vamm };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        let res: Integer = QuerierWrapper::new(querier).query(&query)?;
        Ok(res)
    }
}
