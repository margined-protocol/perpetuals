use cosmwasm_std::{
    Addr, DepsMut, Env, MessageInfo, Order, Response, StdError, StdResult, Storage, SubMsg, Uint128,
};
use margined_utils::{
    contracts::helpers::VammController, tools::price_swap::get_output_price_with_reserves,
};

use crate::{
    contract::{
        CLOSE_POSITION_REPLY_ID, INCREASE_POSITION_REPLY_ID, LIQUIDATION_REPLY_ID,
        PARTIAL_CLOSE_POSITION_REPLY_ID, PARTIAL_LIQUIDATION_REPLY_ID, PAY_FUNDING_REPLY_ID,
    },
    messages::{execute_transfer_from, withdraw},
    query::{query_free_collateral, query_margin_ratio, query_positions},
    state::{
        increase_last_position_id, read_config, read_position, read_state, store_config,
        store_position, store_sent_funds, store_state, store_tmp_liquidator, store_tmp_swap,
        SentFunds, TmpReserveInfo, TmpSwapInfo,
    },
    tick::query_ticks,
    utils::{
        calc_remain_margin_with_funding_payment, calculate_tp_sl_spread, check_tp_sl_price,
        direction_to_side, get_asset, get_margin_ratio_calc_option,
        get_position_notional_unrealized_pnl, keccak_256, position_is_bad_debt,
        position_is_liquidated, position_to_side, require_additional_margin, require_bad_debt,
        require_insufficient_margin, require_non_zero_input, require_not_paused,
        require_not_restriction_mode, require_position_not_zero, require_vamm, side_to_direction,
        update_reserve,
    },
};
use margined_common::{
    asset::{Asset, AssetInfo},
    integer::Integer,
    messages::wasm_execute,
    validate::{validate_margin_ratios, validate_ratio},
};
use margined_perp::margined_engine::{
    PnlCalcOption, Position, PositionFilter, PositionUnrealizedPnlResponse, Side,
};
use margined_perp::margined_vamm::{CalcFeeResponse, Direction, ExecuteMsg};

#[allow(clippy::too_many_arguments)]
pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    insurance_fund: Option<String>,
    fee_pool: Option<String>,
    initial_margin_ratio: Option<Uint128>,
    maintenance_margin_ratio: Option<Uint128>,
    partial_liquidation_ratio: Option<Uint128>,
    tp_sl_spread: Option<Uint128>,
    liquidation_fee: Option<Uint128>,
) -> StdResult<Response> {
    let mut config = read_config(deps.storage)?;

    // check permission
    if info.sender != config.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    // change owner of engine
    if let Some(owner) = owner {
        config.owner = deps.api.addr_validate(owner.as_str())?;
    }

    // update insurance fund - note altering insurance fund could lead to vAMMs being unusable maybe make this a migration
    if let Some(insurance_fund) = insurance_fund {
        config.insurance_fund = Some(deps.api.addr_validate(insurance_fund.as_str())?);
    }

    // update fee pool
    if let Some(fee_pool) = fee_pool {
        config.fee_pool = deps.api.addr_validate(fee_pool.as_str())?;
    }

    // update initial margin ratio
    if let Some(initial_margin_ratio) = initial_margin_ratio {
        validate_ratio(initial_margin_ratio, config.decimals)?;
        validate_margin_ratios(initial_margin_ratio, config.maintenance_margin_ratio)?;
        config.initial_margin_ratio = initial_margin_ratio;
    }

    // update maintenance margin ratio
    if let Some(maintenance_margin_ratio) = maintenance_margin_ratio {
        validate_ratio(maintenance_margin_ratio, config.decimals)?;
        validate_margin_ratios(config.initial_margin_ratio, maintenance_margin_ratio)?;
        config.maintenance_margin_ratio = maintenance_margin_ratio;
    }

    // update partial liquidation ratio
    if let Some(partial_liquidation_ratio) = partial_liquidation_ratio {
        validate_ratio(partial_liquidation_ratio, config.decimals)?;
        config.partial_liquidation_ratio = partial_liquidation_ratio;
    }

    // update take_profit and stop_loss spread ratio
    if let Some(tp_sl_spread) = tp_sl_spread {
        validate_ratio(tp_sl_spread, config.decimals)?;
        config.tp_sl_spread = tp_sl_spread;
    }

    // update liquidation fee
    if let Some(liquidation_fee) = liquidation_fee {
        validate_ratio(liquidation_fee, config.decimals)?;
        config.liquidation_fee = liquidation_fee;
    }

    store_config(deps.storage, &config)?;

    Ok(Response::default().add_attribute("action", "update_config"))
}

