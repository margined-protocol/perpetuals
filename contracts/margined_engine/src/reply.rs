use cosmwasm_std::{DepsMut, Env, Response, StdError, StdResult, SubMsg, Uint128};
use margined_utils::contracts::helpers::VammController;

use crate::{
    messages::{
        execute_insurance_fund_withdrawal, execute_transfer, execute_transfer_from,
        execute_transfer_to_insurance_fund, transfer_fees, withdraw,
    },
    state::{
        append_cumulative_premium_fraction, enter_restriction_mode, read_config, read_position,
        read_sent_funds, read_state, read_tmp_liquidator, read_tmp_swap, remove_position,
        remove_sent_funds, remove_tmp_liquidator, remove_tmp_swap, store_position, store_state,
        State,
    },
    utils::{
        calc_remain_margin_with_funding_payment, check_base_asset_holding_cap, keccak_256,
        realize_bad_debt, side_to_direction, update_open_interest_notional,
    },
};

use margined_common::{asset::AssetInfo, integer::Integer};
use margined_perp::{
    margined_engine::{Position, RemainMarginResponse, Side},
    margined_vamm::Direction,
};

// Updates position after successful execution of the swap
pub fn open_position_reply(
    deps: DepsMut,
    env: Env,
    input: Uint128,
    output: Uint128,
    position_id: u64,
) -> StdResult<Response> {
    let mut swap = read_tmp_swap(deps.storage, &position_id.to_be_bytes())?;

    let mut position = Position {
        position_id: swap.position_id,
        vamm: swap.vamm.clone(),
        trader: swap.trader.clone(),
        pair: swap.pair,
        side: swap.side.clone(),
        direction: side_to_direction(&swap.side),
        size: Integer::zero(),
        margin: Uint128::zero(),
        notional: Uint128::zero(),
        entry_price: Uint128::zero(),
        take_profit: swap.take_profit,
        stop_loss: swap.stop_loss,
        last_updated_premium_fraction: Integer::zero(),
        spread_fee: swap.spread_fee,
        toll_fee: swap.toll_fee,
        block_time: env.block.time.seconds(),
    };

    // depending on the direction the output is positive or negative
    let signed_output = match &swap.side {
        Side::Buy => Integer::new_positive(output),
        Side::Sell => Integer::new_negative(output),
    };

    let mut state = read_state(deps.storage)?;
    let config = read_config(deps.storage)?;

    update_open_interest_notional(
        &deps.as_ref(),
        &mut state,
        swap.vamm.clone(),
        Integer::new_positive(input),
        swap.trader.clone(),
    )?;

    // define variables that differ across increase and decrease scenario
    let swap_margin;
    let margin_delta;

    // calculate margin needed given swap
    swap_margin = swap
        .open_notional
        .checked_mul(config.decimals)?
        .checked_div(swap.leverage)?;

    swap.margin_to_vault = swap
        .margin_to_vault
        .checked_add(Integer::new_positive(swap_margin))?;

    margin_delta = Integer::new_positive(swap_margin);

    // calculate the remaining margin
    let RemainMarginResponse {
        funding_payment: _,
        margin,
        bad_debt: _,
        latest_premium_fraction,
    } = calc_remain_margin_with_funding_payment(deps.as_ref(), &position, margin_delta)?;

    // set the new position
    position.notional = swap.open_notional;
    position.size += signed_output;
    position.margin = margin;
    position.last_updated_premium_fraction = latest_premium_fraction;
    position.entry_price = position
        .notional
        .checked_mul(config.decimals)?
        .checked_div(position.size.value)?;
    position.block_time = env.block.time.seconds();

    let vamm_key = keccak_256(&[position.vamm.as_bytes()].concat());
    store_position(deps.storage, &vamm_key, &position, true)?;

    // check the new position doesn't exceed any caps
    check_base_asset_holding_cap(
        &deps.as_ref(),
        swap.vamm.clone(),
        position.size.value,
        swap.trader.clone(),
    )?;

    let mut msgs: Vec<SubMsg> = vec![];
    let mut funds = read_sent_funds(deps.storage)?;

    // create transfer messages depending on PnL
    if swap.margin_to_vault.is_positive() {
        match config.eligible_collateral {
            AssetInfo::NativeToken { .. } => {
                funds.required = funds.required.checked_add(swap_margin)?;
            }
            AssetInfo::Token { .. } => {
                msgs.push(execute_transfer_from(
                    deps.storage,
                    &swap.trader,
                    &env.contract.address,
                    swap.margin_to_vault.value,
                )?);
            }
        }
    };

    // create messages to pay for toll and spread fees, check flag is true if this follows a reverse
    let mut fees_messages = transfer_fees(
        deps.as_ref(),
        swap.trader,
        swap.spread_fee,
        swap.toll_fee,
        true,
    )?;
    // add the fee transfer messages
    msgs.append(&mut fees_messages);

    // add the total fees to the required funds counter
    funds.required = funds
        .required
        .checked_add(swap.spread_fee)?
        .checked_add(swap.toll_fee)?;

    // check if native tokens are sufficient
    if let AssetInfo::NativeToken { .. } = config.eligible_collateral {
        funds.are_sufficient()?;
    }

    store_state(deps.storage, &state)?;

    remove_tmp_swap(deps.storage, &position_id.to_be_bytes());
    remove_sent_funds(deps.storage);

    Ok(Response::new().add_submessages(msgs).add_attributes(vec![
        ("action", "open_position_reply"),
        ("entry_price", &position.entry_price.to_string()),
        ("spread_fee", &position.spread_fee.to_string()),
        ("toll_fee", &position.toll_fee.to_string()),
    ]))
}

