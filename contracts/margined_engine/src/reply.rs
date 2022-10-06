use cosmwasm_std::{DepsMut, Env, Response, StdError, StdResult, SubMsg, Uint128};

use crate::{
    contract::INCREASE_POSITION_REPLY_ID,
    handle::internal_increase_position,
    messages::{
        execute_insurance_fund_withdrawal, execute_transfer, execute_transfer_from,
        execute_transfer_to_insurance_fund, transfer_fees, withdraw,
    },
    querier::query_vamm_state,
    query::query_margin_ratio,
    state::{
        append_cumulative_premium_fraction, enter_restriction_mode, read_config, read_sent_funds,
        read_state, read_tmp_liquidator, read_tmp_swap, remove_position, remove_sent_funds,
        remove_tmp_liquidator, remove_tmp_swap, store_position, store_sent_funds, store_state,
        store_tmp_swap, Config, State, TmpSwapInfo,
    },
    utils::{
        calc_remain_margin_with_funding_payment, check_base_asset_holding_cap, clear_position,
        get_position, realize_bad_debt, require_additional_margin, side_to_direction,
        update_open_interest_notional,
    },
};

use margined_common::{asset::AssetInfo, integer::Integer};
use margined_perp::{
    margined_engine::{Position, RemainMarginResponse, Side},
    margined_vamm::Direction,
};