// Opens a position
#[allow(clippy::too_many_arguments)]
pub fn open_position(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vamm: String,
    side: Side,
    margin_amount: Uint128,
    leverage: Uint128,
    take_profit: Uint128,
    stop_loss: Option<Uint128>,
    base_asset_limit: Uint128,
) -> StdResult<Response> {
    // validate address inputs
    let vamm = deps.api.addr_validate(&vamm)?;
    let vamm_controller = VammController(vamm.clone());
    let config = read_config(deps.storage)?;
    let state = read_state(deps.storage)?;
    let trader = info.sender.clone();

    require_not_paused(state.pause)?;
    require_vamm(deps.as_ref(), &config.insurance_fund, &vamm)?;

    require_not_restriction_mode(deps.storage, &vamm, env.block.height)?;
    require_non_zero_input(margin_amount)?;
    require_non_zero_input(leverage)?;
    require_non_zero_input(take_profit)?;

    let position_id = increase_last_position_id(deps.storage)?;

    if leverage < config.decimals {
        return Err(StdError::generic_err("Leverage must be greater than 1"));
    }

    let vamm_config = vamm_controller.config(&deps.querier)?;

    // calculate the margin ratio of new position wrt to leverage
    let margin_ratio = config
        .decimals
        .checked_mul(config.decimals)?
        .checked_div(leverage)?;

    require_additional_margin(Integer::from(margin_ratio), config.initial_margin_ratio)?;

    // calculate the position notional
    let mut open_notional = margin_amount
        .checked_mul(leverage)?
        .checked_div(config.decimals)?;

    let CalcFeeResponse {
        spread_fee,
        toll_fee,
    } = vamm_controller.calc_fee(&deps.querier, open_notional)?;

    // calculate the new margin
    let new_margin_amount = margin_amount
        .checked_sub(spread_fee)?
        .checked_sub(toll_fee)?;
    require_non_zero_input(new_margin_amount)?;

    // calculate the new position notional
    open_notional = new_margin_amount
        .checked_mul(leverage)?
        .checked_div(config.decimals)?;

    let entry_price =
        vamm_controller.input_price(&deps.querier, side_to_direction(&side), open_notional)?;

    match side {
        Side::Buy => {
            if take_profit <= entry_price {
                return Err(StdError::generic_err("TP price is too low"));
            }
            if let Some(stop_loss) = stop_loss {
                if stop_loss > entry_price {
                    return Err(StdError::generic_err("SL price is too high"));
                }
            }
        }
        Side::Sell => {
            if take_profit >= entry_price {
                return Err(StdError::generic_err("TP price is too high"));
            }
            if let Some(stop_loss) = stop_loss {
                if stop_loss < entry_price {
                    return Err(StdError::generic_err("SL price is too low"));
                }
            }
        }
    }

    let msg = internal_open_position(
        vamm.clone(),
        side,
        position_id,
        open_notional,
        base_asset_limit,
    )?;

    store_tmp_swap(
        deps.storage,
        &TmpSwapInfo {
            position_id,
            vamm: vamm.clone(),
            pair: format!("{}/{}", vamm_config.base_asset, vamm_config.quote_asset),
            trader: trader.clone(),
            side: side.clone(),
            margin_amount: new_margin_amount,
            leverage,
            open_notional,
            position_notional: Uint128::zero(),
            unrealized_pnl: Integer::zero(),
            margin_to_vault: Integer::zero(),
            spread_fee,
            toll_fee,
            take_profit,
            stop_loss,
        },
    )?;

    store_sent_funds(
        deps.storage,
        &SentFunds {
            asset: get_asset(info, config.eligible_collateral),
            required: Uint128::zero(),
        },
    )?;

    Ok(Response::new().add_submessage(msg).add_attributes(vec![
        ("action", "open_position"),
        ("position_id", &position_id.to_string()),
        ("position_side", &format!("{:?}", side)),
        ("vamm", vamm.as_ref()),
        (
            "pair",
            &format!("{}/{}", vamm_config.base_asset, vamm_config.quote_asset),
        ),
        ("trader", trader.as_ref()),
        ("margin_amount", &margin_amount.to_string()),
        ("leverage", &leverage.to_string()),
        ("take_profit", &take_profit.to_string()),
        ("stop_loss", &stop_loss.unwrap_or_default().to_string()),
    ]))
}