// Closes position after successful execution of the swap
pub fn close_position_reply(
    deps: DepsMut,
    env: Env,
    _input: Uint128,
    output: Uint128,
    position_id: u64,
) -> StdResult<Response> {
    let swap = read_tmp_swap(deps.storage, &position_id.to_be_bytes())?;
    let vamm_key = keccak_256(&[swap.vamm.as_bytes()].concat());
    let position = read_position(deps.storage, &vamm_key, position_id)?;

    let margin_delta = match &position.direction {
        Direction::AddToAmm => {
            Integer::new_positive(output) - Integer::new_positive(swap.open_notional)
        }
        Direction::RemoveFromAmm => {
            Integer::new_positive(swap.open_notional) - Integer::new_positive(output)
        }
    };

    let RemainMarginResponse {
        funding_payment,
        margin,
        bad_debt,
        latest_premium_fraction: _,
    } = calc_remain_margin_with_funding_payment(deps.as_ref(), &position, margin_delta)?;

    let mut msgs: Vec<SubMsg> = vec![];
    let mut withdraw_amount = Integer::new_positive(margin).checked_add(swap.unrealized_pnl)?;
    let mut spread_fee = Uint128::zero();
    let mut toll_fee = Uint128::zero();

    if withdraw_amount.value > position.spread_fee.checked_add(position.toll_fee)? {
        spread_fee = position.spread_fee;
        toll_fee = position.toll_fee;
        withdraw_amount.value = withdraw_amount
            .value
            .checked_sub(position.spread_fee.checked_add(position.toll_fee)?)?;
    } else {
        if !position
            .spread_fee
            .checked_add(position.toll_fee)?
            .is_zero()
        {
            // If withdraw_amount < spread_fee + toll_fee, we need to re-caculate fees
            // new_spread_fee = withdraw_amount * spread_fee / (spread_fee + toll_fee)
            // new_toll_fee = withdraw_amount - new_spread_fee
            spread_fee = withdraw_amount
                .value
                .checked_mul(position.spread_fee)?
                .checked_div(position.spread_fee.checked_add(position.toll_fee)?)?;
            toll_fee = withdraw_amount.value.checked_sub(spread_fee)?;
            withdraw_amount.value = Uint128::zero();
        }
    }

    // to prevent attacker to leverage the bad debt to withdraw extra token from insurance fund
    if !bad_debt.is_zero() {
        return Err(StdError::generic_err("Cannot close position - bad debt"));
    }

    let mut state = read_state(deps.storage)?;
    if !withdraw_amount.is_zero() {
        let config = read_config(deps.storage)?;
        msgs.append(&mut withdraw(
            deps.as_ref(),
            env,
            &mut state,
            &swap.trader,
            config.eligible_collateral,
            withdraw_amount.value,
            spread_fee.checked_add(toll_fee)?,
            Uint128::zero(),
        )?);
    }

    if !spread_fee.is_zero() && !toll_fee.is_zero() {
        let mut fees_messages = transfer_fees(
            deps.as_ref(),
            swap.trader.clone(),
            spread_fee,
            toll_fee,
            false,
        )?;
        msgs.append(&mut fees_messages);
    }

    let value =
        margin_delta + Integer::new_positive(bad_debt) + Integer::new_positive(position.notional);

    update_open_interest_notional(
        &deps.as_ref(),
        &mut state,
        swap.vamm,
        value.invert_sign(),
        swap.trader,
    )?;

    remove_position(deps.storage, &vamm_key, &position)?;
    store_state(deps.storage, &state)?;
    remove_tmp_swap(deps.storage, &position_id.to_be_bytes());

    Ok(Response::new().add_submessages(msgs).add_attributes(vec![
        ("action", "close_position_reply"),
        ("take_profit", &position.take_profit.to_string()),
        (
            "stop_loss",
            &position.stop_loss.unwrap_or_default().to_string(),
        ),
        ("pnl", &margin_delta.to_string()),
        ("spread_fee", &position.spread_fee.to_string()),
        ("toll_fee", &position.toll_fee.to_string()),
        ("funding_payment", &funding_payment.to_string()),
        ("bad_debt", &bad_debt.to_string()),
        ("withdraw_amount", &withdraw_amount.value.to_string()),
    ]))
}

