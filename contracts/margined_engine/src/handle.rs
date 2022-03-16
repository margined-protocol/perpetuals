use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, Deps, DepsMut, Env, MessageInfo, ReplyOn, Response, StdError,
    StdResult, SubMsg, Uint128, WasmMsg,
};

use crate::{
    contract::{
        SWAP_CLOSE_REPLY_ID, SWAP_DECREASE_REPLY_ID, SWAP_INCREASE_REPLY_ID,
        SWAP_LIQUIDATE_REPLY_ID, SWAP_REVERSE_REPLY_ID,
    },
    querier::{query_vamm_output_price, query_vamm_output_twap},
    query::query_margin_ratio,
    state::{
        read_config, read_position, store_config, store_tmp_liquidator, store_tmp_swap, Config,
        Position, Swap,
    },
    utils::{
        direction_to_side, get_position, require_insufficient_margin, require_margin, require_vamm,
        side_to_direction,
    },
};
use margined_perp::margined_engine::{
    Pnl, PnlCalcOption, PnlResponse, PositionUnrealizedPnlResponse, RemainMarginResponse, Side,
};
use margined_perp::margined_vamm::{Direction, ExecuteMsg};

pub fn update_config(deps: DepsMut, info: MessageInfo, owner: String) -> StdResult<Response> {
    let config = read_config(deps.storage)?;
    if info.sender != config.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    let new_config = Config {
        owner: deps.api.addr_validate(&owner).unwrap(),
        ..config
    };

    store_config(deps.storage, &new_config)?;

    Ok(Response::default())
}

// Opens a position
// TODO - refactor arguments into a struct
#[allow(clippy::too_many_arguments)]
pub fn open_position(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    vamm: String,
    trader: String,
    side: Side,
    quote_asset_amount: Uint128,
    leverage: Uint128,
) -> StdResult<Response> {
    let config: Config = read_config(deps.storage)?;

    let vamm = deps.api.addr_validate(&vamm)?;
    let trader = deps.api.addr_validate(&trader)?;

    let margin_ratio = Uint128::from(1_000_000_000u64)
        .checked_mul(config.decimals)?
        .checked_div(leverage)?;

    require_vamm(deps.storage, &vamm)?;
    require_margin(config.initial_margin_ratio, margin_ratio)?;

    // calc the input amount wrt to leverage and decimals
    let open_notional = quote_asset_amount
        .checked_mul(leverage)?
        .checked_div(config.decimals)?;

    let position: Position = get_position(env.clone(), deps.storage, &vamm, &trader, side.clone());

    let mut is_increase: bool = true;
    if !(position.direction == Direction::AddToAmm && side == Side::BUY
        || position.direction == Direction::RemoveFromAmm && side == Side::SELL)
    {
        is_increase = false;
    }

    let msg: SubMsg = if is_increase {
        internal_increase_position(vamm.clone(), side.clone(), open_notional).unwrap()
    } else {
        open_reverse_position(
            &deps,
            env,
            vamm.clone(),
            trader.clone(),
            side.clone(),
            open_notional,
        )
    };

    store_tmp_swap(
        deps.storage,
        &Swap {
            vamm,
            trader,
            side,
            quote_asset_amount,
            leverage,
            open_notional,
        },
    )?;

    Ok(Response::new()
        .add_submessage(msg)
        .add_attributes(vec![("action", "open_position")]))
}

pub fn close_position(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    vamm: String,
    trader: String,
) -> StdResult<Response> {
    // validate address inputs
    let vamm = deps.api.addr_validate(&vamm)?;
    let trader = deps.api.addr_validate(&trader)?;

    // read the position for the trader from vamm
    let position = read_position(deps.storage, &vamm, &trader).unwrap();

    let msg = internal_close_position(deps, &position, SWAP_CLOSE_REPLY_ID)?;

    Ok(Response::new()
        .add_attributes(vec![("action", "close_position")])
        .add_submessage(msg))
}