pub fn update_tp_sl(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    vamm: String,
    position_id: u64,
    take_profit: Option<Uint128>,
    stop_loss: Option<Uint128>,
) -> StdResult<Response> {
    let vamm = deps.api.addr_validate(&vamm)?;
    let trader = info.sender;

    // read the position for the trader from vamm
    let vamm_key = keccak_256(&[vamm.as_bytes()].concat());
    let mut position = read_position(deps.storage, &vamm_key, position_id)?;

    let state = read_state(deps.storage)?;
    require_not_paused(state.pause)?;
    require_position_not_zero(position.size.value)?;

    if position.trader != trader {
        return Err(StdError::generic_err("Unauthorized"));
    }

    if take_profit.is_none() && stop_loss.is_none() {
        return Err(StdError::generic_err(
            "Both take profit and stop loss are not set",
        ));
    }

    match position.side {
        Side::Buy => {
            if let Some(take_profit) = take_profit {
                if take_profit <= position.entry_price {
                    return Err(StdError::generic_err("TP price is too low"));
                }
                position.take_profit = take_profit;
            }

            if let Some(sl) = stop_loss {
                if sl > position.entry_price {
                    return Err(StdError::generic_err("SL price is too high"));
                }
                position.stop_loss = stop_loss;
            }
        }
        Side::Sell => {
            if let Some(take_profit) = take_profit {
                if take_profit >= position.entry_price {
                    return Err(StdError::generic_err("TP price is too high"));
                }
                position.take_profit = take_profit;
            }
            if let Some(sl) = stop_loss {
                if sl < position.entry_price {
                    return Err(StdError::generic_err("SL price is too low"));
                }
                position.stop_loss = stop_loss;
            }
        }
    }

    store_position(deps.storage, &vamm_key, &position, false)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "update_tp_sl"),
        ("vamm", vamm.as_ref()),
        ("pair", &position.pair),
        ("trader", trader.as_ref()),
        ("position_id", &position_id.to_string()),
        ("take_profit", &position.take_profit.to_string()),
        (
            "stop_loss",
            &position.stop_loss.unwrap_or_default().to_string(),
        ),
    ]))
}

