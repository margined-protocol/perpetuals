#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Storage, Timestamp, Uint128,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_storage::{singleton, singleton_read};

pub static KEY_CONFIG: &[u8] = b"config";
pub static KEY_PRICES: &[u8] = b"prices";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub decimals: u8,
    pub oracle_hub_contract: String, // address of the oracle hub we are using
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AppendPrice {
        key: String,
        price: Uint128,
        timestamp: u64,
    },
    AppendMultiplePrice {
        key: String,
        prices: Vec<Uint128>,
        timestamps: Vec<u64>,
    },
    UpdateConfig {
        owner: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    GetPrice {
        key: String,
    },
    GetPreviousPrice {
        key: String,
        num_round_back: Uint128,
    },
    GetTwapPrice {
        key: String,
        interval: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: Addr,
    pub decimals: Uint128,
}

#[cfg(not(tarpaulin_include))]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        owner: info.sender,
        decimals: Uint128::from(10u128.pow(msg.decimals as u32)),
    };

    store_config(deps.storage, &config)?;

    Ok(Response::default())
}

#[cfg(not(tarpaulin_include))]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::AppendPrice {
            key,
            price,
            timestamp,
        } => append_price(deps, info, key, price, timestamp),
        ExecuteMsg::AppendMultiplePrice {
            key,
            prices,
            timestamps,
        } => append_multiple_price(deps, info, key, prices, timestamps),
        ExecuteMsg::UpdateConfig { owner } => update_config(deps, info, owner),
    }
}

#[cfg(not(tarpaulin_include))]
pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
) -> StdResult<Response> {
    let mut config: Config = read_config(deps.storage)?;

    // check permission
    if info.sender != config.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    // change owner of amm
    if let Some(owner) = owner {
        config.owner = deps.api.addr_validate(owner.as_str())?;
    }

    store_config(deps.storage, &config)?;

    Ok(Response::default())
}

/// this is a mock function that enables storage of data
/// by the contract owner will be replaced by integration
/// with on-chain price oracles in the future.
#[cfg(not(tarpaulin_include))]
pub fn append_price(
    deps: DepsMut,
    _info: MessageInfo,
    key: String,
    price: Uint128,
    timestamp: u64,
) -> StdResult<Response> {
    store_price_data(deps.storage, key, price, timestamp)?;

    Ok(Response::default())
}

/// this is a mock function that enables storage of data
/// by the contract owner will be replaced by integration
/// with on-chain price oracles in the future.
#[cfg(not(tarpaulin_include))]
pub fn append_multiple_price(
    deps: DepsMut,
    _info: MessageInfo,
    key: String,
    prices: Vec<Uint128>,
    timestamps: Vec<u64>,
) -> StdResult<Response> {
    store_price_data(deps.storage, key, prices[0], timestamps[0])?;

    Ok(Response::default())
}

#[cfg(not(tarpaulin_include))]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::GetPrice { key } => to_binary(&query_get_price(deps, key)?),
        QueryMsg::GetPreviousPrice {
            key,
            num_round_back,
        } => to_binary(&query_get_previous_price(deps, key, num_round_back)?),
        QueryMsg::GetTwapPrice { key, interval } => {
            to_binary(&query_get_twap_price(deps, env, key, interval)?)
        }
    }
}

/// Queries contract Config
#[cfg(not(tarpaulin_include))]
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner,
        decimals: config.decimals,
    })
}

/// Queries latest price for pair stored with key
#[cfg(not(tarpaulin_include))]
pub fn query_get_price(deps: Deps, _key: String) -> StdResult<Uint128> {
    singleton_read(deps.storage, KEY_PRICES).load()
}

/// Queries previous price for pair stored with key
#[cfg(not(tarpaulin_include))]
pub fn query_get_previous_price(
    deps: Deps,
    _key: String,
    _num_round_back: Uint128,
) -> StdResult<Uint128> {
    singleton_read(deps.storage, KEY_PRICES).load()
}

/// Queries contract Config
#[cfg(not(tarpaulin_include))]
pub fn query_get_twap_price(
    deps: Deps,
    _env: Env,
    _key: String,
    _interval: u64,
) -> StdResult<Uint128> {
    singleton_read(deps.storage, KEY_PRICES).load()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub decimals: Uint128,
}

#[cfg(not(tarpaulin_include))]
pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(config)
}

#[cfg(not(tarpaulin_include))]
pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, KEY_CONFIG).load()
}

#[derive(Serialize, Default, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PriceData {
    pub round_id: Uint128,
    pub price: Uint128,
    pub timestamp: Timestamp,
}

#[cfg(not(tarpaulin_include))]
pub fn store_price_data(
    storage: &mut dyn Storage,
    _key: String,
    price: Uint128,
    _timestamp: u64,
) -> StdResult<()> {
    singleton(storage, KEY_PRICES).save(&price)
}

#[cfg(not(tarpaulin_include))]
pub fn read_price_data(storage: &dyn Storage, _key: String) -> StdResult<Uint128> {
    singleton_read(storage, KEY_PRICES).load()
}
