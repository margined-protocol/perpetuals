use cosmwasm_schema::cw_serde;
use margined_perp::margined_vamm::{
    CalcFeeResponse, ConfigResponse, Direction, ExecuteMsg, QueryMsg, StateResponse,
};

use cosmwasm_std::{Addr, CosmosMsg, QuerierWrapper, StdResult, Uint128};

use margined_common::messages::wasm_execute;

/// VammController is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[cw_serde]
pub struct VammController(pub Addr);

impl VammController {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_config(
        &self,
        base_asset_holding_cap: Option<Uint128>,
        open_interest_notional_cap: Option<Uint128>,
        toll_ratio: Option<Uint128>,
        spread_ratio: Option<Uint128>,
        fluctuation_limit_ratio: Option<Uint128>,
        margin_engine: Option<String>,
        insurance_fund: Option<String>,
        pricefeed: Option<String>,
        spot_price_twap_interval: Option<u64>,
    ) -> StdResult<CosmosMsg> {
        wasm_execute(
            &self.0,
            &ExecuteMsg::UpdateConfig {
                base_asset_holding_cap,
                open_interest_notional_cap,
                toll_ratio,
                spread_ratio,
                fluctuation_limit_ratio,
                margin_engine,
                insurance_fund,
                pricefeed,
                spot_price_twap_interval,
            },
            vec![],
        )
    }

    pub fn update_owner(&self, owner: String) -> StdResult<CosmosMsg> {
        wasm_execute(&self.0, &ExecuteMsg::UpdateOwner { owner }, vec![])
    }