pub fn close_position(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vamm: String,
    position_id: u64,
    quote_amount_limit: Uint128,
) -> StdResult<Response> {
    // read configuration and state information
    let config = read_config(deps.storage)?;
    let state = read_state(deps.storage)?;

    // validate address inputs
    let vamm = deps.api.addr_validate(&vamm)?;
    let trader = info.sender;

    // read the position for the trader from vamm
    let vamm_key = keccak_256(&[vamm.as_bytes()].concat());
    let position = read_position(deps.storage, &vamm_key, position_id)?;

    if position.trader != trader {
        return Err(StdError::generic_err("Unauthorized"));
    }

    // check the position isn't zero
    require_not_paused(state.pause)?;
    require_position_not_zero(position.size.value)?;
    require_not_restriction_mode(deps.storage, &vamm, env.block.height)?;

    // if it is long position, close a position means short it (which means base dir is AddToAmm) and vice versa
    let base_direction = if position.size > Integer::zero() {
        Direction::AddToAmm
    } else {
        Direction::RemoveFromAmm
    };

    let vamm_controller = VammController(vamm.clone());
    let is_over_fluctuation_limit = vamm_controller.is_over_fluctuation_limit(
        &deps.querier,
        Direction::RemoveFromAmm,
        position.size.value,
    )?;

    // check if this position exceed fluctuation limit
    // if over fluctuation limit, then close partial position. Otherwise close all.
    // if partialLiquidationRatio is 1, then close whole position
    let msg = if is_over_fluctuation_limit && config.partial_liquidation_ratio < config.decimals {
        let side = position_to_side(position.size);

        let partial_close_amount = position
            .size
            .value
            .checked_mul(config.partial_liquidation_ratio)?
            .checked_div(config.decimals)?;

        let partial_close_notional =
            vamm_controller.output_amount(&deps.querier, base_direction, partial_close_amount)?;

        let PositionUnrealizedPnlResponse {
            position_notional,
            unrealized_pnl,
        } = get_position_notional_unrealized_pnl(
            deps.as_ref(),
            &position,
            PnlCalcOption::SpotPrice,
        )?;

        store_tmp_swap(
            deps.storage,
            &TmpSwapInfo {
                position_id,
                vamm: position.vamm.clone(),
                pair: position.pair.clone(),
                trader: position.trader.clone(),
                side: side.clone(),
                margin_amount: position.size.value,
                leverage: config.decimals,
                open_notional: partial_close_notional,
                position_notional,
                unrealized_pnl,
                margin_to_vault: Integer::zero(),
                spread_fee: position.spread_fee,
                toll_fee: position.toll_fee,
                take_profit: position.take_profit,
                stop_loss: position.stop_loss,
            },
        )?;

        swap_input(
            &position.vamm,
            &side,
            position_id,
            partial_close_notional,
            Uint128::zero(),
            true,
            PARTIAL_CLOSE_POSITION_REPLY_ID,
        )?
    } else {
        internal_close_position(
            deps.storage,
            &position,
            quote_amount_limit,
            CLOSE_POSITION_REPLY_ID,
        )?
    };

    Ok(Response::new().add_submessage(msg).add_attributes(vec![
        ("action", "close_position"),
        ("vamm", vamm.as_ref()),
        ("pair", &position.pair),
        ("trader", trader.as_ref()),
        ("position_id", &position_id.to_string()),
        ("position_side", &format!("{:?}", position.side)),
        ("margin_amount", &position.margin.to_string()),
        ("entry_price", &position.entry_price.to_string()),
        (
            "leverage",
            &position
                .notional
                .checked_mul(config.decimals)?
                .checked_div(position.margin)?
                .to_string(),
        ),
    ]))
}

pub fn trigger_tp_sl(
    deps: DepsMut,
    vamm: String,
    position_id: u64,
    take_profit: bool,
) -> StdResult<Response> {
    let config = read_config(deps.storage)?;
    let vamm_addr = deps.api.addr_validate(&vamm)?;
    let mut msgs: Vec<SubMsg> = vec![];
    let mut tp_sl_flag: bool = false;

    let vamm_controller = VammController(vamm_addr.clone());
    let vamm_state = vamm_controller.state(&deps.querier)?;

    // read the position for the trader from vamm
    let vamm_key = keccak_256(&[vamm.as_bytes()].concat());
    let position = read_position(deps.storage, &vamm_key, position_id)?;

    // check that vamm is open
    if !vamm_state.open {
        return Err(StdError::generic_err("vAMM is not open"));
    }

    // check the position isn't zero
    require_position_not_zero(position.size.value)?;

    if !take_profit {
        // Can not trigger stop loss position if bad debt
        if position_is_bad_debt(deps.as_ref(), &position, &vamm_controller)? {
            return Err(StdError::generic_err("position is bad debt"));
        }

        // Can not trigger stop loss position if liquidate
        if position_is_liquidated(
            deps.as_ref(),
            &position,
            config.maintenance_margin_ratio,
            &vamm_controller,
        )? {
            return Err(StdError::generic_err("position is liquidated"));
        }
    }

    let base_asset_amount = position.size.value;
    let quote_asset_amount = get_output_price_with_reserves(
        config.decimals,
        &position.direction,
        base_asset_amount,
        vamm_state.quote_asset_reserve,
        vamm_state.base_asset_reserve,
    )?;
    let close_price = quote_asset_amount
        .checked_mul(config.decimals)?
        .checked_div(base_asset_amount)?;

    let stop_loss = position.stop_loss.unwrap_or_default();
    let (tp_spread, sl_spread) = calculate_tp_sl_spread(
        config.tp_sl_spread,
        position.take_profit,
        stop_loss,
        config.decimals,
    )?;
    let tp_sl_action = check_tp_sl_price(
        close_price,
        position.take_profit,
        stop_loss,
        tp_spread,
        sl_spread,
        &position.side,
    )?;
    if take_profit {
        if tp_sl_action == "trigger_take_profit" {
            tp_sl_flag = true;
        }
    } else {
        if tp_sl_action == "trigger_stop_loss" {
            tp_sl_flag = true;
        }
    }
    if tp_sl_flag {
        msgs.push(internal_close_position(
            deps.storage,
            &position,
            Uint128::zero(),
            CLOSE_POSITION_REPLY_ID,
        )?);
    }

    let action = if take_profit {
        "trigger_take_profit"
    } else {
        "trigger_stop_loss"
    };

    Ok(Response::new()
        .add_submessages(msgs)
        .add_attribute("action", action)
        .add_attributes(vec![("vamm", &vamm_addr.into_string())]))
}

