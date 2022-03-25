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

    pub fn update_config(
        &self,
        owner: Option<String>,
        toll_ratio: Option<Uint128>,
        spread_ratio: Option<Uint128>,
        fluctuation_limit_ratio: Option<Uint128>,
        margin_engine: Option<String>,
        pricefeed: Option<String>,
        spot_price_twap_interval: Option<u64>,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::UpdateConfig {
            owner,
            toll_ratio,
            spread_ratio,
            fluctuation_limit_ratio,
            margin_engine,
            pricefeed,
            spot_price_twap_interval,
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
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::SwapInput {
            direction,
            quote_asset_amount,
        };
        self.call(msg, vec![])
    }

    pub fn swap_output(
        &self,
        direction: Direction,
        base_asset_amount: Uint128,
    ) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::SwapOutput {
            direction,
            base_asset_amount,
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
}
