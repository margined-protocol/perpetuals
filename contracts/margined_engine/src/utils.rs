use cosmwasm_std::{
    Addr, Response, StdError, StdResult, Storage,
};

use margined_perp::margined_vamm::Direction;
use margined_perp::margined_engine::Side;
use crate::{
    state::{
        VammList, read_vamm,
    },
};

pub fn require_vamm(storage: &dyn Storage, vamm: &Addr) -> StdResult<Response> {
    // check that it is a registered vamm
    let vamm_list: VammList = read_vamm(storage)?;
    if !vamm_list.is_vamm(&vamm.to_string()) {
        return Err(StdError::generic_err("vAMM is not registered"));
    }

    Ok(Response::new())

}

// takes the side (buy|sell) and returns the direction (long|short)
pub fn side_to_direction(
    side: Side,
) -> Direction {
    let direction: Direction = match side {
            Side::BUY => Direction::AddToAmm,
            Side::SELL => Direction::RemoveFromAmm,
    };

    return direction
}

// takes the direction (long|short) and returns the side (buy|sell)
pub fn direction_to_side(
    direction: Direction,
) -> Side {
    let side: Side = match direction {
            Direction::AddToAmm => Side::BUY,
            Direction::RemoveFromAmm => Side::SELL,
    };

    return side
}

// takes the side (buy|sell) and returns opposite (short|long)
// this is useful when closing/reversing a position
pub fn switch_direction(
    dir: Direction,
) -> Direction {
    return match dir {
            Direction::RemoveFromAmm => Direction::AddToAmm,
            Direction::AddToAmm => Direction::RemoveFromAmm,
    };
}