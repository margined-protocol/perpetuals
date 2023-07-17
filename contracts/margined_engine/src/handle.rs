use cosmwasm_std::{
    Addr, DepsMut, Env, MessageInfo, Response, StdError, StdResult, SubMsg, Uint128, QuerierWrapper,
};
use margined_utils::contracts::helpers::VammController;

use crate::{
    contract::{
        CLOSE_POSITION_REPLY_ID, INCREASE_POSITION_REPLY_ID,
        LIQUIDATION_REPLY_ID, PARTIAL_CLOSE_POSITION_REPLY_ID, PARTIAL_LIQUIDATION_REPLY_ID,
        PAY_FUNDING_REPLY_ID, TAKE_PROFIT_REPLY_ID, STOP_LOSS_REPLY_ID,
    },
    messages::{execute_transfer_from, withdraw},
    query::{query_free_collateral, query_margin_ratio},
    state::{
        read_config, read_position, read_state, store_config, store_position, store_sent_funds,
        store_state, store_tmp_liquidator, store_tmp_swap, SentFunds, TmpSwapInfo, increase_last_position_id,
    },
    utils::{
        calc_remain_margin_with_funding_payment, direction_to_side, get_asset,
        get_margin_ratio_calc_option, get_position_notional_unrealized_pnl,
        keccak_256, position_to_side, require_additional_margin, require_bad_debt,
        require_insufficient_margin, require_non_zero_input, require_not_paused,
        require_not_restriction_mode, require_position_not_zero, require_vamm, side_to_direction,
    },
};
use margined_common::{
    asset::{Asset, AssetInfo},
    integer::Integer,
    messages::wasm_execute,
    validate::{validate_margin_ratios, validate_ratio},
};
use margined_perp::margined_engine::{
    PnlCalcOption, Position, PositionUnrealizedPnlResponse, Side,
};
use margined_perp::margined_vamm::{Direction, ExecuteMsg, QueryMsg};

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
        println!("[0] update margined engine - initial_margin_ratio: {}", initial_margin_ratio);
        println!("[0] update margined engine - maintenance_margin_ratio: {}", config.maintenance_margin_ratio);
        config.initial_margin_ratio = initial_margin_ratio;
    }

    // update maintenance margin ratio
    if let Some(maintenance_margin_ratio) = maintenance_margin_ratio {
        validate_ratio(maintenance_margin_ratio, config.decimals)?;
        validate_margin_ratios(config.initial_margin_ratio, maintenance_margin_ratio)?;
        println!("[1] update margined engine - initial_margin_ratio: {}", config.initial_margin_ratio);
        println!("[1] update margined engine - maintenance_margin_ratio: {}", maintenance_margin_ratio);

        config.maintenance_margin_ratio = maintenance_margin_ratio;
    }

    // update partial liquidation ratio
    if let Some(partial_liquidation_ratio) = partial_liquidation_ratio {
        validate_ratio(partial_liquidation_ratio, config.decimals)?;
        config.partial_liquidation_ratio = partial_liquidation_ratio;
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
    let trader = info.sender.clone();

    let state = read_state(deps.storage)?;
    require_not_paused(state.pause)?;
    let config = read_config(deps.storage)?;
    require_vamm(deps.as_ref(), &config.insurance_fund, &vamm)?;
    let position_id = increase_last_position_id(deps.storage)?;

    require_not_restriction_mode(deps.storage, &vamm, env.block.height)?;
    require_non_zero_input(margin_amount)?;
    require_non_zero_input(leverage)?;

    if leverage < config.decimals {
        return Err(StdError::generic_err("Leverage must be greater than 1"));
    }

    // calculate the margin ratio of new position wrt to leverage
    let margin_ratio = config
        .decimals
        .checked_mul(config.decimals)?
        .checked_div(leverage)?;

    println!("open_position - margin_amount: {}", margin_amount);
    println!("open_position - margin_ratio: {}", margin_ratio);

    require_additional_margin(Integer::from(margin_ratio), config.initial_margin_ratio)?;
    
    // creates a new position
    let position: Position = Position {
        position_id,
        vamm: vamm.clone(),
        trader: trader.clone(),
        side: side.clone(),
        direction: side_to_direction(&side),
        size: Integer::zero(),
        margin: Uint128::zero(),
        notional: Uint128::zero(),
        entry_price: Uint128::zero(),
        take_profit: Uint128::zero(),
        stop_loss: Some(Uint128::zero()), 
        last_updated_premium_fraction: Integer::zero(),
        block_time: 0u64
    };

    println!("open_position - direction: {:?}", position.direction);
    println!("open_position - side: {:?}", position.side);
    // calculate the position notional
    let open_notional = margin_amount
        .checked_mul(leverage)?
        .checked_div(config.decimals)?;
    println!("open_position - open_notional: {:?}", open_notional);
    let msg = internal_increase_position(vamm.clone(), side.clone(), position_id, open_notional, base_asset_limit)?;

    let PositionUnrealizedPnlResponse {
        position_notional,
        unrealized_pnl,
    } = get_position_notional_unrealized_pnl(deps.as_ref(), &position, PnlCalcOption::SpotPrice)?;
    println!("open_position - position_notional: {:?}", position_notional);
    
    store_tmp_swap(
        deps.storage,
        &TmpSwapInfo {
            position_id,
            vamm: vamm.clone(),
            trader: trader.clone(),
            side: side.clone(),
            margin_amount,
            leverage,
            open_notional,
            position_notional,
            unrealized_pnl,
            margin_to_vault: Integer::zero(),
            fees_paid: false,
            take_profit,
            stop_loss
        }
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
        ("position_side",  &format!("{:?}", side)),
        ("vamm", vamm.as_ref()),
        ("trader", trader.as_ref()),
        ("margin_amount", &margin_amount.to_string()),
        ("leverage", &leverage.to_string()),
    ]))
}