// Partially closes position
pub fn partial_close_position_reply(
    deps: DepsMut,
    env: Env,
    input: Uint128,
    output: Uint128,
    position_id: u64,
) -> StdResult<Response> {
    let swap = read_tmp_swap(deps.storage, &position_id.to_be_bytes())?;
    let vamm_key = keccak_256(&[swap.vamm.as_bytes()].concat());
    let mut position = read_position(deps.storage, &vamm_key, position_id)?;

    let mut state: State = read_state(deps.storage)?;
    update_open_interest_notional(
        &deps.as_ref(),
        &mut state,
        swap.vamm.clone(),
        Integer::new_negative(input),
        swap.trader.clone(),
    )?;

    // depending on the direction the output is positive or negative
    let signed_output = match &swap.side {
        Side::Buy => Integer::new_positive(output),
        Side::Sell => Integer::new_negative(output),
    };

    // realized_pnl = unrealized_pnl * close_ratio
    let realized_pnl = if !position.size.is_zero() {
        swap.unrealized_pnl.checked_mul(signed_output.abs())? / position.size.abs()
    } else {
        Integer::zero()
    };

    let RemainMarginResponse {
        funding_payment,
        margin,
        bad_debt,
        latest_premium_fraction,
    } = calc_remain_margin_with_funding_payment(deps.as_ref(), &position, realized_pnl)?;

    let unrealized_pnl_after = swap.unrealized_pnl - realized_pnl;

    let remaining_notional = if position.size > Integer::zero() {
        Integer::new_positive(swap.position_notional)
            - Integer::new_positive(swap.open_notional)
            - unrealized_pnl_after
    } else {
        unrealized_pnl_after + Integer::new_positive(swap.position_notional)
            - Integer::new_positive(swap.open_notional)
    };

    // calculate the fees
    let fees_messages = transfer_fees(
        deps.as_ref(),
        swap.trader,
        swap.spread_fee,
        swap.toll_fee,
        false,
    )?;

    // set the new position
    position.size += signed_output;
    position.margin = margin;
    position.notional = remaining_notional.value;
    position.last_updated_premium_fraction = latest_premium_fraction;
    position.block_time = env.block.time.seconds();

    store_position(deps.storage, &vamm_key, &position, false)?;
    store_state(deps.storage, &state)?;

    // to prevent attacker to leverage the bad debt to withdraw extra token from insurance fund
    if !bad_debt.is_zero() {
        return Err(StdError::generic_err("Cannot close position - bad debt"));
    }

    // remove the tmp position
    remove_tmp_swap(deps.storage, &position_id.to_be_bytes());

    Ok(Response::new()
        .add_submessages(fees_messages)
        .add_attributes(vec![
            ("action", "partial_close_position_reply"),
            ("take_profit", &position.take_profit.to_string()),
            (
                "stop_loss",
                &position.stop_loss.unwrap_or_default().to_string(),
            ),
            ("pnl", &unrealized_pnl_after.to_string()),
            ("spread_fee", &swap.spread_fee.to_string()),
            ("toll_fee", &swap.toll_fee.to_string()),
            ("funding_payment", &funding_payment.to_string()),
            ("bad_debt", &bad_debt.to_string()),
        ]))
}

