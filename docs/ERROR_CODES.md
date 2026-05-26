# CloakMesh Structured Error Codes

All CloakMesh errors are structured and logged with a numeric code. This reference documents every code, its meaning, common causes, and resolution.

---

## Code Ranges

| Range | Layer | Domain |
|-------|-------|--------|
| `1xxx` | Node / Config | Node lifecycle, configuration, identity |
| `2xxx` | Network | Transport, gRPC, SOCKS5, TunnelStream |
| `3xxx` | DHT / Routing | Kademlia lookups, peer discovery, descriptors |
| `4xxx` | Crypto | Signature verification, address parsing, key operations |
| `5xxx` | Protocol | Chat, file transfer, gossip, wallet |
| `6xxx` | Storage | DHT key-value store, shard retrieval |
| `7xxx` | Auth / Capability | Token validation, scope, expiry |
| `9xxx` | System / Diagnostic | Configuration warnings, non-fatal notices |

---

## 1xxx — Node & Configuration

| Code | Name | Log Level | Description | Resolution |
|------|------|-----------|-------------|------------|
| `1000` | `NODE_STARTED` | INFO | Node fully initialized and listening | — |
| `1001` | `IDENTITY_LOADED` | INFO | Ed25519 identity loaded from disk | — |
| `1002` | `IDENTITY_GENERATED` | INFO | New Ed25519 keypair generated | Backup `data_<node>/identity.pem` |
| `1003` | `CONFIG_LOADED` | INFO | Config TOML loaded successfully | — |
| `1010` | `CONFIG_INVALID_PORT` | ERROR | Listen port is 0 or > 65535 | Set `listen_port` to a valid port |
| `1011` | `CONFIG_INVALID_HOPS` | ERROR | `default_hops` < 1 or > 10 | Use 3 hops (recommended) |
| `1012` | `CONFIG_INVALID_CELL_SIZE` | ERROR | `cell_size_bytes` not in {256, 1024} | Set to 256 or 1024 |
| `1020` | `DATA_DIR_CREATE_FAILED` | ERROR | Cannot create `data_<node>` directory | Check filesystem permissions |
| `1021` | `IDENTITY_KEY_LOAD_FAILED` | ERROR | Key file exists but cannot be read/parsed | Check file permissions (expect `0o600`) |
| `1022` | `IDENTITY_KEY_SAVE_FAILED` | ERROR | Cannot write new identity key to disk | Check disk space and permissions |
| `1030` | `GRPC_BIND_FAILED` | ERROR | Cannot bind gRPC server on `0.0.0.0:<port>` | Check port is free: `ss -tlnp | grep <port>` |
| `1031` | `PROXY_BIND_FAILED` | ERROR | Cannot bind SOCKS5 on `127.0.0.1:<port+5049>` | Check port conflict |
| `1040` | `NODE_SHUTDOWN` | INFO | Graceful shutdown on SIGINT | — |

---

## 2xxx — Network & Transport

| Code | Name | Log Level | Description | Resolution |
|------|------|-----------|-------------|------------|
| `2000` | `GRPC_CONNECTED` | DEBUG | Outbound gRPC connection established | — |
| `2001` | `GRPC_CONNECT_FAILED` | WARN | Cannot reach peer at `host:port` | Check peer is running; check firewall |
| `2010` | `SOCKS5_SESSION_STARTED` | DEBUG | New SOCKS5 client connected | — |
| `2011` | `SOCKS5_AUTH_FAILED` | WARN | Client sent invalid SOCKS5 auth method | Only `\x05\x01\x00` (no auth) is supported |
| `2012` | `SOCKS5_CONNECT_REFUSED` | WARN | Destination address rejected | Check that `.cloak` address is hosted and descriptor is published |
| `2013` | `SOCKS5_SESSION_FAILED` | ERROR | SOCKS5 session terminated with error | See nested error message |
| `2020` | `TUNNEL_STARTED` | DEBUG | TunnelStream gRPC opened to intro point | — |
| `2021` | `TUNNEL_CONNECT_FAILED` | ERROR | Cannot connect TunnelStream to intro point | Intro point may be offline |
| `2022` | `TUNNEL_EOF` | DEBUG | TunnelStream peer closed connection | Normal termination |
| `2023` | `TUNNEL_RELAY_FAILED` | ERROR | Data relay failed mid-stream | Check target TCP service is running |
| `2030` | `KEEPALIVE_SENT` | DEBUG | KeepAlive ping sent to peer | — |
| `2031` | `KEEPALIVE_TIMEOUT` | WARN | Peer did not respond to ping | Peer may be offline; routing table will age it out |

---

## 3xxx — DHT & Routing

