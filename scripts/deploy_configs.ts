export const testnet: Config = {
  initialAssets: [],
  insuranceFundInitMsg: {},
  priceFeedInitMsg: {
    decimals: 6,
    oracle_hub_contract: '',
  },
  engineInitMsg: {
    decimals: 6,
    insurance_fund: '',
    fee_pool: '',
    eligible_collateral: 'UST',
    initial_margin_ratio: '50000',
    maintenance_margin_ratio: '50000',
    liquidation_fee: '50000',
  },
  vammInitMsg: {
    decimals: 6,
    pricefeed: undefined,
    quote_asset: 'ETH',
    base_asset: 'UST',
    quote_asset_reserve: '1000000000', // 1,000.00
    base_asset_reserve: '100000000', // 100.00
    funding_period: 86_400, // 1 day in seconds
    toll_ratio: '0',
    spread_ratio: '0',
    fluctuation_limit_ratio: '0',
  },
}
export const local: Config = {
  initialAssets: [],
  insuranceFundInitMsg: {},
  priceFeedInitMsg: {
    decimals: 6,
    oracle_hub_contract: '',
  },
  engineInitMsg: {
    decimals: 6,
    insurance_fund: '',
    fee_pool: '',
    eligible_collateral: 'UST',
    initial_margin_ratio: '50000',
    maintenance_margin_ratio: '50000',
    liquidation_fee: '50000',
  },
  vammInitMsg: {
    decimals: 6,
    pricefeed: undefined,
    quote_asset: 'ETH',
    base_asset: 'UST',
    quote_asset_reserve: '1000000000', // 1,000.00
    base_asset_reserve: '100000000', // 100.00
    funding_period: 86_400, // 1 day in seconds
    toll_ratio: '0',
    spread_ratio: '0',
    fluctuation_limit_ratio: '0',
  },
}
