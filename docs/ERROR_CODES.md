# CloakMesh Unified Error Codes

To ensure consistency across the polyglot architecture (Rust, Python, TypeScript), CloakMesh uses a unified error code space. Errors are categorized by subsystem.

## Error Categories

| Range | Category | Description |
| :--- | :--- | :--- |
| 1000-1999 | **CRYPTO** | Cryptographic primitives, handshakes, key rotation. |
| 2000-2999 | **PROTOCOL** | Address derivation, descriptor format, cell framing. |
| 3000-3999 | **ROUTING** | DHT lookups, circuit building, relay selection. |
| 4000-4999 | **NETWORK** | Connection failures, timeouts, gRPC transport. |
| 5000-5999 | **AUTH** | Capability tokens, ZK-proofs, scope violations. |
| 6000-6999 | **STORAGE** | KV store failures, journal corruption, I/O. |

## Detailed Error Codes

### 1000: CRYPTO
- `1001`: `KEY_GEN_FAILED` - OS CSPRNG failure or invalid parameters.
- `1002`: `SIG_VERIFY_FAILED` - Ed25519 signature mismatch.
- `1003`: `AEAD_DECRYPT_FAILED` - Tag mismatch or tampered ciphertext.
- `1004`: `HANDSHAKE_INCOMPLETE` - Noise state machine in wrong state.
- `1005`: `NONCE_EXHAUSTED` - 64-bit counter overflow, rotation required.

### 2000: PROTOCOL
- `2001`: `INVALID_ADDRESS` - .cloak address format or Bech32 error.
- `2002`: `CHECKSUM_MISMATCH` - Derived address checksum does not match payload.
- `2003`: `OVERSIZED_PAYLOAD` - Cell or envelope exceeds protocol limits.

### 3000: ROUTING
- `3001`: `DHT_KEY_NOT_FOUND` - Requested descriptor or value not in DHT.
- `3002`: `CIRCUIT_BUILD_FAILED` - Hop failed to acknowledge or relay.
- `3003`: `INSUFFICIENT_RELAYS` - Not enough healthy nodes to form requested hops.

### 4000: NETWORK
- `4001`: `CONNECTION_REFUSED` - Peer or core node not listening.
- `4002`: `TIMEOUT` - Operation exceeded the configured interval.
- `4003`: `GRPC_INTERNAL` - Low-level transport error.

### 5000: AUTH
- `5001`: `TOKEN_EXPIRED` - Capability token TTL has passed.
- `5002`: `INSUFFICIENT_SCOPE` - Token lacks required permission (e.g. 'write').
- `5003`: `ZK_PROOF_INVALID` - Zero-knowledge proof verification failed.
