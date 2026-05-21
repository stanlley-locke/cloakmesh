# CloakMesh: TypeScript SDK & WASM Testing Suite

This document provides commands to verify the web-based integration and browser-level privacy defenses.

---

## 1. Automated Tests

Verify the SDK structure and gRPC client:
```bash
cd sdk
npm test
```

---

## 2. WASM Cryptographic Verification

Ensure the Rust-compiled WASM module correctly handles identity and handshakes:
```bash
# Run the specific WASM smoke test
npx ts-node src/smoke_test.ts
```

**Manual Verification Steps:**
1.  Check that `Generating Address` output matches the Bech32 format.
2.  Verify `PeerIdentity type check successful`.

---

## 3. Browser Privacy Defenses

Verify the anti-fingerprinting module in `src/browser_privacy.ts`:

### Canvas & WebGL Spoofing
```bash
# Run the test that exercises the BrowserPrivacy class
npx ts-node -e "import { BrowserPrivacy } from './src/browser_privacy'; BrowserPrivacy.enableAll(); console.log('Privacy modules active');"
```

**Checklist:**
- [ ] `navigator.hardwareConcurrency` is hardcoded to `2`.
- [ ] `navigator.deviceMemory` is hardcoded to `4`.
- [ ] `navigator.plugins` is an empty list.
- [ ] Outbound headers are standardized with `standardizeHeaders()`.

---

## 4. Integration Test (End-to-End)

Verify the SDK can talk to a live Rust node:
```bash
npx ts-node -e "import { CloakClient } from './src/client/grpc_client'; const c = new CloakClient('127.0.0.1', 4001); c.ping().then(r => console.log('SDK CONNECTED:', r))"
```

"Privacy isn't a feature. It's a foundation."