pub fn update_tp_sl(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    vamm: String,
    position_id: u64,
    take_profit: Option<Uint128>,
    stop_loss: Option<Uint128>
) -> StdResult<Response> {
    let vamm = deps.api.addr_validate(&vamm)?;
    let trader = info.sender;

    // read the position for the trader from vamm
    let position_key = keccak_256(&[vamm.as_bytes()].concat());
    let mut position = read_position(deps.storage, &position_key, position_id)?;

    let state = read_state(deps.storage)?;
    require_not_paused(state.pause)?;
    require_position_not_zero(position.size.value)?;

    if position.trader != trader {
        return Err(StdError::generic_err("Unauthorized"));
    }
    
    if Some(take_profit).is_none() && Some(stop_loss).is_none() {
        return Err(StdError::generic_err("Both take profit and stop loss are not set"));
    }

    match take_profit {
        Some(tp) => {
            position.take_profit = tp;
        }
        None => {},
    }

    match stop_loss {
        Some(_) => {
            position.stop_loss = stop_loss;
        }
        None => {},
    }
    
    store_position(deps.storage, &position_key, &position, false)?;
    
    Ok(Response::new().add_attributes(vec![
        ("action", "update_tp_sl"),
        ("vamm", vamm.as_ref()),
        ("position_id", &position_id.to_string()),
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
    // validate address inputs
    let vamm = deps.api.addr_validate(&vamm)?;
    let trader = info.sender;

    // read the position for the trader from vamm
    let position_key = keccak_256(&[vamm.as_bytes()].concat());
    let position = read_position(deps.storage, &position_key, position_id)?;

    let state = read_state(deps.storage)?;

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

    let config = read_config(deps.storage)?;

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
                trader: position.trader.clone(),
                side: side.clone(),
                margin_amount: position.size.value,
                leverage: config.decimals,
                open_notional: partial_close_notional,
                position_notional,
                unrealized_pnl,
                margin_to_vault: Integer::zero(),
                fees_paid: false,
                take_profit: position.take_profit,
                stop_loss: position.stop_loss
            }
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
        internal_close_position(deps, &position, quote_amount_limit, CLOSE_POSITION_REPLY_ID)?
    };
    

    Ok(Response::new().add_submessage(msg).add_attributes(vec![
        ("action", "close_position"),
        ("vamm", vamm.as_ref()),
        ("trader", trader.as_ref()),
    ]))
}

pub fn trigger_tp_sl(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    vamm: String,
    position_id: u64,
    quote_asset_limit: Uint128
) -> StdResult<Response> {
    let vamm = deps.api.addr_validate(&vamm)?;
    let vamm_controller = VammController(vamm.clone());

    // retrieve the existing margin ratio of the position
    let mut margin_ratio = query_margin_ratio(deps.as_ref(), vamm.to_string(), position_id)?;

    if vamm_controller.is_over_spread_limit(&deps.querier)? {
        let oracle_margin_ratio = get_margin_ratio_calc_option(
            deps.as_ref(),
            vamm.to_string(),
            position_id,
            PnlCalcOption::Oracle,
        )?;
        println!("tp_sl - oracle_margin_ratio: {:?}", oracle_margin_ratio);

        if oracle_margin_ratio.checked_sub(margin_ratio)? > Integer::zero() {
            margin_ratio = oracle_margin_ratio
        }
    }

    let config = read_config(deps.storage)?;
    require_vamm(deps.as_ref(), &config.insurance_fund, &vamm)?;
    println!("tp_sl - config.maintenance_margin_ratio: {:?}", config.maintenance_margin_ratio);
    println!("tp_sl - margin_ratio: {:?}", margin_ratio);
    // require_insufficient_margin(margin_ratio, config.maintenance_margin_ratio)?;

    // read the position for the trader from vamm
    let position_key = keccak_256(&[vamm.as_bytes()].concat());
    let position = read_position(deps.storage, &position_key, position_id)?;
    println!("tp_sl - position: {:?}", position);

    // check the position isn't zero
    require_position_not_zero(position.size.value)?;

    let spot_price = get_spot_price(&deps.querier, &vamm)?;
    println!("tp_sl - spot_price: {:?}", spot_price);
    let stop_loss = position.stop_loss.unwrap_or_default();
    let mut msgs: Vec<SubMsg> = vec![];

    // if spot_price is ~ take_profit or stop_loss, close position
    if position.side == Side::Buy {
        println!("tp_sl - position.side: {:?}", position.side);
        if spot_price <= position.take_profit && 
            position.take_profit.checked_mul(5u128.into())?.checked_div(1000u128.into())? 
            >= position.take_profit.checked_sub(spot_price)? ||
            spot_price > position.take_profit {
            msgs.push(internal_close_position(deps, &position, quote_asset_limit, TAKE_PROFIT_REPLY_ID)?);
        } else if stop_loss > Uint128::zero() && 
            spot_price >= stop_loss && spot_price.checked_sub(stop_loss)? 
            <= stop_loss.checked_mul(5u128.into())?.checked_div(1000u128.into())? ||
            spot_price < position.take_profit {
            println!("tp_sl - spot_price: {:?}", spot_price);
            msgs.push(internal_close_position(deps, &position, quote_asset_limit, STOP_LOSS_REPLY_ID)?);
        };
    } else if position.side == Side::Sell {
        println!("tp_sl - position.side: {:?}", position.side);
        if spot_price >= position.take_profit && 
            position.take_profit.checked_mul(5u128.into())?.checked_div(1000u128.into())? 
            >= spot_price.checked_sub(position.take_profit)? ||
            spot_price < position.take_profit {
            msgs.push(internal_close_position(deps, &position, quote_asset_limit, TAKE_PROFIT_REPLY_ID)?);
        } else if stop_loss > Uint128::zero() && 
            spot_price <= stop_loss && stop_loss.checked_sub(spot_price)? 
            <= stop_loss.checked_mul(5u128.into())?.checked_div(1000u128.into())? ||
            spot_price > position.take_profit {
            msgs.push(internal_close_position(deps, &position, quote_asset_limit, STOP_LOSS_REPLY_ID)?);
        };
    }

    Ok(Response::new().add_submessages(msgs).add_attributes(vec![
        ("action", "tp_sl"),
        ("vamm", vamm.as_ref()),
    ]))
}

pub fn liquidate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vamm: String,
    position_id: u64,
    trader: String,
    quote_asset_limit: Uint128,
) -> StdResult<Response> {
    // validate address inputs
    let vamm = deps.api.addr_validate(&vamm)?;
    let trader = deps.api.addr_validate(&trader)?;
    println!("liquidate - vamm: {:?}", vamm);
    println!("liquidate - trader: {:?}", trader);

    // store the liquidator
    store_tmp_liquidator(deps.storage, &info.sender)?;

    // retrieve the existing margin ratio of the position
    let mut margin_ratio = query_margin_ratio(deps.as_ref(), vamm.to_string(), position_id)?;
    println!("liquidate - margin_ratio: {:?}", margin_ratio);

    let vamm_controller = VammController(vamm.clone());
    println!("liquidate - vamm_controller: {:?}", vamm_controller);

    if vamm_controller.is_over_spread_limit(&deps.querier)? {
        let oracle_margin_ratio = get_margin_ratio_calc_option(
            deps.as_ref(),
            vamm.to_string(),
            position_id,
            PnlCalcOption::Oracle,
        )?;
        println!("liquidate - oracle_margin_ratio: {:?}", oracle_margin_ratio);

        if oracle_margin_ratio.checked_sub(margin_ratio)? > Integer::zero() {
            margin_ratio = oracle_margin_ratio
        }
    }

    let config = read_config(deps.storage)?;
    require_vamm(deps.as_ref(), &config.insurance_fund, &vamm)?;
    println!("liquidate - config.maintenance_margin_ratio: {:?}", config.maintenance_margin_ratio);
    require_insufficient_margin(margin_ratio, config.maintenance_margin_ratio)?;

    // read the position for the trader from vamm
    let position_key = keccak_256(&[vamm.as_bytes()].concat());
    let position = read_position(deps.storage, &position_key, position_id)?;
    println!("liquidate - position: {:?}", position);

    // check the position isn't zero
    require_position_not_zero(position.size.value)?;

    // first see if this is a partial liquidation, else get rekt
    let msg = if margin_ratio.value > config.liquidation_fee
        && !config.partial_liquidation_ratio.is_zero()
    {
        partial_liquidation(deps, env, vamm.clone(), position_id, quote_asset_limit)?
    } else {
        internal_close_position(deps, &position, quote_asset_limit, LIQUIDATION_REPLY_ID)?
    };

    Ok(Response::new().add_submessage(msg).add_attributes(vec![
        ("action", "liquidate"),
        ("vamm", vamm.as_ref()),
        ("trader", trader.as_ref()),
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
    println!("pay_funding - config: {:?}", config);
    // check its a valid vamm
    require_vamm(deps.as_ref(), &config.insurance_fund, &vamm)?;

    let funding_msg = SubMsg::reply_always(
        wasm_execute(vamm, &ExecuteMsg::SettleFunding {}, vec![])?,
        PAY_FUNDING_REPLY_ID,
    );
    println!("pay_funding - funding_msg: {:?}", funding_msg);

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
    let position_key = keccak_256(&[vamm.as_bytes()].concat());
    // read the position for the trader from vamm
    let mut position = read_position(deps.storage, &position_key, position_id)?;

    if position.trader != trader {
        return Err(StdError::generic_err("Unauthorized"));
    }

    position.margin = position.margin.checked_add(amount)?; // TODO: @lehieuhust check if leverage is less than 1

    store_position(deps.storage, &position_key, &position, false)?;

    Ok(response.add_attributes([
        ("action", "deposit_margin"),
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
    let position_key = keccak_256(&[vamm.as_bytes()].concat());
    let mut position = read_position(deps.storage, &position_key, position_id)?;

    if position.trader != trader {
        return Err(StdError::generic_err("Unauthorized"));
    }

    let remain_margin = calc_remain_margin_with_funding_payment(
        deps.as_ref(),
        position.clone(),
        Integer::new_negative(amount),
    )?;
    require_bad_debt(remain_margin.bad_debt)?;

    position.margin = remain_margin.margin;
    position.last_updated_premium_fraction = remain_margin.latest_premium_fraction;

    // check if margin is sufficient
    let free_collateral =
        query_free_collateral(deps.as_ref(), vamm.to_string(), position_id)?;
    if free_collateral
        .checked_sub(Integer::new_positive(amount))?
        .is_negative()
    {
        return Err(StdError::generic_err("Insufficient collateral"));
    }

    // withdraw margin
    let msgs = withdraw(
        deps.as_ref(),
        env,
        &mut state,
        &trader,
        config.eligible_collateral,
        amount,
        Uint128::zero(),
    )?;

    store_position(deps.storage, &position_key, &position, false)?;
    store_state(deps.storage, &state)?;

    Ok(Response::new().add_submessages(msgs).add_attributes(vec![
        ("action", "withdraw_margin"),
        ("trader", trader.as_ref()),
        ("withdrawal_amount", &amount.to_string()),
    ]))
}

// Increase the position through a swap
pub fn internal_increase_position(
    vamm: Addr,
    side: Side,
    position_id: u64,
    open_notional: Uint128,
    base_asset_limit: Uint128,
) -> StdResult<SubMsg> {
    println!("internal_increase_position - open_notional: {}", open_notional);
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

pub fn internal_close_position(
    deps: DepsMut,
    position: &Position,
    quote_asset_limit: Uint128,
    id: u64,
) -> StdResult<SubMsg> {
    let side = direction_to_side(&position.direction);
    store_tmp_swap(
        deps.storage,
        &TmpSwapInfo {
            position_id: position.position_id,
            vamm: position.vamm.clone(),
            trader: position.trader.clone(),
            side: side.clone(),
            margin_amount: position.size.value,
            leverage: Uint128::zero(),
            open_notional: position.notional,
            position_notional: Uint128::zero(),
            unrealized_pnl: Integer::zero(),
            margin_to_vault: Integer::zero(),
            fees_paid: false,
            take_profit: position.take_profit,
            stop_loss: position.stop_loss
        }
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
    let position_key = keccak_256(&[vamm.as_bytes()].concat());
    let position = read_position(deps.storage, &position_key, position_id)?;
    let config = read_config(deps.storage)?;
    let partial_position_size = position
        .size
        .value
        .checked_mul(config.partial_liquidation_ratio)?
        .checked_div(config.decimals)?;
    println!("partial_liquidation - partial_position_size: {:?}", partial_position_size);
    let partial_asset_limit = quote_asset_limit
        .checked_mul(config.partial_liquidation_ratio)?
        .checked_div(config.decimals)?;
    println!("partial_liquidation - quote_asset_limit: {:?}", quote_asset_limit);
    let vamm_controller = VammController(vamm.clone());

    let current_notional = vamm_controller.output_amount(
        &deps.querier,
        position.direction.clone(),
        partial_position_size,
    )?;
    println!("partial_liquidation - current_notional: {:?}", current_notional);
    let PositionUnrealizedPnlResponse {
        position_notional: _,
        unrealized_pnl,
    } = get_position_notional_unrealized_pnl(deps.as_ref(), &position, PnlCalcOption::SpotPrice)?;
    println!("partial_liquidation - unrealized_pnl: {:?}", unrealized_pnl);
    println!("partial_liquidation - position.size: {:?}", position.size);
    let side = position_to_side(position.size);

    store_tmp_swap(
        deps.storage,
        &TmpSwapInfo {
            position_id: position.position_id,
            vamm: position.vamm.clone(),
            trader: position.trader.clone(),
            side,
            margin_amount: partial_position_size,
            leverage: Uint128::zero(),
            open_notional: current_notional,
            position_notional: Uint128::zero(),
            unrealized_pnl,
            margin_to_vault: Integer::zero(),
            fees_paid: false,
            take_profit: position.take_profit,
            stop_loss: position.stop_loss
        }
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
    println!("[HIEU_LOG] swap_output: {:?}, id: {:?}", msg, id);

    Ok(SubMsg::reply_always(msg, id))
}

fn get_spot_price(
    querier: &QuerierWrapper,
    vamm: &Addr,
) -> StdResult<Uint128> {
    querier.query_wasm_smart(vamm, &QueryMsg::SpotPrice {})
}