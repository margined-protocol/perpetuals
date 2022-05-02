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

  /************************************** Register vAMM in Insurance Fund ******************************************************/
  console.log('Register vAMM in Insurance Fund...')
  await executeContract(terra, owner, insuranceFundContractAddress, {
    add_vamm: {
      vamm: vammContractAddress,
    },
  })
  console.log('vAMM registered')

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

  /*************************************** Update Toll and Spread Ratio *****************************************/
  console.log('Update:\n\ttoll ratio\n\tspread ratio\n...')
  await executeContract(terra, owner, vammContractAddress, {
    update_config: {
      toll_ratio: '100000',
      spread_ratio: '0',
    },
  })
  console.log('vAMM Updated')

  /******************************************** Alice opens position *****************************************/
  console.log('Alice open position:\n\t200 margin * 1x Short Position\n...')

  await executeContract(
    terra,
    alice,
    marginEngineContractAddress,
    {
      open_position: {
        vamm: vammContractAddress,
        side: 's_e_l_l',
        quote_asset_amount: '200000000',
        base_asset_limit: '25000000',
        leverage: '1000000',
      },
    },
    { coins: `${220000000}uusd` },
  )

  console.log('Alice opened position')

  let aliceBalance1 = await queryBalanceNative(
    terra,
    alice.key.accAddress,
    'uusd',
  )

  /********************************************* Bob opens positions *****************************************/
  console.log('Bob open position:\n\t200 margin * 1x Long Position\n...')

  await executeContract(
    terra,
    bob,
    marginEngineContractAddress,
    {
      open_position: {
        vamm: vammContractAddress,
        side: 'b_u_y',
        quote_asset_amount: '200000000',
        base_asset_limit: '25000000',
        leverage: '1000000',
      },
    },
    { coins: `${220000000}uusd` },
  )

  console.log('Bob opened position')

  /*************************************** Get Alice's unrealized pnl *****************************************/
  console.log("Alice's unrealized pnl...")

  let pnl = await queryContract(terra, marginEngineContractAddress, {
    unrealized_pnl: {
      vamm: vammContractAddress,
      trader: alice.key.accAddress,
      calc_option: 's_p_o_t_p_r_i_c_e',
    },
  })

  console.log('Unrealized Pnl:\n\t', pnl.unrealized_pnl)
  approximateEqual(pnl.unrealized_pnl, -133333334, 0)

  /******************************************** Alice opens position *****************************************/
  console.log('Alice open position:\n\t50 margin * 4x Short Position\n...')

  await executeContract(
    terra,
    alice,
    marginEngineContractAddress,
    {
      open_position: {
        vamm: vammContractAddress,
        side: 's_e_l_l',
        quote_asset_amount: '50000000',
        base_asset_limit: '25000000',
        leverage: '4000000',
      },
    },
    { coins: `${70000000}uusd` },
  )

  console.log('Alice opened position')

  let aliceBalance2 = await queryBalanceNative(
    terra,
    alice.key.accAddress,
    'uusd',
  )
  approximateEqual(aliceBalance1 - aliceBalance2, 70_000_000, 0)

  /************************************************* Query Alice Position *****************************************************/
  console.log('Query Alice Position...')
  let position = await queryContract(terra, marginEngineContractAddress, {
    position: {
      vamm: vammContractAddress,
      trader: alice.key.accAddress,
    },
  })

  approximateEqual(position.size, -50_000_000, 0)
  approximateEqual(position.margin, 250_000_000, 0)
  approximateEqual(position.notional, 400_000_000, 0)

  console.log('OK')

  logger.showGasConsumption()
})()