| Code | Name | Log Level | Description | Resolution |
|------|------|-----------|-------------|------------|
| `3000` | `BOOTSTRAP_STARTED` | INFO | `bootstrap()` beginning | — |
| `3001` | `BOOTSTRAP_PEER_ADDED` | INFO | Authority added to routing table | — |
| `3002` | `BOOTSTRAP_PEER_FAILED` | WARN | Cannot reach bootstrap authority | Verify `--bootstrap` address and port |
| `3003` | `PEER_DISCOVERED` | INFO | New peer found via `FindNode` | — |
| `3010` | `ROUTING_TABLE_FULL` | DEBUG | K-bucket full; peer not added | Normal; XOR-distant peers are deprioritized |
| `3011` | `ROUTING_SELF_EXCLUDED` | DEBUG | Skipping self in routing table (XOR = 0) | Normal for single-node test |
| `3020` | `DHT_LOOKUP_STARTED` | DEBUG | `find_value_network` initiated | — |
| `3021` | `DHT_LOOKUP_FOUND_LOCAL` | DEBUG | Value found in local storage | — |
| `3022` | `DHT_LOOKUP_FOUND_PEER` | INFO | Value found at remote peer | — |
| `3023` | `DHT_LOOKUP_NOT_FOUND` | WARN | Value not found in DHT | Ensure file was uploaded or descriptor published |
| `3024` | `DHT_LOOKUP_FALLBACK` | INFO | Routing table empty; querying bootstrap authority | Normal during early bootstrap |
| `3030` | `DESCRIPTOR_PUBLISHED` | INFO | `CloakDescriptor` stored in DHT | — |
| `3031` | `DESCRIPTOR_FETCHED` | INFO | `CloakDescriptor` retrieved from DHT | — |
| `3032` | `DESCRIPTOR_NOT_FOUND` | WARN | Address not found in DHT | Run `host-static` or `dht-publish` first |
| `3033` | `DESCRIPTOR_INVALID` | ERROR | Descriptor signature or address mismatch | Descriptor may be corrupted or forged |

---

## 4xxx — Cryptography

| Code | Name | Log Level | Description | Resolution |
|------|------|-----------|-------------|------------|
| `4000` | `ADDRESS_DERIVED` | DEBUG | `.cloak` address computed from pubkey | — |
| `4001` | `ADDRESS_INVALID_LENGTH` | ERROR | Decoded payload is not 37 bytes | Address is malformed or truncated |
| `4002` | `ADDRESS_INVALID_VERSION` | ERROR | Version byte ≠ `0x01` | Unknown address version; upgrade CloakMesh |
| `4003` | `ADDRESS_CHECKSUM_FAIL` | ERROR | Double-SHA256 checksum mismatch | Typo in address; re-copy from source |
| `4004` | `ADDRESS_INVALID_BASE32` | ERROR | Non-base32 characters in address | Address must be lowercase `a-z2-7` |
| `4010` | `SIGNATURE_VALID` | DEBUG | Ed25519 signature verified | — |
| `4011` | `SIGNATURE_INVALID` | ERROR | Ed25519 signature verification failed | Data or key mismatch |
| `4012` | `SIGNATURE_MISSING` | WARN | No signature on token/descriptor | Phase 1 compat mode; will be enforced in Phase 4 |
| `4020` | `KEY_LOAD_OK` | INFO | Ed25519 key loaded from `identity.pem` | — |
| `4021` | `KEY_GENERATE_OK` | INFO | New Ed25519 key generated and saved | — |
| `4022` | `KEY_PARSE_FAIL` | ERROR | Cannot parse key bytes from file | File may be corrupted; delete and restart |
| `4030` | `NOISE_HANDSHAKE_OK` | DEBUG | Noise XX handshake complete | — |
| `4031` | `NOISE_HANDSHAKE_FAIL` | ERROR | Noise handshake failed | Version mismatch or key rejection |

---

## 5xxx — Protocol (Chat, File, Gossip, Wallet)

| Code | Name | Log Level | Description | Resolution |
|------|------|-----------|-------------|------------|
| `5000` | `CHAT_RECEIVED` | INFO | `ChatMessage` received on stream | — |
| `5001` | `CHAT_RELAYED` | DEBUG | Message echoed to sender | — |
| `5010` | `FILE_TRANSFER_CHUNK` | DEBUG | File chunk received | — |
| `5011` | `FILE_TRANSFER_COMPLETE` | INFO | File fully received | — |
| `5012` | `FILE_TRANSFER_ERROR` | ERROR | Error during file stream | Check sender; retry |
| `5020` | `GOSSIP_RECEIVED` | INFO | `GossipMessage` received | — |
| `5021` | `GOSSIP_DUPLICATE` | DEBUG | Already-seen gossip ID dropped | Normal; dedup working |
| `5022` | `GOSSIP_FANOUT` | INFO | Gossiping to N peers | — |
| `5023` | `GOSSIP_FANOUT_FAILED` | WARN | Could not reach a peer during fan-out | Peer may have gone offline |
| `5030` | `TX_RECEIVED` | INFO | `BroadcastTx` received | — |
| `5031` | `TX_QUEUED` | DEBUG | Transaction queued in mempool | — |
| `5032` | `TX_INVALID` | ERROR | Transaction failed validation | Malformed tx; check wallet code |
| `5040` | `INTRODUCE_ACCEPTED` | INFO | `Introduce1` accepted | — |
| `5041` | `INTRODUCE_REJECTED` | WARN | `Introduce1` rejected (auth failed) | Present valid capability token |
| `5042` | `RENDEZVOUS_MATCHED` | INFO | `ConnectRendezvous` matched | — |

