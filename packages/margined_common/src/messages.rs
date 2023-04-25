use cosmwasm_schema::serde::Serialize;
use cosmwasm_std::{to_binary, Coin, CosmosMsg, StdResult, WasmMsg};

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
