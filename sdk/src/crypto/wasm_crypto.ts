import init, { KeyPair, X25519Exchange, cloak_address_from_pubkey } from '../../wasm/web/pkg/cloakmesh_wasm';

export class WasmCrypto {
    private static initialized = false;

    public static async ensureInitialized() {
        if (!this.initialized) {
            await init();
            this.initialized = true;
        }
    }

    public static generateKeyPair(): KeyPair {
        return new KeyPair();
    }

    public static async deriveAddress(pubkey: Uint8Array): Promise<string> {
        await this.ensureInitialized();
        return cloak_address_from_pubkey(pubkey);
    }

    public static createExchange(): X25519Exchange {
        return new X25519Exchange();
    }
}
