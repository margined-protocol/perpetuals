import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate'
import { coins } from 'cosmwasm'
import { readFileSync } from 'fs'
import { CustomError } from 'ts-custom-error'

// LCD endpoints are load balanced, so txs can't be sent too fast, otherwise account sequence queries
// may resolve an older state depending on which lcd you end up with. Generally 1000 ms is is enough
// for all nodes to sync up.
let TIMEOUT = 1000

export function setTimeoutDuration(t: number) {
  TIMEOUT = t
}

export function getTimeoutDuration() {
  return TIMEOUT
}

let GAS_ADJUSTMENT = 1.2

export function setGasAdjustment(g: number) {
  GAS_ADJUSTMENT = g
}

export function getGasAdjustment() {
  return GAS_ADJUSTMENT
}

export async function sleep(timeout: number) {
  await new Promise((resolve) => setTimeout(resolve, timeout))
}

export class TransactionError extends CustomError {
  public constructor(
    public code: number | string,
    public codespace: string | undefined,
    public rawLog: string,
  ) {
    super('transaction failed')
  }
}

interface Opts {
  admin?: string
  coins?: string
}

export async function uploadContract(
  client: SigningCosmWasmClient,
  senderAddress: string,
  filepath: string,
) {
  const contract = readFileSync(filepath)
  const fee = {
    gas: '60000000',
    amount: [{ denom: 'ujunox', amount: '1000000' }],
  }

  let code_id = await client.upload(senderAddress, contract, fee)

  return Number(code_id.codeId) // code_id
}

export async function instantiateContract(
  client: SigningCosmWasmClient,
  senderAddress: string,
  codeId: number,
  label: string,
  msg: Record<string, unknown>,
  opts: Opts = {},
) {
  let admin = opts.admin
  if (admin == undefined) {
    admin = senderAddress
  }

  const fee = {
    gas: '60000000',
    amount: [{ denom: 'ujunox', amount: '1000000' }],
  }

  let result = await client.instantiate(
    senderAddress,
    codeId,
    msg,
    label,
    fee,
    opts,
  )
  return result.contractAddress // contract address
}

export async function executeContract(
  client: SigningCosmWasmClient,
  senderAddress: string,
  contractAddress: string,
  msg: Record<string, unknown>,
) {
  const fee = {
    gas: '30000000',
    amount: [{ denom: 'ujunox', amount: '1000000' }],
  }

  const result = await client.execute(senderAddress, contractAddress, msg, fee)

  return result
}

export async function queryContract(
  client: SigningCosmWasmClient,
  contractAddress: string,
  query: Record<string, unknown>,
): Promise<any> {
  let result = await client.queryContractSmart(contractAddress, query)
  console.log(result)
  return result
}

export async function deployContract(
  client: SigningCosmWasmClient,
  senderAddress: string,
  filepath: string,
  label: string,
  initMsg: Record<string, unknown>,
  opts: object,
) {
  const codeId = await uploadContract(client, senderAddress, filepath)

  return await instantiateContract(
    client,
    senderAddress,
    codeId,
    label,
    initMsg,
    opts,
  )
}

export async function sendToken(
  client: SigningCosmWasmClient,
  senderAddress: string,
  recipientAddress: string,
  amount: string,
) {
  const fee = {
    gas: '30000000',
    amount: [{ denom: 'ujunox', amount: '1000000' }],
  }

  return await client.sendTokens(
    senderAddress,
    recipientAddress,
    [{ denom: 'ujunox', amount: amount }],
    fee,
  )
}
