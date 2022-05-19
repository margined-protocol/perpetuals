use cosmwasm_std::{
    to_binary, Addr, BalanceResponse, BankQuery, Deps, QueryRequest, StdResult, Uint128, WasmQuery,
};
use cw20::{BalanceResponse as CW20BalanceResponse, Cw20QueryMsg};
use margined_common::asset::AssetInfo;

pub fn query_token_balance(deps: Deps, token: AssetInfo, account_addr: Addr) -> StdResult<Uint128> {
    let balance: Uint128 = match token {
        AssetInfo::NativeToken { denom } => {
            let res: BalanceResponse = deps
                .querier
                .query(&QueryRequest::Bank(BankQuery::Balance {
                    address: account_addr.to_string(),
                    denom,
                }))
                .unwrap();

            res.amount.amount
        }
        AssetInfo::Token { contract_addr } => {
            let res: CW20BalanceResponse = deps
                .querier
                .query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: contract_addr.to_string(),
                    msg: to_binary(&Cw20QueryMsg::Balance {
                        address: account_addr.to_string(),
                    })?,
                }))
                .unwrap();

            res.balance
        }
    };

    Ok(balance)
}
