import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate'
import { Coin } from 'cosmwasm'
import { readFileSync } from 'fs'

interface Opts {
  admin?: string
  coins?: string
}

export async function uploadContract(
  prefix: String,
  client: SigningCosmWasmClient,
  senderAddress: string,
  filepath: string,
  feeAmount: string
) {
  const contract = readFileSync(filepath)

  const fee =
    prefix == 'osmo'
      ? {
          gas: '0',
          amount: [{ denom: 'uosmo', amount: '0' }],
        }
      : {
          gas: '60000000',
          amount: [{ denom: 'ujunox', amount: feeAmount }],
        }

  let code_id = await client.upload(senderAddress, contract, fee)

  return Number(code_id.codeId) // code_id
}

export async function instantiateContract(
  prefix: String,
  client: SigningCosmWasmClient,
  senderAddress: string,
  codeId: number,
  label: string,
  msg: Record<string, unknown>,
  feeAmount: string,
  opts: Opts = {}
) {
  let admin = opts.admin
  if (admin == undefined) {
    admin = senderAddress
  }

  const fee =
    prefix == 'osmo'
      ? {
          gas: '0',
          amount: [{ denom: 'uosmo', amount: '0' }],
        }
      : {
          gas: '60000000',
          amount: [{ denom: 'ujunox', amount: feeAmount }],
        }

  let result = await client.instantiate(
    senderAddress,
    codeId,
    msg,
    label,
    fee,
    opts
  )
  return result.contractAddress // contract address
}

export async function executeContract(
  prefix: String,
  client: SigningCosmWasmClient,
  senderAddress: string,
  contractAddress: string,
  msg: Record<string, unknown>,
  feeAmount: string,
  funds?: Coin[]
) {
  const fee =
    prefix == 'osmo'
      ? {
          gas: '0',
          amount: [{ denom: 'uosmo', amount: '0' }],
        }
      : {
          gas: '30000000',
          amount: [{ denom: 'ujunox', amount: feeAmount }],
        }

  const result = await client.execute(
    senderAddress,
    contractAddress,
    msg,
    fee,
    undefined,
    funds
  )

  return result
}

export async function queryContract(
  client: SigningCosmWasmClient,
  contractAddress: string,
  query: Record<string, unknown>
): Promise<any> {
  let result = await client.queryContractSmart(contractAddress, query)
  console.log(result)
  return result
}

export async function deployContract(
  prefix: String,
  client: SigningCosmWasmClient,
  senderAddress: string,
  filepath: string,
  label: string,
  initMsg: Record<string, unknown>,
  feeAmount: string,
  opts: object
) {
  const codeId = await uploadContract(
    prefix,
    client,
    senderAddress,
    filepath,
    feeAmount
  )

  return await instantiateContract(
    prefix,
    client,
    senderAddress,
    codeId,
    label,
    initMsg,
    feeAmount,
    opts
  )
}

export async function sendToken(
  prefix: String,
  client: SigningCosmWasmClient,
  senderAddress: string,
  recipientAddress: string,
  amount: string
) {
  const fee =
    prefix == 'osmo'
      ? {
          gas: '0',
          amount: [{ denom: 'uosmo', amount: '0' }],
        }
      : {
          gas: '30000000',
          amount: [{ denom: 'ujunox', amount: '150000' }],
        }
  const coin =
    prefix == 'osmo'
      ? [{ denom: 'uosmo', amount: amount }]
      : [{ denom: 'ujunox', amount: amount }]

  return await client.sendTokens(senderAddress, recipientAddress, coin, fee)
}