pub fn trigger_mutiple_tp_sl(
    deps: DepsMut,
    vamm: String,
    side: Side,
    take_profit: bool,
    limit: u32,
) -> StdResult<Response> {
    let config = read_config(deps.storage)?;
    let vamm_addr = deps.api.addr_validate(&vamm)?;
    let mut msgs: Vec<SubMsg> = vec![];
    let mut tp_sl_flag: bool = false;

    let vamm_controller = VammController(vamm_addr.clone());
    let vamm_state = vamm_controller.state(&deps.querier)?;

    // check that vamm is open
    if !vamm_state.open {
        return Err(StdError::generic_err("vAMM is not open"));
    }

    // query pool reserves of the vamm so that we can simulate it while triggering tp sl.
    // after simulating, we will know if the position is qualified to close or not
    let mut tmp_reserve = TmpReserveInfo {
        quote_asset_reserve: vamm_state.quote_asset_reserve,
        base_asset_reserve: vamm_state.base_asset_reserve,
    };

    let order_by = if take_profit == (side == Side::Buy) {
        Order::Descending
    } else {
        Order::Ascending
    };

    let ticks = query_ticks(
        deps.storage,
        vamm.clone(),
        side,
        None,
        Some(limit),
        Some(order_by.into()),
    )?;

    for tick in &ticks.ticks {
        let position_by_price = query_positions(
            deps.storage,
            vamm.clone(),
            Some(side),
            PositionFilter::Price(tick.entry_price),
            None,
            None,
            Some(Order::Ascending.into()),
        )?;

        for position in &position_by_price {
            // check the position isn't zero
            require_position_not_zero(position.size.value)?;

            if !take_profit {
                // Can not trigger stop loss position if bad debt
                if position_is_bad_debt(deps.as_ref(), position, &vamm_controller)? {
                    continue;
                }

                // Can not trigger stop loss position if liquidate
                if position_is_liquidated(
                    deps.as_ref(),
                    &position,
                    config.maintenance_margin_ratio,
                    &vamm_controller,
                )? {
                    continue;
                }
            }

            let base_asset_amount = position.size.value;
            let quote_asset_amount = get_output_price_with_reserves(
                config.decimals,
                &position.direction,
                base_asset_amount,
                tmp_reserve.quote_asset_reserve,
                tmp_reserve.base_asset_reserve,
            )?;
            let close_price = quote_asset_amount
                .checked_mul(config.decimals)?
                .checked_div(base_asset_amount)?;

            let stop_loss = position.stop_loss.unwrap_or_default();
            let (tp_spread, sl_spread) = calculate_tp_sl_spread(
                config.tp_sl_spread,
                position.take_profit,
                stop_loss,
                config.decimals,
            )?;
            let tp_sl_action = check_tp_sl_price(
                close_price,
                position.take_profit,
                stop_loss,
                tp_spread,
                sl_spread,
                &position.side,
            )?;
            if take_profit {
                if tp_sl_action == "trigger_take_profit" {
                    tp_sl_flag = true;
                }
            } else {
                if tp_sl_action == "trigger_stop_loss" {
                    tp_sl_flag = true;
                }
            }
            if tp_sl_flag {
                tp_sl_flag = false;
                let _ = update_reserve(
                    &mut tmp_reserve,
                    quote_asset_amount,
                    base_asset_amount,
                    &position.direction,
                );
                msgs.push(internal_close_position(
                    deps.storage,
                    &position,
                    Uint128::zero(),
                    CLOSE_POSITION_REPLY_ID,
                )?);
            }
        }
    }

    let action = if take_profit {
        "trigger_take_profit"
    } else {
        "trigger_stop_loss"
    };

    Ok(Response::new()
        .add_submessages(msgs)
        .add_attribute("action", action)
        .add_attributes(vec![
            ("vamm", &vamm_addr.into_string()),
            ("side", &format!("{:?}", &side)),
        ]))
}

