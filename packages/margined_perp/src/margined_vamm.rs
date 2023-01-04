use cosmwasm_schema::{cw_serde, QueryResponses};

use cosmwasm_std::{Addr, Uint128};

use margined_common::integer::Integer;
use strum::Display;

#[cw_serde]
#[derive(Eq, Display)]
pub enum Direction {
    AddToAmm,
    RemoveFromAmm,
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
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

#[cw_serde]
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

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(StateResponse)]
    State {},
    #[returns(OwnerResponse)]
    GetOwner {},
    #[returns(Uint128)]
    InputPrice {
        direction: Direction,
        amount: Uint128,
    },
    #[returns(Uint128)]
    OutputPrice {
        direction: Direction,
        amount: Uint128,
    },
    #[returns(Uint128)]
    InputAmount {
        direction: Direction,
        amount: Uint128,
    },
    #[returns(Uint128)]
    OutputAmount {
        direction: Direction,
        amount: Uint128,
    },
    #[returns(Uint128)]
    InputTwap {
        direction: Direction,
        amount: Uint128,
    },
    #[returns(Uint128)]
    OutputTwap {
        direction: Direction,
        amount: Uint128,
    },
    #[returns(Uint128)]
    SpotPrice {},
    #[returns(Uint128)]
    TwapPrice { interval: u64 },
    #[returns(Uint128)]
    UnderlyingPrice {},
    #[returns(Uint128)]
    UnderlyingTwapPrice { interval: u64 },
    #[returns(CalcFeeResponse)]
    CalcFee { quote_asset_amount: Uint128 },
    #[returns(bool)]
    IsOverSpreadLimit {},
    #[returns(bool)]
    IsOverFluctuationLimit {
        direction: Direction,
        base_asset_amount: Uint128,
    },
}

#[cw_serde]
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

#[cw_serde]
pub struct OwnerResponse {
    pub owner: Addr,
}

#[cw_serde]
pub struct StateResponse {
    pub open: bool,
    pub quote_asset_reserve: Uint128,
    pub base_asset_reserve: Uint128,
    pub total_position_size: Integer,
    pub funding_rate: Integer,
    pub next_funding_time: u64,
}

#[cw_serde]
pub struct CalcFeeResponse {
    pub toll_fee: Uint128,
    pub spread_fee: Uint128,
}