// Updates position after successful execution of the swap
pub fn update_position_reply(
    deps: DepsMut,
    env: Env,
    input: Uint128,
    output: Uint128,
    reply_id: u64,
) -> StdResult<Response> {
    let config: Config = read_config(deps.storage)?;
    let mut state: State = read_state(deps.storage)?;

    let mut swap = read_tmp_swap(deps.storage)?;
    let mut funds = read_sent_funds(deps.storage)?;

    let mut position = get_position(
        env.clone(),
        deps.storage,
        &swap.vamm,
        &swap.trader,
        swap.side.clone(),
    );

    // depending on the direction the output is positive or negative
    let signed_output: Integer = match &swap.side {
        Side::Buy => Integer::new_positive(output),
        Side::Sell => Integer::new_negative(output),
    };

    let amount = if reply_id == INCREASE_POSITION_REPLY_ID {
        Integer::new_positive(input)
    } else {
        Integer::new_negative(input)
    };

    update_open_interest_notional(&deps.as_ref(), &mut state, swap.vamm.clone(), amount)?;

    // define variables that differ across increase and decrease scenario
    let swap_margin: Uint128;
    let margin_delta: Integer;
    let new_direction: Direction;
    let new_notional: Uint128;

    // calculate margin needed given swap
    match reply_id {
        INCREASE_POSITION_REPLY_ID => {
            swap_margin = swap
                .open_notional
                .checked_mul(config.decimals)?
                .checked_div(swap.leverage)?;

            swap.margin_to_vault = swap
                .margin_to_vault
                .checked_add(Integer::new_positive(swap_margin))?;

            margin_delta = Integer::new_positive(swap_margin);
            new_direction = side_to_direction(swap.side);
            new_notional = position.notional.checked_add(swap.open_notional)?;
        }
        // DECREASE_POSITION_REPLY
        _ => {
            swap_margin = Uint128::zero();

            // realized_pnl = unrealized_pnl * close_ratio
            let realized_pnl = if !position.size.is_zero() {
                swap.unrealized_pnl.checked_mul(signed_output.abs())? / position.size.abs()
            } else {
                Integer::zero()
            };

            let unrealized_pnl_after = swap.unrealized_pnl - realized_pnl;

            let remaining_notional = if position.size > Integer::zero() {
                Integer::new_positive(swap.position_notional)
                    - Integer::new_positive(swap.open_notional)
                    - unrealized_pnl_after
            } else {
                unrealized_pnl_after + Integer::new_positive(swap.position_notional)
                    - Integer::new_positive(swap.open_notional)
            };

            margin_delta = realized_pnl;
            new_direction = position.direction.clone();
            new_notional = remaining_notional.value;
        }
    }

    // calculate the remaining margin
    let RemainMarginResponse {
        funding_payment: _,
        margin,
        bad_debt: _,
        latest_premium_fraction,
    } = calc_remain_margin_with_funding_payment(deps.as_ref(), position.clone(), margin_delta)?;

    // set the new position
    position.direction = new_direction;
    position.notional = new_notional;
    position.size += signed_output;
    position.margin = margin;
    position.last_updated_premium_fraction = latest_premium_fraction;
    position.block_number = env.block.height;

    store_position(deps.storage, &position)?;

    // check the new position doesn't exceed any caps
    check_base_asset_holding_cap(&deps.as_ref(), swap.vamm.clone(), position.size.value)?;

    let mut msgs: Vec<SubMsg> = vec![];

    // create transfer messages depending on PnL
    #[allow(clippy::comparison_chain)]
    if swap.margin_to_vault < Integer::zero() {
        msgs.append(
            &mut withdraw(
                deps.as_ref(),
                env,
                &mut state,
                &swap.trader,
                config.eligible_collateral.clone(),
                swap.margin_to_vault.value,
                Uint128::zero(),
            )
            .unwrap(),
        );
    } else if swap.margin_to_vault > Integer::zero() {
        match config.eligible_collateral {
            AssetInfo::NativeToken { .. } => {
                funds.required = funds.required.checked_add(swap_margin)?;
            }
            AssetInfo::Token { .. } => {
                msgs.push(
                    execute_transfer_from(
                        deps.storage,
                        &swap.trader,
                        &env.contract.address,
                        swap.margin_to_vault.value,
                    )
                    .unwrap(),
                );
            }
        }
    };

    // create array for fee amounts
    let mut fees_amount: [Uint128; 2] = [Uint128::zero(), Uint128::zero()];

    // create messages to pay for toll and spread fees, check flag is true if this follows a reverse
    if !swap.fees_paid {
        let mut fees =
            transfer_fees(deps.as_ref(), swap.trader, swap.vamm, swap.open_notional).unwrap();

        // add the fee transfer messages
        msgs.append(&mut fees.messages);

        // add the total fees to the required funds counter
        funds.required = funds
            .required
            .checked_add(fees.spread_fee)?
            .checked_add(fees.toll_fee)?;

        fees_amount[0] = fees.spread_fee;
        fees_amount[1] = fees.toll_fee;
    };

    // check if native tokens are sufficient
    if let AssetInfo::NativeToken { .. } = config.eligible_collateral {
        funds.are_sufficient()?;
    }

    // check that the maintenance margin is correct
    let margin_ratio = query_margin_ratio(
        deps.as_ref(),
        position.vamm.to_string(),
        position.trader.to_string(),
    )?;

    require_additional_margin(margin_ratio, config.maintenance_margin_ratio)?;

    store_state(deps.storage, &state)?;

    remove_tmp_swap(deps.storage);
    remove_sent_funds(deps.storage);

    Ok(Response::new().add_submessages(msgs).add_attributes(vec![
        ("action", "update_position_reply"),
        ("spread_fee", &fees_amount[0].to_string()),
        ("toll_fee", &fees_amount[1].to_string()),
    ]))
}