---

## 6xxx — Storage (DHT Shards)

| Code | Name | Log Level | Description | Resolution |
|------|------|-----------|-------------|------------|
| `6000` | `SHARD_STORED` | INFO | `FileShard` stored in DHT | — |
| `6001` | `SHARD_RETRIEVED` | INFO | `FileShard` retrieved from DHT | — |
| `6002` | `SHARD_NOT_FOUND` | WARN | Shard key not in local DHT | Upload file again; check TTL |
| `6003` | `SHARD_EXPIRED` | DEBUG | Shard evicted after TTL | Re-upload or set `--mine-atk` for persistence |
| `6010` | `STORAGE_CLEANUP` | DEBUG | DHT maintenance: evicting expired entries | — |
| `6011` | `STORAGE_INIT` | INFO | Storage backend initialized | — |
| `6012` | `STORAGE_INIT_SLED` | INFO | Persistent Sled storage initialized (`--mine-atk`) | — |
| `6020` | `VALUE_STORED` | DEBUG | Generic key-value stored in DHT | — |
| `6021` | `VALUE_RETRIEVED` | DEBUG | Generic key-value retrieved from DHT | — |
| `6022` | `VALUE_NOT_FOUND` | DEBUG | Key-value not in local storage | — |

---

## 7xxx — Auth & Capability Tokens

| Code | Name | Log Level | Description | Resolution |
|------|------|-----------|-------------|------------|
| `7000` | `TOKEN_ISSUED` | INFO | Capability token issued | — |
| `7001` | `TOKEN_VALID` | INFO | Token verification passed | — |
| `7002` | `TOKEN_EXPIRED` | WARN | Token `expires_at` is in the past | Issue a new token with `auth-issue` |
| `7003` | `TOKEN_SCOPE_DENIED` | WARN | Token scope doesn't include required access | Re-issue with correct scope |
| `7004` | `TOKEN_SIGNATURE_INVALID` | ERROR | Ed25519 signature verification failed | Token tampered with |
| `7005` | `TOKEN_MISSING` | WARN | No token provided for `CAPABILITY_REQUIRED` service | Present token in `Introduce1.capability_token` |
| `7006` | `TOKEN_PARSE_FAIL` | ERROR | Cannot parse token JSON | Malformed token; re-issue |

---

## 9xxx — System / Diagnostics

| Code | Name | Log Level | Description | Resolution |
|------|------|-----------|-------------|------------|
| `9000` | `CONFIG_PARSE_WARN` | WARN | Config TOML not found or parse error; using defaults | Create `configs/default.toml` or ignore |
| `9001` | `DEPRECATED_FIELD` | WARN | Config field is deprecated | Migrate to new field name |
| `9002` | `MISSING_BOOTSTRAP` | WARN | Node started with no bootstrap peers | Expected for the first bootstrap node |
| `9010` | `CIRCUIT_BUILT` | INFO | 3-hop onion circuit established | — |
| `9011` | `CIRCUIT_EXPIRED` | DEBUG | Circuit exceeded max uptime | New circuit will be built |
| `9012` | `CIRCUIT_BUILD_FAILED` | WARN | Could not build circuit (insufficient peers) | Add more bootstrap peers |

---

## Log Format

Errors appear in structured JSON logs (via `tracing-subscriber`):

```json
{
  "timestamp": "2026-05-25T11:02:04Z",
  "level": "WARN",
  "target": "cloakmesh_core::routing::dht",
  "fields": {
    "message": "[3002] Failed to connect to bootstrap peer",
    "peer_addr": "127.0.0.1:4001",
    "error": "Connection refused (os error 111)"
  }
}
```

**Filtering by code range in logs:**
```bash
# All DHT events (3xxx)
journalctl -u cloakmesh | grep '"3[0-9]\{3\}]'

# All errors
journalctl -u cloakmesh | grep '"level":"ERROR"'

# Bootstrap events
journalctl -u cloakmesh | grep '\[300'
```
