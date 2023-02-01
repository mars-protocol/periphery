# Mars Periphery

Peripheral smart contracts to be deployed on Mars Hub.

## Bug bounty

A bug bounty is currently open for Mars Hub and peripheral contracts. See details [here](https://immunefi.com/bounty/mars/).

## Audits

See reports [here](https://github.com/mars-protocol/mars-audits/tree/main/periphery).

## How to use

Compile the contracts:

```bash
cargo make rust-optimizer
```

At tag `v1.1.0`, the following SHA-256 checksums should be produced:

```plain
524cabbce221579a85653e7a0b0d393843a5781054192e7ead95ca87e8bb2b0f  mars_delegator.wasm
07eb67c9e9b78c314d36d791e003086e8d6d9e0cd523bae3ef714b67215e0a80  mars_vesting.wasm
```

Generate schemas:

```bash
cargo make generate-all-schemas
```

Generate TypeScript code from schemas:

```bash
cd scripts
yarn generate-types
```

## License

Contents of this repository are open source under [GNU General Public License v3](./LICENSE) or later.
