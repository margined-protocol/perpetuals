import 'dotenv/config.js'
import {
  deployCosmWasmContract,
  executeCosmWasmContract,
  queryCosmWasmContract,
} from './helpers.js'
import { setupNodeLocal } from 'cosmwasm'
import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing'
import { local, testnet } from './deploy_configs.js'
import { join } from 'path'

// consts

const config = {
  chainId: 'testing',
  rpcEndpoint: 'http://127.0.0.1:26657',
  prefix: 'juno',
}

const mnemonic =
  'clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose'

const MARGINED_ARTIFACTS_PATH = '../artifacts'

// main

async function main() {
  const client = await setupNodeLocal(config, mnemonic)
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
    prefix: 'juno',
  })
  let deployConfig: Config
  const isTestnet = process.env.NETWORK === 'testnet'

  const [account] = await wallet.getAccounts()

  if (process.env.NETWORK === 'testnet') {
    deployConfig = testnet
  } else {
    deployConfig = local
  }

  console.log(`Wallet address from seed: ${account.address}`)

  /****************************************** Deploy Insurance Fund Contract *****************************************/
  console.log('Deploying Insurance Fund...')
  const insuranceFundContractAddress = await deployCosmWasmContract(
    client,
    account.address,
    join(MARGINED_ARTIFACTS_PATH, 'margined_insurance_fund.wasm'),
    'margined_insurance_fund',
    {},
    {},
  )
  console.log(
    'Insurance Fund Contract Address: ' + insuranceFundContractAddress,
  )

  /******************************************* Deploy Mock PriceFeed Contract *****************************************/
  console.log('Deploying Mock PriceFeed...')
  const priceFeedAddress = await deployCosmWasmContract(
    client,
    account.address,
    join(MARGINED_ARTIFACTS_PATH, 'mock_pricefeed.wasm'),
    'mock_pricefeed',
    deployConfig.priceFeedInitMsg,
    {},
  )
  console.log('Mock PriceFeed Address: ' + priceFeedAddress)

  /******************************************** Deploy ETH:UST vAMM Contract ******************************************/
  console.log('Deploying ETH:UST vAMM...')
  deployConfig.vammInitMsg.pricefeed = priceFeedAddress
  const vammContractAddress = await deployCosmWasmContract(
    client,
    account.address,
    join(MARGINED_ARTIFACTS_PATH, 'margined_vamm.wasm'),
    'margined_vamm',
    deployConfig.vammInitMsg,
    {},
  )
  console.log('ETH:UST vAMM Address: ' + vammContractAddress)

  /*************************************** Deploy Margin Engine Contract *****************************************/
  console.log('Deploy Margin Engine...')
  deployConfig.engineInitMsg.insurance_fund = insuranceFundContractAddress
  deployConfig.engineInitMsg.fee_pool = insuranceFundContractAddress // TODO this needs its own contract
  deployConfig.engineInitMsg.eligible_collateral = 'uusd' // TODO this needs its own contract
  const marginEngineContractAddress = await deployCosmWasmContract(
    client,
    account.address,
    join(MARGINED_ARTIFACTS_PATH, 'margined_engine.wasm'),
    'margined_engine',
    deployConfig.engineInitMsg,
    {},
  )
  console.log('Margin Engine Address: ' + marginEngineContractAddress)

  /************************************* Define Margin engine address in vAMM *************************************/
  console.log('Set Margin Engine in vAMM...')
  await executeCosmWasmContract(client, account.address, vammContractAddress, {
    update_config: {
      margin_engine: marginEngineContractAddress,
    },
  })
  console.log('margin engine set in vAMM')

  /************************************** Register vAMM in Insurance Fund ******************************************************/
  console.log('Register vAMM in Insurance Fund...')
  await executeCosmWasmContract(
    client,
    account.address,
    insuranceFundContractAddress,
    {
      add_vamm: {
        vamm: vammContractAddress,
      },
    },
  )
  console.log('vAMM registered')

  /*********************************************** Set vAMM Open ******************************************************/
  console.log('Set vAMM Open...')
  await executeCosmWasmContract(client, account.address, vammContractAddress, {
    set_open: {
      open: true,
    },
  })
  console.log('vAMM set to open')

  /************************************************ Query vAMM state **********************************************/
  console.log('Querying vAMM state...')
  let state = await queryCosmWasmContract(client, vammContractAddress, {
    state: {},
  })
  console.log('vAMM state:\n', state)
}

main().catch(console.log)
