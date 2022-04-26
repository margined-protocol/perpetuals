import { LocalTerra, MnemonicKey } from '@terra-money/terra.js'
import { join } from 'path'
import 'dotenv/config.js'
import {
  deployContract,
  executeContract,
  Logger,
  setTimeoutDuration,
  queryNativeBalance,
} from '../helpers.js'

// CONSTS

const MARGINED_ARTIFACTS_PATH = '../artifacts'

// MAIN

;(async () => {
  setTimeoutDuration(0)

  const logger = new Logger()

  const terra = new LocalTerra()
  const owner = terra.wallets.test1
  const alice = terra.wallets.test2
  const bob = terra.wallets.test3
  const carol = terra.wallets.test4
  const feePoolContractAddress = terra.wallets.test5

  // mock contract addresses
  //   const protocolRewardsCollector = new MnemonicKey().accAddress

  /****************************************** Deploy Insurance Fund Contract *****************************************/
  console.log('Deploying Insurance Fund...')
  const insuranceFundContractAddress = await deployContract(
    terra,
    owner,
    join(MARGINED_ARTIFACTS_PATH, 'margined_insurance_fund.wasm'),
    {},
  )
  console.log(
    'Insurance Fund Contract Address: ' + insuranceFundContractAddress,
  )

  /******************************************* Deploy Mock PriceFeed Contract *****************************************/
  console.log('Deploying Mock PriceFeed...')
  const priceFeedAddress = await deployContract(
    terra,
    owner,
    join(MARGINED_ARTIFACTS_PATH, 'mock_pricefeed.wasm'),
    {
      decimals: 6,
      oracle_hub_contract: '',
    },
  )
  console.log('Mock PriceFeed Address: ' + priceFeedAddress)

  /******************************************** Deploy ETH:UST vAMM Contract ******************************************/
  console.log('Deploying ETH:UST vAMM...')
  const vammContractAddress = await deployContract(
    terra,
    owner,
    join(MARGINED_ARTIFACTS_PATH, 'margined_vamm.wasm'),
    {
      decimals: 6,
      pricefeed: priceFeedAddress,
      quote_asset: 'ETH',
      base_asset: 'UST',
      quote_asset_reserve: '1000000000', // 1,000.00
      base_asset_reserve: '100000000', // 100.00
      funding_period: 86_400, // 1 day in seconds
      toll_ratio: '0',
      spread_ratio: '0',
      fluctuation_limit_ratio: '0',
    },
  )
  console.log('ETH:UST vAMM Address: ' + vammContractAddress)

  /*************************************** Deploy Margin Engine Contract *****************************************/
  console.log('Deploy Margin Engine...')
  const marginEngineContractAddress = await deployContract(
    terra,
    owner,
    join(MARGINED_ARTIFACTS_PATH, 'margined_engine.wasm'),
    {
      decimals: 6,
      insurance_fund: insuranceFundContractAddress,
      fee_pool: feePoolContractAddress.key.accAddress,
      eligible_collateral: 'uusd',
      initial_margin_ratio: '50000',
      maintenance_margin_ratio: '50000',
      liquidation_fee: '50000',
      vamm: [vammContractAddress],
    },
  )
  console.log('Margin Engine Address: ' + marginEngineContractAddress)

  /************************************* Define Margin engine address in vAMM *************************************/
  console.log('Set Margin Engine in vAMM...')
  await executeContract(terra, owner, vammContractAddress, {
    update_config: {
      margin_engine: marginEngineContractAddress,
    },
  })
  console.log('margin engine set in vAMM')

  /************************************************ verify UST balances **********************************************/
  console.log('Query native token balances...')
  let [ownerBalance] = await queryNativeBalance(terra, owner.key.accAddress)
  let [aliceBalance] = await queryNativeBalance(terra, alice.key.accAddress)

  console.log('Owner:\t', ownerBalance.toData())
  console.log('Alice:\t', aliceBalance.toData())

  console.log('OK')

  logger.showGasConsumption()
})()