// reverse position after successful execution of the swap
pub fn reverse_position_reply(
    deps: DepsMut,
    env: Env,
    _input: Uint128,
    output: Uint128,
) -> StdResult<Response> {
    let config = read_config(deps.storage)?;
    let mut state = read_state(deps.storage)?;
    let mut swap = read_tmp_swap(deps.storage)?;
    let mut funds = read_sent_funds(deps.storage)?;

    let mut position = get_position(
        env.clone(),
        deps.storage,
        &swap.vamm,
        &swap.trader,
        swap.side.clone(),
    );

    update_open_interest_notional(
        &deps.as_ref(),
        &mut state,
        swap.vamm.clone(),
        Integer::new_negative(output),
    )?;

    let previous_margin = Integer::new_negative(position.margin);

    // reset the position in order to reverse
    position = clear_position(env, position)?;

    // now increase the position again if there is additional position
    let current_open_notional = swap.open_notional;
    swap.open_notional = if swap.open_notional > output {
        swap.open_notional.checked_sub(output)?
    } else {
        output.checked_sub(swap.open_notional)?
    };

    // create messages to pay for toll and spread fees
    let fees = transfer_fees(
        deps.as_ref(),
        swap.trader.clone(),
        swap.vamm.clone(),
        current_open_notional,
    )
    .unwrap();

    // add the fee transfer messages
    let mut msgs: Vec<SubMsg> = fees.messages;

    // add the total fees (spread + toll) to the required funds counter
    funds.required = funds
        .required
        .checked_add(fees.spread_fee)?
        .checked_add(fees.toll_fee)?;

    // reduce position if old position is larger
    if swap.open_notional.checked_div(swap.leverage)? == Uint128::zero() {
        // latest margin requirements
        let margin = previous_margin.checked_sub(swap.unrealized_pnl)?;

        // create transfer message
        msgs.push(execute_transfer(deps.storage, &swap.trader, margin.value).unwrap());

        // check if native tokens are sufficient
        if let AssetInfo::NativeToken { .. } = config.eligible_collateral {
            funds.are_sufficient()?;
        }

        remove_sent_funds(deps.storage);
        remove_tmp_swap(deps.storage);
    } else {
        // determine new position
        swap.margin_to_vault = previous_margin.checked_sub(swap.unrealized_pnl)?;
        swap.unrealized_pnl = Integer::zero();

        // set fees_paid flag to true so they aren't paid twice
        swap.fees_paid = true;

        // update the funds required
        funds.required = if swap.margin_to_vault.is_positive() {
            funds.required.checked_add(swap.margin_to_vault.value)?
        } else if funds.required > swap.margin_to_vault.value {
            funds.required.checked_sub(swap.margin_to_vault.value)?
        } else {
            // add both fees
            fees.spread_fee.checked_add(fees.toll_fee)?
        };

        msgs.push(internal_increase_position(
            swap.vamm.clone(),
            swap.side.clone(),
            swap.open_notional,
            Uint128::zero(),
        )?);

        store_tmp_swap(deps.storage, &swap)?;
        store_sent_funds(deps.storage, &funds)?;
    }

    store_position(deps.storage, &position)?;
    store_state(deps.storage, &state)?;

    Ok(Response::new().add_submessages(msgs).add_attributes(vec![
        ("action", "reverse_position_reply"),
        ("spread_fee", "increase_position_reply"),
        ("toll_fee", "increase_position_reply"),
    ]))
}

