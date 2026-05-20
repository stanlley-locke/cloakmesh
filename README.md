# CloakMesh

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Status](https://img.shields.io/badge/Status-Phase_1-orange.svg)](#roadmap)

**CloakMesh** is a next-generation decentralized network designed for privacy, resilience, and modularity. It provides a secure foundation for metadata-resistant communication using onion-style layered routing, a sharded Kademlia DHT, and cryptographic self-sovereign identities.

## Key Features

- **Onion Routing:** 3-6 hop adaptive circuits for metadata-resistant communication.
- **Privacy-First Identity:** Ed25519-based self-sovereign identities (`.cloak` addresses).
- **Hybrid PQ Crypto:** X25519 + Kyber-768 hybrid key exchange (in development).
- **Polyglot SDK:** Native support for Rust, Python, and TypeScript.
- **WASM Ready:** Run full or light nodes directly in the browser.
- **Decentralized Discovery:** Sharded Kademlia DHT with Merkle-verifiable descriptors.

## Project Structure

```text
cloakmesh/
├── core/                 # Rust: Networking, crypto, routing engine
├── orchestrator/         # Python: CLI, node management, analytics
├── sdk/                  # TypeScript: Client SDK, web UI, events
├── proto/                # Shared .proto & CBOR schema definitions
├── wasm/                 # Compiled WASM artifacts & bindings
├── deploy/               # Docker, K8s, CI/CD pipelines
└── docs/                 # Protocol spec, RFCs, threat model
```

## Getting Started

### Prerequisites

- **Rust** >= 1.70
- **Python** >= 3.10
- **Node.js** >= 18
- **protoc** >= 3.20
- **just** (task runner)

### Quick Start

1. **Bootstrap the project:**
   ```bash
   just bootstrap
   ```

2. **Build all components:**
   ```bash
   just build-all
   ```

3. **Run a local testnet:**
   ```bash
   docker compose up -d
   ```

4. **Connect via CLI:**
   ```bash
   cd orchestrator && poetry run cloakcli connect
   ```

## Roadmap

| Phase | Milestone | Status |
| :--- | :--- | :--- |
| **Phase 1** | **Protocol Definition & Cross-Language Bindings** | **Active** |
| Phase 2 | Rust Core Engine (Network, Crypto, DHT) | Planned |
| Phase 3 | WASM & Browser Integration | Planned |
| Phase 4 | Python Orchestrator & CLI | Planned |
| Phase 5 | TypeScript SDK & Web Client | Planned |
| Phase 6 | Security Hardening & Audit | Planned |

## Documentation

- [Protocol Specification](./docs/PROTOCOL.md)
- [Architecture Overview](./docs/ARCHITECTURE.md)
- [Threat Model](./docs/THREAT_MODEL.md)
- [Deployment Guide](./docs/DEPLOYMENT.md)

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under Apache-2.0. See [LICENSE](LICENSE) for details.

---
"Privacy isn't a feature. It's a foundation."
