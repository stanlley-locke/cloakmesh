# CloakMesh: Rust Core Engine Testing Suite

This document provides comprehensive commands to verify the performance, security, and protocol integrity of the Rust core.

---

## 1. Unit & Integration Tests

Run the full suite of 92+ automated tests:
```bash
cd core
cargo test
```

### Specific Module Verification

**Cryptographic Integrity (Double Ratchet, AEAD, KDF):**
```bash
cargo test crypto
```

**Traffic Analysis Resistance (514B Cell Framing & Padding):**
```bash
cargo test cloak_protocol::traffic
```

**Discovery & Routing (Kademlia DHT & K-Buckets):**
```bash
cargo test routing::dht
```

**Onion Circuit Construction (3-Hop Telescoping):**
```bash
cargo test routing::circuit
```

**Address Derivation & Validation:**
```bash
cargo test cloak_protocol::address
```

---

## 2. Production Build Verification

Ensure the engine compiles with maximum optimizations and zero warnings:
```bash
cd core
cargo build --release
```

---

## 3. Manual Node Execution

Start a node with detailed logging to observe circuit and discovery events:
```bash
# RUST_LOG allows you to see the deep protocol state changes
RUST_LOG=debug ./target/release/cloakmesh --port 4001 --id test-node-1
```

**Checklist:**
- [ ] Identity loaded and Bech32 address generated.
- [ ] gRPC server listening on `0.0.0.0:4001`.
- [ ] SOCKS5 Proxy initialized on `127.0.0.1:9050`.
- [ ] Adaptive 3-hop circuit established automatically.

---

## 4. Fuzzing & Memory Safety (Advanced)

Verify that the C++ bindings and unsafe blocks (if any) are secure:
```bash
# Requires cargo-fuzz
cargo +nightly fuzz run parse_address
```

"Privacy isn't a feature. It's a foundation."
