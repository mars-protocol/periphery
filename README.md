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

At tag `v1.2.0`, the following SHA-256 checksums should be produced:

```plain
4285db3268c4a18b506d62110b9dd320c56685e34bc61b93898257cfe5265b98  mars_delegator.wasm
d68c1cc68a2c782e64a346eee7332bf8220ff6201ee1677fb953d5b59f4cba37  mars_vesting.wasm
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
