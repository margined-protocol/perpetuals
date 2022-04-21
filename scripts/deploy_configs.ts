export const testnet: Config = {
  initialAssets: [],
  insuranceFundInitMsg: {
    "config": {
      "address_provider_address": undefined,
    }
  },
  priceFeedInitMsg: {
    "address_provider_address": undefined,
    "decimals": 9,
    "oracle_hub_contract": "",
  },
  engineInitMsg: {
    "config": {
      "address_provider_address": undefined,
      "insurance_fund": "",
      "fee_pool": "",
      "eligible_collateral": "",
      "initial_margin_ratio": 1_000_000,
      "maintenance_margin_ratio": 1_000_000,
      "liquidation_fee": 1_000_000,
      "vamm": ["", ""],
    }
  },
}
  
export const local: Config = {
  initialAssets: [],
  insuranceFundInitMsg: {
    "config": {
      "address_provider_address": undefined,
    }
  },
  priceFeedInitMsg: {
    "address_provider_address": undefined,
    "decimals": 9,
    "oracle_hub_contract": "",
  },
  engineInitMsg: {
    "config": {
      "address_provider_address": undefined,
      "insurance_fund": "",
      "fee_pool": "",
      "eligible_collateral": "",
      "initial_margin_ratio": 1_000_000,
      "maintenance_margin_ratio": 1_000_000,
      "liquidation_fee": 1_000_000,
      "vamm": ["", ""],
    }
  },
}