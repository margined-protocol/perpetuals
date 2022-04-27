import { LocalTerra, MnemonicKey } from '@terra-money/terra.js'
import { join } from 'path'
import 'dotenv/config.js'
import {
  deployContract,
  executeContract,
  queryContract,
  Logger,
  setTimeoutDuration,
} from '../helpers.js'

import {
  approximateEqual,
  queryBalanceNative,
  getLatestBlockInfo,
} from './test_helpers.js'

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

  /******************************************** Set vAMM to open **********************************************/
  console.log('Set vAMM to open...')

  await executeContract(terra, owner, vammContractAddress, {
    set_open: {
      open: true,
    },
  })
  console.log('vAMM is open')

  /************************************ Append latest price to mock pricefeed **********************************************/
  console.log('Append prices to mock pricefeed...')

  let latestBlock = await getLatestBlockInfo(terra)
  let timestamp = new Date(latestBlock.block.header.time).valueOf()

  await executeContract(terra, owner, priceFeedAddress, {
    append_price: {
      key: 'ETH',
      price: '10000000',
      timestamp: timestamp,
    },
  })
  console.log('latest price appended to mock pricefeed')

  /*************************************** Update Margin Engine Contract *****************************************/
  console.log(
    'Update:\n\tmaintenance margin ratio\n\tpartial liquidation ratio\n\tliquidation fee\n...',
  )
  await executeContract(terra, owner, marginEngineContractAddress, {
    update_config: {
      initial_margin_ratio: '100000',
      maintenance_margin_ratio: '250000',
      liquidation_fee: '25000',
    },
  })
  console.log('Margin Engine Updated')

  /*************************************** Bob opens small positions *****************************************/
  console.log('Bob open position:\n\t20 margin * 5x Long Position\n...')

  for (let i = 0; i < 5; i++) {
    await executeContract(
      terra,
      bob,
      marginEngineContractAddress,
      {
        open_position: {
          vamm: vammContractAddress,
          side: 'b_u_y',
          quote_asset_amount: '4000000',
          base_asset_limit: '0',
          leverage: '5000000',
        },
      },
      { coins: `${4000000}uusd` },
    )
  }

  console.log('Margin Engine Updated')

  /*************************************** Alice opens small positions *****************************************/
  console.log('Alice open position:\n\t20 margin * 5x Long Position\n...')

  for (let i = 0; i < 5; i++) {
    await executeContract(
      terra,
      alice,
      marginEngineContractAddress,
      {
        open_position: {
          vamm: vammContractAddress,
          side: 'b_u_y',
          quote_asset_amount: '4000000',
          base_asset_limit: '0',
          leverage: '5000000',
        },
      },
      { coins: `${4000000}uusd` },
    )
  }

  console.log('Margin Engine Updated')

  /*************************************** Bob opens small positions *****************************************/
  console.log(
    'Bob manually closes position:\n\t20 margin * 5x Long Position\n...',
  )

  for (let i = 0; i < 5; i++) {
    await executeContract(
      terra,
      bob,
      marginEngineContractAddress,
      {
        open_position: {
          vamm: vammContractAddress,
          side: 's_e_l_l',
          quote_asset_amount: '4000000',
          base_asset_limit: '0',
          leverage: '5000000',
        },
      },
      { coins: `${4000000}uusd` },
    )
  }

  console.log('Margin Engine Updated')

  /************************************** Query vAMM Spot Balance & Update Pricefeed **********************************************/
  console.log('Query vAMM spot price...')
  let price = await queryContract(terra, vammContractAddress, {
    spot_price: {},
  })
  let out = await queryContract(terra, vammContractAddress, {
    state: {},
  })
  console.log('state: ', out)
  console.log('spot price: ', price)
  latestBlock = await getLatestBlockInfo(terra)
  timestamp = new Date(latestBlock.block.header.time).valueOf()

  await executeContract(terra, owner, priceFeedAddress, {
    append_price: {
      key: 'ETH',
      price: price,
      timestamp: timestamp,
    },
  })
  console.log('latest price appended to mock pricefeed')

  /*********************************************** Carol liquidates Alice **************************************************/
  console.log("Carol liquidates Alice's underwater position...")

  await executeContract(terra, carol, marginEngineContractAddress, {
    liquidate: {
      vamm: vammContractAddress,
      trader: alice.key.accAddress,
      quote_asset_limit: '0',
    },
  })

  console.log('Alice got rekt')

  /************************************************* Query vAMM state *****************************************************/
  console.log('Query vAMM spot price...')
  let state = await queryContract(terra, vammContractAddress, {
    state: {},
  })

  console.log('le state', state)

  approximateEqual(state.quote_asset_reserve, 107751027, 0)
  approximateEqual(state.base_asset_reserve, 92803035, 0)

  // console.log('quote asset reserve: ', state.quote_asset_reserve)
  // console.log('base asset reserve: ', state.base_asset_reserve)

  /************************************************ verify UST balances **************************************************/
  console.log('Query native token balances...')
  let ownerBalance = await queryBalanceNative(
    terra,
    owner.key.accAddress,
    'uusd',
  )
  let aliceBalance = await queryBalanceNative(
    terra,
    alice.key.accAddress,
    'uusd',
  )
  let bobBalance = await queryBalanceNative(terra, bob.key.accAddress, 'uusd')
  let carolBalance = await queryBalanceNative(
    terra,
    carol.key.accAddress,
    'uusd',
  )

  console.log('Owner:\t', ownerBalance)
  console.log('Alice:\t', aliceBalance)
  console.log('Bob:\t', bobBalance)
  console.log('Carol:\t', carolBalance)

  console.log('OK')

  logger.showGasConsumption()
})()