// Closes position after successful execution of the swap
pub fn close_position_reply(
    deps: DepsMut,
    env: Env,
    _input: Uint128,
    output: Uint128,
) -> StdResult<Response> {
    let config = read_config(deps.storage)?;
    let mut state = read_state(deps.storage)?;
    let swap = read_tmp_swap(deps.storage)?;

    let position = get_position(
        env.clone(),
        deps.storage,
        &swap.vamm.clone(),
        &swap.trader,
        swap.side.clone(),
    );

    let margin_delta: Integer = match &position.direction {
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
    } = calc_remain_margin_with_funding_payment(deps.as_ref(), position.clone(), margin_delta)?;

    let withdraw_amount = Integer::new_positive(margin).checked_add(swap.unrealized_pnl)?;

    let mut msgs: Vec<SubMsg> = vec![];

    // to prevent attacker to leverage the bad debt to withdraw extra token from insurance fund
    if !bad_debt.is_zero() {
        return Err(StdError::generic_err("Cannot close position - bad debt"));
    }

    if !withdraw_amount.is_zero() {
        msgs.append(
            &mut withdraw(
                deps.as_ref(),
                env,
                &mut state,
                &swap.trader,
                config.eligible_collateral,
                withdraw_amount.value,
                Uint128::zero(),
            )
            .unwrap(),
        );
    }

    // create array for fee amounts
    let mut fees_amount: [Uint128; 2] = [Uint128::zero(), Uint128::zero()];

    if !position.notional.is_zero() {
        let mut fees = transfer_fees(
            deps.as_ref(),
            swap.trader,
            swap.vamm.clone(),
            position.notional,
        )
        .unwrap();

        fees_amount[0] = fees.spread_fee;
        fees_amount[1] = fees.toll_fee;

        msgs.append(&mut fees.messages);
    }

    let value =
        margin_delta + Integer::new_positive(bad_debt) + Integer::new_positive(position.notional);

    update_open_interest_notional(&deps.as_ref(), &mut state, swap.vamm, value.invert_sign())?;

    remove_position(deps.storage, &position);

    store_state(deps.storage, &state)?;

    remove_tmp_swap(deps.storage);

    Ok(Response::new().add_submessages(msgs).add_attributes(vec![
        ("action", "close_position_reply"),
        ("spread_fee", &fees_amount[0].to_string()),
        ("toll_fee", &fees_amount[1].to_string()),
        ("funding_payment", &funding_payment.to_string()),
        ("bad_debt", &bad_debt.to_string()),
    ]))
}

