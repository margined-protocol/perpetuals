use cosmwasm_std::{Addr, Deps, Env, Response, StdError, StdResult, Storage, Uint128};
use margined_perp::margined_vamm::Direction;

use crate::{
    handle::{get_input_price_with_reserves, get_output_price_with_reserves},
    state::{
        read_config, read_reserve_snapshot, read_reserve_snapshot_counter, read_state,
        store_reserve_snapshot, update_current_reserve_snapshot, Config, ReserveSnapshot,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TwapCalcOption {
    Reserve,
    Input,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TwapInputAsset {
    pub direction: Direction,
    pub amount: Uint128,
    pub quote: bool, // [true|false] -> [quote_in|quote_out]
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TwapPriceCalcParams {
    pub opt: TwapCalcOption,
    pub snapshot_index: u64,
    pub asset: Option<TwapInputAsset>,
}

pub fn require_margin_engine(sender: Addr, margin_engine: Addr) -> StdResult<Response> {
    // check that sender is the margin engine
    if sender != margin_engine {
        return Err(StdError::generic_err("sender not margin engine"));
    }

    Ok(Response::new())
}

pub fn require_open(open: bool) -> StdResult<Response> {
    // check that the vamm is open
    if !open {
        return Err(StdError::generic_err("amm is closed"));
    }

    Ok(Response::new())
}

pub fn check_is_over_block_fluctuation_limit(
    storage: &mut dyn Storage,
    env: Env,
    direction: Direction,
    quote_asset_amount: Uint128,
    base_asset_amount: Uint128,
    can_go_over_limit: bool,
) -> StdResult<Response> {
    let config = read_config(storage)?;
    let state = read_state(storage)?;

    if config.fluctuation_limit_ratio.is_zero() {
        return Ok(Response::new());
    }

    let (upper_limit, lower_limit) = price_boundaries_of_last_block(storage, env)?;

    let current_price = state
        .quote_asset_reserve
        .checked_mul(config.decimals)?
        .checked_div(state.base_asset_reserve)?;

    // ensure that the latest price isn't over the limit which would restrict any further
    // swaps from occurring in this block
    if current_price > upper_limit || current_price < lower_limit {
        return Err(StdError::generic_err(
            "price is already over fluctuation limit",
        ));
    }

    if !can_go_over_limit {
        let price = if direction == Direction::AddToAmm {
            state
                .quote_asset_reserve
                .checked_add(quote_asset_amount)?
                .checked_mul(config.decimals)?
                .checked_div(state.base_asset_reserve.checked_sub(base_asset_amount)?)
        } else {
            state
                .quote_asset_reserve
                .checked_sub(quote_asset_amount)?
                .checked_mul(config.decimals)?
                .checked_div(state.base_asset_reserve.checked_add(base_asset_amount)?)
        }?;
        if price > upper_limit || price < lower_limit {
            return Err(StdError::generic_err("price is over fluctuation limit"));
        }
    }

    Ok(Response::new())
}

pub fn price_boundaries_of_last_block(
    storage: &dyn Storage,
    env: Env,
) -> StdResult<(Uint128, Uint128)> {
    let config = read_config(storage)?;

    // calculate the price boundary of the previous block
    let height = read_reserve_snapshot_counter(storage)?;
    let mut latest_snapshot = read_reserve_snapshot(storage, height)?;

    if latest_snapshot.block_height == env.block.height && height > 1 {
        latest_snapshot = read_reserve_snapshot(storage, height - 1u64)?;
    }

    let last_price = latest_snapshot
        .quote_asset_reserve
        .checked_mul(config.decimals)?
        .checked_div(latest_snapshot.base_asset_reserve)?;

    let upper_limit = last_price
        .checked_mul(config.decimals + config.fluctuation_limit_ratio)?
        .checked_div(config.decimals)?;
    let lower_limit = last_price
        .checked_mul(config.decimals - config.fluctuation_limit_ratio)?
        .checked_div(config.decimals)?;

    Ok((upper_limit, lower_limit))
}

pub fn add_reserve_snapshot(
    storage: &mut dyn Storage,
    env: Env,
    quote_asset_reserve: Uint128,
    base_asset_reserve: Uint128,
) -> StdResult<Response> {
    let height = read_reserve_snapshot_counter(storage)?;
    let mut snapshot = read_reserve_snapshot(storage, height)?;

    snapshot.quote_asset_reserve = quote_asset_reserve;
    snapshot.base_asset_reserve = base_asset_reserve;

    // if there has already been an update in this block we overwrite the existing
    // else we create a new snapshot
    if snapshot.block_height == env.block.height {
        update_current_reserve_snapshot(storage, &snapshot)?;
    } else {
        snapshot.timestamp = env.block.time;
        snapshot.block_height = env.block.height;

        store_reserve_snapshot(storage, &snapshot)?;
    }

    Ok(Response::default())
}

pub fn get_price_with_specific_snapshot(
    deps: Deps,
    params: TwapPriceCalcParams,
) -> StdResult<Uint128> {
    let config: Config = read_config(deps.storage)?;

    let snapshot: ReserveSnapshot = read_reserve_snapshot(deps.storage, params.snapshot_index)?;

    // RESERVE_ASSET means price comes from quoteAssetReserve/baseAssetReserve
    // INPUT_ASSET means getInput/Output price with snapshot's reserve
    if params.opt == TwapCalcOption::Reserve {
        let current_price = snapshot
            .quote_asset_reserve
            .checked_mul(config.decimals)?
            .checked_div(snapshot.base_asset_reserve)?;

        return Ok(current_price);
    } else if params.opt == TwapCalcOption::Input {
        // safe to unwrap as entry requires it to be so,
        // maybe its nicer just to set defaults instead of option
        // ¯\_(ツ)_/¯
        let asset = params.asset.unwrap();
        if asset.amount.is_zero() {
            return Ok(Uint128::zero());
        }

        if asset.quote {
            return get_input_price_with_reserves(
                deps,
                &asset.direction,
                asset.amount,
                snapshot.quote_asset_reserve,
                snapshot.base_asset_reserve,
            );
        } else {
            return get_output_price_with_reserves(
                deps,
                &asset.direction,
                asset.amount,
                snapshot.quote_asset_reserve,
                snapshot.base_asset_reserve,
            );
        }
    }
    Ok(Uint128::zero())
}

/// Calculates the TWAP of the AMM reserves
pub fn calc_twap(
    deps: Deps,
    env: Env,
    mut params: TwapPriceCalcParams,
    interval: u64,
) -> StdResult<Uint128> {
    let current_price = get_price_with_specific_snapshot(deps, params.clone())?;

    if interval == 0 {
        return Ok(current_price);
    }

    let base_timestamp = env.block.time.seconds().checked_sub(interval).unwrap();
    let reserve_snapshot_length = read_reserve_snapshot_counter(deps.storage).unwrap();
    let mut current_snapshot = read_reserve_snapshot(deps.storage, params.snapshot_index)?;

    if reserve_snapshot_length == 1 || current_snapshot.timestamp.seconds() <= base_timestamp {
        return Ok(current_price);
    }

    let mut previous_timestamp = current_snapshot.timestamp.seconds();
    let mut period = Uint128::from(
        env.block
            .time
            .seconds()
            .checked_sub(previous_timestamp)
            .unwrap(),
    );

    let mut weighted_price = current_price.checked_mul(period)?;

    loop {
        params.snapshot_index -= 1;

        // if snapshot history is too short
        if params.snapshot_index == 0 {
            return Ok(weighted_price.checked_div(period)?);
        }

        current_snapshot = read_reserve_snapshot(deps.storage, params.snapshot_index).unwrap();
        let current_price = get_price_with_specific_snapshot(deps, params.clone())?;

        if current_snapshot.timestamp.seconds() <= base_timestamp {
            let delta_timestamp =
                Uint128::from(previous_timestamp.checked_sub(base_timestamp).unwrap());

            weighted_price = weighted_price
                .checked_add(current_price.checked_mul(delta_timestamp).unwrap())
                .unwrap();

            break;
        }

        let delta_timestamp = Uint128::from(
            previous_timestamp
                .checked_sub(current_snapshot.timestamp.seconds())
                .unwrap(),
        );
        weighted_price = weighted_price
            .checked_add(current_price.checked_mul(delta_timestamp).unwrap())
            .unwrap();

        period = period.checked_add(delta_timestamp).unwrap();
        previous_timestamp = current_snapshot.timestamp.seconds();
    }

    Ok(weighted_price.checked_div(Uint128::from(interval))?)
}

/// Does the modulus (%) operator on Uint128.
/// However it follows the design of the perpetual protocol decimals
/// https://github.com/perpetual-protocol/perpetual-protocol/blob/release/v2.1.x/src/utils/Decimal.sol
pub(crate) fn modulo(a: Uint128, b: Uint128, decimals: Uint128) -> Uint128 {
    let a_decimals = a.checked_mul(decimals).unwrap();
    let integral = a_decimals / b;
    a_decimals - (b * integral)
}
