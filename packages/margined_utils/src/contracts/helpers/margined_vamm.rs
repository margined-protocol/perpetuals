use margined_perp::margined_vamm::{
    CalcFeeResponse, ConfigResponse, Direction, ExecuteMsg, QueryMsg, StateResponse,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    to_binary, Addr, Coin, CosmosMsg, Querier, QuerierWrapper, StdResult, Uint128, WasmMsg,
    WasmQuery,
};

/// VammController is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VammController(pub Addr);

impl VammController {
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
        base_asset_holding_cap: Option<Uint128>,
        open_interest_notional_cap: Option<Uint128>,
        toll_ratio: Option<Uint128>,
        spread_ratio: Option<Uint128>,
        fluctuation_limit_ratio: Option<Uint128>,
        margin_engine: Option<String>,
        pricefeed: Option<String>,
        spot_price_twap_interval: Option<u64>,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::UpdateConfig {
            owner,
            base_asset_holding_cap,
            open_interest_notional_cap,
            toll_ratio,
            spread_ratio,
            fluctuation_limit_ratio,
            margin_engine,
            pricefeed,
            spot_price_twap_interval,
        };
        self.call(msg, vec![])
    }

    pub fn set_toll_ratio(&self, toll_ratio: Uint128) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::UpdateConfig {
            owner: None,
            base_asset_holding_cap: None,
            open_interest_notional_cap: None,
            toll_ratio: Some(toll_ratio),
            spread_ratio: None,
            fluctuation_limit_ratio: None,
            margin_engine: None,
            pricefeed: None,
            spot_price_twap_interval: None,
        };
        self.call(msg, vec![])
    }

    pub fn set_spread_ratio(&self, spread_ratio: Uint128) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::UpdateConfig {
            owner: None,
            base_asset_holding_cap: None,
            open_interest_notional_cap: None,
            toll_ratio: None,
            spread_ratio: Some(spread_ratio),
            fluctuation_limit_ratio: None,
            margin_engine: None,
            pricefeed: None,
            spot_price_twap_interval: None,
        };
        self.call(msg, vec![])
    }

    pub fn set_open_interest_notional_cap(
        &self,
        open_interest_notional_cap: Uint128,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::UpdateConfig {
            owner: None,
            base_asset_holding_cap: None,
            open_interest_notional_cap: Some(open_interest_notional_cap),
            toll_ratio: None,
            spread_ratio: None,
            fluctuation_limit_ratio: None,
            margin_engine: None,
            pricefeed: None,
            spot_price_twap_interval: None,
        };
        self.call(msg, vec![])
    }

    pub fn set_fluctuation_limit_ratio(
        &self,
        fluctuation_limit_ratio: Uint128,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::UpdateConfig {
            owner: None,
            base_asset_holding_cap: None,
            open_interest_notional_cap: None,
            toll_ratio: None,
            spread_ratio: None,
            fluctuation_limit_ratio: Some(fluctuation_limit_ratio),
            margin_engine: None,
            pricefeed: None,
            spot_price_twap_interval: None,
        };
        self.call(msg, vec![])
    }

    pub fn set_open(&self, open: bool) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::SetOpen { open };
        self.call(msg, vec![])
    }

    pub fn swap_input(
        &self,
        direction: Direction,
        quote_asset_amount: Uint128,
        base_asset_limit: Uint128,
        can_go_over_fluctuation: bool,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::SwapInput {
            direction,
            quote_asset_amount,
            base_asset_limit,
            can_go_over_fluctuation,
        };
        self.call(msg, vec![])
    }

    pub fn swap_output(
        &self,
        direction: Direction,
        base_asset_amount: Uint128,
        quote_asset_limit: Uint128,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::SwapOutput {
            direction,
            base_asset_amount,
            quote_asset_limit,
        };
        self.call(msg, vec![])
    }

    pub fn settle_funding(&self) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::SettleFunding {};
        self.call(msg, vec![])
    }

    /// get margin vamm configuration
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

    /// get margin vamm state
    pub fn state<Q: Querier>(&self, querier: &Q) -> StdResult<StateResponse> {
        let msg = QueryMsg::State {};
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        let res: StateResponse = QuerierWrapper::new(querier).query(&query)?;
        Ok(res)
    }

    /// get output price
    pub fn output_price<Q: Querier>(
        &self,
        querier: &Q,
        direction: Direction,
        amount: Uint128,
    ) -> StdResult<Uint128> {
        let msg = QueryMsg::OutputPrice { direction, amount };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        let res: Uint128 = QuerierWrapper::new(querier).query(&query)?;
        Ok(res)
    }

    /// get spot price
    pub fn spot_price<Q: Querier>(&self, querier: &Q) -> StdResult<Uint128> {
        let msg = QueryMsg::SpotPrice {};
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        let res: Uint128 = QuerierWrapper::new(querier).query(&query)?;
        Ok(res)
    }

    /// get twap price
    pub fn twap_price<Q: Querier>(&self, querier: &Q, interval: u64) -> StdResult<Uint128> {
        let msg = QueryMsg::TwapPrice { interval };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        let res: Uint128 = QuerierWrapper::new(querier).query(&query)?;
        Ok(res)
    }

    /// get swap fees
    pub fn calc_fee<Q: Querier>(
        &self,
        querier: &Q,
        quote_asset_amount: Uint128,
    ) -> StdResult<CalcFeeResponse> {
        let msg = QueryMsg::CalcFee { quote_asset_amount };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        let res: CalcFeeResponse = QuerierWrapper::new(querier).query(&query)?;
        Ok(res)
    }

    /// is over spread limit
    pub fn is_over_spread_limit<Q: Querier>(&self, querier: &Q) -> StdResult<bool> {
        let msg = QueryMsg::IsOverSpreadLimit {};
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&msg)?,
        }
        .into();

        let res: bool = QuerierWrapper::new(querier).query(&query)?;
        Ok(res)
    }
}