    pub fn set_toll_ratio(&self, toll_ratio: Uint128) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::UpdateConfig {
            base_asset_holding_cap: None,
            open_interest_notional_cap: None,
            toll_ratio: Some(toll_ratio),
            spread_ratio: None,
            fluctuation_limit_ratio: None,
            margin_engine: None,
            insurance_fund: None,
            pricefeed: None,
            spot_price_twap_interval: None,
        };
        wasm_execute(&self.0, &msg, vec![])
    }

    pub fn set_spread_ratio(&self, spread_ratio: Uint128) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::UpdateConfig {
            base_asset_holding_cap: None,
            open_interest_notional_cap: None,
            toll_ratio: None,
            spread_ratio: Some(spread_ratio),
            fluctuation_limit_ratio: None,
            margin_engine: None,
            insurance_fund: None,
            pricefeed: None,
            spot_price_twap_interval: None,
        };
        wasm_execute(&self.0, &msg, vec![])
    }

    pub fn set_open_interest_notional_cap(
        &self,
        open_interest_notional_cap: Uint128,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::UpdateConfig {
            base_asset_holding_cap: None,
            open_interest_notional_cap: Some(open_interest_notional_cap),
            toll_ratio: None,
            spread_ratio: None,
            fluctuation_limit_ratio: None,
            margin_engine: None,
            insurance_fund: None,
            pricefeed: None,
            spot_price_twap_interval: None,
        };
        wasm_execute(&self.0, &msg, vec![])
    }

    pub fn set_base_asset_holding_cap(
        &self,
        base_asset_holding_cap: Uint128,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::UpdateConfig {
            base_asset_holding_cap: Some(base_asset_holding_cap),
            open_interest_notional_cap: None,
            toll_ratio: None,
            spread_ratio: None,
            fluctuation_limit_ratio: None,
            margin_engine: None,
            insurance_fund: None,
            pricefeed: None,
            spot_price_twap_interval: None,
        };
        wasm_execute(&self.0, &msg, vec![])
    }

    pub fn set_fluctuation_limit_ratio(
        &self,
        fluctuation_limit_ratio: Uint128,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::UpdateConfig {
            base_asset_holding_cap: None,
            open_interest_notional_cap: None,
            toll_ratio: None,
            spread_ratio: None,
            fluctuation_limit_ratio: Some(fluctuation_limit_ratio),
            margin_engine: None,
            insurance_fund: None,
            pricefeed: None,
            spot_price_twap_interval: None,
        };
        wasm_execute(&self.0, &msg, vec![])
    }

    pub fn set_open(&self, open: bool) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::SetOpen { open };
        wasm_execute(&self.0, &msg, vec![])
    }

    pub fn swap_input(
        &self,
        direction: Direction,
        position_id: u64,
        quote_asset_amount: Uint128,
        base_asset_limit: Uint128,
        can_go_over_fluctuation: bool,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::SwapInput {
            direction,
            position_id,
            quote_asset_amount,
            base_asset_limit,
            can_go_over_fluctuation,
        };
        wasm_execute(&self.0, &msg, vec![])
    }

    pub fn swap_output(
        &self,
        direction: Direction,
        position_id: u64,
        base_asset_amount: Uint128,
        quote_asset_limit: Uint128,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::SwapOutput {
            direction,
            position_id,
            base_asset_amount,
            quote_asset_limit,
        };
        wasm_execute(&self.0, &msg, vec![])
    }

    pub fn settle_funding(&self) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::SettleFunding {};
        wasm_execute(&self.0, &msg, vec![])
    }

    /// get margin vamm configuration
    pub fn config(&self, querier: &QuerierWrapper) -> StdResult<ConfigResponse> {
        querier.query_wasm_smart(&self.0, &QueryMsg::Config {})
    }

    /// get margin vamm state
    pub fn state(&self, querier: &QuerierWrapper) -> StdResult<StateResponse> {
        querier.query_wasm_smart(&self.0, &QueryMsg::State {})
    }

    /// get output price
    pub fn output_price(
        &self,
        querier: &QuerierWrapper,
        direction: Direction,
        amount: Uint128,
    ) -> StdResult<Uint128> {
        querier.query_wasm_smart(&self.0, &QueryMsg::OutputPrice { direction, amount })
    }

    /// get spot price
    pub fn spot_price(&self, querier: &QuerierWrapper) -> StdResult<Uint128> {
        querier.query_wasm_smart(&self.0, &QueryMsg::SpotPrice {})
    }

    /// get twap price
    pub fn twap_price(&self, querier: &QuerierWrapper, interval: u64) -> StdResult<Uint128> {
        let msg = QueryMsg::TwapPrice { interval };

        querier.query_wasm_smart(&self.0, &msg)
    }

    /// get swap fees
    pub fn calc_fee(
        &self,
        querier: &QuerierWrapper,
        quote_asset_amount: Uint128,
    ) -> StdResult<CalcFeeResponse> {
        querier.query_wasm_smart(&self.0, &QueryMsg::CalcFee { quote_asset_amount })
    }

    /// returns bool if vamm is over spread limit
    pub fn is_over_spread_limit(&self, querier: &QuerierWrapper) -> StdResult<bool> {
        querier.query_wasm_smart(&self.0, &QueryMsg::IsOverSpreadLimit {})
    }

    // returns the state of the request vamm
    // can be used to calculate the input and outputs
    pub fn output_amount(
        &self,
        querier: &QuerierWrapper,
        direction: Direction,
        amount: Uint128,
    ) -> StdResult<Uint128> {
        querier.query_wasm_smart(&self.0, &QueryMsg::OutputAmount { direction, amount })
    }

    // returns the state of the request vamm
    // can be used to calculate the input and outputs
    pub fn output_twap(
        &self,
        querier: &QuerierWrapper,
        direction: Direction,
        amount: Uint128,
    ) -> StdResult<Uint128> {
        querier.query_wasm_smart(&self.0, &QueryMsg::OutputTwap { direction, amount })
    }

    // returns pricefeed price of underlying in vamm
    pub fn underlying_price(&self, querier: &QuerierWrapper) -> StdResult<Uint128> {
        querier.query_wasm_smart(&self.0, &QueryMsg::UnderlyingPrice {})
    }

    // returns bool if swap is over fluctuation limit
    pub fn is_over_fluctuation_limit(
        &self,
        querier: &QuerierWrapper,
        direction: Direction,
        base_asset_amount: Uint128,
    ) -> StdResult<bool> {
        querier.query_wasm_smart(
            &self.0,
            &QueryMsg::IsOverFluctuationLimit {
                direction,
                base_asset_amount,
            },
        )
    }
}
