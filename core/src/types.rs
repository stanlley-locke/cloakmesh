use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

// ── .cloak Address ───────────────────────────────────────────────────────────

/// A validated .cloak address: `<base32(version||pubkey||checksum)>.cloak`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CloakAddress(String);

impl CloakAddress {
    /// Construct without validation — only use when address was already validated.
    pub(crate) fn from_trusted(s: String) -> Self {
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CloakAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for CloakAddress {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// ── Circuit Identifier ───────────────────────────────────────────────────────

/// 16-byte cryptographically random circuit identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CircuitId(pub [u8; 16]);

impl CircuitId {
    pub fn generate() -> Self {
        use rand::RngCore;
        let mut id = [0u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut id);
        Self(id)
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl fmt::Display for CircuitId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

// ── Reputation Score ─────────────────────────────────────────────────────────

/// Reputation score clamped to [0.0, 1.0].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ReputationScore(f64);

impl ReputationScore {
    pub const MIN: ReputationScore = ReputationScore(0.0);
    pub const MAX: ReputationScore = ReputationScore(1.0);
    pub const DEFAULT: ReputationScore = ReputationScore(0.5);
    /// Minimum score a relay must have to be selected for routing.
    pub const ROUTING_THRESHOLD: f64 = 0.3;

    pub fn new(v: f64) -> Self {
        Self(v.clamp(0.0, 1.0))
    }

    pub fn value(&self) -> f64 {
        self.0
    }

    pub fn is_above_threshold(&self) -> bool {
        self.0 >= Self::ROUTING_THRESHOLD
    }
}

impl fmt::Display for ReputationScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.3}", self.0)
    }
}

// ── Node Info ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    /// host:port
    pub address: String,
    pub reputation: ReputationScore,
    /// Ed25519 public key bytes (32B)
    pub pubkey: [u8; 32],
}

impl NodeInfo {
    pub fn cloak_address(&self) -> crate::cloak_protocol::address::CloakAddressRef<'_> {
        crate::cloak_protocol::address::CloakAddressRef(&self.pubkey)
    }
}

// ── Session Nonce ────────────────────────────────────────────────────────────

/// 96-bit (12-byte) nonce for ChaCha20-Poly1305.
/// Constructed from a 64-bit counter + 32-bit circuit prefix to guarantee
/// uniqueness across all cells in a session.
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct CellNonce {
    prefix: [u8; 4],
    counter: u64,
}

impl CellNonce {
    pub fn new(circuit_prefix: [u8; 4]) -> Self {
        Self { prefix: circuit_prefix, counter: 0 }
    }

    /// Advance and return the next nonce. Returns `None` if the counter would
    /// overflow (2^64 cells — practically unreachable, but checked for safety).
    pub fn advance_nonce(&mut self) -> Option<[u8; 12]> {
        let c = self.counter.checked_add(1)?;
        self.counter = c;
        let mut nonce = [0u8; 12];
        nonce[..4].copy_from_slice(&self.prefix);
        nonce[4..].copy_from_slice(&c.to_le_bytes());
        Some(nonce)
    }

    pub fn current_count(&self) -> u64 {
        self.counter
    }
}

// ── Rendezvous Cookie ────────────────────────────────────────────────────────

/// 32-byte random rendezvous cookie used to match client and service circuits at the RP.
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop, Serialize, Deserialize)]
pub struct RendezvousCookie(pub [u8; 32]);

impl RendezvousCookie {
    pub fn generate() -> Self {
        use rand::RngCore;
        let mut cookie = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut cookie);
        Self(cookie)
    }

    /// Constant-time equality check.
    pub fn ct_eq(&self, other: &Self) -> bool {
        use subtle::ConstantTimeEq;
        self.0.ct_eq(&other.0).into()
    }
}

// ── Hop Count ────────────────────────────────────────────────────────────────

/// Validated hop count for circuit construction (2–5 hops).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HopCount(u8);

impl HopCount {
    pub const MIN: u8 = 2;
    pub const MAX: u8 = 5;
    pub const DEFAULT: u8 = 3;

    pub fn new(n: u8) -> crate::errors::CloakResult<Self> {
        if !(Self::MIN..=Self::MAX).contains(&n) {
            return Err(crate::errors::CloakError::ConfigInvalidValue {
                field: "hop_count".into(),
                reason: format!("must be {}-{}, got {}", Self::MIN, Self::MAX, n),
            });
        }
        Ok(Self(n))
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

impl Default for HopCount {
    fn default() -> Self {
        Self(Self::DEFAULT)
    }
}
