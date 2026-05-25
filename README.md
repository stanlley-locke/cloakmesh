# CloakMesh: The Privacy-First Decentralized Mesh Network

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2021_Edition-orange.svg)](core/)
[![Python](https://img.shields.io/badge/Python-3.10%2B-blue.svg)](orchestrator/)
[![Status](https://img.shields.io/badge/Status-Phase_1_Complete-green.svg)](#phase-status)

**CloakMesh** is an advanced, decentralized peer-to-peer mesh network engineered for absolute metadata resistance. Built on a polyglot stack (Rust + Python + TypeScript), it integrates telescoping three-hop onion routing, a sharded Kademlia DHT, real-time gossip propagation, decentralized file storage, and encrypted P2P communication into a single cohesive system.

> *"Privacy isn't a feature. It's a foundation."*

---

## ✨ What CloakMesh Can Do Right Now

| Feature | Status | Command |
|---------|--------|---------|
| **Host a .cloak website** | ✅ Live | `cloakcli host-static <dir> <port>` |
| **Browse .cloak sites** | ✅ Live | `cloakcli browse <addr>.cloak` |
| **DHT peer discovery** | ✅ Live | `cloakcli relays` |
| **Kademlia store/fetch** | ✅ Live | `cloakcli dht-publish / dht-fetch` |
| **Gossip propagation** | ✅ Live | (auto, on upload/transfer) |
| **Decentralized file storage** | ✅ Live | `cloakcli storage upload / download` |
| **ATK token transfers** | ✅ Live | `cloakcli wallet transfer` |
| **P2P encrypted chat** | ✅ Live | `cloakcli chat-send / listen-chat` |
| **Streaming file transfer** | ✅ Live | `cloakcli file-send / file-recv` |
| **Capability tokens** | ✅ Live | `cloakcli auth-issue / auth-verify` |
| **Live telemetry** | ✅ Live | `cloakcli dash / stream-metrics` |
| **Multi-node bootstrap** | ✅ Live | `--bootstrap` (repeatable flag) |

---

## 🔐 Core Privacy Architecture

### Three-Hop Onion Routing
Traffic flows through a telescoping circuit: **Guard → Middle → Exit/Rendezvous**. No single node ever knows both the origin and destination. Circuits are rebuilt automatically via `CircuitManager` with a configurable pool.

### Kademlia DHT
Service descriptors, file shards, and routing metadata are stored in a distributed hash table using XOR-distance key routing. Any node can find any published `.cloak` address without a central directory.

### SOCKS5 Mesh Gateway
Each node exposes a native RFC 1928 SOCKS5 proxy (`gRPC_port + 5049`). Standard tools (`curl`, `Firefox`, `Brave`) can reach `.cloak` addresses through this gateway without any modification.

### Gossip Protocol
File manifests and transactions propagate through the mesh via epidemic gossip with seen-ID deduplication. Every node fans out to all routing-table peers, ensuring network-wide convergence.

### Cryptographic Identity
Nodes derive their `.cloak` address from an Ed25519 keypair using:
```
address = base32( 0x01 || pubkey[32] || SHA256(SHA256(0x01 || pubkey))[:4] ) + ".cloak"
```
Keys persist in `data_<node-id>/identity.pem` — each node has a unique identity.

---

## 📁 Project Structure

```
cloakmesh/
├── core/                   # Rust: gRPC server, DHT, SOCKS5 bridge, crypto, circuits
│   ├── src/
│   │   ├── main.rs         # Entry point, CLI args, node startup
│   │   ├── node.rs         # gRPC service implementations (25+ handlers)
│   │   ├── config.rs       # NodeConfig, validation
│   │   ├── crypto/         # Ed25519, X25519, ChaCha20-Poly1305
│   │   ├── network/
│   │   │   └── bridge.rs   # SOCKS5 proxy + TunnelStream routing
│   │   └── routing/
│   │       ├── dht.rs      # Kademlia DHT (find_node, find_value, store_value)
│   │       └── circuit.rs  # Onion circuit pool management
├── orchestrator/           # Python: CLI, telemetry, chat, storage, wallet
│   └── src/cloakcli/
│       ├── main.py         # All CLI commands (typer app)
│       ├── communication.py # Chat + file transfer logic
│       ├── storage.py      # AetherStore upload/download
│       ├── wallet.py       # ATK token commands
│       ├── dht_seeder.py   # Descriptor publish/fetch
│       ├── auth_generator.py # Capability token issuance
│       ├── node_manager.py # Background node process management
│       └── api/
│           └── grpc_client.py  # Python gRPC stubs wrapper
├── sdk/                    # TypeScript: Browser SDK + WASM bindings
├── wasm/                   # Rust → WASM for browser integration
├── proto/v1/               # Protobuf definitions (source of truth)
│   ├── cloakmesh.proto     # Core RPCs: Chat, DHT, Gossip, Tunnel, Tx
│   ├── cloak_service.proto # Service RPCs: Descriptor, Host, Rendezvous
│   ├── telemetry.proto     # Metrics, health, circuit streaming
│   └── capability.proto    # Token verification service
└── docs/                   # Technical specifications
    ├── ARCHITECTURE.md
    ├── PROTOCOL.md
    ├── CLI_REFERENCE.md    # Complete command reference
    ├── MESSAGING.md        # Chat and file transfer guide
    ├── STORAGE.md          # AetherStore decentralized storage
    ├── WALLET.md           # ATK token system
    ├── NETWORKING.md       # DHT, routing, gossip deep dive
    ├── CRYPTO.md           # Cryptographic primitives
    ├── THREAT_MODEL.md
    └── ERROR_CODES.md
```

---

## 🚀 Quick Start: Three-Node Network

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Python dependencies
cd orchestrator && poetry install

# Or build everything at once
just build-all
```

### 1. Start Node Alpha (Bootstrap Authority)
```bash
cd core
cargo run -- --port 4001 --id node-alpha
```
Copy the `.cloak` address from the startup log:
```
[200] Identity loaded, cloak_address: aedqpcdb...cloak
```

### 2. Start Node Beta (Relay / Browser)
```bash
cargo run -- --port 4002 --id node-beta --bootstrap 127.0.0.1:4001
```

### 3. Start Node Miner (ATK Mining, bootstraps to both)
```bash
cargo run -- --port 4003 --id node-miner --mine-atk \
  --bootstrap 127.0.0.1:4001 \
  --bootstrap 127.0.0.1:4002
```

### 4. Host a .cloak Website
```bash
cd orchestrator
poetry run cloakcli site-init my_site
CLOAK_GRPC_PORT=4001 poetry run cloakcli host-static my_site 8080
```

### 5. Browse From Another Node
```bash
ADDR=<your-address>.cloak
CLOAK_GRPC_PORT=4002 poetry run cloakcli browse $ADDR
```

**Result:** You'll see the full HTML response served from Node Alpha, routed through Node Beta's TunnelStream → Alpha's local HTTP server on port 8080.

---

## 🛠️ CLI Reference (Summary)

Full reference: **[docs/CLI_REFERENCE.md](./docs/CLI_REFERENCE.md)**

```bash
# Node operations
poetry run cloakcli ping                        # Check node health
poetry run cloakcli status                      # Node metrics
poetry run cloakcli dash                        # Live dashboard
poetry run cloakcli stream-metrics              # Streaming telemetry
poetry run cloakcli circuits                    # Active onion circuits
poetry run cloakcli relays                      # Known DHT peers
poetry run cloakcli address                     # Your .cloak address

# Web hosting & browsing
poetry run cloakcli site-init <dir>             # Scaffold a .cloak site
poetry run cloakcli host-static <dir> [port]    # Host + publish to DHT
poetry run cloakcli host <addr> <port>          # Bridge existing port
poetry run cloakcli browse <addr>.cloak         # Browse via SOCKS5

# DHT operations
poetry run cloakcli dht-publish <addr>          # Publish descriptor
poetry run cloakcli dht-fetch <addr>            # Fetch descriptor

# Messaging & file transfer
poetry run cloakcli chat-send "<msg>" <target>  # Send chat message
poetry run cloakcli listen-chat                 # Listen for messages
poetry run cloakcli view-chat-history           # Message history
poetry run cloakcli file-send <path> <target>   # Send file over mesh
poetry run cloakcli file-recv                   # Receive incoming files
poetry run cloakcli list-files                  # List received files

# Decentralized storage
poetry run cloakcli storage upload <file>       # Upload to DHT
poetry run cloakcli storage download <hash> -o <out>  # Download from DHT

# ATK wallet
poetry run cloakcli wallet balance              # Check balance
poetry run cloakcli wallet transfer <amt> <addr>  # Send tokens

# Capability tokens
poetry run cloakcli auth-issue <addr> [scope] [ttl]  # Issue token
poetry run cloakcli auth-verify '<json>'        # Verify token

# Node lifecycle
poetry run cloakcli node-start --port 4001 --id mynode  # Start background node
poetry run cloakcli node-stop --port 4001       # Stop background node
poetry run cloakcli interactive                 # Interactive REPL
```

> **Environment variable:** `CLOAK_GRPC_PORT=<port>` selects which running node to talk to. Defaults to 4001.

---

## 🌐 Port Reference

| Node | gRPC Port | SOCKS5 Proxy | `CLOAK_GRPC_PORT` |
|------|-----------|--------------|-------------------|
| node-alpha | 4001 | 127.0.0.1:9050 | 4001 |
| node-beta  | 4002 | 127.0.0.1:9051 | 4002 |
| node-miner | 4003 | 127.0.0.1:9052 | 4003 |
| node-delta | 4004 | 127.0.0.1:9053 | 4004 |

SOCKS5 port formula: `SOCKS5 = gRPC_port + 5049`

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [TESTING_GUIDE.md](./TESTING_GUIDE.md) | Complete end-to-end testing walkthrough |
| [docs/CLI_REFERENCE.md](./docs/CLI_REFERENCE.md) | Full command reference with examples |
| [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) | System architecture overview |
| [docs/PROTOCOL.md](./docs/PROTOCOL.md) | .cloak protocol specification |
| [docs/NETWORKING.md](./docs/NETWORKING.md) | DHT, routing, gossip deep dive |
| [docs/MESSAGING.md](./docs/MESSAGING.md) | Chat and file transfer guide |
| [docs/STORAGE.md](./docs/STORAGE.md) | AetherStore decentralized storage |
| [docs/WALLET.md](./docs/WALLET.md) | ATK token system |
| [docs/CRYPTO.md](./docs/CRYPTO.md) | Cryptographic primitives reference |
| [docs/THREAT_MODEL.md](./docs/THREAT_MODEL.md) | Privacy threat model |
| [docs/ERROR_CODES.md](./docs/ERROR_CODES.md) | Structured error code reference |
| [core/README.md](./core/README.md) | Rust core engine guide |
| [orchestrator/README.md](./orchestrator/README.md) | Python orchestrator guide |

---

## 🔩 Configuration

The node reads from `configs/default.toml` (falls back to hardcoded defaults if missing). Key options:

```toml
node_id = "my-node"
listen_port = 4001
bootstrap_peers = ["127.0.0.1:4001"]
data_dir = "data"           # Auto-prefixed: data_<node_id>/

[crypto]
identity_key_path = "data/identity.pem"
pq_hybrid_enabled = true    # Kyber KEM hybrid post-quantum

[circuit]
default_hops = 3            # Guard → Middle → Exit

[traffic]
cell_size_bytes = 256
padding_enabled = true      # Cover traffic / timing jitter
```

CLI overrides: `--port`, `--id`, `--bootstrap` (repeatable), `--mine-atk`

---

## 🏗️ Development Workflow

```bash
# Build all components
just build-all

# Regenerate gRPC bindings after .proto changes
just proto-gen

# Run all tests
just test

# Lint all languages
just lint

# Build Rust core only
cd core && cargo build

# Run Python tests
cd orchestrator && poetry run pytest
```

### Commit Convention
Follow conventional commits: `feat:`, `fix:`, `docs:`, `perf:`, `refactor:`

---

## 📊 Phase Status

| Phase | Focus | Status |
|-------|-------|--------|
| **Phase 1** | Protocol definition, Protobuf bindings, gRPC skeleton | ✅ Complete |
| **Phase 2** | Kademlia DHT, SOCKS5 bridge, .cloak hosting & browsing | ✅ Complete |
| **Phase 3** | Gossip propagation, AetherStore, ATK wallet, messaging | ✅ Complete |
| **Phase 4** | Double Ratchet E2E encryption, UTXO validation, ZK gates | 🔄 In Progress |
| **Phase 5** | Multi-hop onion routing with real circuit negotiation | 🔄 Planned |
| **Phase 6** | CloakBrowser integration, WASM SDK, erasure coding | 📅 Planned |

---

## 🤝 Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for style guidelines, security disclosure policy, and branch conventions. All PRs must pass `just lint` and `just test`.

## ⚖️ License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
