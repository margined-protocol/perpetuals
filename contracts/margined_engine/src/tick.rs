use cosmwasm_std::{Order as OrderBy, StdResult, Storage, Uint128};
use cosmwasm_storage::ReadonlyBucket;
use margined_perp::margined_engine::{Side, TickResponse, TicksResponse};
use std::convert::{TryFrom, TryInto};

use crate::{
    state::{DEFAULT_LIMIT, MAX_LIMIT, PREFIX_TICK},
    utils::{calc_range_start, keccak_256},
};

pub fn query_ticks(
    storage: &dyn Storage,
    vamm: String,
    side: Side,
    start_after: Option<Uint128>,
    limit: Option<u32>,
    order_by: Option<i32>,
) -> StdResult<TicksResponse> {
    let order_by = order_by.map_or(None, |val| OrderBy::try_from(val).ok());
    let vamm_key = keccak_256(vamm.as_bytes());

    let position_bucket =
        ReadonlyBucket::multilevel(storage, &[PREFIX_TICK, &vamm_key, side.as_bytes()]);

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_after = start_after.map(|id| id.to_be_bytes().to_vec());
    let (start, end, order_by) = match order_by {
        Some(OrderBy::Ascending) => (calc_range_start(start_after), None, OrderBy::Ascending),
        _ => (None, start_after, OrderBy::Descending),
    };

    let ticks = position_bucket
        .range(start.as_deref(), end.as_deref(), order_by)
        .take(limit)
        .map(|item| {
            let (k, total_positions) = item?;
            let entry_price = Uint128::from(u128::from_be_bytes(k.try_into().unwrap()));
            Ok(TickResponse {
                entry_price,
                total_positions,
            })
        })
        .collect::<StdResult<Vec<TickResponse>>>()?;

    Ok(TicksResponse { ticks })
}

pub fn query_tick(
    storage: &dyn Storage,
    vamm: String,
    side: Side,
    entry_price: Uint128,
) -> StdResult<TickResponse> {
    let price_key = entry_price.to_be_bytes();
    let vamm_key = keccak_256(vamm.as_bytes());

    let total_positions =
        ReadonlyBucket::<u64>::multilevel(storage, &[PREFIX_TICK, &vamm_key, side.as_bytes()])
            .load(&price_key)?;

    Ok(TickResponse {
        total_positions,
        entry_price,
    })
}
