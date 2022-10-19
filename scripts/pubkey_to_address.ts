import { bech32 } from "bech32";
import { sha256, ripemd160 } from "./hash";

// const pubkey = "023b33a8524344061b12364cba20fe0a1ab36d4486abf451bb7cebd11ea2241e5b";
const pubkey = "02ef8bc2e2e1da64c941e2234ec260e59a708c8acc979890b2046985756bff6b21";
const pubkeyBytes = Buffer.from(pubkey, "hex");
console.log("pubkey:", pubkey);

const rawAddress = ripemd160(sha256(pubkeyBytes));
const terraAddress = bech32.encode("terra", bech32.toWords(rawAddress));
console.log("terra address:", terraAddress);
