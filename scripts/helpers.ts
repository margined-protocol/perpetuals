import {
  BlockTxBroadcastResult,
  Coin,
  isTxError,
  LCDClient,
  MnemonicKey,
  Msg,
  MsgExecuteContract,
  MsgInstantiateContract,
  MsgMigrateContract,
  MsgUpdateContractAdmin,
  MsgStoreCode,
  Tx,
  TxError,
  Wallet,
} from '@terra-money/terra.js'
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate'
import { toBase64, toUtf8 } from '@cosmjs/encoding'
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
  logger?: Logger
}

export class Logger {
  private gasConsumptions: Array<{ msg: string; gasUsed: number }> = []

  constructor(readonly logGasConsumption: boolean = true) {}

  addGasConsumption(msg: object, gasUsed: number) {
    const msgStr = JSON.stringify(msg)
    this.gasConsumptions.push({ msg: msgStr, gasUsed: gasUsed })
  }

  showGasConsumption() {
    if (this.gasConsumptions.length == 0) {
      return
    }

    this.gasConsumptions.sort((a, b) => b.gasUsed - a.gasUsed)

    console.log('--- MAX GAS CONSUMPTION ---')
    const maxGasConsumption = this.gasConsumptions[0]
    console.log(
      'gas used: ',
      maxGasConsumption.gasUsed,
      ', msg: ',
      maxGasConsumption.msg,
    )

    console.log('--- AVERAGE GAS CONSUMPTION ---')
    const sumOfGasConsumption = this.gasConsumptions.reduce(
      (a, b) => a + b.gasUsed,
      0,
    )
    const avgOfGasConsumption =
      sumOfGasConsumption / this.gasConsumptions.length
    console.log('avg gas used: ', avgOfGasConsumption)

    console.log('--- SORTED GAS CONSUMPTION (DESCENDING) ---')
    this.gasConsumptions.forEach(function ({ msg, gasUsed }) {
      console.log('gas used: ', gasUsed, ', msg: ', msg)
    })
  }
}

export async function createTransaction(wallet: Wallet, msg: Msg) {
  return await wallet.createTx({
    msgs: [msg],
    gasAdjustment: GAS_ADJUSTMENT,
  })
}

export async function broadcastTransaction(terra: LCDClient, signedTx: Tx) {
  const result = await terra.tx.broadcast(signedTx)
  await sleep(TIMEOUT)
  return result
}

export async function performTransaction(
  terra: LCDClient,
  wallet: Wallet,
  msg: Msg,
) {
  const tx = await createTransaction(wallet, msg)
  const { account_number, sequence } = await wallet.accountNumberAndSequence()
  const signedTx = await wallet.key.signTx(tx, {
    accountNumber: account_number,
    sequence: sequence,
    chainID: terra.config.chainID,
    signMode: 1, // SignMode.SIGN_MODE_DIRECT
  })
  const result = await broadcastTransaction(terra, signedTx)
  if (isTxError(result)) {
    throw transactionErrorFromResult(result)
  }
  return result
}

export function transactionErrorFromResult(
  result: BlockTxBroadcastResult & TxError,
) {
  return new TransactionError(result.code, result.codespace, result.raw_log)
}

export async function uploadContract(
  terra: LCDClient,
  wallet: Wallet,
  filepath: string,
) {
  const contract = readFileSync(filepath, 'base64')
  const uploadMsg = new MsgStoreCode(wallet.key.accAddress, contract)
  let result = await performTransaction(terra, wallet, uploadMsg)
  return Number(result.logs[0].eventsByType.store_code.code_id[0]) // code_id
}

export async function uploadCosmWasmContract(
  client: SigningCosmWasmClient,
  senderAddress: string,
  filepath: string,
) {
  const contract = readFileSync(filepath)
  const fee = {
    gas: '30000000',
    amount: [{ denom: 'ujunox', amount: '1000000' }],
  }

  let code_id = await client.upload(senderAddress, contract, fee)

  return Number(code_id.codeId) // code_id
}

export async function instantiateContract(
  terra: LCDClient,
  wallet: Wallet,
  codeId: number,
  msg: object,
  opts: Opts = {},
) {
  let admin = opts.admin
  if (admin == undefined) {
    admin = wallet.key.accAddress
  }
  const instantiateMsg = new MsgInstantiateContract(
    wallet.key.accAddress,
    admin,
    codeId,
    msg,
    undefined,
  )
  let result = await performTransaction(terra, wallet, instantiateMsg)
  const attributes = result.logs[0].events[0].attributes
  return attributes[attributes.length - 1].value // contract address
}

export async function instantiateCosmWasmContract(
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
    gas: '30000000',
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
  terra: LCDClient,
  wallet: Wallet,
  contractAddress: string,
  msg: object,
  opts: Opts = {},
) {
  const coins = opts.coins
  const logger = opts.logger

  const executeMsg = new MsgExecuteContract(
    wallet.key.accAddress,
    contractAddress,
    msg,
    coins,
  )
  const result = await performTransaction(terra, wallet, executeMsg)

  if (logger !== undefined && logger.logGasConsumption) {
    // save gas consumption during contract execution
    logger.addGasConsumption(msg, result.gas_used)
  }

  return result
}

export async function queryContract(
  terra: LCDClient,
  contractAddress: string,
  query: object,
): Promise<any> {
  return await terra.wasm.contractQuery(contractAddress, query)
}

export async function queryCosmWasmContract(
  client: SigningCosmWasmClient,
  contractAddress: string,
  query: Record<string, unknown>,
): Promise<any> {
  let result = await client.queryContractSmart(contractAddress, query)
  console.log(result)
  return result
}

export async function deployContract(
  terra: LCDClient,
  wallet: Wallet,
  filepath: string,
  initMsg: object,
) {
  const codeId = await uploadContract(terra, wallet, filepath)
  return await instantiateContract(terra, wallet, codeId, initMsg)
}

export async function deployCosmWasmContract(
  client: SigningCosmWasmClient,
  senderAddress: string,
  filepath: string,
  label: string,
  initMsg: Record<string, unknown>,
  opts: object,
) {
  const codeId = await uploadCosmWasmContract(client, senderAddress, filepath)

  return await instantiateCosmWasmContract(
    client,
    senderAddress,
    codeId,
    label,
    initMsg,
    opts,
  )
}

export async function updateContractAdmin(
  terra: LCDClient,
  admin: Wallet,
  newAdmin: string,
  contractAddress: string,
) {
  const updateContractAdminMsg = new MsgUpdateContractAdmin(
    admin.key.accAddress,
    newAdmin,
    contractAddress,
  )
  return await performTransaction(terra, admin, updateContractAdminMsg)
}

export async function migrate(
  terra: LCDClient,
  wallet: Wallet,
  contractAddress: string,
  newCodeId: number,
) {
  const migrateMsg = new MsgMigrateContract(
    wallet.key.accAddress,
    contractAddress,
    newCodeId,
    {},
  )
  return await performTransaction(terra, wallet, migrateMsg)
}

export function recover(terra: LCDClient, mnemonic: string) {
  const mk = new MnemonicKey({ mnemonic: mnemonic })
  return terra.wallet(mk)
}

export function initialize(terra: LCDClient) {
  const mk = new MnemonicKey()

  console.log(`Account Address: ${mk.accAddress}`)
  console.log(`MnemonicKey: ${mk.mnemonic}`)

  return terra.wallet(mk)
}

export function toEncodedBinary(object: any) {
  return Buffer.from(JSON.stringify(object)).toString('base64')
}
