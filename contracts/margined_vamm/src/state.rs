use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Uint256};
use cosmwasm_std::{Addr, Api, StdResult};
use cw_storage_plus::Item;

use margined_perp::margined_vamm::{
    ConfigResponse,
};

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");

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
}

impl State {}
