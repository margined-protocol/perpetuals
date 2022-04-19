use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terraswap::asset::AssetInfo;

use cosmwasm_std::{Addr, Uint128};
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        owner: Option<String>,
        beneficiary: Option<String>,
    },
    AddVamm {
        vamm: String,
    },
    RemoveVamm {
        vamm: String,
    },
    Withdraw {
        token: AssetInfo,
        amount: Uint128,
    },
    SwitchVammOn {
        vamm: String,
    },
    SwitchVammOff {
        vamm: String,
    },
    ShutdownAllVamm {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    IsVamm { vamm: String },
    GetAllVamm {},
    GetAllVammStatus {},
    GetVammStatus {vamm: String},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: Addr,
    pub beneficiary: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VammResponse {
    pub is_vamm: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VammStatusResponse {
    pub vamm_status: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AllVammResponse {
    pub vamm_list: Vec<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AllVammStatusResponse {
    pub vamm_list_status: Vec<(Addr, bool)>,
}