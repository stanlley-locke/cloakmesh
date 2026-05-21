# CloakMesh: Rust Core Engine

The high-performance, memory-safe backbone of the CloakMesh network.

## 🏗️ Architecture

The `core` crate handles the low-level heavy lifting for the entire network:

*   **`src/crypto/`**: Implementation of `Ed25519` for identity, `X25519` for key exchange, and the **Double Ratchet** protocol for forward-secure messaging.
*   **`src/routing/`**: 
    *   **Kademlia DHT**: Distributed storage for service descriptors.
    *   **Circuit Manager**: Builds and maintains 3-6 hop telescoping onion circuits.
    *   **GossipSub**: Peer-assisted metadata dissemination.
*   **`src/network/`**:
    *   **TCP Transport**: Framed, length-prefixed transport with strict resource limits.
    *   **Mesh Bridge**: The native SOCKS5 gateway and local hosting proxy.
*   **`src/cloak_protocol/`**:
    *   **Traffic Engine**: 514B cell framing, cover traffic injection, and timing jitter.
    *   **Address Module**: Bech32 `.cloak` address derivation and validation.

## 🛡️ Security Mandates

1.  **Volatile RAM Storage**: All sensitive data is stored in memory-only structures and zeroed out on drop using the `zeroize` crate.
2.  **Metadata Resistance**: Mandatory 514B cell framing ensures that payload sizes never leak information to global observers.
3.  **Profiling Defense**: Long-Term Guard Selection pins the first hop of circuits to a small set of trusted nodes.

## 🛠️ Build & Test

### Development Build
```bash
cargo build
```

### Release Build (Optimized)
```bash
cargo build --release
```

### Run Tests
```bash
# Run all 92+ tests
cargo test

# Test specific modules
cargo test crypto
cargo test routing::dht
cargo test routing::circuit
```

## 🚀 Running a Node

```bash
# Basic start
cargo run -- --port 4001 --id my-node-id

# Start with debug logging
RUST_LOG=debug cargo run -- --port 4001
```

---
"Privacy isn't a feature. It's a foundation."
