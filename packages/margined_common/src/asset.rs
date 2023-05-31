use cosmwasm_schema::cw_serde;
use std::fmt;

use cosmwasm_std::{
    Addr, Api, BankMsg, Coin, CosmosMsg, MessageInfo, QuerierWrapper, StdError, StdResult, Uint128,
};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, TokenInfoResponse};
use cw_utils::must_pay;

use crate::messages::wasm_execute;

pub const NATIVE_DENOM: &str = "orai";

/// ## Description
/// This enum describes a Cosmos asset (native or CW20).
#[cw_serde]
pub struct Asset {
    /// Information about an asset stored in a [`AssetInfo`] struct
    pub info: AssetInfo,
    /// A token amount
    pub amount: Uint128,
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.amount, self.info)
    }
}

impl Asset {
    /// Returns true if the token is native. Otherwise returns false.
    /// ## Params
    /// * **self** is the type of the caller object.
    pub fn is_native_token(&self) -> bool {
        self.info.is_native_token()
    }

    pub fn into_msg(self, recipient: String, sender: Option<String>) -> StdResult<CosmosMsg> {
        self.info.into_msg(recipient, self.amount, sender)
    }

    /// Validates an amount of native tokens being sent. Returns [`Ok`] if successful, otherwise returns [`Err`].
    /// ## Params
    /// * **self** is the type of the caller object.
    ///
    /// * **message_info** is an object of type [`MessageInfo`]
    pub fn assert_sent_native_token_balance(&self, message_info: &MessageInfo) -> StdResult<()> {
        // grab the denom from self so we can test
        let msg_amount = if let AssetInfo::NativeToken { denom } = &self.info {
            // call `must_pay` to ensure its the right denom + funds are sent
            must_pay(message_info, denom)
                .map_err(|error| StdError::generic_err(error.to_string()))?
        } else {
            // this error occurs if self is of type `AssetInfo::Token`
            return Err(StdError::generic_err("self is not native token"));
        };

        if self.amount == msg_amount {
            Ok(())
        } else {
            Err(StdError::generic_err(
                "Native token balance mismatch between the argument and the transferred",
            ))
        }
    }
}

#[cw_serde]
pub enum AssetInfo {
    /// Non-native Token
    Token { contract_addr: Addr },
    /// Native token
    NativeToken { denom: String },
}

impl fmt::Display for AssetInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AssetInfo::NativeToken { denom } => write!(f, "{}", denom),
            AssetInfo::Token { contract_addr } => write!(f, "{}", contract_addr),
        }
    }
}

impl AssetInfo {
    /// Returns true if the caller is a native token. Otherwise returns false.
    /// ## Params
    /// * **self** is the caller object type
    pub fn is_native_token(&self) -> bool {
        match self {
            AssetInfo::NativeToken { .. } => true,
            AssetInfo::Token { .. } => false,
        }
    }

    /// Returns True if the calling token is the same as the token specified in the input parameters.
    /// Otherwise returns False.
    /// ## Params
    /// * **self** is the type of the caller object.
    ///
    /// * **asset** is object of type [`AssetInfo`].
    pub fn equal(&self, asset: &AssetInfo) -> bool {
        match self {
            AssetInfo::Token { contract_addr, .. } => {
                let self_contract_addr = contract_addr;
                match asset {
                    AssetInfo::Token { contract_addr, .. } => self_contract_addr == contract_addr,
                    AssetInfo::NativeToken { .. } => false,
                }
            }
            AssetInfo::NativeToken { denom, .. } => {
                let self_denom = denom;
                match asset {
                    AssetInfo::Token { .. } => false,
                    AssetInfo::NativeToken { denom, .. } => self_denom == denom,
                }
            }
        }
    }

