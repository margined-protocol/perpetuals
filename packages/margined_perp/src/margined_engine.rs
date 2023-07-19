use crate::margined_vamm::Direction;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, SubMsg, Uint128};
use margined_common::{asset::AssetInfo, integer::Integer};

#[cw_serde]
#[derive(Copy)]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Side::Buy => &[0u8],
            Side::Sell => &[1u8],
        }
    }
}

#[cw_serde]
pub enum PnlCalcOption {
    SpotPrice,
    Twap,
    Oracle,
}

#[cw_serde]
pub enum PositionFilter {
    Trader(String), // filter by trader
    Price(Uint128), // filter by price
    None,           // no filter
}

#[cw_serde]
pub struct InstantiateMsg {
    pub pauser: String,
    pub insurance_fund: Option<String>, // insurance_fund need engine addr, so there is senario when we re-deploy engine
    pub fee_pool: String,
    pub eligible_collateral: String,
    pub initial_margin_ratio: Uint128,
    pub maintenance_margin_ratio: Uint128,
    pub liquidation_fee: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        owner: Option<String>,
        insurance_fund: Option<String>,
        fee_pool: Option<String>,
        initial_margin_ratio: Option<Uint128>,
        maintenance_margin_ratio: Option<Uint128>,
        partial_liquidation_ratio: Option<Uint128>,
        liquidation_fee: Option<Uint128>,
    },
    UpdatePauser {
        pauser: String,
    },
    AddWhitelist {
        address: String,
    },
    RemoveWhitelist {
        address: String,
    },
    OpenPosition {
        vamm: String,
        side: Side,
        margin_amount: Uint128,
        leverage: Uint128,
        take_profit: Uint128,
        stop_loss: Option<Uint128>,
        base_asset_limit: Uint128,
    },
    UpdateTpSl {
        vamm: String,
        position_id: u64,
        take_profit: Option<Uint128>,
        stop_loss: Option<Uint128>,
    },
    ClosePosition {
        vamm: String,
        position_id: u64,
        quote_asset_limit: Uint128,
    },
    TriggerTpSl {
        vamm: String,
        position_id: u64,
        quote_asset_limit: Uint128,
    },
    Liquidate {
        vamm: String,
        position_id: u64,
        trader: String,
        quote_asset_limit: Uint128,
    },
    PayFunding {
        vamm: String,
    },
    DepositMargin {
        vamm: String,
        position_id: u64,
        amount: Uint128,
    },
    WithdrawMargin {
        vamm: String,
        position_id: u64,
        amount: Uint128,
    },
    SetPause {
        pause: bool,
    },
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(StateResponse)]
    State {},
    #[returns(PauserResponse)]
    GetPauser {},
    #[returns(bool)]
    IsWhitelisted { address: String },
    #[returns(cw_controllers::HooksResponse)]
    GetWhitelist {},
    #[returns(Position)]
    Position { vamm: String, position_id: u64 },
    #[returns(Vec<Position>)]
    AllPositions {
        trader: String,
        start_after: Option<u64>,
        limit: Option<u32>,
        order_by: Option<i32>,
    },
    #[returns(Vec<Position>)]
    Positions {
        vamm: String,
        filter: PositionFilter,
        side: Option<Side>,
        start_after: Option<u64>,
        limit: Option<u32>,
        order_by: Option<i32>, 
    },
    #[returns(PositionUnrealizedPnlResponse)]
    UnrealizedPnl {
        vamm: String,
        position_id: u64,
        calc_option: PnlCalcOption,
    },
    #[returns(Integer)]
    CumulativePremiumFraction { vamm: String },
    #[returns(Integer)]
    MarginRatio { vamm: String, position_id: u64 },
    #[returns(Integer)]
    FreeCollateral { vamm: String, position_id: u64 },
    #[returns(Uint128)]
    BalanceWithFundingPayment { position_id: u64},
    #[returns(Position)]
    PositionWithFundingPayment { vamm: String, position_id: u64 },
    #[returns(LastPositionIdResponse)]
    LastPositionId {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: Addr,
    pub insurance_fund: Option<Addr>,
    pub fee_pool: Addr,
    pub eligible_collateral: AssetInfo,
    pub decimals: Uint128,
    pub initial_margin_ratio: Uint128,
    pub maintenance_margin_ratio: Uint128,
    pub partial_liquidation_ratio: Uint128,
    pub liquidation_fee: Uint128,
}

#[cw_serde]
pub struct StateResponse {
    pub open_interest_notional: Uint128,
    pub bad_debt: Uint128,
}

#[cw_serde]
pub struct PauserResponse {
    pub pauser: Addr,
}

#[cw_serde]
pub struct LastPositionIdResponse {
    pub last_order_id: u64,
}

#[cw_serde]
pub struct Position {
    pub position_id: u64,
    pub vamm: Addr,
    pub trader: Addr,
    pub side: Side,
    pub direction: Direction,
    pub size: Integer,
    pub margin: Uint128,
    pub notional: Uint128,
    pub entry_price: Uint128,
    pub take_profit: Uint128,
    pub stop_loss: Option<Uint128>,
    pub last_updated_premium_fraction: Integer,
    pub block_time: u64,
}

impl Default for Position {
    fn default() -> Position {
        Position {
            position_id: 0u64,
            vamm: Addr::unchecked(""),
            trader: Addr::unchecked(""),
            side: Side::Buy,
            direction: Direction::AddToAmm,
            size: Integer::zero(),
            margin: Uint128::zero(),
            notional: Uint128::zero(),
            entry_price: Uint128::zero(),
            take_profit: Uint128::zero(),
            stop_loss: Some(Uint128::zero()),
            last_updated_premium_fraction: Integer::zero(),
            block_time: 0u64,
        }
    }
}

#[cw_serde]
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

#[cw_serde]
pub struct PositionUnrealizedPnlResponse {
    pub position_notional: Uint128,
    pub unrealized_pnl: Integer,
}

#[cw_serde]
pub struct RemainMarginResponse {
    pub funding_payment: Integer,
    pub margin: Uint128,
    pub bad_debt: Uint128,
    pub latest_premium_fraction: Integer,
}

#[cw_serde]
pub struct TransferResponse {
    pub messages: Vec<SubMsg>,
    pub spread_fee: Uint128,
    pub toll_fee: Uint128,
}
