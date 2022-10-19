import { setupDeployer } from './setupDeployer'
import { DeploymentConfig, MultisigConfig } from '../../types/config'
import { printRed } from '../../utils/chalk'
import { atomAsset, osmoAsset } from '../osmosis/config'

export const taskRunner = async (config: DeploymentConfig, multisig: MultisigConfig) => {
  const deployer = await setupDeployer(config, multisig)

  try {
    await deployer.assertDeployerBalance()
    await deployer.saveDeploymentAddrsToFile()

    // Upload contracts
    await deployer.upload('liquidationFilterer', 'mars-liquidation-filterer.wasm')

    // Instantiate contracts
    await deployer.instantiateLiquidationFilterer()

    //update owner to multisig address
    await deployer.updateFiltererContractOwner()

  } catch (e) {
    printRed(e)
  } finally {
    await deployer.saveStorage()
  }
}
