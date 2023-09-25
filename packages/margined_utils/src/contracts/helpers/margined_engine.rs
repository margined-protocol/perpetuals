use cosmwasm_schema::cw_serde;
use cw_controllers::HooksResponse;
use margined_perp::margined_engine::{
    ConfigResponse, ExecuteMsg, PnlCalcOption, Position, PositionFilter,
    PositionUnrealizedPnlResponse, QueryMsg, Side, StateResponse, TickResponse, TicksResponse, PositionTpSlResponse,
};

use cosmwasm_std::{Addr, Coin, CosmosMsg, QuerierWrapper, StdResult, Uint128};

use margined_common::integer::Integer;

use margined_common::messages::wasm_execute;

/// EngineController is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[cw_serde]
pub struct EngineController(pub Addr);

impl EngineController {
    pub fn addr(&self) -> Addr {
        self.0.clone()
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
        tp_sl_spread: Option<Uint128>,
        liquidation_fee: Option<Uint128>,
    ) -> StdResult<CosmosMsg> {
        wasm_execute(
            &self.0,
            &ExecuteMsg::UpdateConfig {
                owner,
                insurance_fund,
                fee_pool,
                initial_margin_ratio,
                maintenance_margin_ratio,
                partial_liquidation_ratio,
                tp_sl_spread,
                liquidation_fee,
            },
            vec![],
        )
    }

    pub fn set_initial_margin_ratio(&self, initial_margin_ratio: Uint128) -> StdResult<CosmosMsg> {
        wasm_execute(
            &self.0,
            &ExecuteMsg::UpdateConfig {
                owner: None,
                insurance_fund: None,
                fee_pool: None,
                initial_margin_ratio: Some(initial_margin_ratio),
                maintenance_margin_ratio: None,
                partial_liquidation_ratio: None,
                tp_sl_spread: None,
                liquidation_fee: None,
            },
            vec![],
        )
    }

    pub fn set_maintenance_margin_ratio(
        &self,
        maintenance_margin_ratio: Uint128,
    ) -> StdResult<CosmosMsg> {
        wasm_execute(
            &self.0,
            &ExecuteMsg::UpdateConfig {
                owner: None,
                insurance_fund: None,
                fee_pool: None,
                initial_margin_ratio: None,
                maintenance_margin_ratio: Some(maintenance_margin_ratio),
                partial_liquidation_ratio: None,
                tp_sl_spread: None,
                liquidation_fee: None,
            },
            vec![],
        )
    }