// Liquidates position after successful execution of the swap
pub fn liquidate_reply(
    deps: DepsMut,
    env: Env,
    _input: Uint128,
    output: Uint128,
    position_id: u64,
) -> StdResult<Response> {
    let swap = read_tmp_swap(deps.storage, &position_id.to_be_bytes())?;
    let liquidator = read_tmp_liquidator(deps.storage)?;

    let vamm_key = keccak_256(&[swap.vamm.as_bytes()].concat());
    let position = read_position(deps.storage, &vamm_key, position_id)?;

    // calculate delta from trade and whether it was profitable or a loss
    let margin_delta = match &position.direction {
        Direction::RemoveFromAmm => {
            Integer::new_positive(swap.open_notional) - Integer::new_positive(output)
        }
        Direction::AddToAmm => {
            Integer::new_positive(output) - Integer::new_positive(swap.open_notional)
        }
    };

    let mut remain_margin =
        calc_remain_margin_with_funding_payment(deps.as_ref(), &position, margin_delta)?;

    let config = read_config(deps.storage)?;

    // calculate liquidation penalty and fee for liquidator
    let liquidation_penalty = output
        .checked_mul(config.liquidation_fee)?
        .checked_div(config.decimals)?;

    let liquidation_fee = liquidation_penalty.checked_div(Uint128::from(2u64))?;

    if liquidation_fee > remain_margin.margin {
        let bad_debt = liquidation_fee.checked_sub(remain_margin.margin)?;
        remain_margin.bad_debt = remain_margin.bad_debt.checked_add(bad_debt)?;

        // any margin is going to be taken as part of liquidation fee
        remain_margin.margin = Uint128::zero();
    } else {
        remain_margin.margin = remain_margin.margin.checked_sub(liquidation_fee)?;
    }

    let mut msgs: Vec<SubMsg> = vec![];

    let mut state = read_state(deps.storage)?;
    let pre_paid_shortfall = if !remain_margin.bad_debt.is_zero() {
        realize_bad_debt(deps.as_ref(), remain_margin.bad_debt, &mut msgs, &mut state)?
    } else {
        Uint128::zero()
    };

    // any remaining margin goes to the insurance contract
    if !remain_margin.margin.is_zero() {
        let msg = match config.insurance_fund {
            Some(insurance_fund) => {
                execute_transfer(deps.storage, &insurance_fund, remain_margin.margin)?
            }
            None => return Err(StdError::generic_err("insurance fund is not registered")),
        };

        msgs.push(msg);
    }

    msgs.append(&mut withdraw(
        deps.as_ref(),
        env.clone(),
        &mut state,
        &liquidator,
        config.eligible_collateral,
        liquidation_fee,
        Uint128::zero(),
        pre_paid_shortfall,
    )?);

    store_state(deps.storage, &state)?;

    remove_position(deps.storage, &vamm_key, &position)?;

    remove_tmp_swap(deps.storage, &position_id.to_be_bytes());
    remove_tmp_liquidator(deps.storage);

    enter_restriction_mode(deps.storage, swap.vamm, env.block.height)?;

    Ok(Response::new().add_submessages(msgs).add_attributes(vec![
        ("action", "liquidation_reply"),
        ("take_profit", &position.take_profit.to_string()),
        (
            "stop_loss",
            &position.stop_loss.unwrap_or_default().to_string(),
        ),
        ("liquidation_fee", &liquidation_fee.to_string()),
        ("pnl", &margin_delta.to_string()),
        (
            "funding_payment",
            &remain_margin.funding_payment.to_string(),
        ),
        ("bad_debt", &remain_margin.bad_debt.to_string()),
    ]))
}

