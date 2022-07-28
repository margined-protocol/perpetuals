import 'dotenv/config.js'
import { deployContract, executeContract, queryContract } from './helpers.js'
import { setupNodeLocal } from 'cosmwasm'
import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing'
import { local, testnet } from './deploy_configs.js'
import { join } from 'path'

// consts

const config = {
  chainId: 'uni-3',
  rpcEndpoint: 'https://rpc.margined.io',
  prefix: 'juno',
}

const MARGINED_ARTIFACTS_PATH = '../artifacts'

// main
async function main() {
  const mnemonic = process.env.MNEMONIC

  // just check mnemonic has actually been defined
  if (mnemonic === null || mnemonic === undefined) {
    const message = `mnemonic undefined`

    throw new Error(message)
  }

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

  ///
  /// Deploy Fee Pool Contract
  ///
  console.log('Deploying Fee Pool...')
  const feePoolContractAddress = await deployContract(
    client,
    account.address,
    join(MARGINED_ARTIFACTS_PATH, 'margined_fee_pool.wasm'),
    'margined_fee_pool',
    {},
    '150000',
    {},
  )
  console.log('Fee Pool Contract Address: ' + feePoolContractAddress)

  ///
  /// Deploy Insurance Fund Contract
  ///
  console.log('Deploying Insurance Fund...')
  const insuranceFundContractAddress = await deployContract(
    client,
    account.address,
    join(MARGINED_ARTIFACTS_PATH, 'margined_insurance_fund.wasm'),
    'margined_insurance_fund',
    {},
    '150000',
    {},
  )
  console.log(
    'Insurance Fund Contract Address: ' + insuranceFundContractAddress,
  )

  ///
  /// Deploy Mock PriceFeed Contract
  ///
  console.log('Deploying Mock PriceFeed...')
  const priceFeedAddress = await deployContract(
    client,
    account.address,
    join(MARGINED_ARTIFACTS_PATH, 'margined_pricefeed.wasm'),
    'margined_pricefeed',
    deployConfig.priceFeedInitMsg,
    '150000',
    {},
  )
  console.log('Mock PriceFeed Address: ' + priceFeedAddress)

  ///
  /// Deploy ETH:UST vAMM Contract
  ///
  console.log('Deploying ETH:UST vAMM...')
  deployConfig.vammInitMsg.pricefeed = priceFeedAddress
  const vammContractAddress = await deployContract(
    client,
    account.address,
    join(MARGINED_ARTIFACTS_PATH, 'margined_vamm.wasm'),
    'margined_vamm',
    deployConfig.vammInitMsg,
    '300000',
    {},
  )
  console.log('ETH:UST vAMM Address: ' + vammContractAddress)

  ///
  /// Deploy CW20 Token Contract
  ///
  console.log('Deploy Margined CW20...')

  let marginCW20ContractAddress = await deployContract(
    client,
    account.address,
    join(MARGINED_ARTIFACTS_PATH, 'cw20_base.wasm'),
    'margined_cw20',
    {
      name: 'Margined USD',
      symbol: 'USDm',
      decimals: 6,
      initial_balances: [
        {
          address: account.address,
          amount: '1000000000000000000', // 1mn
        },
        {
          address: insuranceFundContractAddress,
          amount: '5000000000', // 5,000
        },
        {
          address: 'juno1w5jfhzm93vkpevpverdgkj33dw3dfus825mfnm',
          amount: '1000000000', // 1,000
        },
        {
          address: 'juno1dedkvygl3kx903axl7ypnrhu0g2p855sflz305',
          amount: '1000000000', // 1,000
        },
        {
          address: 'juno1evd2a75k42450nkkteatsmpmlq8kzk50vja0n8',
          amount: '1000000000', // 1,000
        },
        {
          address: 'juno18da0wya36037qq73whp4vkaf8fw078hl9y2kf5',
          amount: '1000000000', // 1,000
        },
        {
          address: 'juno1peu2fm3rtuc3hrpaskazzh68qle8g654z68y2w',
          amount: '1000000000', // 1,000
        },
        {
          address: 'juno1qrqw3650zq7md6txk4g3pyt98vr6f02neq0krc',
          amount: '1000000000', // 1,000
        },
      ],
      mint: {
        minter: account.address,
      },
    },
    '150000',
    {},
  )
  console.log('Margin CW20 Address: ' + marginCW20ContractAddress)

  ///
  /// Deploy Margin Engine Contract
  ///

  console.log('Deploy Margin Engine...')
  deployConfig.engineInitMsg.insurance_fund = insuranceFundContractAddress
  deployConfig.engineInitMsg.fee_pool = feePoolContractAddress
  deployConfig.engineInitMsg.eligible_collateral = 'ujunox'
  const marginEngineContractAddress = await deployContract(
    client,
    account.address,
    join(MARGINED_ARTIFACTS_PATH, 'margined_engine.wasm'),
    'margined_engine',
    deployConfig.engineInitMsg,
    '300000',
    {},
  )
  console.log('Margin Engine Address: ' + marginEngineContractAddress)

  // Define Margin engine address in vAMM
  console.log('Set Margin Engine in vAMM...')
  await executeContract(
    client,
    account.address,
    vammContractAddress,
    {
      update_config: {
        margin_engine: marginEngineContractAddress,
      },
    },
    '150000',
  )
  console.log('Margin Engine set in vAMM')

  ///
  /// Define the token address in the Margin Engine
  ///
  console.log('Set Eligible Collateral in Margin Engine...')
  await executeContract(
    client,
    account.address,
    marginEngineContractAddress,
    {
      update_config: {
        eligible_collateral: marginCW20ContractAddress,
      },
    },
    '150000',
  )
  console.log('Margin Engine set in vAMM')

  ///
  /// Register vAMM in Insurance Fund
  ///
  console.log('Register vAMM in Insurance Fund...')
  await executeContract(
    client,
    account.address,
    insuranceFundContractAddress,
    {
      add_vamm: {
        vamm: vammContractAddress,
      },
    },
    '150000',
  )
  console.log('vAMM registered')

  ///
  ///
  /// Define Margin Engine as Insurance Fund Beneficiary
  ///
  ///
  console.log('Define Margin Engine as Insurance Fund Beneficiary...')
  await executeContract(
    client,
    account.address,
    insuranceFundContractAddress,
    {
      update_config: {
        beneficiary: marginEngineContractAddress,
      },
    },
    '150000',
  )
  console.log('Margin Engine set as beneficiary')

  ///
  /// Set vAMM Open
  ///
  console.log('Set vAMM Open...')
  await executeContract(
    client,
    account.address,
    vammContractAddress,
    {
      set_open: {
        open: true,
      },
    },
    '150000',
  )
  console.log('vAMM set to open')

  ///
  /// Query vAMM state
  ///
  console.log('Querying vAMM state...')
  let state = await queryContract(client, vammContractAddress, {
    state: {},
  })
  console.log('vAMM state:\n', state)
}

main().catch(console.log)
