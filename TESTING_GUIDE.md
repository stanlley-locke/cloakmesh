# CloakMesh Phase 2: Milestone Testing & Utilization Guide

This guide provides the necessary commands and procedures to verify that all features of the Phase 2 milestone (Core Engine, Transport, Crypto, and Cross-Language Integration) are working correctly and securely.

## 1. Automated Test Suites

The first step is to run the internal unit and integration tests for each component.

### Rust (Core Engine)
Run the complete Rust test suite (96+ tests) covering Crypto, DHT, Transport, and Protocol logic.
```bash
cd core
cargo test
```

### Python (Orchestrator)
Run the Python protocol and CLI smoke tests.
```bash
cd orchestrator
# Verify protobuf bindings and basic logic
poetry run pytest
# Run the specific proto instantiation test
poetry run python tests/smoke_proto.py
```

### TypeScript (SDK)
Run the SDK unit tests and type verification.
```bash
cd sdk
# Verify generated types and basic client structure
npm test
# Run the proto smoke test
npx ts-node src/smoke_test.ts
```

---

## 2. Live End-to-End Integration Test

This test verifies that all three layers (Rust, Python, TS) can communicate in a live environment.

### Step A: Start the Core Node
Open a terminal and start a Rust node listening on port 4001.
```bash
cd core
cargo run -- --port 4001 --id local-test-node
```
*Expected Output:* You should see `info: gRPC server starting` and `info: Node fully initialized and listening`.

### Step B: Verify via Python Orchestrator
In a second terminal, use the Python CLI to check the node's health.
```bash
cd orchestrator
# Ensure bindings are fixed if not done via just proto-gen
find src/proto -name "*.py" -exec sed -i 's/^import \(.*_pb2\)/from . import \1/g' {} +
poetry run cloakcli node status
```
*Expected Output:* `SUCCESS: Node is alive. Received pong with nonce: 123`.

### Step C: Verify via TypeScript SDK
In a third terminal, run a small script to verify the SDK's gRPC connectivity.
```bash
cd sdk
npx ts-node -e "import { CloakClient } from './src/client/grpc_client'; const c = new CloakClient('127.0.0.1', 4001); c.ping().then(r => { console.log('SDK PING SUCCESS:', (r as any).nonce); process.exit(0); }).catch(e => { console.error('SDK PING FAILED:', e); process.exit(1); })"
```
*Expected Output:* `SDK PING SUCCESS: 123`.

---

## 3. Feature Utilization Examples

### Generating and Validating Addresses
You can test the address derivation logic directly via the core library (used for .cloak identities).
```bash
# Within the core directory, run an example (if added) or check via tests
cargo test cloak_protocol::address
```

### Inspecting Noise Handshake
The Noise handshake logic can be exercised to verify mutual authentication.
```bash
cargo test crypto::noise
```

### Manual DHT Storage Test
The node currently supports local DHT storage via its internal API. This can be verified during the core test run:
```bash
cargo test routing::dht
```

---

## 4. Troubleshooting

- **Port Conflict:** If you get an error that port 4001 is in use, use the `--port` flag to specify a different one.
- **Missing Bindings:** If you get import errors in Python or TS, run `just proto-gen` from the root directory.
- **Rust Toolchain:** Ensure you have the latest stable Rust installed (`rustup update`).

"Privacy isn't a feature. It's a foundation."
