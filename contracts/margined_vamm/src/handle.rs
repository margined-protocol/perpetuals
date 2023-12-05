use cosmwasm_std::{
    DepsMut, Env, MessageInfo, Response, StdError, StdResult, Storage, Uint128,
};

use margined_common::{integer::Integer, validate::validate_ratio};
use margined_perp::margined_vamm::Direction;
use margined_utils::{
    contracts::helpers::PricefeedController,
    tools::price_swap::{get_input_price_with_reserves, get_output_price_with_reserves},
};

use crate::{
    contract::{
        ONE_DAY_IN_SECONDS, ONE_HOUR_IN_SECONDS, ONE_MINUTE_IN_SECONDS, ONE_WEEK_IN_SECONDS, OWNER,
    },
    query::query_twap_price,
    state::{read_config, read_state, store_config, store_state, Config},
    utils::{
        add_reserve_snapshot, check_is_over_block_fluctuation_limit, require_margin_engine,
        require_open, TwapCalcOption,
    },
};

#[allow(clippy::too_many_arguments)]
pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    base_asset_holding_cap: Option<Uint128>,
    open_interest_notional_cap: Option<Uint128>,
    toll_ratio: Option<Uint128>,
    spread_ratio: Option<Uint128>,
    fluctuation_limit_ratio: Option<Uint128>,
    margin_engine: Option<String>,
    insurance_fund: Option<String>,
    pricefeed: Option<String>,
    spot_price_twap_interval: Option<u64>,
) -> StdResult<Response> {
    let mut config: Config = read_config(deps.storage)?;

    // check permission
    if !OWNER.is_admin(deps.as_ref(), &info.sender)? {
        return Err(StdError::generic_err("unauthorized"));
    }

    // change base asset holding cap
    if let Some(base_asset_holding_cap) = base_asset_holding_cap {
        config.base_asset_holding_cap = base_asset_holding_cap;
    }

    // change open interest notional cap
    if let Some(open_interest_notional_cap) = open_interest_notional_cap {
        config.open_interest_notional_cap = open_interest_notional_cap;
    }

    // set and update margin engine
    if let Some(margin_engine) = margin_engine {
        config.margin_engine = deps.api.addr_validate(margin_engine.as_str())?;
    }

    // set and update insurance fund
    if let Some(insurance_fund) = insurance_fund {
        config.insurance_fund = deps.api.addr_validate(insurance_fund.as_str())?;
    }
    // change toll ratio
    if let Some(toll_ratio) = toll_ratio {
        validate_ratio(toll_ratio, config.decimals)?;
        config.toll_ratio = toll_ratio;
    }

    // change spread ratio
    if let Some(spread_ratio) = spread_ratio {
        validate_ratio(spread_ratio, config.decimals)?;
        config.spread_ratio = spread_ratio;
    }

    // change fluctuation limit ratio
    if let Some(fluctuation_limit_ratio) = fluctuation_limit_ratio {
        validate_ratio(fluctuation_limit_ratio, config.decimals)?;
        config.fluctuation_limit_ratio = fluctuation_limit_ratio;
    }

    // change pricefeed
    if let Some(pricefeed) = pricefeed {
        config.pricefeed = deps.api.addr_validate(&pricefeed)?;
    }

    // change spot price twap interval - check that the twap interval is between 1 min and 1 week
    if let Some(spot_price_twap_interval) = spot_price_twap_interval {
        if !(ONE_MINUTE_IN_SECONDS..=ONE_WEEK_IN_SECONDS).contains(&spot_price_twap_interval) {
            return Err(StdError::generic_err(
                "spot_price_twap_interval should be between one minute and one week",
            ));
        }
        config.spot_price_twap_interval = spot_price_twap_interval;
    }

    store_config(deps.storage, &config)?;

    Ok(Response::default().add_attribute("action", "update_config"))
}

pub fn update_owner(deps: DepsMut, info: MessageInfo, owner: String) -> StdResult<Response> {
    // validate the address
    let valid_owner = deps.api.addr_validate(&owner)?;

    OWNER
        .execute_update_admin(deps, info, Some(valid_owner))
        .map_err(|error| StdError::generic_err(error.to_string()))
}

pub fn set_open(deps: DepsMut, env: Env, info: MessageInfo, open: bool) -> StdResult<Response> {
    let config = read_config(deps.storage)?;
    let mut state = read_state(deps.storage)?;

    // check permission and if state matches
    if (!OWNER.is_admin(deps.as_ref(), &info.sender)? && info.sender != config.insurance_fund)
        || state.open == open
    {
        return Err(StdError::generic_err("unauthorized"));
    }

    state.open = open;

    // if state.open is true then we update the next funding time
    if state.open {
        state.next_funding_time = env.block.time.seconds()
            + config.funding_period / ONE_HOUR_IN_SECONDS * ONE_HOUR_IN_SECONDS;
    }

    store_state(deps.storage, &state)?;

    Ok(Response::new().add_attribute("action", "set_open"))
}

