# Hub Periphery

Peripheral smart contracts to be deployed on Mars Hub

## License

TBD

## Deploy scripts set up 

Everything related to deployment must be ran from the `scripts` directory:
```
cd scripts
```
Set up yarn:
```
yarn install
```
Create the build folder:
```
yarn build
```
Compile all contracts:
```
yarn compile
```
This compiles and optimizes all contracts, storing them in `/artifacts` directory along with `checksum.txt` which contains sha256 hashes of each of the `.wasm` files (The script just uses CosmWasm's [rust-optimizer](https://github.com/CosmWasm/rust-optimizer)).

Note: Docker deamon must be running to run this command. 

Formating must be done before running lint:
```
yarn format
```
Linting:
```
yarn lint
```
Now you're ready to deploy for an outpost.

## Deploying to an Outpost
Each outpost has a config file for its respective deployment and assets

For Osmosis:
```
yarn deploy:osmosis
```