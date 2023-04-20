use cosmwasm_schema::cw_serde;
use cw_controllers::HooksResponse;
use margined_perp::margined_engine::{
    ConfigResponse, ExecuteMsg, PnlCalcOption, Position, PositionUnrealizedPnlResponse, QueryMsg,
    Side, StateResponse,
};

use cosmwasm_std::{
    to_binary, Addr, Coin, CosmosMsg, QuerierWrapper, StdResult, Uint128, WasmMsg, WasmQuery,
};

use margined_common::integer::Integer;

/// EngineController is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[cw_serde]
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

    #[allow(clippy::too_many_arguments)]
    pub fn update_config(
        &self,
        owner: Option<String>,
        insurance_fund: Option<String>,
        fee_pool: Option<String>,
        initial_margin_ratio: Option<Uint128>,
        maintenance_margin_ratio: Option<Uint128>,
        partial_liquidation_ratio: Option<Uint128>,
        liquidation_fee: Option<Uint128>,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::UpdateConfig {
            owner,
            insurance_fund,
            fee_pool,
            initial_margin_ratio,
            maintenance_margin_ratio,
            partial_liquidation_ratio,
            liquidation_fee,
        };
        self.call(msg, vec![])
    }

    pub fn set_initial_margin_ratio(&self, initial_margin_ratio: Uint128) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::UpdateConfig {
            owner: None,
            insurance_fund: None,
            fee_pool: None,
            initial_margin_ratio: Some(initial_margin_ratio),
            maintenance_margin_ratio: None,
            partial_liquidation_ratio: None,
            liquidation_fee: None,
        };
        self.call(msg, vec![])
    }

    pub fn set_maintenance_margin_ratio(
        &self,
        maintenance_margin_ratio: Uint128,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::UpdateConfig {
            owner: None,
            insurance_fund: None,
            fee_pool: None,
            initial_margin_ratio: None,
            maintenance_margin_ratio: Some(maintenance_margin_ratio),
            partial_liquidation_ratio: None,
            liquidation_fee: None,
        };
        self.call(msg, vec![])
    }

    pub fn set_margin_ratios(&self, margin_ratio: Uint128) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::UpdateConfig {
            owner: None,
            insurance_fund: None,
            fee_pool: None,
            initial_margin_ratio: Some(margin_ratio),
            maintenance_margin_ratio: Some(margin_ratio),
            partial_liquidation_ratio: None,
            liquidation_fee: None,
        };
        self.call(msg, vec![])
    }

    pub fn set_partial_liquidation_ratio(
        &self,
        partial_liquidation_ratio: Uint128,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::UpdateConfig {
            owner: None,
            insurance_fund: None,
            fee_pool: None,
            initial_margin_ratio: None,
            maintenance_margin_ratio: None,
            partial_liquidation_ratio: Some(partial_liquidation_ratio),
            liquidation_fee: None,
        };
        self.call(msg, vec![])
    }

    pub fn set_liquidation_fee(&self, liquidation_fee: Uint128) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::UpdateConfig {
            owner: None,
            insurance_fund: None,
            fee_pool: None,
            initial_margin_ratio: None,
            maintenance_margin_ratio: None,
            partial_liquidation_ratio: None,
            liquidation_fee: Some(liquidation_fee),
        };
        self.call(msg, vec![])
    }

    pub fn set_pause(&self, pause: bool) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::SetPause { pause };
        self.call(msg, vec![])
    }

    pub fn open_position(
        &self,
        vamm: String,
        side: Side,
        margin_amount: Uint128,
        leverage: Uint128,
        base_asset_limit: Uint128,
        funds: Vec<Coin>,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::OpenPosition {
            vamm,
            side,
            margin_amount,
            leverage,
            base_asset_limit,
        };
        self.call(msg, funds)
    }

    pub fn close_position(&self, vamm: String, quote_asset_limit: Uint128) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::ClosePosition {
            vamm,
            quote_asset_limit,
        };
        self.call(msg, vec![])
    }

    pub fn liquidate(
        &self,
        vamm: String,
        trader: String,
        quote_asset_limit: Uint128,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::Liquidate {
            vamm,
            trader,
            quote_asset_limit,
        };
        self.call(msg, vec![])
    }

    pub fn pay_funding(&self, vamm: String) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::PayFunding { vamm };
        self.call(msg, vec![])
    }

    pub fn deposit_margin(
        &self,
        vamm: String,
        amount: Uint128,
        funds: Vec<Coin>,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::DepositMargin { vamm, amount };
        self.call(msg, funds)
    }

    pub fn withdraw_margin(&self, vamm: String, amount: Uint128) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::WithdrawMargin { vamm, amount };
        self.call(msg, vec![])
    }

    pub fn add_whitelist(&self, address: String) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::AddWhitelist { address };
        self.call(msg, vec![])
    }

    pub fn remove_whitelist(&self, address: String) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::RemoveWhitelist { address };
        self.call(msg, vec![])
    }

    /// get margin engine configuration
    pub fn config(&self, querier: &QuerierWrapper) -> StdResult<ConfigResponse> {
        let msg = QueryMsg::Config {};
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        querier.query(&query)
    }

    /// get margin engine state
    pub fn state(&self, querier: &QuerierWrapper) -> StdResult<StateResponse> {
        let msg = QueryMsg::State {};
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        querier.query(&query)
    }

    /// get traders position for a particular vamm
    pub fn position(
        &self,
        querier: &QuerierWrapper,
        vamm: String,
        trader: String,
    ) -> StdResult<Position> {
        let msg = QueryMsg::Position { vamm, trader };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        querier.query(&query)
    }

    /// get traders positions for all registered vamms
    pub fn get_all_positions(
        &self,
        querier: &QuerierWrapper,
        trader: String,
    ) -> StdResult<Vec<Position>> {
        let msg = QueryMsg::AllPositions { trader };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        querier.query(&query)
    }

    /// get the whitelist
    pub fn get_whitelist(&self, querier: &QuerierWrapper) -> StdResult<Vec<String>> {
        let msg = QueryMsg::GetWhitelist {};
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        querier.query::<HooksResponse>(&query).map(|res| res.hooks)
    }

    /// checks if the address supplied is in the whitelist
    pub fn is_whitelist(&self, querier: &QuerierWrapper, address: String) -> StdResult<bool> {
        let msg = QueryMsg::IsWhitelisted { address };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        querier.query(&query)
    }

    /// get unrealized profit and loss for a position
    pub fn get_unrealized_pnl(
        &self,
        querier: &QuerierWrapper,
        vamm: String,
        trader: String,
        calc_option: PnlCalcOption,
    ) -> StdResult<PositionUnrealizedPnlResponse> {
        let msg = QueryMsg::UnrealizedPnl {
            vamm,
            trader,
            calc_option,
        };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        querier.query(&query)
    }

    /// get free collateral
    pub fn get_free_collateral(
        &self,
        querier: &QuerierWrapper,
        vamm: String,
        trader: String,
    ) -> StdResult<Integer> {
        let msg = QueryMsg::FreeCollateral { vamm, trader };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        querier.query(&query)
    }

    /// get margin ratio
    pub fn get_margin_ratio(
        &self,
        querier: &QuerierWrapper,
        vamm: String,
        trader: String,
    ) -> StdResult<Integer> {
        let msg = QueryMsg::MarginRatio { vamm, trader };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        querier.query(&query)
    }

    /// get traders margin balance
    pub fn get_balance_with_funding_payment(
        &self,
        querier: &QuerierWrapper,
        trader: String,
    ) -> StdResult<Uint128> {
        let msg = QueryMsg::BalanceWithFundingPayment { trader };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        querier.query(&query)
    }

    /// get personal position with funding payment
    pub fn get_position_with_funding_payment(
        &self,
        querier: &QuerierWrapper,
        vamm: String,
        trader: String,
    ) -> StdResult<Position> {
        let msg = QueryMsg::PositionWithFundingPayment { vamm, trader };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        querier.query(&query)
    }

    /// get the latest cumulative premium fraction
    pub fn get_latest_cumulative_premium_fraction(
        &self,
        querier: &QuerierWrapper,
        vamm: String,
    ) -> StdResult<Integer> {
        let msg = QueryMsg::CumulativePremiumFraction { vamm };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        querier.query(&query)
    }
}