pub fn liquidate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    vamm: String,
    trader: String,
) -> StdResult<Response> {
    let config: Config = read_config(deps.storage)?;

    // validate address inputs
    let vamm = deps.api.addr_validate(&vamm)?;
    let trader = deps.api.addr_validate(&trader)?;

    // store the liquidator
    store_tmp_liquidator(deps.storage, &info.sender)?;

    // check if margin ratio has been
    let margin = query_margin_ratio(deps.as_ref(), vamm.to_string(), trader.to_string())?;

    require_vamm(deps.storage, &vamm)?;
    require_insufficient_margin(
        config.maintenance_margin_ratio,
        margin.ratio,
        margin.polarity,
    )?;

    // read the position for the trader from vamm
    let position = read_position(deps.storage, &vamm, &trader).unwrap();

    // TODO First we should see if it is a partial or full liqudiation, but not today
    let msg: SubMsg;
    let mut response = Response::default();
    if false {
        // NOTHING in future this condition will be there to see if the liquidation is partial
    } else {
        msg = internal_close_position(deps, &position, SWAP_LIQUIDATE_REPLY_ID)?;
        response = response.add_submessage(msg);
    }

    Ok(response.add_attributes(vec![("action", "liquidate")]))
}

// Increase the position, just basically wraps swap input though it may do more in the future
pub fn internal_increase_position(
    vamm: Addr,
    side: Side,
    open_notional: Uint128,
) -> StdResult<SubMsg> {
    swap_input(&vamm, side, open_notional, SWAP_INCREASE_REPLY_ID)
}
pub fn internal_close_position(deps: DepsMut, position: &Position, id: u64) -> StdResult<SubMsg> {
    let swap_msg = WasmMsg::Execute {
        contract_addr: position.vamm.to_string(),
        funds: vec![],
        msg: to_binary(&ExecuteMsg::SwapOutput {
            direction: position.direction.clone(),
            base_asset_amount: position.size,
        })?,
    };

    store_tmp_swap(
        deps.storage,
        &Swap {
            vamm: position.vamm.clone(),
            trader: position.trader.clone(),
            side: direction_to_side(position.direction.clone()),
            quote_asset_amount: position.size,
            leverage: Uint128::zero(),
            open_notional: position.notional,
        },
    )?;

    Ok(SubMsg {
        msg: CosmosMsg::Wasm(swap_msg),
        gas_limit: None, // probably should set a limit in the config
        id,
        reply_on: ReplyOn::Always,
    })
}
// Increase the position, just basically wraps swap input though it may do more in the future
fn open_reverse_position(
    deps: &DepsMut,
    env: Env,
    vamm: Addr,
    trader: Addr,
    side: Side,
    open_notional: Uint128,
) -> SubMsg {
    let position: Position = get_position(env, deps.storage, &vamm, &trader, side.clone());
    let current_notional = query_vamm_output_price(
        &deps.as_ref(),
        vamm.to_string(),
        position.direction.clone(),
        position.size,
    )
    .unwrap();

    // if position.notional > open_notional {
    let msg: SubMsg = if current_notional > open_notional {
        // then we are opening a new position or adding to an existing
        swap_input(&vamm, side, open_notional, SWAP_DECREASE_REPLY_ID).unwrap()
    } else {
        // first close position swap out the entire position
        swap_output(
            &vamm,
            direction_to_side(position.direction.clone()),
            position.size,
            SWAP_REVERSE_REPLY_ID,
        )
        .unwrap()
    };

    msg
}

fn swap_input(vamm: &Addr, side: Side, open_notional: Uint128, id: u64) -> StdResult<SubMsg> {
    let direction: Direction = side_to_direction(side);

    let msg = WasmMsg::Execute {
        contract_addr: vamm.to_string(),
        funds: vec![],
        msg: to_binary(&ExecuteMsg::SwapInput {
            direction,
            quote_asset_amount: open_notional,
        })?,
    };

    let execute_submsg = SubMsg {
        msg: CosmosMsg::Wasm(msg),
        gas_limit: None, // probably should set a limit in the config
        id,
        reply_on: ReplyOn::Always,
    };

    Ok(execute_submsg)
}

