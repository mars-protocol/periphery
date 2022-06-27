import { bech32 } from "bech32";
import * as bip32 from "bip32";
import * as bip39 from "bip39";
import * as secp256k1 from "secp256k1";

import { sha256, ripemd160 } from "./hash";

// localterra test account 3: https://github.com/terra-money/LocalTerra#accounts
const mnemonic = "symbol force gallery make bulk round subway violin worry mixture penalty kingdom boring survey tool fringe patrol sausage hard admit remember broken alien absorb";

// Terra's derivation path, with coin type 330
const path = "m/44'/330'/0'/0/0";

// mnemonic --> privkey
const seed: Buffer = bip39.mnemonicToSeedSync(mnemonic);
const masterKey = bip32.fromSeed(seed);
const privKey = masterKey.derivePath(path).privateKey!;
console.log("privkey:", privKey.toString("hex"));

// privkey --> pubkey
const pubkey = Buffer.from(secp256k1.publicKeyCreate(privKey, true));
console.log("pubkey:", pubkey.toString("hex"));

// pubkey --> addresses (terra, mars)
// address is derived as `ripemd160(sha256(pk_bytes))[:20]`
// according to ADR-028: https://docs.cosmos.network/master/architecture/adr-028-public-key-addresses.html
const rawAddress = ripemd160(sha256(pubkey));
const terraAddress = bech32.encode("terra", bech32.toWords(rawAddress));
const marsAddress = bech32.encode("mars", bech32.toWords(rawAddress));
console.log("terra address:", terraAddress);
console.log("mars address:", marsAddress);

// sign message
// see `1_build_merkle_tree.ts` for the Merkle tree used in this example
const msg = `airdrop for ${terraAddress} of ${42069} umars shall be released to ${marsAddress}`;
const msgBytes = Buffer.from(msg, "utf8");
const msgHashBytes = sha256(msgBytes);
const { signature } = secp256k1.ecdsaSign(msgHashBytes, privKey);
console.log("msg:", msg);
console.log("signature:", Buffer.from(signature).toString("hex"));
