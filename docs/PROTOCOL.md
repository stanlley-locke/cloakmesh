# The .cloak Protocol Specification

**Version:** 1  
**Status:** Phase 1 Complete, Phase 2 Active  
**Last Updated:** 2026-05

---

## Overview

The `.cloak` protocol is the application-layer protocol running over CloakMesh's onion-routed gRPC transport. It defines how nodes discover each other, how hidden services are published and reached, how messages are routed, and how all parties authenticate without revealing identities.

A `.cloak` address is a **self-authenticating identifier** — it encodes the service's Ed25519 public key, so any client can cryptographically verify they are speaking to the correct service without a CA or PKI.

---

## Address Format

```
<address>.cloak

Payload = version_byte[1] || ed25519_pubkey[32] || checksum[4]   = 37 bytes
Encoded = lowercase_base32_no_padding(payload)                   = 59 chars
Full    = encoded + ".cloak"                                      ≈ 65 chars
```

| Field | Bytes | Description |
|-------|-------|-------------|
| version | 1 | Always `0x01` (current) |
| pubkey | 32 | Ed25519 public key (service identity) |
| checksum | 4 | `SHA256(SHA256(0x01 ‖ pubkey))[0:4]` |

**Example:**
```
aedqpcdbqryj2lbj66mcup6relejktkzcp2kzt5fdz72tgvlpiemavqil5uq.cloak
```

See [CRYPTO.md](./CRYPTO.md) for the full address derivation algorithm.

---

## Service Descriptors (`CloakDescriptor`)

A `CloakDescriptor` is the signed announcement of a hidden service — it tells the network where and how to reach a `.cloak` address.

```proto
// proto/v1/cloak_service.proto
message CloakDescriptor {
    string cloak_address              = 1;   // The .cloak address being announced
    repeated IntroductionPoint intro_points = 2;   // Reachability relays
    bytes  identity_pubkey            = 3;   // Ed25519 public key (32 bytes)
    bytes  hybrid_pq_pubkey           = 4;   // X25519 + Kyber-768 combined public key
    bytes  signature                  = 5;   // Ed25519 signature of descriptor fields
    bytes  merkle_root                = 6;   // SHA-256 Merkle root of all fields
    google.protobuf.Timestamp issued_at = 7;
    uint64 nonce                      = 8;   // Random nonce (prevents replay)
    AuthPolicy auth_policy            = 9;   // PUBLIC | CAPABILITY_REQUIRED | ZK_GATED
    uint32 version                    = 10;  // Descriptor format version
}

message IntroductionPoint {
    string peer_id   = 1;  // Node ID of the introduction relay
    string address   = 2;  // Host:port of the relay (e.g., "127.0.0.1:4001")
    bytes  auth_key  = 3;  // One-time authentication key for this intro point
}

enum AuthPolicy {
    PUBLIC               = 0;  // Anyone can connect
    CAPABILITY_REQUIRED  = 1;  // Must present a valid CapabilityToken
    ZK_GATED             = 2;  // Must present a zero-knowledge proof (roadmap)
}
```

### Descriptor Lifecycle

```
1. Service generates Ed25519 keypair → derives .cloak address
2. Service creates CloakDescriptor:
   - Sets intro_points to currently-reachable relay nodes
   - Signs with identity_key
   - Sets TTL (default: 1 hour)
3. Service calls PublishDescriptor RPC → stored in DHT
   - DHT key: SHA-256(identity_pubkey)
   - Replicated to K=20 closest peers
4. Clients call FetchDescriptor(cloak_address) → get descriptor
5. Client verifies signature using pubkey derived from address
6. Client contacts intro_point and establishes rendezvous
```

### Descriptor Publication (`PublishDescriptor`)

```proto
service CloakService {
    rpc PublishDescriptor (CloakDescriptor) returns (PublishAck);
    rpc FetchDescriptor   (DescriptorRequest) returns (CloakDescriptor);
}
```

The `PublishDescriptor` handler validates:
1. `cloak_address` matches `identity_pubkey` (computed via `derive_address`)
2. Signature is non-empty (Phase 2+: full Ed25519 verification)
3. Encodes as protobuf bytes → stores in DHT under `SHA-256(pubkey)` with 1-hour TTL

---

## Introduction & Rendezvous Protocol

The introduction protocol allows a client to reach a hidden service through an introduction point without either party knowing the other's IP address.

### Phase 1: Introduce

```proto
message Introduce1 {
    string intro_point_id    = 1;  // Which intro point to use
    bytes  encrypted_payload = 2;  // Encrypted: rendezvous cookie + client's ephemeral pubkey
    bytes  capability_token  = 3;  // Required if auth_policy = CAPABILITY_REQUIRED
}

message IntroduceAck {
    bool accepted = 1;
}
```

### Phase 2: Rendezvous

```proto
message Introduce2 {
    bytes  cookie             = 1;  // Rendezvous cookie from Introduce1 response
    bytes  session_key_material = 2;  // Client's contribution to shared session key
    string circuit_id         = 3;  // Circuit ID for the rendezvous connection
}

message RendezvousAck {
    bool   matched    = 1;  // true = rendezvous established
    string circuit_id = 2;  // Circuit to use for subsequent communication
}
```

### Full Sequence

