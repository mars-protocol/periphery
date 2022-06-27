# Mars Airdrop

Contract for distributing MARS token airdrop on Mars Hub

## Instantiation

The deployer must transfer MARS token corresponding to the total airdrop amount to the contract at instantiation.

The `instantiate` function does not check whether the correct amount of tokens is received, or whether the Merkle root is valid (e.g. of correct length). The deployer must make sure these are accurate.

## Claiming

In order to claim MARS token airdrop, a Terra user needs to:

- [add Mars Hub to Keplr wallet](https://docs.keplr.app/api/suggest-chain.html) and obtain a Mars address to receive the airdrop
- sign a message with the Terra account's private key to prove their ownership of the account and to specify the recipient Mars address
- generate the Merkle proof

The message to be signed is:

```plain
airdrop for {terra-address} of {amount} umars shall be released to {mars-address}
```

The `execute_msg` is as follows:

```json
{
  "claim": {
    "terra_acct_pk": "...",
    "mars_acct": "...",
    "amount": "...",
    "proof": [
      "...",
      "...",
      "..."
    ],
    "signature": "..."
  }
}
```

Where `terra_acct_pk`, `proof`, and `signature` are hex-encoded strings.

See scripts [1](../../scripts/1_build_merkle_tree.ts) and [2](../../scripts/2_sign_message.ts) for examples on how to construct the Merkle proof and sign the message.

## Clawback

The contract also defines a deadline for claiming. Once the deadline is passed, anyone can invoke the `clawback` function to transfer the unclaimed tokens to the community pool.