pub fn liquidate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vamm: String,
    position_id: u64,
    quote_asset_limit: Uint128,
) -> StdResult<Response> {
    let config = read_config(deps.storage)?;

    // validate address inputs
    let vamm = deps.api.addr_validate(&vamm)?;

    // read the position for the trader from vamm
    let vamm_key = keccak_256(&[vamm.as_bytes()].concat());
    let position = read_position(deps.storage, &vamm_key, position_id)?;

    // store the liquidator
    store_tmp_liquidator(deps.storage, &info.sender)?;

    // retrieve the existing margin ratio of the position
    let mut margin_ratio = query_margin_ratio(deps.as_ref(), &position)?;

    let vamm_controller = VammController(vamm.clone());

    if vamm_controller.is_over_spread_limit(&deps.querier)? {
        let oracle_margin_ratio =
            get_margin_ratio_calc_option(deps.as_ref(), &position, PnlCalcOption::Oracle)?;

        if oracle_margin_ratio.checked_sub(margin_ratio)? > Integer::zero() {
            margin_ratio = oracle_margin_ratio
        }
    }

    require_vamm(deps.as_ref(), &config.insurance_fund, &vamm)?;
    require_insufficient_margin(margin_ratio, config.maintenance_margin_ratio)?;

    // check the position isn't zero
    require_position_not_zero(position.size.value)?;

    // first see if this is a partial liquidation, else get rekt
    let msg = if margin_ratio.value > config.liquidation_fee
        && !config.partial_liquidation_ratio.is_zero()
    {
        partial_liquidation(deps, env, vamm.clone(), position_id, quote_asset_limit)?
    } else {
        internal_close_position(
            deps.storage,
            &position,
            quote_asset_limit,
            LIQUIDATION_REPLY_ID,
        )?
    };

    Ok(Response::new().add_submessage(msg).add_attributes(vec![
        ("action", "liquidate"),
        ("vamm", vamm.as_ref()),
        ("pair", &position.pair),
        ("position_id", &position_id.to_string()),
        ("margin_ratio", &margin_ratio.to_string()),
        (
            "maintenance_margin_ratio",
            &config.maintenance_margin_ratio.to_string(),
        ),
        ("trader", &position.trader.as_ref()),
    ]))
}

/// settles funding in amm specified
pub fn pay_funding(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    vamm: String,
) -> StdResult<Response> {
    // validate address inputs
    let vamm = deps.api.addr_validate(&vamm)?;
    let config = read_config(deps.storage)?;
    // check its a valid vamm
    require_vamm(deps.as_ref(), &config.insurance_fund, &vamm)?;

    let funding_msg = SubMsg::reply_always(
        wasm_execute(vamm, &ExecuteMsg::SettleFunding {}, vec![])?,
        PAY_FUNDING_REPLY_ID,
    );

    Ok(Response::new()
        .add_submessage(funding_msg)
        .add_attribute("action", "pay_funding"))
}

