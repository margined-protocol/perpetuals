use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{Addr, Api, StdResult, Uint128};
use cw_storage_plus::Item;

use margined_perp::margined_vamm::{
    ConfigResponse, StateResponse,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub decimals: u8,
    pub quote_asset: String,
    pub base_asset: String,
}

impl Config {
    pub fn as_res(&self, _api: &dyn Api) -> StdResult<ConfigResponse> {
        let res = ConfigResponse {
            owner: self.owner.clone(),
            decimals: self.decimals,
            quote_asset: self.quote_asset.clone(),
            base_asset: self.base_asset.clone(),
        };
        Ok(res)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub quote_asset_reserve: Uint256,
    pub base_asset_reserve: Uint256,
    pub funding_rate: Uint256,
    pub funding_period: Uint128,
}

impl State {
    pub fn as_res(&self, _api: &dyn Api) -> StdResult<StateResponse> {
        let res = StateResponse {
            quote_asset_reserve: self.quote_asset_reserve,
            base_asset_reserve: self.base_asset_reserve,
            funding_rate: self.funding_rate,
            funding_period: self.funding_period,
        };
        Ok(res)
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
