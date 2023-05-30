use cosmwasm_std::{from_slice, to_vec, StdResult, Storage, Timestamp, Uint128};
use margined_perp::margined_pricefeed::{ConfigResponse, PriceData};

pub static KEY_CONFIG: &[u8] = b"config";

pub const PRICES: &[u8] = b"prices";

pub type Config = ConfigResponse;

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    Ok(storage.set(KEY_CONFIG, &to_vec(config)?))
}

pub fn store_price_data(
    storage: &mut dyn Storage,
    key: String,
    price: Uint128,
    timestamp: u64,
) -> StdResult<()> {
    // load the existing data
    let mut prices = read_price_data(storage, key.clone())?;

    let price_data: PriceData = PriceData {
        round_id: prices.len() as u64,
        price,
        timestamp: Timestamp::from_seconds(timestamp),
    };

    prices.push(price_data);

    Ok(storage.set(&[PRICES, key.as_bytes()].concat(), &to_vec(&prices)?))
}

pub fn read_price_data(storage: &dyn Storage, key: String) -> StdResult<Vec<PriceData>> {
    // let prices = ReadonlyBucket::new(storage, PRICES).may_load(key.as_bytes())?;
    match storage.get(&[PRICES, key.as_bytes()].concat()) {
        None => Ok(vec![PriceData::default()]),
        Some(data) => from_slice(&data),
    }
}
