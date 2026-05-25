# CloakMesh System Architecture

## Overview

CloakMesh is a polyglot, privacy-first decentralized network. The system is divided into four primary layers, each communicating via Protocol Buffers over gRPC.

```
┌─────────────────────────────────────────────────────────────────┐
│                    User / Application Layer                      │
│   Browser (SOCKS5)  │  CLI (cloakcli)  │  SDK (TypeScript)      │
└────────────┬─────────────────┬──────────────────┬───────────────┘
             │                 │                  │
             ▼                 ▼                  ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Orchestrator Layer (Python)                    │
│   cloakcli  │  wallet.py  │  storage.py  │  communication.py    │
│   dht_seeder.py  │  auth_generator.py  │  node_manager.py       │
└──────────────────────────────┬──────────────────────────────────┘
                                │ gRPC (proto/v1)
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Core Engine (Rust)                         │
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐    │
│  │  gRPC Server │  │  SOCKS5      │  │  Kademlia DHT      │    │
│  │  (node.rs)   │  │  Bridge      │  │  (dht.rs)          │    │
│  │  25+ handlers│  │  (bridge.rs) │  │  find/store/gossip │    │
│  └──────┬───────┘  └──────┬───────┘  └──────────┬─────────┘    │
│         │                 │                       │              │
│  ┌──────▼─────────────────▼───────────────────────▼──────────┐  │
│  │              Circuit Manager (circuit.rs)                  │  │
│  │       3-hop telescoping onion routing pool                 │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                Crypto Layer (crypto/)                      │  │
│  │  Ed25519 · X25519 · ChaCha20-Poly1305 · Kyber KEM (PQ)   │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                │
                         P2P gRPC (TunnelStream)
                                │
┌─────────────────────────────────────────────────────────────────┐
│                    Peer Nodes (same stack)                       │
│  node-alpha:4001  │  node-beta:4002  │  node-miner:4003        │
└─────────────────────────────────────────────────────────────────┘
```

---

## Core Engine (Rust)

### Entry Point: `core/src/main.rs`
- Parses CLI args with `clap`: `--port`, `--id`, `--bootstrap` (repeatable), `--mine-atk`
- Loads `NodeConfig` from TOML (defaults if missing)
- Generates or loads Ed25519 identity from `data_<node_id>/identity.pem`
- Spawns gRPC server, SOCKS5 proxy, DHT bootstrap
- Handles `SIGINT` for graceful shutdown

### Node Services: `core/src/node.rs`
Implements four gRPC service traits on `CloakNode`:

| Service | Handlers |
|---------|---------|
| `CloakMeshNode` | Handshake, SendEnvelope, KeepAlive, ChatStream, FileTransfer, TunnelStream, BroadcastTx, StoreShard, RetrieveShard, Gossip, FindNode, FindValue, StoreValueNetwork |
| `CloakService` | PublishDescriptor, FetchDescriptor, Introduce, ConnectRendezvous, HostSite |
| `TelemetryService` | GetMetrics, GetHealth, StreamMetrics, GetCircuits, GetRelays |
| `CapabilityService` | Verify |

### SOCKS5 Bridge: `core/src/network/bridge.rs`
- Listens on `127.0.0.1:<gRPC_port + 5049>`
- Handles RFC 1928 SOCKS5 handshake
- For locally-hosted addresses: proxies directly to `127.0.0.1:<local_port>`
- For remote addresses: queries DHT for descriptor, connects to intro_point via `TunnelStream` gRPC
- `TunnelStream` is bidirectional: chunks from client are forwarded; responses stream back

### Kademlia DHT: `core/src/routing/dht.rs`
- `DhtNode`: manages `RoutingTable` (k-buckets), `StorageBackend`, `authorities` list
- `bootstrap()`: pings each authority, then issues `FindNode` for own ID to discover network peers
- `find_value_network()`: checks local storage, then iteratively queries routing table peers (falls back to authorities if table empty)
- `store_value_network()`: stores locally, replicates to K closest peers (falls back to authorities if table empty)
- Maintenance: 60-second cleanup tick via `start_maintenance()`
- Storage backends: volatile `ZeroizeWrapper<HashMap>` (default) or persistent `sled` DB (`--mine-atk`)

### Onion Circuits: `core/src/routing/circuit.rs`
- `CircuitManager` maintains a pool of pre-built 3-hop circuits
- `build_circuit()`: telescoping construction (Guard → Middle → Exit)
- Guard nodes are pinned for long-term consistency
- Each circuit has a UUID, uptime tracking, and status (READY/EXPIRED)

