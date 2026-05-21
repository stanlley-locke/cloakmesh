# CloakMesh: The Privacy-First Decentralized Mesh Network

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/Status-Production--Ready-green.svg)](#roadmap)
[![Browser](https://img.shields.io/badge/Browser-CloakBrowser-orange.svg)](./cloak-browser)

**CloakMesh** is an advanced, decentralized peer-to-peer network designed for total metadata resistance. It combines telescoping onion routing, a sharded Kademlia DHT, and the Signal Double Ratchet protocol to provide a secure foundation for anonymous web hosting, private messaging, and encrypted file sharing.

---

## 🚀 Key Features

### 🔐 Advanced Cryptography & Privacy
*   **Three-Hop Onion Routing:** Adaptive, telescoping circuits (Guard -> Middle -> Exit/Rendezvous) ensure that no single node knows both the sender and receiver.
*   **Double Ratchet Protocol:** Forward-secure, break-in recoverable end-to-end encryption for all P2P communication.
*   **514-Byte Cell Framing:** Strictly enforced fixed-size cells with cryptographic random padding to defeat traffic analysis.
*   **Volatile RAM Storage:** Sensitive session data and descriptors are stored in-memory only and cryptographically zeroed out upon eviction.

### 🌐 Network Infrastructure
*   **SOCKS5 Mesh Gateway:** A native RFC 1928 gateway allowing standard browsers and tools (`curl`, `firefox`) to visit `.cloak` addresses anonymously.
*   **Decentralized Discovery:** Sharded Kademlia DHT with cryptographically verifiable service descriptors.
*   **Mesh Hosting:** Map local TCP services to unique `.cloak` identities with a single command.
*   **Traffic Analysis Defense:** Integrated cover flow (chaff traffic) and timing jitter to defeat global observers.

### 🛠️ Polyglot Ecosystem
*   **Core Engine (Rust):** High-performance, memory-safe backbone for routing and crypto.
*   **Orchestrator (Python):** Production-grade CLI for node management, observability, and chat.
*   **SDK (TypeScript & WASM):** Native browser integration for building decentralized web apps.
*   **CloakBrowser:** A hardened fork of the Mullvad Browser pre-configured for the CloakMesh network.

---

## 📁 Project Structure

```text
cloakmesh/
├── core/                 # Rust: Onion routing, DHT, Double Ratchet, TCP Transport
├── orchestrator/         # Python: Management CLI, Chat Listener, File Manager
├── sdk/                  # TypeScript: Web integration & Browser WASM Crypto
├── cloak-browser/        # C++/JS: Mullvad-based hardened privacy browser
├── proto/                # Cross-language gRPC/Protobuf definitions
├── docs/                 # Specifications, Feature Compliance, Error Codes
└── tests/                # System-wide integration and smoke tests
```

---

## 🚦 Quick Start

### 1. Bootstrap the System
```bash
just bootstrap
just proto-gen
cd core && cargo build --release
```

### 2. Launch your Gateway
```bash
./target/release/cloakmesh --port 4001 --id my-gateway
```
*Note your `.cloak` address in the logs.*

### 3. Host and Browse
```bash
# Terminal B: Start a local server
python3 -m http.server 8080 &
# Terminal B: Map it to your .cloak address
poetry run cloakcli node host your-address.cloak 8080

# Terminal C: Browse anonymously
curl -x socks5h://127.0.0.1:9050 http://your-address.cloak
```

---

## 📚 Documentation Portal

*   **[Master Operations Manual](./TESTING_GUIDE_ADVANCED.md):** Step-by-step guide to all network features.
*   **[Advanced Feature Compliance](./docs/FEATURES_COMPLIANCE.md):** Audit of all 36+ integrated privacy features.
*   **[Unified Error Codes](./docs/ERROR_CODES.md):** Troubleshooting guide for the polyglot stack.
*   **[Protocol Specification](./docs/PROTOCOL.md):** Technical deep-dive into the .cloak protocol.
*   **[Architecture Overview](./docs/ARCHITECTURE.md):** High-level system design and data lifecycle.

### Sub-Module Guides
*   **[Rust Core Engine](./TEST_RUST.md)**
*   **[Python Orchestrator CLI](./TEST_PYTHON.md)**
*   **[TypeScript/WASM SDK](./TEST_SDK.md)**
*   **[CloakBrowser Build Guide](./TEST_BROWSER.md)**

---

## 🤝 Contributing

We welcome contributions to the mesh! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for style guidelines and security disclosure policies.

## ⚖️ License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

---
"Privacy isn't a feature. It's a foundation."