/// Enables a user to directly deposit margin into their position
pub fn deposit_margin(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vamm: String,
    position_id: u64,
    amount: Uint128,
) -> StdResult<Response> {
    let vamm = deps.api.addr_validate(&vamm)?;
    let trader = info.sender.clone();

    let state = read_state(deps.storage)?;
    require_not_paused(state.pause)?;
    require_non_zero_input(amount)?;

    // first try to execute the transfer
    let mut response = Response::new();

    let config = read_config(deps.storage)?;

    match config.eligible_collateral.clone() {
        AssetInfo::NativeToken { .. } => {
            let token = Asset {
                info: config.eligible_collateral,
                amount,
            };

            token.assert_sent_native_token_balance(&info)?;
        }

        AssetInfo::Token { .. } => {
            let msg = execute_transfer_from(deps.storage, &trader, &env.contract.address, amount)?;
            response = response.add_submessage(msg);
        }
    };
    let vamm_key = keccak_256(&[vamm.as_bytes()].concat());
    // read the position for the trader from vamm
    let mut position = read_position(deps.storage, &vamm_key, position_id)?;

    if position.trader != trader {
        return Err(StdError::generic_err("Unauthorized"));
    }

    position.margin = position.margin.checked_add(amount)?;

    store_position(deps.storage, &vamm_key, &position, false)?;

    Ok(response.add_attributes([
        ("action", "deposit_margin"),
        ("position_id", &position_id.to_string()),
        ("trader", trader.as_ref()),
        ("deposit_amount", &amount.to_string()),
    ]))
}

/// Enables a user to directly withdraw excess margin from their position
pub fn withdraw_margin(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vamm: String,
    position_id: u64,
    amount: Uint128,
) -> StdResult<Response> {
    // get and validate address inputs
    let vamm = deps.api.addr_validate(&vamm)?;
    let trader = info.sender;

    let config = read_config(deps.storage)?;
    require_vamm(deps.as_ref(), &config.insurance_fund, &vamm)?;
    let mut state = read_state(deps.storage)?;
    require_not_paused(state.pause)?;
    require_non_zero_input(amount)?;

    // read the position for the trader from vamm
    let vamm_key = keccak_256(&[vamm.as_bytes()].concat());
    let mut position = read_position(deps.storage, &vamm_key, position_id)?;

    if position.trader != trader {
        return Err(StdError::generic_err("Unauthorized"));
    }

    let remain_margin = calc_remain_margin_with_funding_payment(
        deps.as_ref(),
        &position,
        Integer::new_negative(amount),
    )?;
    require_bad_debt(remain_margin.bad_debt)?;

    position.margin = remain_margin.margin;
    position.last_updated_premium_fraction = remain_margin.latest_premium_fraction;

    // check if margin is sufficient
    let free_collateral = query_free_collateral(deps.as_ref(), vamm.to_string(), position_id)?;
    if free_collateral
        .checked_sub(Integer::new_positive(amount))?
        .is_negative()
    {
        return Err(StdError::generic_err("Insufficient collateral"));
    }

    let fees = position.spread_fee.checked_add(position.toll_fee)?;
    // withdraw margin
    let msgs = withdraw(
        deps.as_ref(),
        env,
        &mut state,
        &trader,
        config.eligible_collateral,
        amount,
        fees,
        Uint128::zero(),
    )?;

    store_position(deps.storage, &vamm_key, &position, false)?;
    store_state(deps.storage, &state)?;

    Ok(Response::new().add_submessages(msgs).add_attributes(vec![
        ("action", "withdraw_margin"),
        ("position_id", &position_id.to_string()),
        ("trader", trader.as_ref()),
        ("withdrawal_amount", &amount.to_string()),
    ]))
}

// Open position via vamm
pub fn internal_open_position(
    vamm: Addr,
    side: Side,
    position_id: u64,
    open_notional: Uint128,
    base_asset_limit: Uint128,
) -> StdResult<SubMsg> {
    swap_input(
        &vamm,
        &side,
        position_id,
        open_notional,
        base_asset_limit,
        false,
        INCREASE_POSITION_REPLY_ID,
    )
}

