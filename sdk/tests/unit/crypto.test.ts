import { WasmCrypto } from '../../src/crypto/wasm_crypto';

describe('WasmCrypto', () => {
    it('should generate a keypair and derive a valid address', async () => {
        await WasmCrypto.ensureInitialized();
        const kp = WasmCrypto.generateKeyPair();
        const pubkey = kp.public_key();
        expect(pubkey.length).toBe(32);

        const address = await WasmCrypto.deriveAddress(pubkey);
        expect(address).toContain('.cloak');
        console.log('Generated Address:', address);
    });

    it('should perform a mock diffie-hellman exchange', async () => {
        await WasmCrypto.ensureInitialized();
        const alice = WasmCrypto.createExchange();
        const bob = WasmCrypto.createExchange();

        const aliceShared = alice.diffie_hellman(bob.public_key());
        const bobShared = bob.diffie_hellman(alice.public_key());

        expect(aliceShared).toEqual(bobShared);
        expect(aliceShared.length).toBe(32);
    });
});
