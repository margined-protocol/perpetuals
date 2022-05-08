use crate::margined_vamm::Direction;
use cosmwasm_std::{Addr, SubMsg, Uint128};
use margined_common::integer::Integer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terraswap::asset::AssetInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PnlCalcOption {
    SpotPrice,
    Twap,
    Oracle,
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
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // Receive(Cw20ReceiveMsg),
    UpdateConfig {
        owner: Option<String>,
        insurance_fund: Option<String>,
        fee_pool: Option<String>,
        eligible_collateral: Option<String>,
        decimals: Option<Uint128>,
        initial_margin_ratio: Option<Uint128>,
        maintenance_margin_ratio: Option<Uint128>,
        partial_liquidation_margin_ratio: Option<Uint128>,
        liquidation_fee: Option<Uint128>,
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
        quote_asset_limit: Uint128,
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
    SetPause {
        pause: bool,
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
    State {},
    Position {
        vamm: String,
        trader: String,
    },
    AllPositions {
        trader: String,
    },
    UnrealizedPnl {
        vamm: String,
        trader: String,
        calc_option: PnlCalcOption,
    },
    CumulativePremiumFraction {
        vamm: String,
    },
    MarginRatio {
        vamm: String,
        trader: String,
    },
    FreeCollateral {
        vamm: String,
        trader: String,
    },
    BalanceWithFundingPayment {
        trader: String,
    },
    PositionWithFundingPayment {
        vamm: String,
        trader: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: Addr,
    pub eligible_collateral: AssetInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub open_interest_notional: Uint128,
    pub bad_debt: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Position {
    pub vamm: Addr,
    pub trader: Addr,
    pub direction: Direction,
    pub size: Integer,
    pub margin: Uint128,
    pub notional: Uint128,
    pub last_updated_premium_fraction: Integer,
    pub block_number: u64,
}

impl Default for Position {
    fn default() -> Position {
        Position {
            vamm: Addr::unchecked(""),
            trader: Addr::unchecked(""),
            direction: Direction::AddToAmm,
            size: Integer::zero(),
            margin: Uint128::zero(),
            notional: Uint128::zero(),
            last_updated_premium_fraction: Integer::zero(),
            block_number: 0u64,
        }
    }
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
    pub unrealized_pnl: Integer,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RemainMarginResponse {
    pub funding_payment: Integer,
    pub margin: Uint128,
    pub bad_debt: Uint128,
    pub latest_premium_fraction: Integer,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TransferResponse {
    pub messages: Vec<SubMsg>,
    pub amount: Uint128,
}
