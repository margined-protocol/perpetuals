use cosmwasm_std::{from_slice, to_vec, StdResult, Storage, Timestamp, Uint128};
use margined_perp::margined_pricefeed::{ConfigResponse, PriceData};

pub static KEY_CONFIG: &[u8] = b"config";

pub static PRICES: &[u8] = b"prices";
pub static KEY_LAST_ROUND_ID: &[u8] = b"last_round_id";
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
    let last_round_id = read_last_round_id(storage, &key)?;
    let price_data = PriceData {
        round_id: last_round_id + 1,
        price,
        timestamp: Timestamp::from_seconds(timestamp),
    };
    store_last_round_id(storage, &key, price_data.round_id)?;
    Ok(storage.set(
        &[PRICES, key.as_bytes(), &price_data.round_id.to_be_bytes()].concat(),
        &to_vec(&price_data)?,
    ))
}

pub fn read_price_data(storage: &dyn Storage, key: String, round_id: u64) -> StdResult<PriceData> {
    match storage.get(&[PRICES, key.as_bytes(), &round_id.to_be_bytes()].concat()) {
        None => Ok(PriceData::default()),
        Some(data) => from_slice(&data),
    }
}

pub fn store_last_round_id(
    storage: &mut dyn Storage,
    key: &String,
    round_id: u64,
) -> StdResult<()> {
    Ok(storage.set(
        &[KEY_LAST_ROUND_ID, key.as_bytes()].concat(),
        &to_vec(&round_id)?,
    ))
}

pub fn read_last_round_id(storage: &dyn Storage, key: &String) -> StdResult<u64> {
    Ok(
        match storage.get(&[KEY_LAST_ROUND_ID, key.as_bytes()].concat()) {
            Some(data) => from_slice(&data)?,
            None => 0,
        },
    )
}