    /// If the caller object is a native token of type ['AssetInfo`] then his `denom` field converts to a byte string.
    ///
    /// If the caller object is a token of type ['AssetInfo`] then his `contract_addr` field converts to a byte string.
    /// ## Params
    /// * **self** is the type of the caller object.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            AssetInfo::NativeToken { denom } => denom.as_bytes(),
            AssetInfo::Token { contract_addr } => contract_addr.as_bytes(),
        }
    }

    /// Returns [`Ok`] if the token of type [`AssetInfo`] is in lowercase and valid. Otherwise returns [`Err`].
    /// ## Params
    /// * **self** is the type of the caller object.
    ///
    /// * **api** is a object of type [`Api`]
    pub fn check(&self, api: &dyn Api) -> StdResult<()> {
        match self {
            AssetInfo::Token { contract_addr } => {
                api.addr_validate(contract_addr.as_ref())?;
            }
            AssetInfo::NativeToken { denom } => {
                if !denom.starts_with("ibc/") && denom != &denom.to_lowercase() {
                    return Err(StdError::generic_err(format!(
                        "Non-IBC token denom {} should be lowercase",
                        denom
                    )));
                }
            }
        }
        Ok(())
    }

    /// returns the decimal places used by the token, places some assumption that native tokens
    /// use the `utoken` convention, will fail if native token does not follow this
    pub fn get_decimals(&self, querier: &QuerierWrapper) -> StdResult<u8> {
        match self {
            AssetInfo::NativeToken { denom } => {
                // orai is 6 decimals
                if denom.eq(NATIVE_DENOM) {
                    return Ok(6);
                }
                // prefix must follow -> https://github.com/osmosis-labs/osmosis/pull/2223
                match denom.chars().next() {
                    // default is empty char => go to Err case
                    Some('u') => Ok(6u8),  // micro
                    Some('n') => Ok(9u8),  // nano
                    Some('p') => Ok(12u8), // pico
                    _ => Err(StdError::generic_err(
                        "Native token does not follow prefix standards",
                    )),
                }
            }
            AssetInfo::Token { contract_addr } => {
                // query the CW20 contract for its decimals
                let response: TokenInfoResponse =
                    querier.query_wasm_smart(contract_addr, &Cw20QueryMsg::TokenInfo {})?;

                Ok(response.decimals)
            }
        }
    }

    /// Returns a message of type [`CosmosMsg`].
    ///
    /// For native tokens of type [`AssetInfo`] uses the default method [`BankMsg::Send`] to send a token amount to a recipient.
    /// Before the token is sent, we need to deduct a tax.
    ///
    /// For a token of type [`AssetInfo`] we use the default method [`Cw20ExecuteMsg::Transfer`] and so there's no need to deduct any other tax.
    /// ## Params
    /// * **self** is the type of the caller object.
    ///
    /// * **querier** is an object of type [`QuerierWrapper`]
    ///
    /// * **recipient** is the address where the funds will be sent.
    pub fn into_msg(
        &self,
        recipient: String,
        amount: Uint128,
        sender: Option<String>,
    ) -> StdResult<CosmosMsg> {
        match self {
            AssetInfo::Token { contract_addr } => wasm_execute(
                contract_addr,
                &match sender {
                    Some(owner) => Cw20ExecuteMsg::TransferFrom {
                        owner,
                        recipient,
                        amount,
                    },
                    None => Cw20ExecuteMsg::Transfer { recipient, amount },
                },
                vec![],
            ),
            AssetInfo::NativeToken { denom, .. } => Ok(CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient,
                amount: vec![Coin {
                    denom: denom.to_string(),
                    amount,
                }],
            })),
        }
    }

    pub fn query_balance(
        &self,
        querier: &QuerierWrapper,
        account_addr: Addr,
    ) -> StdResult<Uint128> {
        let balance = match self {
            AssetInfo::NativeToken { denom } => {
                let res = querier.query_balance(account_addr, denom)?;
                res.amount
            }
            AssetInfo::Token { contract_addr } => {
                let res: BalanceResponse = querier.query_wasm_smart(
                    contract_addr,
                    &Cw20QueryMsg::Balance {
                        address: account_addr.to_string(),
                    },
                )?;

                res.balance
            }
        };

        Ok(balance)
    }
}

#[cfg(test)]
mod test {
    use super::AssetInfo;
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::StdError;

    #[test]
    fn decimal_checks() {
        let deps = mock_dependencies();

        let utoken = AssetInfo::NativeToken {
            denom: "uwasm".to_string(),
        };
        assert_eq!(utoken.get_decimals(&deps.as_ref().querier).unwrap(), 6u8);

        let ntoken = AssetInfo::NativeToken {
            denom: "nwasm".to_string(),
        };
        assert_eq!(ntoken.get_decimals(&deps.as_ref().querier).unwrap(), 9u8);

        let ptoken = AssetInfo::NativeToken {
            denom: "pwasm".to_string(),
        };
        assert_eq!(ptoken.get_decimals(&deps.as_ref().querier).unwrap(), 12u8);

        let token = AssetInfo::NativeToken {
            denom: "wasm".to_string(),
        };

        let err = token.get_decimals(&deps.as_ref().querier).unwrap_err();
        assert_eq!(
            StdError::generic_err("Native token does not follow prefix standards"),
            err
        );
    }
}
