import * as fs from 'fs'
import * as path from 'path'
import { MerkleTree } from 'merkletreejs'

import { sha256 } from './hash'

type InputData = {
  address: string
  amount: number
}[]

type OutputData = {
  address: string
  amount: number
  merkle_proof: string[]
}[]

/**
 * Reads a JSON of `InputData` type, generates merkle proofs, and write the output to file in
 * `OutputData` format.
 */
function generateProofs(inputFile: string, outputFile: string) {
  const input: InputData = JSON.parse(fs.readFileSync(inputFile, 'utf8'))
  console.log(`read input data of ${input.length} users`)

  const leaves = input.map(({ address, amount }) =>
    sha256(Buffer.from(`${address}:${amount}`, 'utf8')),
  )
  const tree = new MerkleTree(leaves, sha256, {
    sortLeaves: false,
    sortPairs: true,
  })
  console.log('created merkle tree')

  const output: OutputData = []
  for (const [idx, { address, amount }] of input.entries()) {
    const leaf = leaves[idx]!
    const proof = tree.getProof(leaf).map((p) => p.data.toString('hex'))
    output.push({
      address,
      amount,
      merkle_proof: proof,
    })
    console.log(`generated proof for user ${idx + 1}`)
  }

  fs.writeFileSync(outputFile, JSON.stringify(output, null, 2))
}

generateProofs(
  path.join(__dirname, 'data/testnet_amount_no_contracts.json'),
  path.join(__dirname, 'data/testnet_amount_no_contracts_with_proofs.json'),
)
