use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cosmwasm_bignumber::{Decimal256};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    AddToAmm,
    RemoveFromAmm,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub decimals: u8,
    pub quote_asset: String,
    pub base_asset: String,
    pub quote_asset_reserve: Decimal256,
    pub base_asset_reserve: Decimal256,
    pub funding_period: u64,
    pub toll_ratio: Decimal256,
    pub spread_ratio: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SwapInput {
        direction: Direction,
        quote_asset_amount: Decimal256,
    },
    SwapOutput {
        direction: Direction,
        base_asset_amount: Decimal256,
    },
    UpdateConfig {
        owner: Option<String>,
        toll_ratio: Option<Decimal256>,
        spread_ratio: Option<Decimal256>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {},
    OutputPrice {
        direction: Direction,
        amount: Decimal256,
    },
    CalcFee {
        quote_asset_amount: Decimal256,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: Addr,
    pub quote_asset: String,
    pub base_asset: String,
    pub toll_ratio: Decimal256,
    pub spread_ratio: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub quote_asset_reserve: Decimal256,
    pub base_asset_reserve: Decimal256,
    pub funding_rate: Decimal256,
    pub funding_period: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CalcFeeResponse {
    pub toll_fee: Decimal256,
    pub spread_fee: Decimal256,
}
