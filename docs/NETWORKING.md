# CloakMesh Networking Reference

Deep-dive technical documentation for the CloakMesh network layer: Kademlia DHT, onion routing circuits, SOCKS5 proxy, gossip system, and the introduction/rendezvous protocol.

---

## Table of Contents

- [Node Identity](#node-identity)
  - [Ed25519 Keypair Generation](#ed25519-keypair-generation)
  - [Persistent Identity per Data Directory](#persistent-identity-per-data-directory)
- [Port Assignment](#port-assignment)
- [Kademlia DHT](#kademlia-dht)
  - [Key Space and Node IDs](#key-space-and-node-ids)
  - [XOR Distance Metric](#xor-distance-metric)
  - [K-Buckets](#k-buckets)
  - [PeerInfo and NodeFlags](#peerinfo-and-nodeflags)
  - [DHT RPCs](#dht-rpcs)
  - [Bootstrap Process](#bootstrap-process)
  - [DHT Maintenance](#dht-maintenance)
- [Routing Table](#routing-table)
- [Onion Circuits](#onion-circuits)
  - [Circuit Construction](#circuit-construction)
  - [Guard Node Selection](#guard-node-selection)
  - [Telescoping Construction](#telescoping-construction)
  - [Circuit Lifecycle](#circuit-lifecycle)
  - [CircuitManager Pool](#circuitmanager-pool)
- [SOCKS5 Proxy](#socks5-proxy)
  - [RFC 1928 Implementation](#rfc-1928-implementation)
  - [.cloak Address Resolution](#cloak-address-resolution)
  - [TunnelStream Relay](#tunnelstream-relay)
- [Gossip Protocol](#gossip-protocol)
  - [Fan-out Strategy](#fan-out-strategy)
  - [Deduplication](#deduplication)
- [Introduction Protocol](#introduction-protocol)
  - [CloakDescriptor](#cloakdescriptor)
  - [Introduce1/Introduce2/RendezvousAck Flow](#introduce1introduce2rendezvousack-flow)
- [Multi-Node Local Setup](#multi-node-local-setup)
- [Network Configuration Reference](#network-configuration-reference)
- [Proto Reference](#proto-reference)

---

## Node Identity

### Ed25519 Keypair Generation

Each CloakMesh node has a unique **Ed25519 identity keypair** generated on first startup and stored persistently. The keypair serves as:
- The node's cryptographic identity for handshakes and signatures
- The derivation source for the node's `.cloak` address
- The derivation source for the node's ATK wallet address
- The DHT node ID (Kademlia key = `Ed25519 pubkey bytes`)

```rust
// core/src/main.rs — Identity initialization
let identity = Ed25519KeyPair::load_or_generate(&config.crypto.identity_key_path)?;
let pubkey = identity.public_key_bytes();   // [u8; 32]
let cloak_address = derive_address(&pubkey); // e.g., "ae3kqhd4...cloak"
```

**`load_or_generate` behavior:**
- If `identity_key_path` exists: loads the 32-byte secret seed.
- If not: generates a new keypair via OS CSPRNG (`OsRng`) and saves it with Unix permissions `0o600`.

### Persistent Identity per Data Directory

To support multiple nodes on the same machine (local testing), each node gets its own isolated data directory:

```rust
// core/src/main.rs
config.data_dir = format!("data_{}", config.node_id).into();
config.crypto.identity_key_path = format!("{}/identity.pem", config.data_dir.display()).into();
```

**Directory layout:**
```
data_node-1/
├── identity.pem     # 32-byte Ed25519 secret seed (mode 0o600)
└── sled_db/         # Persistent storage (only with --mine-atk)

data_node-2/
├── identity.pem
└── sled_db/
```

---

## Port Assignment

CloakMesh nodes use a fixed port-offset scheme to automatically assign the SOCKS5 proxy port:

```
SOCKS5_PORT = gRPC_PORT + 5049
```

| gRPC Port | SOCKS5 Port | Node Instance |
|---|---|---|
| `4001` (default) | `9050` | Compatible with Tor default |
| `4002` | `9051` | Multi-node 2nd instance |
| `4003` | `9052` | Multi-node 3rd instance |
| `4004` | `9053` | Multi-node 4th instance |

The offset of **5049** was chosen to make the default node (port 4001) use port 9050, which is Tor's default SOCKS5 port, enabling drop-in compatibility with Tor-aware applications.

```rust
// core/src/main.rs
let proxy_port = config.listen_port + 5049; // 4001 -> 9050
tokio::spawn(async move {
    node_clone.start_proxy(proxy_port).await
});
```

---

## Kademlia DHT

The DHT is implemented in `core/src/routing/dht.rs` and provides:
- Peer discovery via `FIND_NODE`
- Key-value storage for descriptors and file shards via `STORE`/`FIND_VALUE`
- Gossip peer list for message propagation

### Key Space and Node IDs

All node IDs and storage keys are **256-bit (32-byte)** values, matching the Ed25519 public key size:

```rust
// core/src/routing/dht.rs
pub const KEY_LEN: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Zeroize, ZeroizeOnDrop)]
pub struct DhtKey(pub [u8; KEY_LEN]);
```

**Node ID derivation:** `DhtKey(Ed25519_pubkey_bytes[32])`

**Storage key derivation:** `DhtKey(SHA256(content_key_string))`

### XOR Distance Metric

Kademlia uses XOR distance to define the keyspace topology:

```rust
// core/src/routing/dht.rs
pub fn distance(&self, other: &DhtKey) -> DhtKey {
    let mut dist = [0u8; KEY_LEN];
    for (i, byte) in dist.iter_mut().enumerate() {
        *byte = self.0[i] ^ other.0[i];
    }
    DhtKey(dist)
}
```

**XOR properties:**
- `distance(A, A) = 0` — A node is distance 0 from itself.
- `distance(A, B) = distance(B, A)` — Symmetric.
- `distance(A, C) ≤ distance(A, B) + distance(B, C)` — Triangle inequality.
- The k closest nodes to a key are stored in the same k-bucket region.

### K-Buckets

The routing table contains up to **256 k-buckets**, one per bit of the 256-bit keyspace:

```rust
// core/src/routing/dht.rs
pub const K_VALUE: usize = 20;  // Maximum peers per bucket

pub struct KBucket {
    pub peers: Vec<PeerInfo>,
    pub last_updated: tokio::time::Instant,
}
```

**Bucket index determination:**
```rust
pub fn bucket_index(&self, local_id: &DhtKey) -> Option<usize> {
    let dist = self.distance(local_id);
    // Find the index of the highest set bit in the XOR distance
    for (byte_idx, &byte) in dist.0.iter().enumerate() {
        if byte != 0 {
            let bit_idx = 7 - byte.leading_zeros() as usize;
            return Some((KEY_LEN - 1 - byte_idx) * 8 + bit_idx);
        }
    }
    None // Same ID
}
```

**Bucket fullness policy (Phase 1):** If a bucket is full (`peers.len() == K_VALUE`), new peers are dropped. A full Kademlia implementation would ping the oldest peer and evict it if unresponsive.

**Bucket contents:** Ordered from least-recently-seen (LRU) to most-recently-seen. Existing peers are moved to the most-recently-seen position on contact (`add_peer` updates `last_seen` in place).

### PeerInfo and NodeFlags

```rust
// core/src/routing/dht.rs
pub struct PeerInfo {
    pub id: DhtKey,                    // 32-byte node ID
    pub address: String,               // "IP:port" or Multiaddr
    pub last_seen: tokio::time::Instant,
    pub reputation: u32,               // 0–1000 reputation score
    pub flags: HashSet<NodeFlag>,      // Auto-classified role flags
}

pub enum NodeFlag {
    Guard,      // High-stability entry node; first hop in circuits
    Middle,     // Middle relay hop
    Exit,       // Exit relay; connects to destination .cloak address
    Authority,  // Directory Authority (hardcoded bootstrap peers)
    Stable,     // Has been online consistently (uptime > threshold)
}
```

**Flag auto-classification:** Node flags are assigned based on observed behavior, uptime, and reputation score during DHT maintenance cycles. Nodes with high reputation and stable uptime may be classified as `Guard` nodes, making them preferred first-hop candidates.

### DHT RPCs

Three primary Kademlia RPCs are implemented:

#### FIND_NODE

```protobuf
// proto/v1/cloakmesh.proto
message FindNodeRequest  { string target_id = 1; }  // Base32 encoded target ID
message FindNodeResponse { repeated NodeContact nodes = 1; }  // K closest nodes

message NodeContact {
  string node_id = 1;   // Base32 encoded public key
  string address = 2;   // IP:port
}
```

**Behavior:** Returns up to `K_VALUE` (20) nodes closest to `target_id` in XOR distance from the local routing table.

#### FIND_VALUE

```protobuf
// proto/v1/cloakmesh.proto
message FindValueRequest  { string target_key = 1; }
message FindValueResponse {
  oneof result {
    bytes            value   = 1;  // Value if found locally
    FindNodeResponse closest = 2;  // K closest nodes if not found
  }
}
```

**Behavior:** Returns the stored value if held locally, otherwise returns the k closest nodes to guide iterative lookup.

#### STORE (StoreValueNetwork)

```protobuf
// proto/v1/cloakmesh.proto
message StoreValueRequest  { string key = 1; bytes value = 2; uint64 ttl_seconds = 3; }
message StoreValueResponse { bool success = 1; }
```

**Behavior:** Stores a key-value pair with a TTL. Called on the k closest nodes to the target key.

### Bootstrap Process

```rust
// core/src/node.rs — Bootstrap spawn
tokio::spawn(async move {
    if let Err(e) = dht_clone.bootstrap().await {
        tracing::error!("DHT bootstrap failed: {}", e);
    }
});
```

**Bootstrap sequence:**
1. **Ping authority nodes:** Send `KeepAlive(Ping{nonce})` to each configured `bootstrap_peers`.
2. **FIND_NODE for self:** Send `FindNode(target_id=my_node_id)` to each bootstrap peer to discover the nodes closest to our own ID.
3. **Populate routing table:** Add all returned `NodeContact` entries to the appropriate k-buckets.
4. **FIND_NODE for random IDs:** Query for random IDs in each bucket region to populate those buckets with live peers.
5. **Start maintenance timer:** Begin periodic bucket refresh and peer liveness checks.

**Bootstrap peers in TOML config:**
```toml
# configs/default.toml
bootstrap_peers = ["192.168.1.10:4001", "192.168.1.11:4001"]
```

**Multiple bootstrap peers via CLI:**
```bash
./target/release/cloakmesh \
  --bootstrap 192.168.1.10:4001 \
  --bootstrap 192.168.1.11:4001 \
  --bootstrap 192.168.1.12:4001
```

### DHT Maintenance

The DHT runs a background maintenance task (`DhtNode::start_maintenance()`) that periodically:

1. **Bucket refresh:** For each bucket not updated in the last hour, send `FIND_NODE` for a random ID in that bucket's region.
2. **Peer liveness check:** Send `KeepAlive` pings to peers in each bucket; remove unresponsive peers.
3. **Value republication:** Republish stored values before their TTL expires (interval: `republish_interval_secs = 1800`).
4. **Storage cleanup:** Call `StorageBackend::cleanup()` to evict expired entries.

---

## Routing Table

```rust
// core/src/routing/dht.rs
pub struct RoutingTable {
    pub local_id: DhtKey,       // This node's own ID
    buckets: Vec<KBucket>,      // 256 k-buckets
}
```

**Table size:** 256 buckets × 20 peers = up to **5,120 known peers** in steady state.

**Lookup algorithm** (`find_closest_peers`):
1. Compute XOR distance from target to all known peers.
2. Return the K (20) peers with smallest XOR distance.
3. These are the peers to contact next during iterative lookup.

**Iterative FIND_NODE:**
```
Round 1: Query α=3 closest known peers → get up to 3×K new candidates
Round 2: Query α closest from new candidates not yet queried
Round 3: If no new closer nodes found: query all unreached nodes in current K-closest
Converge: When K closest nodes have all been queried
```

---

## Onion Circuits

### Circuit Construction

Circuits are managed by `CircuitManager` (`core/src/routing/circuit.rs`). Each circuit consists of `n` hops (default: 3):

```
Client → [Guard] → [Middle] → [Exit] → Destination .cloak Address
          Hop 0      Hop 1      Hop 2
```

**Hop roles:**
| Role | Position | Knows About |
|---|---|---|
| Guard | First hop | Client IP, next hop (Middle) |
| Middle | Middle hop(s) | Guard (previous), Exit (next) |
| Exit | Last hop | Middle (previous), destination |

### Guard Node Selection

```rust
// core/src/routing/circuit.rs
pub async fn select_guard(&self, candidates: &[(String, String)]) -> CloakResult<(String, String)> {
    let mut guards = self.guards.write().await;
    if guards.is_empty() && !candidates.is_empty() {
        // Select up to 3 long-term guards from candidates (random sample)
        guards.extend(candidates.choose_multiple(&mut rng, 3).cloned());
        info!("[202] Long-term guard nodes selected and pinned.");
    }
    // Randomly choose one of the pinned guards
    guards.choose(&mut rand::thread_rng()).cloned()
}
```

**Guard pinning:** A small set of 3 guard nodes is selected once and pinned for the duration of the node's session. This prevents an adversary from discovering your guard by observing circuit rebuilds.

**Guard selection criteria:** Guards are chosen from `NodeFlag::Guard` candidates — nodes with high reputation and `NodeFlag::Stable` classification.

### Telescoping Construction

```rust
// core/src/routing/circuit.rs
pub async fn build_circuit(&self, hops_count: usize, candidates: Vec<(String, String)>) -> CloakResult<String> {
    // 1. First hop: Long-Term Guard Selection
    let (guard_id, guard_addr) = self.select_guard(&candidates).await?;
    let guard_session = BidirectionalSession::generate_mock();
    hops.push(CircuitHop { peer_id: guard_id, address: guard_addr, session: Arc::new(RwLock::new(guard_session)) });

    // 2. Telescoping: extend through each subsequent hop
    for i in 1..hops_count {
        let (peer_id, address) = &candidates[i % candidates.len()];
        let session = BidirectionalSession::generate_mock();
        hops.push(CircuitHop { peer_id: peer_id.clone(), address: address.clone(), session: Arc::new(RwLock::new(session)) });
    }

    let circuit = Arc::new(Circuit::new(hops));
    self.circuits.write().await.insert(circuit.id.clone(), circuit.clone());
    info!("[202] Three-hop circuit established via telescoping construction, id: {}", circuit.id);
}
```

**Telescoping construction** means each hop is established sequentially through the previous hop:
1. Client ↔ Guard: Establish Noise handshake, get session key K_guard
2. Client → (via Guard) ↔ Middle: Establish Noise handshake through Guard, get K_middle
3. Client → (via Guard → Middle) ↔ Exit: Establish handshake through both, get K_exit

**Layered encryption:**
```rust
pub async fn encrypt_all_layers(&self, mut payload: Vec<u8>) -> CloakResult<Vec<u8>> {
    for hop in self.hops.iter().rev() {  // Exit → Middle → Guard
        let mut session = hop.session.write().await;
        payload = session.encrypt(&payload, b"circuit-layer")?;
    }
    Ok(payload)  // payload is now: Enc(K_guard, Enc(K_middle, Enc(K_exit, data)))
}
```

Each relay peels one layer:
- Guard decrypts outer K_guard layer, forwards to Middle
- Middle decrypts K_middle layer, forwards to Exit
- Exit decrypts K_exit layer, gets plaintext, forwards to destination

### Circuit Lifecycle

```rust
// core/src/routing/circuit.rs
pub fn is_expired(&self) -> bool {
    self.created_at.elapsed() > Duration::from_secs(3600) // 1 hour
}
```

| Event | Action |
|---|---|
| Circuit created | Added to `CircuitManager.circuits` map |
| Circuit built | `is_active = true` |
| 1 hour elapsed | Marked expired by `is_expired()` |
| `maintain_pool()` runs | Expired circuits removed, new ones built |

### CircuitManager Pool

```rust
// core/src/node.rs — Pool initialization
let circuit_manager = Arc::new(CircuitManager::new(5)); // Pool of 5 circuits
```

The `maintain_pool()` function runs periodically to:
1. Remove expired circuits (`is_expired() == true`)
2. Build new circuits until `pool_size` (5) is reached

**Pre-built circuit pool benefits:**
- Eliminates circuit build latency for first message (10s timeout avoided)
- Allows immediate selection of a random circuit for each new request

---

## SOCKS5 Proxy

### RFC 1928 Implementation

The SOCKS5 proxy listens on `0.0.0.0:<gRPC_PORT + 5049>` (e.g., `9050` for default node) and is started by `CloakNode::start_proxy`:

```rust
// core/src/node.rs
pub async fn start_proxy(self: Arc<Self>, port: u16) -> CloakResult<()> {
    // Pre-build a mock circuit for demo
    self.circuit_manager.build_circuit(3, vec![...]).await?;
    self.bridge.clone().start_client_proxy(port).await
}
```

**Handshake sequence (RFC 1928):**

```
Client → Proxy:  \x05\x01\x00           (Version 5, 1 method: No Auth)
Proxy → Client:  \x05\x00               (Version 5, method: No Auth accepted)

Client → Proxy:  \x05\x01\x00\x03       (Version 5, CONNECT, Reserved, ATYP=Domain)
                 \x1a                    (Domain name length = 26)
                 ae3kqhd4....cloak       (Domain name bytes)
                 \x00\x50               (Port = 80)

Proxy → Client:  \x05\x00\x00\x01...   (Success: Version 5, Status=0, ...)
```

**ATYP=0x03 (Domain Name):** The SOCKS5 proxy uses domain-name addressing rather than IP addresses, allowing `.cloak` addresses to be passed directly through the SOCKS5 tunnel without DNS lookup.

### .cloak Address Resolution

When a CONNECT request arrives with a `.cloak` domain:

1. **Parse address:** Extract the `.cloak` address from ATYP=0x03 domain field.
2. **DHT lookup:** Call `DhtNode::find_value(SHA256(cloak_address))` to get the `CloakDescriptor`.
3. **Extract introduction point:** Get `intro_points[0].address` from the descriptor.
4. **Select circuit:** Use `CircuitManager::select_random_circuit()` to pick an active circuit.
5. **Open TunnelStream:** Send `TunnelData{target_address: cloak_address, circuit_id, ...}` through the circuit.
6. **Relay TCP data:** Forward all incoming TCP bytes as `TunnelData.chunk` frames.

### TunnelStream Relay

```protobuf
// proto/v1/cloakmesh.proto
message TunnelData {
  string circuit_id     = 1;
  string target_address = 2;  // Destination .cloak address
  repeated string path  = 5;  // [next_hop_ip:port, ...] — decremented at each hop
  bytes  chunk          = 3;  // Raw TCP payload
  bool   is_eof         = 4;  // TCP FIN signal
}
```

**`path` field:** Contains the remaining hop addresses. Each relay removes its own address from the front and forwards to `path[0]`. When `path` is empty, the Exit relay connects to `target_address`.

---

## Gossip Protocol

### Fan-out Strategy

When a node receives a `Gossip(GossipMessage)` RPC:

```rust
// core/src/node.rs
async fn gossip(&self, request: Request<GossipMessage>) -> Result<Response<TransferAck>, Status> {
    let msg = request.into_inner();

    // 1. Deduplication check
    {
        let mut seen = self.seen_gossip.write().await;
        if seen.contains(&msg.message_id) {
            return Ok(Response::new(TransferAck { success: true, message: "already seen".into() }));
        }
        seen.insert(msg.message_id.clone());
    }

    // 2. Process payload (update UTXO set or cache manifest)

    // 3. Fan-out to DHT peers (spawn-and-forget)
    let peers = self.dht.get_all_peers().await;
    for peer_addr in peers {
        let msg_clone = msg.clone();
        tokio::spawn(async move {
            // Connect to peer and forward gossip
            let _ = peer_client.gossip(msg_clone).await;
        });
    }

    Ok(Response::new(TransferAck { success: true, message: "gossiped".into() }))
}
```

**Fan-out scope:** All peers known to the DHT routing table. In a fully populated routing table with `K_VALUE=20` and 256 buckets, this could be up to ~5,000 peers, but typically a much smaller subset of actively connected peers.

### Deduplication

```rust
pub struct CloakNode {
    seen_gossip: Arc<tokio::sync::RwLock<HashSet<String>>>,
}
```

- `message_id` is inserted **before** fan-out to prevent re-processing if a peer echoes the message back.
- The set is held in RAM — it is cleared on node restart (intentional for decentralized design).
- No TTL on `seen_gossip` entries in Phase 1; Phase 2 will add a time-bounded sliding window.

---

## Introduction Protocol

The introduction protocol enables a client to establish a rendezvous circuit to a `.cloak` service without either party knowing the other's IP address.

### CloakDescriptor

```protobuf
// proto/v1/cloak_service.proto
message CloakDescriptor {
  string cloak_address        = 1;  // The .cloak address of the service
  repeated IntroductionPoint  intro_points = 2;
  bytes  identity_pubkey      = 3;  // Ed25519 32B
  bytes  hybrid_pq_pubkey     = 4;  // X25519 + Kyber-768 combined key
  bytes  signature            = 5;  // Ed25519 over descriptor body
  bytes  merkle_root          = 6;  // SHA-256 Merkle root of all fields
  google.protobuf.Timestamp issued_at = 7;
  uint64 nonce                = 8;  // Anti-replay nonce
  AuthPolicy auth_policy      = 9;  // PUBLIC | CAPABILITY_REQUIRED | ZK_GATED
  uint32 version              = 10;
}

message IntroductionPoint {
  string peer_id  = 1;  // Introduction node ID
  string address  = 2;  // Introduction node host:port
  bytes  auth_key = 3;  // Short-term authentication key
}

enum AuthPolicy {
  PUBLIC               = 0;  // Open access
  CAPABILITY_REQUIRED  = 1;  // Requires CapabilityToken
  ZK_GATED             = 2;  // Requires ZK proof + ATK payment (roadmap)
}
```

### Introduce1/Introduce2/RendezvousAck Flow

```
Client                  Intro Point              Service
  │                          │                      │
  │── Introduce1 ───────────►│                      │
  │   (encrypted payload     │                      │
  │    for Service:          │── forward ──────────►│
  │    RP addr + cookie +    │                      │
  │    capability)           │                      │
  │                          │◄── IntroduceAck ─────│
  │                          │   (accepted=true)    │
  │◄── IntroduceAck ─────────│                      │
  │                          │                      │
  │                          │   Service ──────────►│ Rendezvous Point
  │                          │   Introduce2:         │   (RP)
  │                          │   {cookie, session_key│
  │                          │    material, circuit_id}
  │                          │                      │
  │◄─── RendezvousAck ──────────────────────────────│
  │     (matched=true,                               │
  │      circuit_id)                                 │
  │                                                  │
  │ ←══════════════ Bidirectional circuit ══════════►│
```

```protobuf
// proto/v1/cloak_service.proto
message Introduce1 {
  string intro_point_id    = 1;
  bytes  encrypted_payload = 2;  // Encrypted for Service: RP addr + cookie + capability
  bytes  capability_token  = 3;  // Optional CapabilityToken for CAPABILITY_REQUIRED services
}

message Introduce2 {
  bytes  cookie             = 1;  // 32-byte random rendezvous cookie
  bytes  session_key_material = 2;
  string circuit_id         = 3;
}

message RendezvousAck {
  bool   matched    = 1;
  string circuit_id = 2;
}

service CloakService {
  rpc Introduce(Introduce1)         returns (IntroduceAck);
  rpc ConnectRendezvous(Introduce2) returns (RendezvousAck);
}
```

---

## Multi-Node Local Setup

To run a multi-node local test network:

```bash
# Terminal 1: Start bootstrap authority node (node-1)
./target/release/cloakmesh \
  --config configs/default.toml \
  --port 4001 \
  --id node-1

# Terminal 2: Start node-2, bootstrapping from node-1
./target/release/cloakmesh \
  --config configs/default.toml \
  --port 4002 \
  --id node-2 \
  --bootstrap 127.0.0.1:4001

# Terminal 3: Start node-3, bootstrapping from both
./target/release/cloakmesh \
  --config configs/default.toml \
  --port 4003 \
  --id node-3 \
  --bootstrap 127.0.0.1:4001 \
  --bootstrap 127.0.0.1:4002

# Python orchestrator targeting node-2
CLOAK_GRPC_PORT=4002 poetry run cloakcli status
```

**Multiple `--bootstrap` flags** are fully supported. Each value is appended to `config.bootstrap_peers`:

```rust
// core/src/main.rs
for peer in cli.bootstrap {
    if !peer.is_empty() {
        config.bootstrap_peers.push(peer);
    }
}
```

---

## Network Configuration Reference

Full `NodeConfig` with all networking-relevant fields:

```toml
# configs/default.toml

node_id = "node-1"
listen_port = 4001
bootstrap_peers = ["127.0.0.1:4002"]
data_dir = "data"
log_level = "info"

[circuit]
min_hops = 2         # Security floor (anonymity guarantee)
max_hops = 5         # Latency ceiling
default_hops = 3     # Standard 3-hop circuit
build_timeout_ms = 10000  # 10s timeout for circuit building
pool_size = 3        # Warm circuits maintained in pool

[traffic]
cell_size_bytes = 256    # Fixed cell size: 256 or 1024
cover_flow_pps = 1       # Cover traffic packets per second
jitter_max_ms = 50       # Max random send jitter
padding_enabled = true
key_rotation_cells = 10000  # Rotate session keys every 10,000 cells
key_rotation_secs = 300     # Or every 5 minutes (whichever first)

[crypto]
identity_key_path = "data/identity.key"
nonce_window_secs = 300       # Replay protection window
descriptor_max_age_secs = 3600  # Max descriptor age
pq_hybrid_enabled = true      # Enable Kyber-768 hybrid PQ key exchange

[dht]
k_bucket_size = 20       # K-bucket capacity (Kademlia K value)
alpha = 3                # Parallel lookup concurrency (Kademlia α)
descriptor_ttl_secs = 3600      # 1 hour descriptor TTL
republish_interval_secs = 1800  # Republish every 30 minutes

[telemetry]
prometheus_enabled = true
prometheus_port = 9090
metrics_interval_secs = 15
```

---

## Proto Reference

```protobuf
// ── Peer and DHT ─────────────────────────────────────────────
message NodeContact {
  string node_id = 1;   // Base32 encoded public key (DHT Node ID)
  string address = 2;   // IP:port or Multiaddr
}

message FindNodeRequest  { string target_id = 1; }
message FindNodeResponse { repeated NodeContact nodes = 1; }

message FindValueRequest  { string target_key = 1; }
message FindValueResponse {
  oneof result {
    bytes            value   = 1;
    FindNodeResponse closest = 2;
  }
}

message StoreValueRequest  { string key = 1; bytes value = 2; uint64 ttl_seconds = 3; }
message StoreValueResponse { bool success = 1; }

// ── Handshake ─────────────────────────────────────────────────
message HandshakeInit {
  bytes ephemeral_pubkey = 1;  // X25519 ephemeral key
  bytes kyber_kem_ct     = 2;  // Kyber-768 ciphertext (PQ hybrid)
  bytes payload          = 3;  // Encrypted identity + nonce
  uint32 version         = 4;
}

message HandshakeResponse {
  bytes ephemeral_pubkey = 1;
  bytes kyber_kem_ct     = 2;
  bytes payload          = 3;
}

// ── Tunnel ────────────────────────────────────────────────────
message TunnelData {
  string circuit_id     = 1;
  string target_address = 2;
  repeated string path  = 5;
  bytes  chunk          = 3;
  bool   is_eof         = 4;
}

// ── Introduction ──────────────────────────────────────────────
message Introduce1    { string intro_point_id = 1; bytes encrypted_payload = 2; bytes capability_token = 3; }
message Introduce2    { bytes cookie = 1; bytes session_key_material = 2; string circuit_id = 3; }
message IntroduceAck  { bool accepted = 1; }
message RendezvousAck { bool matched = 1; string circuit_id = 2; }

// ── Service RPCs ──────────────────────────────────────────────
service CloakMeshNode {
  rpc Handshake(HandshakeInit)       returns (HandshakeResponse);
  rpc SendEnvelope(Envelope)         returns (Envelope);
  rpc KeepAlive(Ping)               returns (Pong);
  rpc TunnelStream(stream TunnelData) returns (stream TunnelData);
  rpc FindNode(FindNodeRequest)      returns (FindNodeResponse);
  rpc FindValue(FindValueRequest)    returns (FindValueResponse);
  rpc StoreValueNetwork(StoreValueRequest) returns (StoreValueResponse);
}

service CloakService {
  rpc PublishDescriptor(CloakDescriptor) returns (PublishAck);
  rpc FetchDescriptor(DescriptorRequest) returns (CloakDescriptor);
  rpc Introduce(Introduce1)              returns (IntroduceAck);
  rpc ConnectRendezvous(Introduce2)      returns (RendezvousAck);
  rpc HostSite(HostRequest)              returns (HostAck);
}
```
