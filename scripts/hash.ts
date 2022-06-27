import { SHA256 } from 'jscrypto/SHA256';
import { RIPEMD160 } from 'jscrypto/RIPEMD160';
import { Word32Array } from 'jscrypto';

export function sha256(data: Buffer): Buffer {
  return Buffer.from(SHA256.hash(new Word32Array(data)).toUint8Array());
}

export function ripemd160(data: Buffer): Buffer {
  return Buffer.from(RIPEMD160.hash(new Word32Array(data)).toUint8Array());
}
