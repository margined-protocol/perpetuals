use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use strum_macros::Display;

use margined_common::integer::Integer;

#[derive(Serialize, Display, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    AddToAmm,
    RemoveFromAmm,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    pub decimals: u8,
    pub pricefeed: String,
    pub margin_engine: Option<String>,
    pub insurance_fund: Option<String>,
    pub quote_asset: String,
    pub base_asset: String,
    pub quote_asset_reserve: Uint128,
    pub base_asset_reserve: Uint128,
    pub funding_period: u64,
    pub toll_ratio: Uint128,
    pub spread_ratio: Uint128,
    pub fluctuation_limit_ratio: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteMsg {
    UpdateConfig {
        base_asset_holding_cap: Option<Uint128>,
        open_interest_notional_cap: Option<Uint128>,
        toll_ratio: Option<Uint128>,
        spread_ratio: Option<Uint128>,
        fluctuation_limit_ratio: Option<Uint128>,
        margin_engine: Option<String>,
        insurance_fund: Option<String>,
        pricefeed: Option<String>,
        spot_price_twap_interval: Option<u64>,
    },
    UpdateOwner {
        owner: String,
    },
    SwapInput {
        direction: Direction,
        quote_asset_amount: Uint128,
        base_asset_limit: Uint128,
        can_go_over_fluctuation: bool,
    },
    SwapOutput {
        direction: Direction,
        base_asset_amount: Uint128,
        quote_asset_limit: Uint128,
    },
    SettleFunding {},
    SetOpen {
        open: bool,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {},
    GetOwner {},
    InputPrice {
        direction: Direction,
        amount: Uint128,
    },
    OutputPrice {
        direction: Direction,
        amount: Uint128,
    },
    InputAmount {
        direction: Direction,
        amount: Uint128,
    },
    OutputAmount {
        direction: Direction,
        amount: Uint128,
    },
    InputTwap {
        direction: Direction,
        amount: Uint128,
    },
    OutputTwap {
        direction: Direction,
        amount: Uint128,
    },
    SpotPrice {},
    TwapPrice {
        interval: u64,
    },
    UnderlyingPrice {},
    UnderlyingTwapPrice {
        interval: u64,
    },
    CalcFee {
        quote_asset_amount: Uint128,
    },
    IsOverSpreadLimit {},
    IsOverFluctuationLimit {
        direction: Direction,
        base_asset_amount: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ConfigResponse {
    pub base_asset_holding_cap: Uint128,
    pub open_interest_notional_cap: Uint128,
    pub margin_engine: Addr,
    pub insurance_fund: Addr,
    pub pricefeed: Addr,
    pub quote_asset: String,
    pub base_asset: String,
    pub toll_ratio: Uint128,
    pub spread_ratio: Uint128,
    pub fluctuation_limit_ratio: Uint128,
    pub decimals: Uint128,
    pub funding_period: u64,
    pub spot_price_twap_interval: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct OwnerResponse {
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct StateResponse {
    pub open: bool,
    pub quote_asset_reserve: Uint128,
    pub base_asset_reserve: Uint128,
    pub total_position_size: Integer,
    pub funding_rate: Integer,
    pub next_funding_time: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CalcFeeResponse {
    pub toll_fee: Uint128,
    pub spread_fee: Uint128,
}
