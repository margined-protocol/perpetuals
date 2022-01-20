use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{Addr, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    SHORT,
    LONG
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub decimals: u8,
    pub quote_asset: String,
    pub base_asset: String,
    pub quote_asset_reserve: Uint256,
    pub base_asset_reserve: Uint256,
    pub funding_period: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SwapInput {
        direction: Direction,
        quote_asset_amount: Uint256,
    },
    SwapOutput {
        direction: Direction,
        base_asset_amount: Uint256,
    },
    UpdateConfig {
        owner: String,
        decimals: u8,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: Addr,
    pub decimals: u8,
    pub quote_asset: String,
    pub base_asset: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub quote_asset_reserve: Uint256,
    pub base_asset_reserve: Uint256,
    pub funding_rate: Uint256,
    pub funding_period: Uint128,
}
