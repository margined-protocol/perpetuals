use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, StdResult, Storage, Timestamp};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_storage::{singleton, singleton_read};
use cw_storage_plus::Map;

pub static KEY_CONFIG: &[u8] = b"config";

pub const PRICES: Map<String, Vec<PriceData>> = Map::new("prices");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub decimals: Decimal256,
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, KEY_CONFIG).load()
}

#[derive(Serialize, Default, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PriceData {
    pub round_id: Decimal256,
    pub price: Decimal256,
    pub timestamp: Timestamp,
}

pub fn store_price_data(
    storage: &mut dyn Storage,
    key: String,
    price: Decimal256,
    timestamp: u64,
) -> StdResult<()> {
    // load the existing data
    let mut prices = read_price_data(storage, key.clone()).unwrap();

    let price_data: PriceData = PriceData {
        round_id: Decimal256::from_uint256(Uint256::from(prices.len() as u64)),
        price,
        timestamp: Timestamp::from_seconds(timestamp),
    };

    prices.push(price_data);

    PRICES.save(storage, key, &prices)
}

pub fn read_price_data(storage: &dyn Storage, key: String) -> StdResult<Vec<PriceData>> {
    let prices = PRICES.may_load(storage, key)?;
    let mut result = Vec::new();

    if let Some(prices) = prices {
        result = prices;
    } else {
        result.push(PriceData::default());
    }

    Ok(result)
}
