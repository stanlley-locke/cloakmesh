# CloakMesh Cryptography Reference

Technical specification for all cryptographic primitives, protocols, and implementations used in CloakMesh. Every security decision is documented with its rationale.

---

## Table of Contents

- [Cryptographic Primitives Summary](#cryptographic-primitives-summary)
- [Ed25519 Keypairs](#ed25519-keypairs)
  - [Key Generation](#key-generation)
  - [Key Storage](#key-storage)
  - [Signing and Verification](#signing-and-verification)
  - [Zeroize on Drop](#zeroize-on-drop)
- [.cloak Address Derivation](#cloak-address-derivation)
  - [Address Format Specification](#address-format-specification)
  - [Checksum Algorithm](#checksum-algorithm)
  - [Encoding and Decoding](#encoding-and-decoding)
  - [Address Validation](#address-validation)
- [X25519 Key Exchange](#x25519-key-exchange)
- [Kyber-768 KEM (Post-Quantum Hybrid)](#kyber-768-kem-post-quantum-hybrid)
  - [pq_hybrid_enabled Config Flag](#pq_hybrid_enabled-config-flag)
  - [Hybrid Handshake Design](#hybrid-handshake-design)
- [Noise Protocol Handshake](#noise-protocol-handshake)
  - [HandshakeInit Message](#handshakeinit-message)
  - [HandshakeResponse Message](#handshakeresponse-message)
  - [Noise_XX Pattern](#noise_xx-pattern)
- [ChaCha20-Poly1305 Symmetric Encryption](#chacha20-poly1305-symmetric-encryption)
  - [Session Encryption](#session-encryption)
  - [Nonce Management](#nonce-management)
- [Cell Framing and Padding](#cell-framing-and-padding)
  - [Envelope Message](#envelope-message)
  - [Fixed-Size Cell Enforcement](#fixed-size-cell-enforcement)
  - [Cover Traffic](#cover-traffic)
- [Key Derivation Function (KDF)](#key-derivation-function-kdf)
  - [HKDF Usage](#hkdf-usage)
  - [Key Rotation](#key-rotation)
- [Zeroize: Volatile Memory Security](#zeroize-volatile-memory-security)
  - [ZeroizeWrapper](#zeroziwrapper)
  - [What Gets Zeroed](#what-gets-zeroed)
- [Double Ratchet (Signal Protocol)](#double-ratchet-signal-protocol)
  - [Ratchet State](#ratchet-state)
  - [Roadmap](#double-ratchet-roadmap)
- [Capability Tokens](#capability-tokens)
  - [Token Structure](#token-structure)
  - [Issuance](#issuance)
  - [Verification](#verification)
- [SHA-256 Usage](#sha-256-usage)
- [Merkle Trees](#merkle-trees)
- [ZK Proofs (Roadmap)](#zk-proofs-roadmap)
- [Security Properties Summary](#security-properties-summary)
- [Cryptographic Dependency Inventory](#cryptographic-dependency-inventory)

---

## Cryptographic Primitives Summary

| Primitive | Algorithm | Usage | Implementation |
|---|---|---|---|
| Asymmetric signing | Ed25519 | Identity, descriptors, transactions | `ed25519-dalek` |
| Key exchange | X25519 | Noise handshake, ephemeral sessions | `x25519-dalek` |
| Post-quantum KEM | Kyber-768 | Hybrid PQ handshake | `pqcrypto-kyber` |
| Symmetric encryption | ChaCha20-Poly1305 | Cell encryption, session data | `chacha20poly1305` |
| Hash function | SHA-256 | Content addressing, checksums, KDF | `sha2` |
| KDF | HKDF-SHA256 | Session key derivation from shared secret | `hkdf` |
| Handshake protocol | Noise_XX | Mutual authentication + key exchange | Custom (`noise.rs`) |
| Address encoding | Base32 (RFC 4648) | `.cloak` address serialization | `base32` |
| Memory security | Zeroize | Cryptographic erasure of secrets | `zeroize` |
| Ratchet (roadmap) | Double Ratchet | Per-message PFS for chat | `ratchet.rs` |

---

## Ed25519 Keypairs

### Key Generation

```rust
// core/src/crypto/ed25519.rs
use ed25519_dalek::{SigningKey, Signer, Verifier, VerifyingKey};
use rand::rngs::OsRng;

pub struct Ed25519KeyPair {
    signing_key: SigningKey,  // Contains both private seed and derived public key
}

impl Ed25519KeyPair {
    /// Generate a fresh keypair using the OS CSPRNG (cryptographically secure RNG).
    pub fn generate() -> Self {
        Self { signing_key: SigningKey::generate(&mut OsRng) }
    }
}
```

**Algorithm:** Ed25519 as specified in [RFC 8032](https://www.rfc-editor.org/rfc/rfc8032).

**Key sizes:**
| Component | Size |
|---|---|
| Secret seed | 32 bytes |
| Public key | 32 bytes |
| Signature | 64 bytes |

**Security properties:**
- 128-bit classical security level
- Resistant to fault attacks (deterministic signing; no random nonce to bias)
- Cofactor-safe (small-subgroup attacks mitigated)
- Constant-time implementation in `ed25519-dalek`

### Key Storage

```rust
// core/src/crypto/ed25519.rs
pub fn save_to_file(&self, path: &Path) -> CloakResult<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .write(true).create(true).truncate(true)
            .mode(0o600)  // Owner read/write only
            .open(path)?
    }
    file.write_all(&*seed)?;
}
```

**Key file format:** Raw 32-byte Ed25519 secret seed. No PEM encoding — direct binary.

**File permissions:** `0o600` (Unix: `-rw-------`) — only the file owner can read or write.

**Load on startup:**
```rust
pub fn load_or_generate(path: &Path) -> CloakResult<Self> {
    if path.exists() {
        Self::load_from_file(path)  // Load existing key
    } else {
        let kp = Self::generate();
        kp.save_to_file(path)?;     // Generate and persist
        Ok(kp)
    }
}
```

### Signing and Verification

```rust
// core/src/crypto/ed25519.rs

// Sign: message is pre-hashed internally by ed25519-dalek
pub fn sign(&self, msg: &[u8]) -> Ed25519Signature {
    Ed25519Signature(self.signing_key.sign(msg))
}

// Verify (standalone, constant-time)
pub fn verify(pubkey_bytes: &[u8; 32], msg: &[u8], sig_bytes: &[u8; 64]) -> CloakResult<()> {
    let vk = VerifyingKey::from_bytes(pubkey_bytes)?;
    let sig = Signature::from_bytes(sig_bytes);
    vk.verify(msg, &sig).map_err(|e| CloakError::SignatureVerification(e.to_string()))
}
```

**Verification is constant-time:** `ed25519-dalek` uses `subtle::ConstantTimeEq` internally, preventing timing side-channels.

**What is signed:**
| Item | Signer | Signed Data |
|---|---|---|
| `CloakDescriptor` | Service node | Full descriptor fields (serialized) |
| `Transaction` | Wallet owner | `inputs || outputs || timestamp` |
| `CapabilityToken` | Issuer | `token_id || address || scopes || expires_at` |
| Circuit cells | (session key) | Not Ed25519; ChaCha20-Poly1305 AEAD |

### Zeroize on Drop

```rust
// ed25519-dalek's SigningKey implements ZeroizeOnDrop internally.
// The 32-byte secret seed is zeroed when SigningKey is dropped.
impl Drop for Ed25519KeyPair {
    fn drop(&mut self) {
        // SigningKey automatically zeros its memory (ZeroizeOnDrop trait)
    }
}
```

---

## .cloak Address Derivation

### Address Format Specification

```
Format: <base32(payload)>.cloak

Payload (37 bytes):
  [0]     : version byte = 0x01
  [1..32] : Ed25519 public key (32 bytes)
  [33..36]: checksum (4 bytes) = SHA256(SHA256(0x01 || pubkey))[:4]

Encoded:
  base32(payload, RFC4648, no-padding) → lowercase → + ".cloak"
  37 bytes → 59 base32 characters + ".cloak" suffix = 65 characters total
```

**Constants:**
```rust
// core/src/cloak_protocol/address.rs
pub const VERSION_BYTE: u8 = 0x01;
const PUBKEY_LEN: usize = 32;
const CHECKSUM_LEN: usize = 4;
const PAYLOAD_LEN: usize = 1 + PUBKEY_LEN + CHECKSUM_LEN; // 37 bytes
```

### Checksum Algorithm

```rust
// core/src/cloak_protocol/address.rs
fn compute_checksum(pubkey: &[u8; 32]) -> [u8; 4] {
    // First SHA-256: over version_byte || pubkey
    let mut first = Sha256::new();
    first.update([VERSION_BYTE]);  // 0x01
    first.update(pubkey);          // 32 bytes
    let first_hash = first.finalize();

    // Second SHA-256: over first hash
    let second_hash = Sha256::digest(first_hash);

    // Take first 4 bytes as checksum
    let mut checksum = [0u8; 4];
    checksum.copy_from_slice(&second_hash[..4]);
    checksum
}
```

**Checksum purpose:** Detects typos and bit errors in `.cloak` addresses. A single character change will (with overwhelming probability 1 - 2^-32 ≈ 99.9999998%) cause checksum verification to fail.

### Encoding and Decoding

```rust
// Encoding
let encoded = base32::encode(
    base32::Alphabet::RFC4648 { padding: false },
    &payload
).to_lowercase();
CloakAddress::from_trusted(format!("{}.cloak", encoded))

// Decoding
let bytes = base32::decode(
    base32::Alphabet::RFC4648 { padding: false },
    &host.to_uppercase(),  // base32 is case-insensitive; normalize to uppercase
).ok_or(/* decode error */)?;
```

**Base32 alphabet (RFC 4648):** `A-Z` + `2-7` (mapped to lowercase `a-z` + `2-7`). No `0`, `1`, `8`, `9` to avoid visual confusion.

### Address Validation

```rust
// core/src/cloak_protocol/address.rs
pub fn parse_address(addr: &str) -> CloakResult<[u8; 32]> {
    // 1. Strip ".cloak" suffix
    let host = addr.strip_suffix(".cloak")?;

    // 2. Validate base32 characters (reject non-alphabet chars)
    if !host.chars().all(|c| matches!(c, 'a'..='z' | '2'..='7')) {
        return Err(CloakError::InvalidAddress(...));
    }

    // 3. Base32 decode
    let bytes = base32::decode(...)?;

    // 4. Length check (must be exactly 37 bytes)
    if bytes.len() != PAYLOAD_LEN { return Err(...); }

    // 5. Version check
    if bytes[0] != VERSION_BYTE { return Err(CloakError::UnsupportedAddressVersion(bytes[0])); }

    // 6. Extract pubkey
    let pubkey: [u8; 32] = bytes[1..33].try_into().unwrap();

    // 7. Constant-time checksum verification
    let expected = compute_checksum(&pubkey);
    let actual = &bytes[33..];
    use subtle::ConstantTimeEq;
    if expected.ct_eq(actual).unwrap_u8() != 1 {
        return Err(CloakError::AddressChecksum);
    }

    Ok(pubkey)
}
```

**Constant-time checksum comparison:** Uses `subtle::ConstantTimeEq` to prevent timing attacks that could distinguish valid addresses from invalid ones through response latency.

---

## X25519 Key Exchange

X25519 Diffie-Hellman key exchange is used for ephemeral session establishment in:
- Noise handshakes (ephemeral keypair per session)
- Rendezvous cookie encryption (`RendezvousCookie.ephemeral_pubkey`)

```rust
// core/src/crypto/x25519.rs
use x25519_dalek::{EphemeralSecret, PublicKey, StaticSecret};

pub struct X25519KeyPair {
    secret: StaticSecret,   // 32-byte scalar
    public: PublicKey,      // 32-byte u-coordinate on Curve25519
}

impl X25519KeyPair {
    pub fn generate() -> Self {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);
        Self { secret, public }
    }

    pub fn diffie_hellman(&self, their_public: &PublicKey) -> SharedSecret {
        self.secret.diffie_hellman(their_public)
    }
}
```

**Output:** 32-byte shared secret (Montgomery u-coordinate of the DH result).

**Ephemeral vs. Static use:**
| Use Case | Key Type | Lifecycle |
|---|---|---|
| Noise handshake ephemeral | `EphemeralSecret` | Single use; zeroed on drop |
| Hybrid PQ key in descriptor | `StaticSecret` | Per-descriptor lifetime |
| Rendezvous cookie | `EphemeralSecret` | Per-circuit |

---

## Kyber-768 KEM (Post-Quantum Hybrid)

Kyber-768 is a lattice-based Key Encapsulation Mechanism (KEM) selected by NIST as the post-quantum standard (FIPS 203 / ML-KEM).

### pq_hybrid_enabled Config Flag

```toml
# configs/default.toml
[crypto]
pq_hybrid_enabled = true  # Default: enabled
```

When `pq_hybrid_enabled = false`, the Kyber KEM fields in `HandshakeInit` are empty/ignored, falling back to classical X25519 only.

### Hybrid Handshake Design

```
Classical:  shared_secret_x25519 = X25519_DH(ephemeral_priv, peer_pub)
PQ:         (kyber_ct, shared_secret_kyber) = Kyber_Encaps(peer_kyber_pub)

Combined:   session_key = HKDF(
              input_key_material = shared_secret_x25519 || shared_secret_kyber,
              salt = "CloakMesh-v1-handshake",
              info = "session-key"
            )
```

**Security rationale:** The hybrid design ensures that security is maintained even if:
1. Kyber is broken (still secured by X25519 classical DH)
2. X25519 is broken by a future quantum computer (still secured by Kyber)

A single adversary needs to break **both** algorithms to compromise the session key.

**Proto fields:**
```protobuf
// proto/v1/cloakmesh.proto
message HandshakeInit {
  bytes ephemeral_pubkey = 1;  // X25519 32-byte ephemeral public key
  bytes kyber_kem_ct     = 2;  // Kyber-768 ciphertext (1088 bytes)
  bytes payload          = 3;  // Encrypted {identity_pubkey, nonce}
  uint32 version         = 4;
}

// proto/v1/cloak_service.proto
message CloakDescriptor {
  bytes hybrid_pq_pubkey = 4;  // X25519 (32B) + Kyber-768 pubkey (1184B) combined
}
```

---

## Noise Protocol Handshake

### HandshakeInit Message

The `HandshakeInit` implements the first message of the **Noise_XX** pattern:

```
→ e                     (Ephemeral X25519 public key)
→ Kyber_Encaps(rs)     (Kyber ciphertext for recipient's static key)
→ es                    (X25519 DH between our ephemeral and their static)
→ payload               (Encrypted: our identity pubkey + challenge nonce)
```

### HandshakeResponse Message

```
← e                     (Responder's ephemeral X25519 public key)
← Kyber_Encaps(re)     (Kyber ciphertext for initiator's static key)
← ee, se               (X25519 DH combinations)
← payload               (Encrypted: responder's identity + response)
```

### Noise_XX Pattern

After both handshake messages are exchanged:
- **Mutual authentication:** Both parties know each other's static public keys.
- **Perfect forward secrecy:** Ephemeral keys are used and discarded; compromise of static keys does not expose past sessions.
- **Session establishment:** Two directional symmetric keys are derived: `(send_key, recv_key)`.

```rust
// core/src/crypto/noise.rs
pub struct NoiseState {
    role: HandshakeRole,
    chaining_key: [u8; 32],    // Running hash of all handshake messages
    hash: [u8; 32],            // Mixed state hash
    local_ephemeral: X25519KeyPair,
    remote_ephemeral: Option<[u8; 32]>,
    remote_static: Option<[u8; 32]>,
}
```

---

## ChaCha20-Poly1305 Symmetric Encryption

### Session Encryption

All onion circuit cells are encrypted using **ChaCha20-Poly1305** (RFC 8439):

```rust
// core/src/crypto/session.rs
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use chacha20poly1305::aead::{Aead, NewAead};

pub struct BidirectionalSession {
    cipher: ChaCha20Poly1305,  // Keyed with 32-byte session key
    send_counter: AtomicU64,   // Monotonic nonce counter
}

impl BidirectionalSession {
    pub fn encrypt(&mut self, plaintext: &[u8], aad: &[u8]) -> CloakResult<Vec<u8>> {
        let nonce = self.next_nonce(); // 12-byte nonce from counter
        let ciphertext = self.cipher.encrypt(&nonce, plaintext)?;
        Ok(ciphertext)  // ciphertext = encrypted_data + 16-byte Poly1305 tag
    }

    pub fn decrypt(&self, nonce: &[u8; 12], ciphertext: &[u8], aad: &[u8]) -> CloakResult<Vec<u8>> {
        self.cipher.decrypt(Nonce::from_slice(nonce), ciphertext)
    }
}
```

**Properties:**
- **ChaCha20:** 256-bit key stream cipher; constant-time; no timing side-channels
- **Poly1305:** 128-bit MAC (AEAD authentication tag)
- **Combined:** Authenticated Encryption with Associated Data (AEAD)
- **Authentication scope:** `aad` parameter (e.g., `b"circuit-layer"`) is authenticated but not encrypted

**Per-hop encryption:** Each circuit hop has its own independent `BidirectionalSession` with a unique key derived during circuit construction. Onion encryption stacks these:

```
Ciphertext = Enc(K_guard, Enc(K_middle, Enc(K_exit, plaintext)))
```

### Nonce Management

```rust
fn next_nonce(&self) -> Nonce {
    let counter = self.send_counter.fetch_add(1, Ordering::SeqCst);
    let mut nonce = [0u8; 12];
    nonce[4..12].copy_from_slice(&counter.to_le_bytes());
    // First 4 bytes remain 0 (reserved for future node ID prefix)
    Nonce::from(nonce)
}
```

**Nonce format (12 bytes):**
```
[0..3]  : Reserved (zero)
[4..11] : 64-bit little-endian counter
```

**Nonce uniqueness guarantee:** The monotonic counter ensures each nonce is used at most once per session key. Key rotation (every 10,000 cells) resets the counter.

**Replay protection:** The `sequence` field in the `Envelope` message provides replay detection at the application layer, complementing nonce uniqueness at the cryptographic layer.

---

## Cell Framing and Padding

### Envelope Message

All inter-node traffic is wrapped in fixed-size `Envelope` cells:

```protobuf
// proto/v1/cloakmesh.proto
message Envelope {
  bytes  circuit_id  = 1;   // 16-byte circuit ID (UUID)
  uint64 sequence    = 2;   // Monotonic counter for replay detection
  bytes  nonce       = 3;   // 12-byte ChaCha20-Poly1305 nonce
  bytes  ciphertext  = 4;   // Payload padded to cell_size_bytes
  google.protobuf.Timestamp timestamp = 5;
}
```

### Fixed-Size Cell Enforcement

```rust
// NodeConfig validation
impl TrafficConfig {
    pub fn validate(&self) -> CloakResult<()> {
        if self.cell_size_bytes != 256 && self.cell_size_bytes != 1024 {
            return Err(CloakError::ConfigInvalidValue {
                field: "traffic.cell_size_bytes".into(),
                reason: "must be 256 or 1024".into(),
            });
        }
        // ...
    }
}
```

**Padding algorithm:**
1. Encrypt the actual payload.
2. Determine `pad_len = cell_size_bytes - len(ciphertext)`.
3. Append `pad_len` random bytes.
4. The receiver uses the `sequence` counter and authenticated ciphertext to determine actual payload length.

**Why fixed cells?** Variable-size cells reveal packet length, which is a traffic analysis vector. Fixed cells ensure all traffic is indistinguishable by size.

**Cell sizes:**
| Size | Use Case |
|---|---|
| 256 bytes | Default; lower bandwidth overhead |
| 1024 bytes | High-security mode; better padding coverage for large payloads |

### Cover Traffic

```toml
[traffic]
cover_flow_pps = 1       # 1 cover packet per second
jitter_max_ms = 50       # Up to 50ms random delay per cell
padding_enabled = true
```

During idle periods, the node generates **dummy `Envelope` cells** filled with random data and sends them at `cover_flow_pps` packets per second. This prevents timing correlation attacks that exploit traffic silence as a signal.

---

## Key Derivation Function (KDF)

### HKDF Usage

```rust
// core/src/crypto/kdf.rs
use hkdf::Hkdf;
use sha2::Sha256;

pub fn derive_key(
    input_key_material: &[u8],
    salt: &[u8],
    info: &[u8],
    output: &mut [u8]  // Output key length (e.g., 32 bytes)
) -> CloakResult<()> {
    let hkdf = Hkdf::<Sha256>::new(Some(salt), input_key_material);
    hkdf.expand(info, output).map_err(|_| CloakError::KeyDerivation("HKDF expand failed".into()))
}
```

**HKDF-SHA256 is used for:**
| Derivation | Input | Salt | Info |
|---|---|---|---|
| Session key from Noise shared secret | X25519_SS \|\| Kyber_SS | `"CloakMesh-v1-handshake"` | `"session-key"` |
| Send/Recv key split | Session key | — | `"send"` / `"recv"` |
| Cover traffic key | Session key | `"cover"` | Circuit ID |

### Key Rotation

```toml
[traffic]
key_rotation_cells = 10000  # Rotate after 10,000 cells
key_rotation_secs = 300     # Or after 5 minutes (whichever first)
```

On key rotation:
1. Derive new session key from `HKDF(current_key, "rotate", sequence_number)`.
2. Reset nonce counter to 0.
3. Both parties must independently perform the same rotation at the same trigger point.

---

## Zeroize: Volatile Memory Security

### ZeroizeWrapper

```rust
// core/src/routing/dht.rs
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct ZeroizeWrapper(pub Vec<u8>);
```

`ZeroizeWrapper` is used in `VolatileStorage` to wrap all stored DHT values:

```rust
// core/src/storage/db.rs
pub struct VolatileStorage {
    store: HashMap<DhtKey, (Arc<ZeroizeWrapper>, tokio::time::Instant)>,
}
```

### What Gets Zeroed

| Component | Zeroed When | Mechanism |
|---|---|---|
| Ed25519 secret seed | Key dropped | `ed25519-dalek` internal `ZeroizeOnDrop` |
| X25519 static secret | Secret dropped | `x25519-dalek` `ZeroizeOnDrop` |
| DHT stored values | Entry evicted or node shutdown | `ZeroizeWrapper` `ZeroizeOnDrop` |
| Session encryption key | Session dropped | `chacha20poly1305` key zeroed on drop |
| Noise handshake state | Handshake complete | `NoiseState` implements `Zeroize` |
| `DhtKey` | Key dropped | `#[derive(Zeroize, ZeroizeOnDrop)]` |

**Why this matters:** If an attacker gains read access to RAM (e.g., cold boot attack, memory dump), zeroed secrets cannot be recovered. This is particularly important for relay nodes that may store hundreds of active session keys simultaneously.

---

## Double Ratchet (Signal Protocol)

### Ratchet State

The `core/src/crypto/ratchet.rs` module scaffolds the Double Ratchet state machine. When implemented, each chat session will maintain:

```rust
// core/src/crypto/ratchet.rs (planned)
pub struct RatchetState {
    root_key: [u8; 32],          // Root key (rotated on DH ratchet)
    chain_key_send: [u8; 32],    // Sending chain key
    chain_key_recv: [u8; 32],    // Receiving chain key
    dh_ratchet_key: X25519KeyPair,  // Current DH ratchet keypair
    peer_dh_pub: Option<[u8; 32]>,  // Peer's most recent DH ratchet public key
    send_message_n: u32,         // Number of messages sent in current chain
    recv_message_n: u32,         // Number of messages received in current chain
    prev_chain_n: u32,           // Number of messages in previous sending chain
    skipped_keys: HashMap<(PublicKey, u32), [u8; 32]>, // Out-of-order key cache
}
```

### Double Ratchet Roadmap

**Phase 3.2: Double Ratchet Integration**

1. **Initialization:** During Noise handshake, both parties agree on initial `root_key` and `chain_key` values derived from the Noise shared secret.
2. **Symmetric Ratchet:** Every message advances the sending chain key:
   ```
   (chain_key, message_key) = HKDF(chain_key, "message")
   ```
3. **DH Ratchet:** On first message from the other party, perform a new X25519 DH exchange and rotate the root key:
   ```
   (root_key, chain_key) = HKDF(root_key, DH(ratchet_priv, peer_pub))
   ```
4. **Out-of-order messages:** Skipped message keys are cached in `skipped_keys` with a cap of 1,000 entries.

**Security properties provided:**
- **Perfect Forward Secrecy (PFS):** Old chain keys are deleted after use.
- **Break-in Recovery:** After a compromise, the DH ratchet restores security.
- **Message ordering:** Out-of-order delivery handled via skipped key cache.

---

## Capability Tokens

### Token Structure

```protobuf
// proto/v1/capability.proto
message CapabilityToken {
  string token_id              = 1;  // 16-byte random hex string (128-bit entropy)
  string cloak_address         = 2;  // Address this token grants access to
  repeated string scopes       = 3;  // e.g., ["read", "write"]
  google.protobuf.Timestamp expires_at = 4;  // UTC expiry time
  bytes issuer_pubkey          = 5;  // Ed25519 issuer public key (32 bytes)
  bytes signature              = 6;  // Ed25519 signature (64 bytes) [TODO Phase 4.3]
}
```

### Issuance

```python
# orchestrator/src/cloakcli/auth_generator.py
import secrets
import time
from pydantic import BaseModel

class CapabilityToken(BaseModel):
    token_id: str      # secrets.token_hex(16) → 16 random bytes = 32 hex chars
    cloak_address: str
    scope: str
    expires_at: int    # Unix timestamp: time.time() + ttl_secs
    signature: str = "" # TODO Phase 4.3: Ed25519 sign via gRPC

def issue_token(address: str, scope: str, ttl_secs: int) -> str:
    token = CapabilityToken(
        token_id=secrets.token_hex(16),
        cloak_address=address,
        scope=scope,
        expires_at=int(time.time()) + ttl_secs,
    )
    return token.model_dump_json()
```

**`token_id` entropy:** `secrets.token_hex(16)` generates 16 random bytes (128 bits) from the OS CSPRNG, formatted as 32 hex characters. This provides sufficient entropy to prevent brute-force token ID guessing.

### Verification

```rust
// core/src/node.rs — CapabilityService::Verify implementation
fn verify_token_internal(&self, request: &CapabilityVerifyRequest) -> Result<(), Box<Status>> {
    let token = request.token.as_ref()?;

    // 1. Check expiry
    if let Some(expires_at) = &token.expires_at {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        if (expires_at.seconds as u64) < now {
            return Err(Box::new(Status::unauthenticated("Token expired")));
        }
    }

    // 2. Check scope
    if !token.scopes.contains(&request.required_scope) {
        return Err(Box::new(Status::permission_denied(
            format!("Missing required scope: {}", request.required_scope)
        )));
    }

    // 3. TODO Phase 4.3: Verify Ed25519 signature
    if token.signature.is_empty() {
        // Warn but not reject (during Phase 1/2)
        tracing::warn!("Capability token has no signature — accepting for Phase 1 compatibility");
    }

    Ok(())
}
```

**gRPC service:**
```protobuf
// proto/v1/capability.proto
service CapabilityService {
  rpc Verify(CapabilityVerifyRequest) returns (CapabilityVerifyResponse);
}

message CapabilityVerifyRequest {
  CapabilityToken token = 1;
  string required_scope = 2;
}

message CapabilityVerifyResponse {
  bool   valid  = 1;
  string reason = 2;
}
```

---

## SHA-256 Usage

SHA-256 (FIPS 180-4) is the primary hash function throughout CloakMesh:

| Usage | Input | Output |
|---|---|---|
| Content addressing | File data | SHA-256 file hash (32 bytes) |
| Address checksum | `0x01 \|\| pubkey` | Double-SHA256, first 4 bytes |
| DHT key derivation | `file_hash + ":" + shard_index` | SHA-256 DHT key (32 bytes) |
| Transaction ID | Serialized `inputs \|\| outputs \|\| timestamp` | SHA-256 tx ID |
| Merkle tree leaves | Shard data | SHA-256 leaf hash |
| KDF (HKDF) | Session secrets | SHA-256 as HKDF PRF |

**Implementation:**
```rust
use sha2::{Digest, Sha256};

let hash = Sha256::digest(data);           // Single SHA-256
let double = Sha256::digest(Sha256::digest(data)); // Double SHA-256 (for checksum)
```

---

## Merkle Trees

```rust
// core/src/cloak_protocol/merkle.rs
// core/src/storage/proofs.rs
```

Merkle trees are used for:
- **`CloakDescriptor.merkle_root`:** 32-byte Merkle root over all descriptor fields.
- **`DhtFetchResponse.merkle_proof`:** Inclusion proof for a stored value.
- **`FileManifest` shard integrity:** Ordered SHA-256 hashes in `shard_hashes` form the leaves.

**Leaf computation:** `SHA256(shard_data)`

**Internal node computation:** `SHA256(left_child_hash || right_child_hash)`

**Root:** Single 32-byte value committed to in the descriptor.

---

## ZK Proofs (Roadmap)

`core/src/crypto/zk.rs` scaffolds a zero-knowledge proof system for `AuthPolicy.ZK_GATED` services. Planned implementation:

- **Proof system:** zk-SNARKs (Groth16 or PLONK)
- **Use case:** Prove you hold ≥ N ATK without revealing your balance or identity
- **Integration:** `Introduce1.capability_token` carries the ZK proof alongside or instead of an `CapabilityToken`

---

## Security Properties Summary

| Property | Status | Mechanism |
|---|---|---|
| Mutual authentication | ✅ | Noise_XX handshake, Ed25519 |
| Forward secrecy | ✅ | Ephemeral X25519 keys per session |
| Post-quantum resistance | ✅ (configurable) | Kyber-768 hybrid KEM |
| Traffic analysis resistance | ✅ | Fixed 256/1024B cells, cover traffic, jitter |
| Replay protection | ✅ | Monotonic sequence counter + nonce |
| Address integrity | ✅ | Double-SHA256 checksum, constant-time verify |
| Memory security | ✅ | Zeroize on drop for all key material |
| Content integrity | ✅ | SHA-256 content addressing |
| Per-hop isolation | ✅ | Independent ChaCha20 session per hop |
| Message authentication | ✅ | Poly1305 AEAD tag on every cell |
| Perfect forward secrecy (per-message) | 🔷 Roadmap | Double Ratchet (Phase 3.2) |
| ZK-gated access | 🔷 Roadmap | zk-SNARKs (Phase 5) |
| Signature on capability tokens | 🔷 Roadmap | Phase 4.3 |

---

## Cryptographic Dependency Inventory

| Crate | Version | Usage |
|---|---|---|
| `ed25519-dalek` | 2.x | Ed25519 signing, verification |
| `x25519-dalek` | 2.x | X25519 ECDH key exchange |
| `chacha20poly1305` | 0.10.x | ChaCha20-Poly1305 AEAD |
| `sha2` | 0.10.x | SHA-256 hashing |
| `hkdf` | 0.12.x | HKDF-SHA256 key derivation |
| `rand` | 0.8.x | OS CSPRNG (`OsRng`) |
| `zeroize` | 1.x | Cryptographic memory erasure |
| `subtle` | 2.x | Constant-time comparisons |
| `base32` | 0.4.x | RFC 4648 base32 encoding |
| `uuid` | 1.x | Circuit ID generation |
| `pqcrypto-kyber` | 0.7.x | Kyber-768 KEM (PQ hybrid) |

All cryptographic crates are pinned in `Cargo.lock` and reviewed for `#![forbid(unsafe_code)]` status where possible. Any `unsafe` usage in cryptographic code must be explicitly documented and reviewed.
