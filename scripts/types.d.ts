type cw20coin = {
  address: string,
  amount: number,
}

// Init Messages
type InsuranceFundInitMsg = {}

type EngineInitMsg = {
  decimals: number
  insurance_fund: string
  fee_pool: string
  eligible_collateral?: string
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

type cw20InitMsg = {
  name: string
  symbol: string
  decimals: number
  initial_balances: Array<cw20coin>
  mint?: string
  marketing?: string
}

interface Config {
  insuranceFundInitMsg: InsuranceFundInitMsg
  engineInitMsg: EngineInitMsg
  priceFeedInitMsg: PriceFeedInitMsg
  vammInitMsg: VammInitMsg
  cw20InitMsg: cw20InitMsg
  initialAssets: Asset[]
}
