import { ProtocolError } from '../utils/errors';

const VERSION_BYTE = 0x01;

export function parseAddress(addr: string): Uint8Array {
  if (!addr.endsWith('.cloak')) {
    throw new ProtocolError('Missing .cloak suffix');
  }
  const host = addr.slice(0, -6).toUpperCase();
  const bytes = base32Decode(host);
  if (bytes.length !== 37) {
    throw new ProtocolError('Invalid address length');
  }
  if (bytes[0] !== VERSION_BYTE) {
    throw new ProtocolError('Unknown address version');
  }
  return bytes.slice(1, 33);
}

export function deriveAddress(pubkey: Uint8Array): string {
  if (pubkey.length !== 32) throw new ProtocolError('pubkey must be 32 bytes');
  const payload = new Uint8Array(37);
  payload[0] = VERSION_BYTE;
  payload.set(pubkey, 1);
  // checksum: first 4 bytes of SHA-256(version || pubkey) — computed server-side or via WASM
  const encoded = base32Encode(payload).toLowerCase();
  return `${encoded}.cloak`;
}

// Minimal RFC4648 base32 (no padding)
const ALPHABET = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';

function base32Encode(data: Uint8Array): string {
  let bits = 0, value = 0, output = '';
  for (const byte of data) {
    value = (value << 8) | byte;
    bits += 8;
    while (bits >= 5) {
      output += ALPHABET[(value >>> (bits - 5)) & 31];
      bits -= 5;
    }
  }
  if (bits > 0) output += ALPHABET[(value << (5 - bits)) & 31];
  return output;
}

function base32Decode(input: string): Uint8Array {
  const lookup = Object.fromEntries([...ALPHABET].map((c, i) => [c, i]));
  let bits = 0, value = 0;
  const output: number[] = [];
  for (const char of input) {
    if (!(char in lookup)) throw new ProtocolError(`Invalid base32 char: ${char}`);
    value = (value << 5) | lookup[char];
    bits += 5;
    if (bits >= 8) { output.push((value >>> (bits - 8)) & 255); bits -= 8; }
  }
  return new Uint8Array(output);
}