```
Client                   Intro Point (Alpha)        Hidden Service
  │                            │                          │
  │── FindDescriptor() ───────→│                          │
  │←─ CloakDescriptor ─────────│                          │
  │                            │                          │
  │── Introduce1{              │                          │
  │     intro_point_id,        │                          │
  │     encrypted_payload,     │                          │
  │     capability_token} ────→│                          │
  │                            │── Forward to Service ──→│
  │                            │                          │
  │←─ IntroduceAck{true} ──────│                          │
  │                            │                          │
  │── ConnectRendezvous{       │                          │
  │     cookie, key_material,  │                          │
  │     circuit_id} ──────────→│                          │
  │                            │                          │
  │←─ RendezvousAck{           │                          │
  │     matched: true,         │                          │
  │     circuit_id} ───────────│                          │
  │                            │                          │
  │═══════ TunnelStream via circuit_id ═════════════════│
```

---

## DHT Protocol (Kademlia RPCs)

The DHT layer provides the peer discovery and key-value storage backbone.

### FindNode

```proto
rpc FindNode (FindNodeRequest) returns (FindNodeResponse);

message FindNodeRequest  { string target_id = 1; }       // Hex-encoded 32-byte key
message FindNodeResponse { repeated NodeContact nodes = 1; }

message NodeContact {
    string node_id = 1;   // Hex-encoded 32-byte pubkey
    string address = 2;   // "host:port"
}
```

Returns the K=20 closest peers to `target_id` (XOR distance), **always including self**.

### FindValue

```proto
rpc FindValue (FindValueRequest) returns (FindValueResponse);

message FindValueRequest { string target_key = 1; }

message FindValueResponse {
    oneof result {
        bytes          value   = 1;  // Found: raw bytes
        FindNodeResponse closest = 2;  // Not found: K closest peers
    }
}
```

If the value is found locally, returns it directly. Otherwise returns closest peers for iterative lookup.

### StoreValueNetwork

```proto
rpc StoreValueNetwork (StoreValueRequest) returns (StoreValueResponse);

message StoreValueRequest {
    string key         = 1;   // Hex-encoded 32-byte key
    bytes  value       = 2;   // Raw bytes to store
    uint64 ttl_seconds = 3;   // Time-to-live
}

message StoreValueResponse { bool success = 1; }
```

Stores value locally and replicates to K closest peers.

---

## Onion Cell Protocol (`Envelope`)

```proto
message Envelope {
    bytes  circuit_id  = 1;   // 16-byte UUID identifying the circuit
    uint64 sequence    = 2;   // Monotonic counter for replay prevention
    bytes  nonce       = 3;   // 12-byte ChaCha20-Poly1305 nonce
    bytes  ciphertext  = 4;   // Payload, padded to cell_size_bytes
    google.protobuf.Timestamp timestamp = 5;
}
```

Each relay in the circuit strips one layer of encryption (using its per-hop session key) to reveal the next layer or the plaintext. Cell size is fixed to prevent traffic analysis.

**Cell size options:** 256 bytes (default) or 1024 bytes (high-security).

---

## Gossip Protocol

```proto
rpc Gossip (GossipMessage) returns (TransferAck);

message GossipMessage {
    string message_id     = 1;   // SHA-256 content hash (dedup key)
    string sender_address = 2;   // ATK address of originator
    oneof payload {
        Transaction  transaction = 3;
        FileManifest manifest    = 4;
    }
}
```

Gossip is epidemic broadcast with seen-ID deduplication. Every node fans out to all routing-table peers. Convergence in O(log N) rounds.

---

## Chat & File Transfer Protocol

```proto
// Bidirectional chat stream
rpc ChatStream (stream ChatMessage) returns (stream ChatMessage);

message ChatMessage {
    string    sender   = 1;
    string    text     = 2;
    Timestamp sent_at  = 3;
}

// Client-streaming file transfer
rpc FileTransfer (stream FileChunk) returns (TransferAck);

message FileChunk {
    string file_id     = 1;  // Unique session identifier
    string filename    = 2;
    bytes  data        = 3;  // Up to 64KB per chunk
    uint64 chunk_index = 4;
    bool   is_last     = 5;
}
```

---

## Keepalive Protocol

```proto
rpc KeepAlive (Ping) returns (Pong);

message Ping { uint64 nonce = 1; Timestamp sent_at = 2; }
message Pong { uint64 nonce = 1; Timestamp sent_at = 2; string node_id = 3; }
```

`node_id` in the `Pong` is the hex-encoded Ed25519 public key of the responding node. Used by DHT bootstrap to identify peers.

---

## Access Control Levels

| `AuthPolicy` | Description | Token Required |
|--------------|-------------|----------------|
| `PUBLIC` | Anyone can connect | No |
| `CAPABILITY_REQUIRED` | Must present valid `CapabilityToken` | Yes — checked at intro point |
| `ZK_GATED` | Must present zero-knowledge proof of ATK balance | Yes — ZK proof (roadmap) |

---

## Error Handling

See [ERROR_CODES.md](./ERROR_CODES.md) for the complete structured error code reference.

gRPC status codes used:
| gRPC Status | Situation |
|-------------|-----------|
| `NOT_FOUND` | Descriptor or shard not in DHT |
| `INVALID_ARGUMENT` | Malformed address, invalid hex, length mismatch |
| `UNAUTHENTICATED` | Expired or missing capability token |
| `PERMISSION_DENIED` | Valid token but insufficient scope |
| `INTERNAL` | DHT storage failure, serialization error |
| `UNAVAILABLE` | Peer unreachable during DHT lookup |

---

## Protocol Version Negotiation

The `version` field in `HandshakeInit` and `CloakDescriptor` carries the protocol version:

| Version | Features |
|---------|---------|
| `1` | Current: Kademlia DHT, SOCKS5, gossip, chat, file transfer, capability tokens |
| `2` (planned) | Double Ratchet, full UTXO validation, erasure coding |
| `3` (planned) | ZK-gated access, sealed sender, group messaging |

Nodes refuse connections from versions they don't support (return `FAILED_PRECONDITION`).
