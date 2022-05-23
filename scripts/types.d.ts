// Init Messages
type InsuranceFundInitMsg = {}

type EngineInitMsg = {
  decimals: number
  insurance_fund: string
  fee_pool: string
  eligible_collateral: string
  initial_margin_ratio: string
  maintenance_margin_ratio: string
  liquidation_fee: string
}

type PriceFeedInitMsg = {
  decimals: number
  oracle_hub_contract: string
}

type VammInitMsg = {
  decimals: number
  pricefeed?: string
  margin_engine?: string
  quote_asset: string
  base_asset: string
  quote_asset_reserve: string
  base_asset_reserve: string
  funding_period: number
  toll_ratio: string
  spread_ratio: string
  fluctuation_limit_ratio: string
}

interface Config {
  insuranceFundInitMsg: InsuranceFundInitMsg
  engineInitMsg: EngineInitMsg
  priceFeedInitMsg: PriceFeedInitMsg
  vammInitMsg: VammInitMsg
  initialAssets: Asset[]
}
