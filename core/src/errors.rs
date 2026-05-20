use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloakErrorCode {
    // 1000: CRYPTO
    KeyGenFailed = 1001,
    SigVerifyFailed = 1002,
    AeadDecryptFailed = 1003,
    HandshakeIncomplete = 1004,
    NonceExhausted = 1005,
    InvalidKeyMaterial = 1006,

    // 2000: PROTOCOL
    InvalidAddress = 2001,
    ChecksumMismatch = 2002,
    OversizedPayload = 2003,

    // 3000: ROUTING
    DhtKeyNotFound = 3001,
    CircuitBuildFailed = 3002,
    InsufficientRelays = 3003,

    // 4000: NETWORK
    ConnectionRefused = 4001,
    Timeout = 4002,
    GrpcInternal = 4003,

    // 5000: AUTH
    TokenExpired = 5001,
    InsufficientScope = 5002,
    ZkProofInvalid = 5003,

    // 6000: STORAGE
    StorageIo = 6001,
    StorageSerde = 6002,
    JournalCorrupted = 6003,

    // 9000: OTHER
    Unknown = 9999,
}

#[derive(Debug, Error)]
pub enum CloakError {
    // ── Crypto ──────────────────────────────────────────────────────────────
    #[error("[1001] crypto: key generation failed: {0}")]
    KeyGeneration(String),

    #[error("[1002] crypto: signature verification failed: {0}")]
    SignatureVerification(String),

    #[error("[1003] crypto: AEAD encryption failed")]
    AeadEncrypt,

    #[error("[1003] crypto: AEAD decryption failed (authentication tag mismatch)")]
    AeadDecrypt,

    #[error("[1006] crypto: invalid key material: {0}")]
    InvalidKeyMaterial(String),

    #[error("[1005] crypto: nonce exhausted — session key must be rotated")]
    NonceExhausted,

    #[error("[1004] crypto: Noise handshake error at step {step}: {reason}")]
    NoiseHandshake { step: u8, reason: String },

    #[error("[1000] crypto: KDF failed: {0}")]
    Kdf(String),

    // ── Protocol / Address ──────────────────────────────────────────────────
    #[error("[2001] address: invalid format — {0}")]
    InvalidAddress(String),

    #[error("[2002] address: checksum mismatch")]
    AddressChecksum,

    #[error("[2000] address: unsupported version byte {0:#04x}")]
    UnsupportedAddressVersion(u8),

    // ── Descriptor ──────────────────────────────────────────────────────────
    #[error("[2002] descriptor: signature invalid")]
    DescriptorSignatureInvalid,

    #[error("[2000] descriptor: expired (issued_at={issued_at}, now={now})")]
    DescriptorExpired { issued_at: u64, now: u64 },

    #[error("[2000] descriptor: nonce replay detected")]
    DescriptorReplay,

    #[error("[2000] descriptor: Merkle proof verification failed")]
    MerkleProofInvalid,

    #[error("[2000] descriptor: missing required field '{0}'")]
    DescriptorMissingField(String),

    // ── Capability / Auth ───────────────────────────────────────────────────
    #[error("[5001] auth: capability token expired")]
    TokenExpired,

    #[error("[5002] auth: capability token signature invalid")]
    TokenSignatureInvalid,

    #[error("[5002] auth: required scope '{required}' not present in token scopes {present:?}")]
    InsufficientScope { required: String, present: Vec<String> },

    #[error("[5000] auth: token issued for '{token_addr}' but used against '{request_addr}'")]
    TokenAddressMismatch { token_addr: String, request_addr: String },

    // ── Network / Transport ─────────────────────────────────────────────────
    #[error("[4003] protocol error: {0}")]
    Protocol(String),

    #[error("[5000] authentication error: {0}")]
    Auth(String),

    #[error("[4001] network: connection to '{addr}' failed: {reason}")]
    ConnectionFailed { addr: String, reason: String },

    #[error("[4003] network: send failed: {0}")]
    SendFailed(String),

    #[error("[4003] network: receive failed: {0}")]
    RecvFailed(String),

    #[error("[4000] network: connection closed unexpectedly")]
    ConnectionClosed,

    #[error("[4002] network: timeout after {ms}ms")]
    Timeout { ms: u64 },

    // ── DHT ─────────────────────────────────────────────────────────────────
    #[error("[3001] dht: key not found")]
    DhtKeyNotFound,

    #[error("[3000] dht: store rejected: {0}")]
    DhtStoreRejected(String),

    #[error("[3000] dht: proof verification failed")]
    DhtProofInvalid,

    // ── Routing / Circuit ───────────────────────────────────────────────────
    #[error("[3003] routing: insufficient relays — need {need}, have {have}")]
    InsufficientRelays { need: usize, have: usize },

    #[error("[3002] routing: circuit build failed at hop {hop}: {reason}")]
    CircuitBuildFailed { hop: u8, reason: String },

    #[error("[3000] routing: rendezvous cookie mismatch")]
    RendezvousCookieMismatch,

    // ── Storage ─────────────────────────────────────────────────────────────
    #[error("[6001] storage: I/O error: {0}")]
    StorageIo(#[from] std::io::Error),

    #[error("[6002] storage: serialization error: {0}")]
    StorageSerde(String),

    #[error("[6003] storage: journal corrupted at offset {0}")]
    JournalCorrupted(u64),

    // ── Configuration ───────────────────────────────────────────────────────
    #[error("[9000] config: parse error: {0}")]
    ConfigParse(String),

    #[error("[9000] config: invalid value for '{field}': {reason}")]
    ConfigInvalidValue { field: String, reason: String },

    // ── Catch-all ───────────────────────────────────────────────────────────
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl CloakError {
    pub fn code(&self) -> CloakErrorCode {
        match self {
            CloakError::KeyGeneration(_) => CloakErrorCode::KeyGenFailed,
            CloakError::SignatureVerification(_) => CloakErrorCode::SigVerifyFailed,
            CloakError::AeadEncrypt => CloakErrorCode::AeadDecryptFailed, // Grouped
            CloakError::AeadDecrypt => CloakErrorCode::AeadDecryptFailed,
            CloakError::InvalidKeyMaterial(_) => CloakErrorCode::InvalidKeyMaterial,
            CloakError::NonceExhausted => CloakErrorCode::NonceExhausted,
            CloakError::NoiseHandshake { .. } => CloakErrorCode::HandshakeIncomplete,
            
            CloakError::InvalidAddress(_) => CloakErrorCode::InvalidAddress,
            CloakError::AddressChecksum => CloakErrorCode::ChecksumMismatch,
            
            CloakError::DhtKeyNotFound => CloakErrorCode::DhtKeyNotFound,
            CloakError::InsufficientRelays { .. } => CloakErrorCode::InsufficientRelays,
            CloakError::CircuitBuildFailed { .. } => CloakErrorCode::CircuitBuildFailed,
            
            CloakError::ConnectionFailed { .. } => CloakErrorCode::ConnectionRefused,
            CloakError::Timeout { .. } => CloakErrorCode::Timeout,
            
            CloakError::TokenExpired => CloakErrorCode::TokenExpired,
            CloakError::InsufficientScope { .. } => CloakErrorCode::InsufficientScope,
            
            CloakError::StorageIo(_) => CloakErrorCode::StorageIo,
            CloakError::StorageSerde(_) => CloakErrorCode::StorageSerde,
            CloakError::JournalCorrupted(_) => CloakErrorCode::JournalCorrupted,
            
            _ => CloakErrorCode::Unknown,
        }
    }
}

pub type CloakResult<T> = Result<T, CloakError>;
