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

At tag `v1.0.0`, the following SHA-256 checksums should be produced:

```plain
a251d4669097ba31f68d0b832af8bd0568bc7ea6b6bd320eeb2b9b59636fbc6e  mars_delegator.wasm
3464ed7751342865ffbd7b0750cfac5bffb9fbdaceb3d33f913aa92bd37423d1  mars_vesting.wasm
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
