use cosmwasm_std::{Deps, StdResult};
use margined_perp::margined_pricefeed::ConfigResponse;

use crate::state::{read_config, read_price_data, Config, PriceData};

/// Queries contract Config
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner,
        decimals: config.decimals,
    })
}

/// Queries contract Config
pub fn query_get_price(deps: Deps, key: String) -> StdResult<PriceData> {
    let prices_response = read_price_data(deps.storage, key);

    // if prices.is_err() {
    //     Ok(PriceData {});
    // }

    let prices = prices_response.unwrap();
    let price = prices.last().unwrap();

    Ok(price.clone())
}
