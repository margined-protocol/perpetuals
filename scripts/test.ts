import {
  uploadCosmWasmContract,
  deployCosmWasmContract,
  instantiateCosmWasmContract,
  queryCosmWasmContract,
} from './helpers.js'
import { CosmWasmClient, setupNodeLocal } from 'cosmwasm'
import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing'

import { SigningStargateClient, StargateClient } from '@cosmjs/stargate'
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
  // This is your rpc endpoint
  //   const rpcEndpoint = 'https://rpc.cliffnet.cosmwasm.com:443/'
  //   const

  //   const client = await CosmWasmClient.connect(rpcEndpoint)
  const client = await setupNodeLocal(config, mnemonic)
  // console.log(await client.getBlock())
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
    prefix: 'juno',
  })
  console.log(wallet)
  const [firstAccount] = await wallet.getAccounts()
  console.log(firstAccount)

  // const client = await SigningStargateClient.connectWithSigner(
  //   config.rpcEndpoint,
  //   wallet,
  // )
  // console.log(await client.getBlock())

  /****************************************** Deploy Insurance Fund Contract *****************************************/
  console.log('Deploying Insurance Fund...')

  let msg = {}

  const insuranceFundContractAddress = await deployCosmWasmContract(
    client,
    firstAccount.address,
    join(MARGINED_ARTIFACTS_PATH, 'margined_insurance_fund.wasm'),
    'insurance-fund',
    msg,
    {},
  )
  console.log(
    'Insurance Fund Contract Address: ' + insuranceFundContractAddress,
  )

  let queryMsg = {
    config: {},
  }

  let result = await queryCosmWasmContract(
    client,
    insuranceFundContractAddress,
    queryMsg,
  )

  console.log(result)
}

main().catch(console.log)
