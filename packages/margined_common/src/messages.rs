use cosmwasm_schema::serde::Serialize;
use cosmwasm_std::{
    to_binary, Coin, CosmosMsg, Event, StdError, StdResult, SubMsgResponse, WasmMsg,
};

// util to create Wasm execute message as CosmosMsg
pub fn wasm_execute<T: Serialize + ?Sized>(
    contract_addr: impl Into<String>,
    msg: &T,
    funds: Vec<Coin>,
) -> StdResult<CosmosMsg> {
    let msg = to_binary(msg)?;
    Ok(WasmMsg::Execute {
        contract_addr: contract_addr.into(),
        msg,
        funds,
    }
    .into())
}

pub fn read_response<'a>(key: &str, response: &'a SubMsgResponse) -> StdResult<&'a Event> {
    response
        .events
        .iter()
        .find(|&e| e.ty.eq(key))
        .ok_or_else(|| StdError::generic_err("No event found"))
}

// reads attribute from an event by name
pub fn read_event<'a>(key: &str, event: &'a Event) -> StdResult<&'a str> {
    event
        .attributes
        .iter()
        .find_map(|attr| {
            if attr.key.eq(key) {
                Some(attr.value.as_str())
            } else {
                None
            }
        })
        .ok_or_else(|| StdError::generic_err("No attribute found"))
}