// Partially closes position
pub fn partial_close_position_reply(
    deps: DepsMut,
    env: Env,
    input: Uint128,
    output: Uint128,
) -> StdResult<Response> {
    let mut state: State = read_state(deps.storage)?;

    let swap: TmpSwapInfo = read_tmp_swap(deps.storage)?;

    let mut position: Position = get_position(
        env.clone(),
        deps.storage,
        &swap.vamm,
        &swap.trader,
        swap.side.clone(),
    );

    update_open_interest_notional(
        &deps.as_ref(),
        &mut state,
        swap.vamm.clone(),
        Integer::new_negative(input),
    )?;

    // depending on the direction the output is positive or negative
    let signed_output: Integer = match &swap.side {
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
    } = calc_remain_margin_with_funding_payment(deps.as_ref(), position.clone(), realized_pnl)?;

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
    let fees = transfer_fees(deps.as_ref(), swap.trader, swap.vamm, swap.open_notional).unwrap();

    // set the new position
    position.size += signed_output;
    position.margin = margin;
    position.notional = remaining_notional.value;
    position.last_updated_premium_fraction = latest_premium_fraction;
    position.block_number = env.block.height;

    store_position(deps.storage, &position)?;
    store_state(deps.storage, &state)?;

    // to prevent attacker to leverage the bad debt to withdraw extra token from insurance fund
    if !bad_debt.is_zero() {
        return Err(StdError::generic_err("Cannot close position - bad debt"));
    }

    // remove the tmp position
    remove_tmp_swap(deps.storage);

    Ok(Response::new()
        .add_submessages(fees.messages)
        .add_attributes(vec![
            ("action", "partial_close_position_reply"),
            ("spread_fee", &fees.spread_fee.to_string()),
            ("toll_fee", &fees.toll_fee.to_string()),
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
) -> StdResult<Response> {
    let config = read_config(deps.storage)?;
    let mut state = read_state(deps.storage)?;

    let swap = read_tmp_swap(deps.storage)?;
    let liquidator = read_tmp_liquidator(deps.storage)?;

    let position = get_position(
        env.clone(),
        deps.storage,
        &swap.vamm,
        &swap.trader,
        swap.side.clone(),
    );

    // calculate delta from trade and whether it was profitable or a loss
    let margin_delta: Integer = match &position.direction {
        Direction::RemoveFromAmm => {
            Integer::new_positive(swap.open_notional) - Integer::new_positive(output)
        }
        Direction::AddToAmm => {
            Integer::new_positive(output) - Integer::new_positive(swap.open_notional)
        }
    };

    let mut remain_margin =
        calc_remain_margin_with_funding_payment(deps.as_ref(), position.clone(), margin_delta)?;

    // calculate liquidation penalty and fee for liquidator
    let liquidation_penalty: Uint128 = output
        .checked_mul(config.liquidation_fee)?
        .checked_div(config.decimals)?;

    let liquidation_fee: Uint128 = liquidation_penalty.checked_div(Uint128::from(2u64))?;

    if liquidation_fee > remain_margin.margin {
        let bad_debt = liquidation_fee.checked_sub(remain_margin.margin)?;
        remain_margin.bad_debt = remain_margin.bad_debt.checked_add(bad_debt)?;

        // any margin is going to be taken as part of liquidation fee
        remain_margin.margin = Uint128::zero();
    } else {
        remain_margin.margin = remain_margin.margin.checked_sub(liquidation_fee)?;
    }

    let mut msgs: Vec<SubMsg> = vec![];

    let pre_paid_shortfall: Uint128 = if !remain_margin.bad_debt.is_zero() {
        realize_bad_debt(deps.as_ref(), remain_margin.bad_debt, &mut msgs, &mut state)
    } else {
        Uint128::zero()
    };

    // any remaining margin goes to the insurance contract
    if !remain_margin.margin.is_zero() {
        msgs.push(
            execute_transfer(deps.storage, &config.insurance_fund, remain_margin.margin).unwrap(),
        );
    }

    msgs.append(
        &mut withdraw(
            deps.as_ref(),
            env.clone(),
            &mut state,
            &liquidator,
            config.eligible_collateral,
            liquidation_fee,
            pre_paid_shortfall,
        )
        .unwrap(),
    );

    store_state(deps.storage, &state)?;

    remove_position(deps.storage, &position);
    remove_tmp_swap(deps.storage);
    remove_tmp_liquidator(deps.storage);

    enter_restriction_mode(deps.storage, swap.vamm, env.block.height)?;

    Ok(Response::new().add_submessages(msgs).add_attributes(vec![
        ("action", "liquidation_reply"),
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
) -> StdResult<Response> {
    let config = read_config(deps.storage)?;
    let mut state = read_state(deps.storage)?;

    let swap = read_tmp_swap(deps.storage)?;

    let liquidator = read_tmp_liquidator(deps.storage)?;

    let mut position = get_position(
        env.clone(),
        deps.storage,
        &swap.vamm,
        &swap.trader,
        swap.side.clone(),
    );

    // calculate delta from trade and whether it was profitable or a loss
    let realized_pnl = (swap.unrealized_pnl
        * Integer::new_positive(config.partial_liquidation_ratio))
        / Integer::new_positive(config.decimals);

    let liquidation_penalty: Uint128 = output
        .checked_mul(config.liquidation_fee)?
        .checked_div(config.decimals)?;

    let liquidation_fee: Uint128 = liquidation_penalty.checked_div(Uint128::from(2u64))?;

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

    if !liquidation_fee.is_zero() {
        messages
            .push(execute_transfer(deps.storage, &config.insurance_fund, liquidation_fee).unwrap());

        // calculate token balance that should be remaining once
        // insurance fees have been paid
        messages.append(
            &mut withdraw(
                deps.as_ref(),
                env.clone(),
                &mut state,
                &liquidator,
                config.eligible_collateral,
                liquidation_fee,
                Uint128::zero(),
            )
            .unwrap(),
        );
    }

    store_position(deps.storage, &position)?;
    store_state(deps.storage, &state)?;

    remove_tmp_swap(deps.storage);
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
    sender: String,
) -> StdResult<Response> {
    let config = read_config(deps.storage)?;
    let vamm = deps.api.addr_validate(&sender)?;

    // update the cumulative premium fraction
    append_cumulative_premium_fraction(deps.storage, vamm.clone(), premium_fraction)?;

    let total_position_size =
        query_vamm_state(&deps.as_ref(), vamm.to_string())?.total_position_size;

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
