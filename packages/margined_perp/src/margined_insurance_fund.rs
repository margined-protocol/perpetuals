use margined_common::asset::AssetInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    pub beneficiary: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig { owner: Option<String> },
    AddVamm { vamm: String },
    RemoveVamm { vamm: String },
    Withdraw { token: AssetInfo, amount: Uint128 },
    ShutdownVamms {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    IsVamm { vamm: String },
    GetAllVamm { limit: Option<u32> },
    GetAllVammStatus { limit: Option<u32> },
    GetVammStatus { vamm: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: Addr,
    pub beneficiary: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct VammResponse {
    pub is_vamm: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct VammStatusResponse {
    pub vamm_status: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AllVammResponse {
    pub vamm_list: Vec<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AllVammStatusResponse {
    pub vamm_list_status: Vec<(Addr, bool)>,
}
