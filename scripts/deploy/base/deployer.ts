import { DeploymentConfig, Addresses } from '../../types/config'
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate'
import * as fs from 'fs'
import { printBlue, printGreen, printRed, printYellow } from '../../utils/chalk'
import { ARTIFACTS_PATH, Storage } from './storage'
import { InstantiateMsgs } from '../../types/msg.js'
import { writeFile } from 'fs/promises'
import { join, resolve } from 'path'
import assert from 'assert'

export class Deployer {
  constructor(
    public config: DeploymentConfig,
    public client: SigningCosmWasmClient,
    public deployerAddress: string,
    private storage: Storage,
    public addresses: Addresses,
  ) {}

  async saveStorage() {
    await this.storage.save()
  }

  async assertDeployerBalance() {
    const accountBalance = await this.client.getBalance(
      this.deployerAddress,
      this.config.baseAssetDenom,
    )
    printYellow(
      `${this.config.baseAssetDenom} account balance is: ${accountBalance.amount} (${
        Number(accountBalance.amount) / 1e6
      } ${this.config.chainPrefix})`,
    )
    if (Number(accountBalance.amount) < 1_000_000 && this.config.chainId === 'osmo-test-4') {
      printRed(
        `not enough ${this.config.chainPrefix} tokens to complete action, you may need to go to a test faucet to get more tokens.`,
      )
    }
  }

  async upload(name: keyof Storage['codeIds'], file: string) {
    if (this.storage.codeIds[name]) {
      printBlue(`Wasm already uploaded :: ${name} :: ${this.storage.codeIds[name]}`)
      return
    }

    const wasm = fs.readFileSync(ARTIFACTS_PATH + file)
    const uploadResult = await this.client.upload(this.deployerAddress, wasm, 'auto')
    this.storage.codeIds[name] = uploadResult.codeId
    printGreen(`${this.config.chainId} :: ${name} : ${this.storage.codeIds[name]}`)
  }

  async instantiate(name: keyof Storage['addresses'], codeId: number, msg: InstantiateMsgs) {
    if (this.storage.addresses[name]) {
      printBlue(`Contract already instantiated :: ${name} :: ${this.storage.addresses[name]}`)
      return
    }

    const { contractAddress: redBankContractAddress } = await this.client.instantiate(
      this.deployerAddress,
      codeId,
      // @ts-expect-error msg expecting too general of a type
      msg,
      `mars-${name}`,
      'auto',
      { admin: this.addresses.multisig },
    )

    this.storage.addresses[name] = redBankContractAddress
    printGreen(
      `${this.config.chainId} :: ${name} Contract Address : ${this.storage.addresses[name]}`,
    )
  }
 
  async instantiateLiquidationFilterer() {
    const msg = {
      owner: this.deployerAddress,
      address_provider: this.addresses.addressProvider,
    }
    await this.instantiate('liquidationFilterer', this.storage.codeIds.liquidationFilterer!, msg)
  }

  async saveDeploymentAddrsToFile() {
    const addressesDir = resolve(join(__dirname, '../../../deploy/addresses'))
    await writeFile(
      `${addressesDir}/${this.config.chainId}.json`,
      JSON.stringify(this.storage.addresses),
    )
  }

  async updateFiltererContractOwner() {
    const msg = {
      update_config: {
        owner: this.addresses.multisig,
      },
    }
    await this.client.execute(this.deployerAddress, this.storage.addresses.liquidationFilterer!, msg, 'auto')
    printYellow('Owner updated to Mutlisig for Liquidation Filterer')
    const filtererConfig = (await this.client.queryContractSmart(
      this.storage.addresses.liquidationFilterer!,
      {
        config: {},
      },
    )) as { owner: string; prefix: string }

    assert.equal(filtererConfig.owner, this.addresses.multisig)
    printGreen('It is confirmed that all contracts have transferred ownership to the Multisig')
  }
}