// pub fn change_reserve(
//     deps: DepsMut,
//     info: MessageInfo,
//     quote_asset_reserve: Uint128,
//     base_asset_reserve: Uint128,
// ) -> StdResult<Response> {
//     let config = read_config(deps.storage)?;
//     let mut state = read_state(deps.storage)?;

//     // check permission and if state matches
//     if !OWNER.is_admin(deps.as_ref(), &info.sender)? {
//         return Err(StdError::generic_err("unauthorized"));
//     }

//     if quote_asset_reserve.is_zero() || base_asset_reserve.is_zero() {
//         return Err(StdError::generic_err("Input must be non-zero"));
//     }

//     state.quote_asset_reserve = quote_asset_reserve;
//     state.base_asset_reserve = base_asset_reserve;

//     store_state(deps.storage, &state)?;

//     Ok(Response::new().add_attributes(vec![
//         ("action", "change_reserve"),
//         ("quote_asset_reserve", &quote_asset_reserve.to_string()),
//         ("base_asset_reserve", &base_asset_reserve.to_string()),
//     ]))
// }

// Function should only be called by the margin engine
pub fn swap_input(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    direction: Direction,
    position_id: u64,
    quote_asset_amount: Uint128,
    base_asset_limit: Uint128,
    can_go_over_fluctuation: bool,
) -> StdResult<Response> {
    let state = read_state(deps.storage)?;

    require_open(state.open)?;
    let config = read_config(deps.storage)?;
    require_margin_engine(info.sender, config.margin_engine)?;

    let base_asset_amount = if !quote_asset_amount.is_zero() {
        let base_asset_amount = get_input_price_with_reserves(
            &direction,
            quote_asset_amount,
            state.quote_asset_reserve,
            state.base_asset_reserve,
        )?;

        // If AddToAmm, exchanged base amount should be more than base_asset_limit,
        // otherwise(RemoveFromAmm), exchanged base amount should be less than base_asset_limit.
        // In RemoveFromAmm case, more position means more debt so should not be larger than base_asset_limit
        if !base_asset_limit.is_zero() {
            if direction == Direction::AddToAmm && base_asset_amount < base_asset_limit {
                return Err(StdError::generic_err(
                    "Less than minimum base asset amount limit",
                ));
            } else if direction == Direction::RemoveFromAmm && base_asset_amount > base_asset_limit
            {
                return Err(StdError::generic_err(
                    "Greater than maximum base asset amount limit",
                ));
            }
        }

        base_asset_amount
    } else {
        Uint128::zero()
    };

    let response = update_reserve(
        deps.storage,
        env,
        direction.clone(),
        quote_asset_amount,
        base_asset_amount,
        can_go_over_fluctuation,
    )?;

    Ok(response.add_attributes(vec![
        ("action", "swap"),
        ("type", "input"),
        ("direction", &direction.to_string()),
        ("position_id", &position_id.to_string()),
        ("quote_asset_amount", &quote_asset_amount.to_string()),
        ("base_asset_amount", &base_asset_amount.to_string()),
    ]))
}

// Function should only be called by the margin engine
pub fn swap_output(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    direction: Direction,
    position_id: u64,
    base_asset_amount: Uint128,
    quote_asset_limit: Uint128,
) -> StdResult<Response> {
    let state = read_state(deps.storage)?;
    require_open(state.open)?;
    let config = read_config(deps.storage)?;
    require_margin_engine(info.sender, config.margin_engine)?;

    // flip direction when updating reserve
    let update_direction = match direction {
        Direction::AddToAmm => Direction::RemoveFromAmm,
        Direction::RemoveFromAmm => Direction::AddToAmm,
    };

    let quote_asset_amount = if !base_asset_amount.is_zero() {
        let quote_asset_amount = get_output_price_with_reserves(
            &direction,
            base_asset_amount,
            state.quote_asset_reserve,
            state.base_asset_reserve,
        )?;

        // If AddToAmm, exchanged base amount should be more than quote_asset_limit,
        // otherwise(RemoveFromAmm), exchanged base amount should be less than quote_asset_limit.
        // In RemoveFromAmm case, more position means more debt so should not be larger than quote_asset_limit
        if !quote_asset_limit.is_zero() {
            if update_direction == Direction::RemoveFromAmm
                && quote_asset_amount < quote_asset_limit
            {
                return Err(StdError::generic_err(
                    "Less than minimum quote asset amount limit",
                ));
            } else if update_direction == Direction::AddToAmm
                && quote_asset_amount > quote_asset_limit
            {
                return Err(StdError::generic_err(
                    "Greater than maximum quote asset amount limit",
                ));
            }
        }

        quote_asset_amount
    } else {
        Uint128::zero()
    };

    let response = update_reserve(
        deps.storage,
        env,
        update_direction,
        quote_asset_amount,
        base_asset_amount,
        true,
    )?;

    Ok(response.add_attributes(vec![
        ("action", "swap"),
        ("type", "output"),
        ("direction", &direction.to_string()),
        ("position_id", &position_id.to_string()),
        ("quote_asset_amount", &quote_asset_amount.to_string()),
        ("base_asset_amount", &base_asset_amount.to_string()),
    ]))
}

