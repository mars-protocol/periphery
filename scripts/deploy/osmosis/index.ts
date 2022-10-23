import { taskRunner } from '../base'
import { osmosisAddresses, osmosisTestnetConfig } from './config.js'

void (async function () {
  await taskRunner(osmosisTestnetConfig, osmosisAddresses)
})()
