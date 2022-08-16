use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    AddToAmm,
    RemoveFromAmm,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    pub oracle_hub_contract: String, // address of the oracle hub we are using
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AppendPrice {
        key: String,
        price: Uint128,
        timestamp: u64,
    },
    AppendMultiplePrice {
        key: String,
        prices: Vec<Uint128>,
        timestamps: Vec<u64>,
    },
    UpdateConfig {
        owner: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    GetPrice {
        key: String,
    },
    GetPreviousPrice {
        key: String,
        num_round_back: Uint128,
    },
    GetTwapPrice {
        key: String,
        interval: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: Addr,
}
