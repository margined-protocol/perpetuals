use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Timestamp, Uint128};

#[cw_serde]
#[derive(Default)]
pub struct PriceData {
    pub round_id: u64,
    pub price: Uint128,
    pub timestamp: Timestamp,
}

#[cw_serde]
pub enum Direction {
    AddToAmm,
    RemoveFromAmm,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub oracle_hub_contract: String, // address of the oracle hub we are using
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
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
    UpdateOwner {
        owner: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(OwnerResponse)]
    GetOwner {},
    #[returns(Uint128)]
    GetPrice { key: String },
    #[returns(Uint128)]
    GetPreviousPrice { key: String, round_id: u64 },
    #[returns(Uint128)]
    GetTwapPrice { key: String, interval: u64 },
}

#[cw_serde]
pub struct ConfigResponse {}

#[cw_serde]
pub struct OwnerResponse {
    pub owner: Addr,
}
