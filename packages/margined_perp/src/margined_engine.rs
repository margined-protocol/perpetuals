use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Timestamp, Uint128};
use margined_common::integer::Integer;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    BUY,
    SELL,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PnlCalcOption {
    SPOTPRICE,
    TWAP,
    ORACLE,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub decimals: u8,
    pub insurance_fund: String,
    pub fee_pool: String,
    pub eligible_collateral: String,
    pub initial_margin_ratio: Uint128,
    pub maintenance_margin_ratio: Uint128,
    pub liquidation_fee: Uint128,
    pub vamm: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // Receive(Cw20ReceiveMsg),
    UpdateConfig {
        owner: String,
    },
    OpenPosition {
        vamm: String,
        side: Side,
        quote_asset_amount: Uint128,
        leverage: Uint128,
        base_asset_limit: Uint128,
    },
    ClosePosition {
        vamm: String,
        quote_asset_limit: Uint128,
    },
    Liquidate {
        vamm: String,
        trader: String,
    },
    PayFunding {
        vamm: String,
    },
    DepositMargin {
        vamm: String,
        amount: Uint128,
    },
    WithdrawMargin {
        vamm: String,
        amount: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    // allows you to open a position and directly transfer funds
    OpenPosition {
        vamm: String,
        side: Side,
        leverage: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Position { vamm: String, trader: String },
    UnrealizedPnl { vamm: String, trader: String },
    CumulativePremiumFraction { vamm: String },
    MarginRatio { vamm: String, trader: String },
    BalanceWithFundingPayment { trader: String },
    PositionWithFundingPayment { vamm: String, trader: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: Addr,
    pub eligible_collateral: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MarginRatioResponse {
    pub ratio: Uint128,
    // TODO think if i128 should be used or
    // if there is a better solution to this
    pub polarity: bool, // true = positive, false = negative
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PositionResponse {
    pub size: Integer,
    pub margin: Uint128,
    pub notional: Uint128,
    pub last_updated_premium_fraction: Integer,
    pub liquidity_history_index: Uint128,
    pub timestamp: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SwapResponse {
    pub vamm: String,
    pub trader: String,
    pub side: String,
    pub quote_asset_amount: Uint128,
    pub leverage: Uint128,
    pub open_notional: Uint128,
    pub input: Uint128,
    pub output: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PositionUnrealizedPnlResponse {
    pub position_notional: Uint128,
    pub unrealized_pnl: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RemainMarginResponse {
    pub funding_payment: Integer,
    pub margin: Uint128,
    pub bad_debt: Uint128,
    pub latest_premium_fraction: Integer,
}