// Partially liquidates the position
pub fn partial_liquidation_reply(
    deps: DepsMut,
    env: Env,
    input: Uint128,
    output: Uint128,
    position_id: u64,
) -> StdResult<Response> {
    let swap = read_tmp_swap(deps.storage, &position_id.to_be_bytes())?;
    let liquidator = read_tmp_liquidator(deps.storage)?;

    let vamm_key = keccak_256(&[swap.vamm.as_bytes()].concat());
    let mut position = read_position(deps.storage, &vamm_key, position_id)?;
    let config = read_config(deps.storage)?;

    // calculate delta from trade and whether it was profitable or a loss
    let realized_pnl = (swap.unrealized_pnl
        * Integer::new_positive(config.partial_liquidation_ratio))
        / Integer::new_positive(config.decimals);

    let liquidation_penalty = output
        .checked_mul(config.liquidation_fee)?
        .checked_div(config.decimals)?;

    let liquidation_fee = liquidation_penalty.checked_div(Uint128::from(2u64))?;

    if position.size < Integer::zero() {
        position.size += Integer::new_positive(input);
    } else {
        position.size += Integer::new_negative(input);
    }

    // reduce the traders margin
    position.margin = position
        .margin
        .checked_sub(realized_pnl.value)?
        .checked_sub(liquidation_penalty)?;

    // calculate openNotional (it's different depends on long or short side)
    // long: unrealizedPnl = positionNotional - openNotional => openNotional = positionNotional - unrealizedPnl
    // short: unrealizedPnl = openNotional - positionNotional => openNotional = positionNotional + unrealizedPnl
    // positionNotional = oldPositionNotional - exchangedQuoteAssetAmount
    position.notional = match position.size {
        Integer {
            negative: false, ..
        } => position
            .notional
            .checked_sub(swap.open_notional)?
            .checked_sub(realized_pnl.value)?,
        Integer { negative: true, .. } => realized_pnl
            .value
            .checked_add(position.notional)?
            .checked_sub(swap.open_notional)?,
    };

    let mut messages: Vec<SubMsg> = vec![];
    let mut state = read_state(deps.storage)?;
    if !liquidation_fee.is_zero() {
        let msg = match config.insurance_fund {
            Some(insurance_fund) => {
                execute_transfer(deps.storage, &insurance_fund, liquidation_fee)?
            }
            None => return Err(StdError::generic_err("insurance fund is not registered")),
        };

        messages.push(msg);

        // calculate token balance that should be remaining once
        // insurance fees have been paid
        messages.append(&mut withdraw(
            deps.as_ref(),
            env.clone(),
            &mut state,
            &liquidator,
            config.eligible_collateral,
            liquidation_fee,
            Uint128::zero(),
            Uint128::zero(),
        )?);
    }

    store_position(deps.storage, &vamm_key, &position, false)?;
    store_state(deps.storage, &state)?;

    remove_tmp_swap(deps.storage, &position_id.to_be_bytes());
    remove_tmp_liquidator(deps.storage);

    enter_restriction_mode(deps.storage, swap.vamm, env.block.height)?;

    Ok(Response::new()
        .add_submessages(messages)
        .add_attributes(vec![
            ("action", "partial_liquidation_reply"),
            ("liquidation_fee", &liquidation_fee.to_string()),
            ("pnl", &realized_pnl.to_string()),
        ]))
}

/// pays funding, if funding rate is positive, traders with long position
/// pay traders with short position and vice versa.
pub fn pay_funding_reply(
    deps: DepsMut,
    env: Env,
    premium_fraction: Integer,
    sender: &str,
) -> StdResult<Response> {
    let vamm = deps.api.addr_validate(sender)?;

    // update the cumulative premium fraction
    append_cumulative_premium_fraction(deps.storage, vamm.clone(), premium_fraction)?;

    let vamm_controller = VammController(vamm);
    let total_position_size = vamm_controller.state(&deps.querier)?.total_position_size;

    let config = read_config(deps.storage)?;
    let funding_payment =
        total_position_size * premium_fraction / Integer::new_positive(config.decimals);

    let mut response: Response = Response::new();

    if funding_payment.is_negative() && !funding_payment.is_zero() {
        let msg = execute_insurance_fund_withdrawal(deps.as_ref(), funding_payment.value)?;
        response = response.add_submessage(msg);
    } else if funding_payment.is_positive() && !funding_payment.is_zero() {
        let msg = execute_transfer_to_insurance_fund(deps.as_ref(), env, funding_payment.value)?;
        response = response.add_submessage(msg);
    };

    Ok(response.add_attributes(vec![
        ("action", "pay_funding_reply"),
        ("funding_payment", &funding_payment.to_string()),
    ]))
}
