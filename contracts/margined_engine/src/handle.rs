use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, Deps, DepsMut, Env, MessageInfo, ReplyOn, Response, StdError,
    StdResult, Storage, SubMsg, Uint128, WasmMsg,
};

use crate::{
    contract::{
        SWAP_CLOSE_REPLY_ID, SWAP_DECREASE_REPLY_ID, SWAP_INCREASE_REPLY_ID, SWAP_REVERSE_REPLY_ID,
    },
    querier::{query_vamm_output_price, query_vamm_output_twap},
    state::{read_config, read_position, store_config, store_tmp_swap, Config, Position, Swap},
    utils::{direction_to_side, require_margin, require_vamm, side_to_direction},
};
use margined_perp::margined_engine::{Pnl, PnlCalcOption, PositionUnrealizedPnlResponse, Side};
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
    // println!("{} {}", config.initial_margin_ratio, leverage);
    // println!("{} {}", config.initial_margin_ratio, config.decimals);
    let margin_ratio = Uint128::from(1_000_000_000u64)
        .checked_mul(config.decimals)?
        .checked_div(leverage)?;
    println!("{} {}", config.initial_margin_ratio, margin_ratio);
    // _047_619_047
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
    let position = read_position(deps.storage, &vamm, &trader)?.unwrap();

    let swap_msg = WasmMsg::Execute {
        contract_addr: vamm.to_string(),
        funds: vec![],
        msg: to_binary(&ExecuteMsg::SwapOutput {
            direction: position.direction.clone(),
            base_asset_amount: position.size,
        })?,
    };

    let msg = SubMsg {
        msg: CosmosMsg::Wasm(swap_msg),
        gas_limit: None, // probably should set a limit in the config
        id: SWAP_CLOSE_REPLY_ID,
        reply_on: ReplyOn::Always,
    };

    store_tmp_swap(
        deps.storage,
        &Swap {
            vamm,
            trader,
            side: direction_to_side(position.direction),
            quote_asset_amount: position.size,
            leverage: Uint128::zero(),
            open_notional: position.notional,
        },
    )?;

    Ok(Response::new()
        .add_attributes(vec![("action", "close_position")])
        .add_submessage(msg))
}

// Increase the position, just basically wraps swap input though it may do more in the future
pub fn internal_increase_position(
    vamm: Addr,
    side: Side,
    open_notional: Uint128,
) -> StdResult<SubMsg> {
    swap_input(&vamm, side, open_notional, SWAP_INCREASE_REPLY_ID)
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

// reads position from storage but also handles the case where there is no
// previously stored position, i.e. a new position
pub fn get_position(
    env: Env,
    storage: &dyn Storage,
    vamm: &Addr,
    trader: &Addr,
    side: Side,
) -> Position {
    // read the position for the trader from vamm
    let current_position = read_position(storage, vamm, trader).unwrap();
    let mut position = Position::default();

    // so if the position returned is None then its new
    if let Some(current_position) = current_position {
        position = current_position;
    } else {
        let direction: Direction = side_to_direction(side);

        // update the default position
        position.vamm = vamm.clone();
        position.trader = trader.clone();
        position.direction = direction;
        position.timestamp = env.block.time;
    }

    position
}

pub fn get_position_notional_unrealized_pnl(
    deps: Deps,
    position: &Position,
    calc_option: PnlCalcOption,
) -> StdResult<PositionUnrealizedPnlResponse> {
    let mut position_notional = Uint128::zero();
    let mut unrealized_pnl = Uint128::zero();
    let mut side = Pnl::ITM; // doesn't matter the direction if zero

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

        side = if position.notional > position_notional {
            unrealized_pnl = position.notional.checked_sub(position_notional)?;
            Pnl::OTM
        } else {
            unrealized_pnl = position_notional.checked_sub(position.notional)?;
            Pnl::ITM
        }
    }

    Ok(PositionUnrealizedPnlResponse {
        position_notional,
        unrealized_pnl,
        side,
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
