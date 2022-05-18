import { uploadCosmWasmContract } from './helpers.js'
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
  const insuranceFundContractAddress = await uploadCosmWasmContract(
    client,
    firstAccount.address,
    join(MARGINED_ARTIFACTS_PATH, 'cw_erc20.wasm'),
  )
  console.log(
    'Insurance Fund Contract Address: ' + insuranceFundContractAddress,
  )
}

main().catch(console.log)
