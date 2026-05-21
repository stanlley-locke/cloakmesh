# CloakMesh: TypeScript & WASM SDK

Native browser and Node.js integration for building decentralized web applications on the CloakMesh network.

## 🛠️ Features

*   **WASM Cryptography**: High-performance, Rust-compiled crypto engine for `Ed25519` identity and `Noise` handshakes.
*   **Anti-Fingerprinting**: Integrated module for **Canvas/WebGL spoofing** and **Hardware API stripping** to protect web users.
*   **Decentralized Client**: Full gRPC integration for interacting with mesh nodes from TypeScript.
*   **Privacy-First Fetch**: Standardized headers and User-Agent profiles for all outbound mesh requests.

## 📦 Installation

```bash
cd sdk
npm install
```

## 🚀 Quick Start

### 1. Initialize WASM Identity
```typescript
import { WasmCrypto } from './src/crypto/wasm_crypto';

async function init() {
    await WasmCrypto.ensureInitialized();
    const keyPair = WasmCrypto.generateKeyPair();
    const address = await WasmCrypto.deriveAddress(keyPair.public_key());
    console.log('My .cloak address:', address);
}
```

### 2. Connect to a Node
```typescript
import { CloakClient } from './src/client/grpc_client';

const client = new CloakClient('127.0.0.1', 4001);
const descriptor = await client.fetchDescriptor('some-address.cloak');
```

## 🧪 Testing & Audit

```bash
# Run unit tests (WASM & Types)
npm test

# Verify Browser Privacy module
npx ts-node src/smoke_test.ts
```

---
"Privacy isn't a feature. It's a foundation."