// Close position via vamm
pub fn internal_close_position(
    storage: &mut dyn Storage,
    position: &Position,
    quote_asset_limit: Uint128,
    id: u64,
) -> StdResult<SubMsg> {
    let side = direction_to_side(&position.direction);
    store_tmp_swap(
        storage,
        &TmpSwapInfo {
            position_id: position.position_id,
            vamm: position.vamm.clone(),
            pair: position.pair.clone(),
            trader: position.trader.clone(),
            side: side.clone(),
            margin_amount: position.size.value,
            leverage: Uint128::zero(),
            open_notional: position.notional,
            position_notional: Uint128::zero(),
            unrealized_pnl: Integer::zero(),
            margin_to_vault: Integer::zero(),
            take_profit: position.take_profit,
            stop_loss: position.stop_loss,
            spread_fee: position.spread_fee,
            toll_fee: position.toll_fee,
        },
    )?;

    swap_output(
        &position.vamm,
        &side,
        position.position_id,
        position.size.value,
        quote_asset_limit,
        id,
    )
}

fn partial_liquidation(
    deps: DepsMut,
    _env: Env,
    vamm: Addr,
    position_id: u64,
    quote_asset_limit: Uint128,
) -> StdResult<SubMsg> {
    let vamm_key = keccak_256(&[vamm.as_bytes()].concat());
    let position = read_position(deps.storage, &vamm_key, position_id)?;
    let config = read_config(deps.storage)?;
    let partial_position_size = position
        .size
        .value
        .checked_mul(config.partial_liquidation_ratio)?
        .checked_div(config.decimals)?;

    let partial_asset_limit = quote_asset_limit
        .checked_mul(config.partial_liquidation_ratio)?
        .checked_div(config.decimals)?;

    let vamm_controller = VammController(vamm.clone());

    let current_notional = vamm_controller.output_amount(
        &deps.querier,
        position.direction.clone(),
        partial_position_size,
    )?;

    let PositionUnrealizedPnlResponse {
        position_notional: _,
        unrealized_pnl,
    } = get_position_notional_unrealized_pnl(deps.as_ref(), &position, PnlCalcOption::SpotPrice)?;

    let side = position_to_side(position.size);

    store_tmp_swap(
        deps.storage,
        &TmpSwapInfo {
            position_id: position.position_id,
            vamm: position.vamm.clone(),
            pair: position.pair.clone(),
            trader: position.trader.clone(),
            side,
            margin_amount: partial_position_size,
            leverage: Uint128::zero(),
            open_notional: current_notional,
            position_notional: Uint128::zero(),
            unrealized_pnl,
            margin_to_vault: Integer::zero(),
            take_profit: position.take_profit,
            stop_loss: position.stop_loss,
            spread_fee: position.spread_fee,
            toll_fee: position.toll_fee,
        },
    )?;

    let msg = if current_notional > position.notional {
        swap_input(
            &vamm,
            &direction_to_side(&position.direction),
            position.position_id,
            position.notional,
            Uint128::zero(),
            true,
            PARTIAL_LIQUIDATION_REPLY_ID,
        )?
    } else {
        swap_output(
            &vamm,
            &direction_to_side(&position.direction),
            position.position_id,
            partial_position_size,
            partial_asset_limit,
            PARTIAL_LIQUIDATION_REPLY_ID,
        )?
    };

    Ok(msg)
}

fn swap_input(
    vamm: &Addr,
    side: &Side,
    position_id: u64,
    open_notional: Uint128,
    base_asset_limit: Uint128,
    can_go_over_fluctuation: bool,
    id: u64,
) -> StdResult<SubMsg> {
    let msg = wasm_execute(
        vamm,
        &ExecuteMsg::SwapInput {
            direction: side_to_direction(side),
            position_id,
            quote_asset_amount: open_notional,
            base_asset_limit,
            can_go_over_fluctuation,
        },
        vec![],
    )?;

    Ok(SubMsg::reply_always(msg, id))
}

fn swap_output(
    vamm: &Addr,
    side: &Side,
    position_id: u64,
    open_notional: Uint128,
    quote_asset_limit: Uint128,
    id: u64,
) -> StdResult<SubMsg> {
    let msg = wasm_execute(
        vamm,
        &ExecuteMsg::SwapOutput {
            direction: side_to_direction(side),
            position_id,
            base_asset_amount: open_notional,
            quote_asset_limit,
        },
        vec![],
    )?;

    Ok(SubMsg::reply_always(msg, id))
}
