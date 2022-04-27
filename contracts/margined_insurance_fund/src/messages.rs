use cosmwasm_std::{to_binary, Addr, CosmosMsg, ReplyOn, StdResult, SubMsg, WasmMsg};

use margined_perp::margined_vamm::ExecuteMsg as VammExecuteMessage;

pub fn execute_vamm_shutdown(vamm: Addr) -> StdResult<SubMsg> {
    let msg = WasmMsg::Execute {
        contract_addr: vamm.to_string(),
        funds: vec![],
        msg: to_binary(&VammExecuteMessage::SetOpen { open: false })?,
    };

    let status_msg = SubMsg {
        msg: CosmosMsg::Wasm(msg),
        gas_limit: None, // probably should set a limit in the config
        id: 0u64,
        reply_on: ReplyOn::Never,
    };

    Ok(status_msg)
}
