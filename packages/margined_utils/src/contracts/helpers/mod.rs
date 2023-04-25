use cosmwasm_schema::serde::Serialize;
use cosmwasm_std::{to_binary, Coin, CosmosMsg, StdResult, WasmMsg};

pub use crate::contracts::helpers::margined_engine::EngineController;
pub use crate::contracts::helpers::margined_fee_pool::FeePoolController;
pub use crate::contracts::helpers::margined_insurance_fund::InsuranceFundController;
pub use crate::contracts::helpers::margined_pricefeed::PricefeedController;
pub use crate::contracts::helpers::margined_vamm::VammController;

pub mod margined_engine;
pub mod margined_fee_pool;
pub mod margined_insurance_fund;
pub mod margined_pricefeed;
pub mod margined_vamm;

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
