use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdResult, Storage, Timestamp, Uint128};
use cosmwasm_storage::{singleton, Bucket, ReadonlyBucket};
use margined_perp::margined_pricefeed::PriceData;

pub static KEY_CONFIG: &[u8] = b"config";

pub const PRICES: &[u8] = b"prices";

#[cw_serde]
pub struct Config {}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

pub fn store_price_data(
    storage: &mut dyn Storage,
    key: String,
    price: Uint128,
    timestamp: u64,
) -> StdResult<()> {
    // load the existing data
    let mut prices = read_price_data(storage, key.clone()).unwrap();

    let price_data: PriceData = PriceData {
        round_id: prices.len() as u64,
        price,
        timestamp: Timestamp::from_seconds(timestamp),
    };

    prices.push(price_data);

    Bucket::new(storage, PRICES).save(key.as_bytes(), &prices)
}

pub fn read_price_data(storage: &dyn Storage, key: String) -> StdResult<Vec<PriceData>> {
    // let prices = ReadonlyBucket::new(storage, PRICES).may_load(key.as_bytes())?;
    let result = match ReadonlyBucket::new(storage, PRICES).may_load(key.as_bytes())? {
        None => vec![PriceData::default()],
        Some(prices) => prices,
    };

    Ok(result)
}