pub fn settle_funding(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let mut state = read_state(deps.storage)?;
    require_open(state.open)?;
    let config = read_config(deps.storage)?;
    require_margin_engine(info.sender, config.margin_engine)?;

    if env.block.time.seconds() < state.next_funding_time {
        return Err(StdError::generic_err("settle funding called too early"));
    }

    let pricefeed_controller = PricefeedController(config.pricefeed);
    let pair = format!("{}/{}", config.base_asset, config.quote_asset);
    // twap price from oracle
    let underlying_price = pricefeed_controller.twap_price(
        &deps.querier,
        config.base_asset,
        config.spot_price_twap_interval,
    )?;

    // twap price from here, i.e. the amm
    let index_price = query_twap_price(
        deps.as_ref(),
        env.clone(),
        config.spot_price_twap_interval,
        TwapCalcOption::Reserve,
        None,
    )?;

    let premium =
        Integer::new_positive(index_price).checked_sub(Integer::new_positive(underlying_price))?;

    let premium_fraction = premium
        .checked_mul(Integer::new_positive(config.funding_period))?
        .checked_div(Integer::new_positive(ONE_DAY_IN_SECONDS))?;

    // update funding rate = premiumFraction / twapIndexPrice
    state.funding_rate = premium_fraction
        .checked_mul(Integer::new_positive(config.decimals))?
        .checked_div(Integer::new_positive(underlying_price))?;

    // in order to prevent multiple funding settlement during very short time after network congestion
    let block_time_seconds = env.block.time.seconds();
    let min_next_funding_time = block_time_seconds + config.funding_period / 2;

    // floor((nextFundingTime + fundingPeriod) / 3600) * 3600
    let next_funding_time =
        (block_time_seconds + config.funding_period) / ONE_HOUR_IN_SECONDS * ONE_HOUR_IN_SECONDS;

    // max(nextFundingTimeOnHourStart, minNextValidFundingTime)
    state.next_funding_time = next_funding_time.max(min_next_funding_time);

    store_state(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "settle_funding"),
        ("pair", &pair),
        ("time", &block_time_seconds.to_string()),
        ("premium_fraction", &premium_fraction.to_string()),
        ("underlying_price", &underlying_price.to_string()),
        ("index_price", &index_price.to_string()),
        ("next_funding_time", &state.next_funding_time.to_string()),
    ]))
}

pub fn update_reserve(
    storage: &mut dyn Storage,
    env: Env,
    direction: Direction,
    quote_asset_amount: Uint128,
    base_asset_amount: Uint128,
    can_go_over_fluctuation: bool,
) -> StdResult<Response> {
    check_is_over_block_fluctuation_limit(
        storage,
        env.clone(),
        direction.clone(),
        quote_asset_amount,
        base_asset_amount,
        can_go_over_fluctuation,
    )?;

    let mut state = read_state(storage)?;

    match direction {
        Direction::AddToAmm => {
            state.quote_asset_reserve =
                state.quote_asset_reserve.checked_add(quote_asset_amount)?;
            state.base_asset_reserve = state.base_asset_reserve.checked_sub(base_asset_amount)?;

            state.total_position_size += Integer::from(base_asset_amount);
        }
        Direction::RemoveFromAmm => {
            state.base_asset_reserve = state.base_asset_reserve.checked_add(base_asset_amount)?;
            state.quote_asset_reserve =
                state.quote_asset_reserve.checked_sub(quote_asset_amount)?;

            state.total_position_size -= Integer::from(base_asset_amount);
        }
    }

    store_state(storage, &state)?;

    add_reserve_snapshot(
        storage,
        env.clone(),
        state.quote_asset_reserve,
        state.base_asset_reserve,
    )?;

    Ok(Response::new().add_attributes(vec![
        (
            "quote_asset_reserve",
            &state.quote_asset_reserve.to_string(),
        ),
        ("base_asset_reserve", &state.base_asset_reserve.to_string()),
        ("timestamp", &env.block.time.seconds().to_string()),
    ]))
}