### Cryptography: `core/src/crypto/`
- `Ed25519KeyPair`: load or generate identity key; sign/verify
- `derive_address()`: `base32(0x01 || pubkey[32] || checksum[4]).cloak`
- `parse_address()`: validates and extracts pubkey bytes
- `ChaCha20Poly1305` for symmetric cell encryption
- `zeroize` crate: `ZeroizeWrapper<T>` zeroes memory on drop

---

## Orchestrator Layer (Python)

### CLI Entry: `orchestrator/src/cloakcli/main.py`
Typer app with 25+ commands. Uses `CLOAK_GRPC_PORT` env var to select target node.

### gRPC Client: `api/grpc_client.py`
`CloakGrpcClient(host, port)` wraps four stubs:
- `CloakMeshNodeStub` — core mesh operations
- `CloakServiceStub` — descriptor and hosting
- `TelemetryServiceStub` — metrics and health
- `CapabilityServiceStub` — token verification

### Modules
| Module | Responsibility |
|--------|---------------|
| `communication.py` | ChatStream, FileTransfer, chat history, file list |
| `storage.py` | AetherStore upload/download via StoreShard/RetrieveShard |
| `wallet.py` | ATK balance display, BroadcastTx for transfers |
| `dht_seeder.py` | PublishDescriptor with IntroductionPoint, FetchDescriptor |
| `auth_generator.py` | CapabilityToken JSON issuance (Pydantic model) |
| `node_manager.py` | Start/stop node processes via subprocess + PID files |

---

## Protocol Layer (Protobuf)

All cross-language communication uses Protocol Buffers 3. See `proto/v1/`:

| File | Services |
|------|---------|
| `cloakmesh.proto` | `CloakMeshNode`: core P2P RPCs |
| `cloak_service.proto` | `CloakService`: descriptor + hosting RPCs |
| `telemetry.proto` | `TelemetryService`: metrics, health, streaming |
| `capability.proto` | `CapabilityService`: token verification |

After modifying any `.proto` file, run:
```bash
just proto-gen
```

---

## Data Flow: .cloak Site Request

```
Browser/curl
    │  HTTP GET http://<addr>.cloak/
    ▼
SOCKS5 Proxy (127.0.0.1:9051 on node-beta)
    │  RFC 1928 CONNECT <addr>.cloak:80
    ▼
MeshBridge (bridge.rs)
    │  find_value_network(key=<pubkey>) → Bootstrap peer query
    │  Returns: CloakDescriptor{intro_points:[{address:"127.0.0.1:4001"}]}
    ▼
TunnelStream gRPC (node-beta → node-alpha:4001)
    │  Bidirectional gRPC stream
    │  First chunk: {target_address, circuit_id, path:[]}
    │  Subsequent chunks: raw HTTP bytes
    ▼
TunnelStream handler (node-alpha)
    │  address hosted locally? → look up local_port from hosted_sites
    │  Open TCP connection to 127.0.0.1:8080
    ▼
HTTP server (python3 -m http.server 8080)
    │  Returns index.html
    ▼
Response streams back through TunnelStream → SOCKS5 → Browser
```

---

## Data Flow: DHT Storage

```
cloakcli storage upload secret.txt
    │
    ▼
storage.py
    ├── SHA-256 hash of file data
    ├── Create FileManifest protobuf
    ├── Gossip(manifest) → node-alpha gRPC
    │       → seen_gossip dedup
    │       → fan-out to routing table peers
    ├── StoreShard(shard) → node-alpha gRPC
    │       → store_local(key=SHA256("hash:0"), value=data, ttl=24h)
    └── Print FILE_HASH

cloakcli storage download FILE_HASH
    │
    ▼
storage.py
    └── RetrieveShard(file_hash, shard_index=0) → node gRPC
            → get_local(key=SHA256("hash:0"))
            → return FileShard{data}
            → write to output file
```

---

## Security Properties

| Property | Mechanism |
|----------|-----------|
| Sender anonymity | 3-hop onion routing, guard node pinning |
| Recipient anonymity | .cloak address + descriptor + introduction protocol |
| Forward secrecy | Double Ratchet (roadmap), ephemeral X25519 (Noise handshake) |
| Traffic analysis resistance | Fixed 514-byte cells, cover traffic, timing jitter |
| Memory safety | Rust ownership model + zeroize crate |
| No PII in logs | Structured tracing with no sensitive field logging |
| Metadata resistance | No central directory; DHT is fully decentralized |
| Post-quantum readiness | Kyber KEM fields in HandshakeInit (pq_hybrid_enabled) |
