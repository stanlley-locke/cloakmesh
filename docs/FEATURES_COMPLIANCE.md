# CloakMesh Features & Implementation Status

Complete inventory of every feature, its implementation status, and the phase it targets. Use this as the authoritative reference for what is live, what is partial, and what is roadmapped.

---

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Fully implemented and tested |
| ⚠️ | Implemented but partial / mocked |
| 🔷 | Scaffolded / skeleton exists in code |
| 📅 | Planned for a future phase |

---

## Core Network

| Feature | Status | Notes |
|---------|--------|-------|
| Ed25519 identity keypair generation | ✅ | `crypto/ed25519.rs`, persisted to `data_<id>/identity.pem` |
| .cloak address derivation | ✅ | Both Rust (`address.rs`) and Python (`cloak_protocol.py`) |
| gRPC server (Tokio) | ✅ | `0.0.0.0:<port>` with all service traits |
| Multi-bootstrap (`--bootstrap` repeatable) | ✅ | `Vec<String>` in `main.rs` |
| `--mine-atk` persistent storage flag | ✅ | Switches to Sled backend |
| SIGINT graceful shutdown | ✅ | Drops all channels, stops server |
| Config TOML with fallback defaults | ✅ | `[9000]` warning when missing |
| Per-node isolated data directories | ✅ | `data_<node_id>/` |

---

## DHT (Kademlia)

| Feature | Status | Notes |
|---------|--------|-------|
| 256-bucket routing table (K=20) | ✅ | XOR distance, k-bucket split |
| `FindNode` RPC (with self-inclusion) | ✅ | Returns closest peers + self |
| `FindValue` RPC | ✅ | Returns value or closest peers |
| `StoreValueNetwork` RPC | ✅ | Local + peer replication |
| Bootstrap peer discovery (FindNode for self) | ✅ | Authority fallback when table empty |
| DHT maintenance (TTL eviction, 60s interval) | ✅ | `start_maintenance()` |
| Volatile in-memory backend (ZeroizeWrapper) | ✅ | Zeroes on eviction |
| Persistent Sled backend (`--mine-atk`) | ✅ | Survives restart |
| Peer reputation scoring | ⚠️ | Field exists; scoring not yet weighted |
| K-bucket refresh / node health probing | 📅 | Phase 3 |
| Proof-of-work for DHT write access | 📅 | Phase 5 (Sybil resistance) |

---

## SOCKS5 Proxy & Service Hosting

| Feature | Status | Notes |
|---------|--------|-------|
| RFC 1928 SOCKS5 server | ✅ | `127.0.0.1:<gRPC+5049>` |
| Domain-type ATYP (`0x03`) | ✅ | `.cloak` domain resolution |
| Local service bridging (`hosted_sites` map) | ✅ | In-memory address→port map |
| Remote service via TunnelStream | ✅ | DHT lookup → gRPC relay |
| `HostSite` gRPC handler | ✅ | Registers address in bridge |
| `host-static` auto-Python HTTP server | ✅ | `python3 -m http.server` |
| `host` CLI (bridge existing port) | ✅ | |
| `browse` CLI (fetch via SOCKS5) | ✅ | |
| `site-init` scaffold | ✅ | Dark-themed HTML template |
| IPv4 address-type SOCKS5 (`0x01`) | 📅 | Phase 4 |

---

## Onion Circuits

| Feature | Status | Notes |
|---------|--------|-------|
| `CircuitManager` pool | ✅ | 5 circuits pre-built at startup |
| 3-hop minimum enforcement | ✅ | `default_hops = 3` |
| Guard node pinning | ✅ | Long-term guard node selection |
| Circuit UUID tracking | ✅ | UUIDs in `Envelope.circuit_id` |
| `GetCircuits` telemetry | ✅ | Returns circuit list with status |
| Telescoping circuit construction (crypto) | 🔷 | Scaffold in `circuit.rs`; full crypto in Phase 5 |
| Circuit key rotation | 🔷 | Config fields exist; implementation Phase 4 |
| Tor-compatible guard selection algorithm | 📅 | Phase 5 |

---

## Cryptography

| Feature | Status | Notes |
|---------|--------|-------|
| Ed25519 sign / verify | ✅ | `ed25519-dalek` |
| X25519 key exchange | ✅ | `x25519-dalek`, used in Noise handshake |
| ChaCha20-Poly1305 AEAD | ✅ | Per-hop session encryption |
| Kyber-768 KEM (PQ hybrid) | ✅ | `pq_hybrid_enabled` config flag |
| HKDF-SHA256 key derivation | ✅ | Session keys from shared secrets |
| Noise_XX handshake | ✅ | `noise.rs` |
| Zeroize on drop (all key material) | ✅ | `zeroize` crate |
| Fixed-size cell padding (256/1024B) | ✅ | `cell_size_bytes` config |
| Cover traffic (dummy cells) | ✅ | `cover_flow_pps` config |
| Timing jitter | ✅ | `jitter_max_ms` config |
| Double Ratchet (per-message PFS) | 🔷 | `ratchet.rs` scaffold; Phase 4 |
| Sealed sender (Signal-style) | 📅 | Phase 5 |
| ZK proofs (zk-SNARKs) | 📅 | Phase 5 |
| Capability token Ed25519 signing | 🔷 | Field exists; signing Phase 4.3 |

---

## Messaging

