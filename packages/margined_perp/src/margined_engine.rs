use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Timestamp };
use cw20::Cw20ReceiveMsg;
use cosmwasm_bignumber::Decimal256;

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
    pub eligible_collateral: String,
    pub initial_margin_ratio: Decimal256,
    pub maintenance_margin_ratio: Decimal256,
    pub liquidation_fee: Decimal256,
    pub vamm: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    UpdateConfig {
        owner: String,
    },
    OpenPosition {
        vamm: String,
        side: Side,
        quote_asset_amount: Decimal256,
        leverage: Decimal256,
    },
    ClosePosition {
        vamm: String,
    },
    // Liquidate {},
    // PayFunding {},
    // DepositMargin {},
    // WithdrawMargin {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    // allows you to open a position and directly transfer funds
    OpenPosition {
        vamm: String,
        side: Side,
        leverage: Decimal256,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Position { vamm: String, trader: String },
    TraderBalance { trader: String },
    UnrealizedPnl { vamm: String, trader: String },
    // MarginRatio {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: Addr,
    pub eligible_collateral: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PositionResponse {
    pub size: Decimal256,
    pub margin: Decimal256,
    pub notional: Decimal256,
    pub premium_fraction: Decimal256,
    pub liquidity_history_index: Decimal256,
    pub timestamp: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SwapResponse {
    pub vamm: String,
    pub trader: String,
    pub side: String,
    pub quote_asset_amount: Decimal256,
    pub leverage: Decimal256,
    pub open_notional: Decimal256,
    pub input: Decimal256,
    pub output: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PositionUnrealizedPnlResponse {
    pub position_notional: Decimal256,
    pub unrealized_pnl: Decimal256,
}