fn swap_output(vamm: &Addr, side: Side, open_notional: Uint128, id: u64) -> StdResult<SubMsg> {
    let direction: Direction = side_to_direction(side);

    let swap_msg = WasmMsg::Execute {
        contract_addr: vamm.to_string(),
        funds: vec![],
        msg: to_binary(&ExecuteMsg::SwapOutput {
            direction,
            base_asset_amount: open_notional,
        })?,
    };

    let execute_submsg = SubMsg {
        msg: CosmosMsg::Wasm(swap_msg),
        gas_limit: None, // probably should set a limit in the config
        id,
        reply_on: ReplyOn::Always,
    };

    Ok(execute_submsg)
}

pub fn get_position_notional_unrealized_pnl(
    deps: Deps,
    position: &Position,
    calc_option: PnlCalcOption,
) -> StdResult<PositionUnrealizedPnlResponse> {
    let mut position_notional = Uint128::zero();
    let mut unrealized_pnl = Uint128::zero();

    let position_size = position.size;
    if !position_size.is_zero() {
        match calc_option {
            PnlCalcOption::TWAP => {
                position_notional = query_vamm_output_twap(
                    &deps,
                    position.vamm.to_string(),
                    position.direction.clone(),
                    position_size,
                )?;
            }
            PnlCalcOption::SPOTPRICE => {
                position_notional = query_vamm_output_price(
                    &deps,
                    position.vamm.to_string(),
                    position.direction.clone(),
                    position_size,
                )?;
            }
            PnlCalcOption::ORACLE => {}
        }
        if position.notional > position_notional {
            unrealized_pnl = position.notional.checked_sub(position_notional)?;
        } else {
            unrealized_pnl = position_notional.checked_sub(position.notional)?;
        }
    }

    Ok(PositionUnrealizedPnlResponse {
        position_notional,
        unrealized_pnl,
    })
}

pub fn calc_pnl(
    output: Uint128,
    previous_notional: Uint128,
    direction: Direction,
) -> StdResult<PnlResponse> {
    // calculate delta from the trade
    let profit_loss: Pnl;
    let value: Uint128 = if output > previous_notional {
        if direction == Direction::AddToAmm {
            profit_loss = Pnl::Profit;
        } else {
            profit_loss = Pnl::Loss;
        }
        output.checked_sub(previous_notional)?
    } else {
        if direction == Direction::AddToAmm {
            profit_loss = Pnl::Loss;
        } else {
            profit_loss = Pnl::Profit;
        }
        previous_notional.checked_sub(output)?
    };

    Ok(PnlResponse { value, profit_loss })
}

pub fn calc_remain_margin_with_funding_payment(
    position: &Position,
    margin_delta: Uint128,
    pnl: Pnl,
) -> StdResult<RemainMarginResponse> {
    // calculate the funding payment

    // calculate the remaining margin
    let mut bad_debt = Uint128::zero();
    let remaining_margin: Uint128 = if pnl == Pnl::Profit {
        position.margin.checked_add(margin_delta)?
    } else if margin_delta < position.margin {
        position.margin.checked_sub(margin_delta)?
    } else {
        // if the delta is bigger than margin we
        // will have some bad debt and margin out is gonna
        // be zero
        bad_debt = margin_delta.checked_sub(position.margin)?;
        Uint128::zero()
    };

    // if the remain is negative, set it to zero
    // and set the rest to
    Ok(RemainMarginResponse {
        funding_payment: Uint128::zero(),
        margin: remaining_margin,
        bad_debt,
    })
}

// this resets the main variables of a position
pub fn clear_position(env: Env, mut position: Position) -> StdResult<Position> {
    position.size = Uint128::zero();
    position.margin = Uint128::zero();
    position.notional = Uint128::zero();
    position.timestamp = env.block.time;

    Ok(position)
}