    pub fn set_margin_ratios(&self, margin_ratio: Uint128) -> StdResult<CosmosMsg> {
        wasm_execute(
            &self.0,
            &ExecuteMsg::UpdateConfig {
                owner: None,
                insurance_fund: None,
                fee_pool: None,
                initial_margin_ratio: Some(margin_ratio),
                maintenance_margin_ratio: Some(margin_ratio),
                partial_liquidation_ratio: None,
                tp_sl_spread: None,
                liquidation_fee: None,
            },
            vec![],
        )
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
            tp_sl_spread: None,
            liquidation_fee: None,
        };
        wasm_execute(&self.0, &msg, vec![])
    }

    pub fn set_liquidation_fee(&self, liquidation_fee: Uint128) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::UpdateConfig {
            owner: None,
            insurance_fund: None,
            fee_pool: None,
            initial_margin_ratio: None,
            maintenance_margin_ratio: None,
            partial_liquidation_ratio: None,
            tp_sl_spread: None,
            liquidation_fee: Some(liquidation_fee),
        };
        wasm_execute(&self.0, &msg, vec![])
    }

    pub fn set_pause(&self, pause: bool) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::SetPause { pause };
        wasm_execute(&self.0, &msg, vec![])
    }

    pub fn open_position(
        &self,
        vamm: String,
        side: Side,
        margin_amount: Uint128,
        leverage: Uint128,
        take_profit: Uint128,
        stop_loss: Option<Uint128>,
        base_asset_limit: Uint128,
        funds: Vec<Coin>,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::OpenPosition {
            vamm,
            side,
            margin_amount,
            leverage,
            base_asset_limit,
            take_profit,
            stop_loss,
        };
        wasm_execute(&self.0, &msg, funds)
    }

    pub fn close_position(
        &self,
        vamm: String,
        position_id: u64,
        quote_asset_limit: Uint128,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::ClosePosition {
            vamm,
            position_id,
            quote_asset_limit,
        };
        wasm_execute(&self.0, &msg, vec![])
    }

    pub fn liquidate(
        &self,
        vamm: String,
        position_id: u64,
        quote_asset_limit: Uint128,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::Liquidate {
            vamm,
            position_id,
            quote_asset_limit,
        };
        wasm_execute(&self.0, &msg, vec![])
    }

    pub fn pay_funding(&self, vamm: String) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::PayFunding { vamm };
        wasm_execute(&self.0, &msg, vec![])
    }

    pub fn deposit_margin(
        &self,
        vamm: String,
        position_id: u64,
        amount: Uint128,
        funds: Vec<Coin>,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::DepositMargin {
            vamm,
            position_id,
            amount,
        };
        wasm_execute(&self.0, &msg, funds)
    }

    pub fn withdraw_margin(
        &self,
        vamm: String,
        position_id: u64,
        amount: Uint128,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::WithdrawMargin {
            vamm,
            position_id,
            amount,
        };
        wasm_execute(&self.0, &msg, vec![])
    }

    pub fn update_tp_sl(
        &self,
        vamm: String,
        position_id: u64,
        take_profit: Option<Uint128>,
        stop_loss: Option<Uint128>,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::UpdateTpSl {
            vamm,
            position_id,
            take_profit,
            stop_loss,
        };
        wasm_execute(&self.0, &msg, vec![])
    }

    pub fn trigger_tp_sl(
        &self,
        vamm: String,
        position_id: u64,
        quote_asset_limit: Uint128,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::TriggerTpSl {
            vamm,
            position_id,
            quote_asset_limit,
        };
        wasm_execute(&self.0, &msg, vec![])
    }

    pub fn add_whitelist(&self, address: String) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::AddWhitelist { address };
        wasm_execute(&self.0, &msg, vec![])
    }

    pub fn remove_whitelist(&self, address: String) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::RemoveWhitelist { address };
        wasm_execute(&self.0, &msg, vec![])
    }

    /// get margin engine configuration
    pub fn config(&self, querier: &QuerierWrapper) -> StdResult<ConfigResponse> {
        let msg = QueryMsg::Config {};

        querier.query_wasm_smart(&self.0, &msg)
    }

    /// get margin engine state
    pub fn state(&self, querier: &QuerierWrapper) -> StdResult<StateResponse> {
        let msg = QueryMsg::State {};

        querier.query_wasm_smart(&self.0, &msg)
    }

    /// get traders position for a particular vamm
    pub fn position(
        &self,
        querier: &QuerierWrapper,
        vamm: String,
        position_id: u64,
    ) -> StdResult<Position> {
        let msg = QueryMsg::Position { vamm, position_id };

        querier.query_wasm_smart(&self.0, &msg)
    }

    /// get traders positions for all registered vamms
    pub fn get_all_positions(
        &self,
        querier: &QuerierWrapper,
        trader: String,
        start_after: Option<u64>,
        limit: Option<u32>,
        order_by: Option<i32>,
    ) -> StdResult<Vec<Position>> {
        let msg = QueryMsg::AllPositions {
            trader,
            start_after,
            limit,
            order_by,
        };

        querier.query_wasm_smart(&self.0, &msg)
    }

    /// get positions from vamm
    pub fn get_positions(
        &self,
        querier: &QuerierWrapper,
        vamm: String,
        filter: PositionFilter,
        side: Option<Side>,
        start_after: Option<u64>,
        limit: Option<u32>,
        order_by: Option<i32>,
    ) -> StdResult<Vec<Position>> {
        let msg = QueryMsg::Positions {
            vamm,
            filter,
            side,
            start_after,
            limit,
            order_by,
        };

        querier.query_wasm_smart(&self.0, &msg)
    }

    /// get price tick from vamm
    pub fn get_tick(
        &self,
        querier: &QuerierWrapper,
        vamm: String,
        side: Side,
        entry_price: Uint128,
    ) -> StdResult<TickResponse> {
        let msg = QueryMsg::Tick {
            vamm,
            side,
            entry_price,
        };

        querier.query_wasm_smart(&self.0, &msg)
    }

    /// get price ticks from vamm
    pub fn get_ticks(
        &self,
        querier: &QuerierWrapper,
        vamm: String,
        side: Side,
        start_after: Option<Uint128>,
        limit: Option<u32>,
        order_by: Option<i32>,
    ) -> StdResult<TicksResponse> {
        let msg = QueryMsg::Ticks {
            vamm,
            side,
            start_after,
            limit,
            order_by,
        };

        querier.query_wasm_smart(&self.0, &msg)
    }

    /// get the whitelist
    pub fn get_whitelist(&self, querier: &QuerierWrapper) -> StdResult<Vec<String>> {
        let msg = QueryMsg::GetWhitelist {};

        querier
            .query_wasm_smart::<HooksResponse>(&self.0, &msg)
            .map(|res| res.hooks)
    }

    /// checks if the address supplied is in the whitelist
    pub fn is_whitelist(&self, querier: &QuerierWrapper, address: String) -> StdResult<bool> {
        let msg = QueryMsg::IsWhitelisted { address };

        querier.query_wasm_smart(&self.0, &msg)
    }

    /// get unrealized profit and loss for a position
    pub fn get_unrealized_pnl(
        &self,
        querier: &QuerierWrapper,
        vamm: String,
        position_id: u64,
        calc_option: PnlCalcOption,
    ) -> StdResult<PositionUnrealizedPnlResponse> {
        let msg = QueryMsg::UnrealizedPnl {
            vamm,
            position_id,
            calc_option,
        };

        querier.query_wasm_smart(&self.0, &msg)
    }

    /// get free collateral
    pub fn get_free_collateral(
        &self,
        querier: &QuerierWrapper,
        vamm: String,
        position_id: u64,
    ) -> StdResult<Integer> {
        let msg = QueryMsg::FreeCollateral { vamm, position_id };

        querier.query_wasm_smart(&self.0, &msg)
    }

    /// get margin ratio
    pub fn get_margin_ratio(
        &self,
        querier: &QuerierWrapper,
        vamm: String,
        position_id: u64,
    ) -> StdResult<Integer> {
        let msg = QueryMsg::MarginRatio { vamm, position_id };

        querier.query_wasm_smart(&self.0, &msg)
    }

    /// get traders margin balance
    pub fn get_balance_with_funding_payment(
        &self,
        querier: &QuerierWrapper,
        position_id: u64,
    ) -> StdResult<Uint128> {
        let msg = QueryMsg::BalanceWithFundingPayment { position_id };

        querier.query_wasm_smart(&self.0, &msg)
    }

    /// get personal position with funding payment
    pub fn get_position_with_funding_payment(
        &self,
        querier: &QuerierWrapper,
        vamm: String,
        position_id: u64,
    ) -> StdResult<Position> {
        let msg = QueryMsg::PositionWithFundingPayment { vamm, position_id };

        querier.query_wasm_smart(&self.0, &msg)
    }

    /// get the latest cumulative premium fraction
    pub fn get_latest_cumulative_premium_fraction(
        &self,
        querier: &QuerierWrapper,
        vamm: String,
    ) -> StdResult<Integer> {
        let msg = QueryMsg::CumulativePremiumFraction { vamm };

        querier.query_wasm_smart(&self.0, &msg)
    }

    /// get the latest cumulative premium fraction
    pub fn get_tp_sl_status(
        &self,
        querier: &QuerierWrapper,
        vamm: String,
        position_id: u64,
    ) -> StdResult<PositionTpSlResponse> {
        let msg = QueryMsg::PositionIsTpSL { vamm, position_id };

        querier.query_wasm_smart(&self.0, &msg)
    }
}