| Feature | Status | Notes |
|---------|--------|-------|
| `ChatStream` gRPC (bidirectional) | ✅ | Echo relay |
| `FileTransfer` gRPC (client streaming) | ✅ | 64KB chunks |
| `chat-send` CLI | ✅ | |
| `listen-chat` CLI | ✅ | Blocking listener |
| `view-chat-history` CLI | ✅ | Message history table |
| `file-send` CLI | ✅ | |
| `file-recv` CLI | ✅ | |
| `list-files` CLI | ✅ | |
| Recipient address routing (not just echo) | 📅 | Phase 4 |
| Persistent encrypted mailbox | 📅 | Phase 4 |
| Group messaging | 📅 | Phase 5 |

---

## Decentralized Storage (AetherStore)

| Feature | Status | Notes |
|---------|--------|-------|
| `StoreShard` gRPC | ✅ | Stores shard in DHT (24h TTL) |
| `RetrieveShard` gRPC | ✅ | Retrieves shard from local DHT |
| `Gossip` manifest broadcast | ✅ | FileManifest propagated to all peers |
| SHA-256 content addressing | ✅ | DHT key = SHA256(`hash:shard_index`) |
| `storage upload` CLI | ✅ | |
| `storage download` CLI | ✅ | |
| Single-shard files (no erasure coding) | ✅ | `data_shards=1, parity_shards=0` |
| Cross-node shard retrieval | ⚠️ | Gossip manifest propagates; shard must exist on queried node |
| Reed-Solomon erasure coding (N+M shards) | 📅 | Phase 6 |
| Merkle proof inclusion verification | 🔷 | Fields in proto; Phase 4 |

---

## ATK Wallet

| Feature | Status | Notes |
|---------|--------|-------|
| `BroadcastTx` gRPC | ✅ | Receives and gossips transactions |
| `wallet transfer` CLI | ✅ | Builds and broadcasts mock tx |
| `wallet balance` CLI | ⚠️ | Returns mock 100.0 ATK balance |
| Transaction gossip propagation | ✅ | Real fan-out via `Gossip` RPC |
| Ed25519 tx signing | 🔷 | Scaffold; Phase 4 |
| UTXO input validation | 📅 | Phase 4.1 |
| UTXO set persistence (sled) | 📅 | Phase 4.2 |
| Block formation | 📅 | Phase 4.3 |
| Proof-of-relay reward distribution | 📅 | Phase 4.4 |
| Governance voting | 📅 | Phase 6 |

---

## Capability Tokens & Access Control

| Feature | Status | Notes |
|---------|--------|-------|
| Token issuance (`auth-issue` CLI) | ✅ | 128-bit random token_id, Pydantic model |
| Token verification (expiry + scope) | ✅ | `CapabilityService.Verify` gRPC |
| `auth-verify` CLI | ✅ | Displays token fields + validity status |
| `auth-issue` scope + TTL options | ✅ | read / write / admin, configurable TTL |
| Ed25519 signature on tokens | 🔷 | Field in proto; Phase 4.3 |
| `AUTH_POLICY.CAPABILITY_REQUIRED` enforcement | ⚠️ | Intro point checks token presence |
| `AUTH_POLICY.ZK_GATED` | 📅 | Phase 5 |

---

## Telemetry & Monitoring

| Feature | Status | Notes |
|---------|--------|-------|
| `GetMetrics` gRPC | ✅ | Uptime, circuits, DHT entries, bytes relayed |
| `GetHealth` gRPC | ✅ | Liveness check |
| `GetCircuits` gRPC | ✅ | Active circuit list |
| `GetRelays` gRPC | ✅ | Routing table peer list |
| `StreamMetrics` gRPC (server streaming) | 🔷 | Stub; pushes every 2s in Phase 2 |
| `status` CLI | ✅ | Node metrics table |
| `dash` CLI | ✅ | Live terminal dashboard |
| `ping` CLI | ✅ | Keepalive + latency |
| `circuits` CLI | ✅ | Active circuit table |
| `relays` CLI | ✅ | Peer table |
| `stream-metrics` CLI | 🔷 | Awaits `StreamMetrics` full impl |

---

## Node Lifecycle (CLI)

| Feature | Status | Notes |
|---------|--------|-------|
| `node-start` CLI | ✅ | Starts background node via `NodeManager` |
| `node-stop` CLI | ✅ | Stops node by port via PID file |
| `interactive` REPL | ✅ | Full command menu |
| `address` CLI | ✅ | Shows .cloak address |
| Python `NodeManager` (subprocess + PID files) | ✅ | `/tmp/cloakmesh/node-<port>.pid` |

---

## SDK & Browser Integration

| Feature | Status | Notes |
|---------|--------|-------|
| TypeScript SDK skeleton (`sdk/`) | 🔷 | Directory structure exists |
| WASM Rust bindings (`wasm/`) | 🔷 | `wasm-bindgen` scaffold |
| Browser SOCKS5 routing | 📅 | Phase 6 |
| CloakBrowser extension | 📅 | Phase 6 |

---

## Build & Developer Tooling

| Feature | Status | Notes |
|---------|--------|-------|
| `just build-all` | ✅ | Builds Rust + Python + TypeScript |
| `just proto-gen` | ✅ | Regenerates all gRPC bindings |
| `just test` | ✅ | Runs all test suites |
| `just lint` | ✅ | Runs all linters |
| `cargo build` (core) | ✅ | |
| `poetry install` (orchestrator) | ✅ | |
| Conventional commits enforced | ✅ | Via PR template |
