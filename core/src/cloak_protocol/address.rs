//! .cloak address encoding and validation.
//!
//! Format: `<base32(version || pubkey || checksum)>.cloak`
//!
//! - version:  1 byte  (0x01)
//! - pubkey:   32 bytes (Ed25519 public key)
//! - checksum: 4 bytes  (first 4 bytes of SHA-256(SHA-256(version || pubkey)))
//!
//! Total payload: 37 bytes → base32 → 59 characters + ".cloak" suffix.

use sha2::{Digest, Sha256};

use crate::errors::{CloakError, CloakResult};
use crate::types::CloakAddress;

// ── Constants ────────────────────────────────────────────────────────────────

pub const VERSION_BYTE: u8 = 0x01;
const PUBKEY_LEN: usize = 32;
const CHECKSUM_LEN: usize = 4;
const PAYLOAD_LEN: usize = 1 + PUBKEY_LEN + CHECKSUM_LEN; // 37 bytes

// ── Address derivation ───────────────────────────────────────────────────────

/// Derive a .cloak address from an Ed25519 public key.
pub fn derive_address(pubkey: &[u8; 32]) -> CloakAddress {
    let checksum = compute_checksum(pubkey);
    let mut payload = [0u8; PAYLOAD_LEN];
    payload[0] = VERSION_BYTE;
    payload[1..33].copy_from_slice(pubkey);
    payload[33..].copy_from_slice(&checksum);
    let encoded = base32::encode(base32::Alphabet::RFC4648 { padding: false }, &payload)
        .to_lowercase();
    CloakAddress::from_trusted(format!("{}.cloak", encoded))
}

/// Compute the 4-byte double-SHA256 checksum over `version || pubkey`.
fn compute_checksum(pubkey: &[u8; 32]) -> [u8; CHECKSUM_LEN] {
    let mut first = Sha256::new();
    first.update([VERSION_BYTE]);
    first.update(pubkey);
    let first_hash = first.finalize();

    let second_hash = Sha256::digest(first_hash);
    let mut checksum = [0u8; CHECKSUM_LEN];
    checksum.copy_from_slice(&second_hash[..CHECKSUM_LEN]);
    checksum
}

// ── Address parsing ──────────────────────────────────────────────────────────

/// Parse and validate a .cloak address, returning the embedded public key.
pub fn parse_address(addr: &str) -> CloakResult<[u8; 32]> {
    // 1. Strip suffix
    let host = addr
        .strip_suffix(".cloak")
        .ok_or_else(|| CloakError::InvalidAddress("missing .cloak suffix".into()))?;

    // 2. Reject obviously malformed inputs before decoding
    if host.is_empty() {
        return Err(CloakError::InvalidAddress("empty host part".into()));
    }
    // base32 alphabet: a-z, 2-7 (lowercase after our encoding)
    if !host.chars().all(|c| matches!(c, 'a'..='z' | '2'..='7')) {
        return Err(CloakError::InvalidAddress(
            "address contains characters outside RFC4648 base32 alphabet".into(),
        ));
    }

    // 3. Decode
    let bytes = base32::decode(
        base32::Alphabet::RFC4648 { padding: false },
        &host.to_uppercase(),
    )
    .ok_or_else(|| CloakError::InvalidAddress("base32 decode failed".into()))?;

    // 4. Length check
    if bytes.len() != PAYLOAD_LEN {
        return Err(CloakError::InvalidAddress(format!(
            "decoded payload is {} bytes, expected {}",
            bytes.len(),
            PAYLOAD_LEN
        )));
    }

    // 5. Version check
    if bytes[0] != VERSION_BYTE {
        return Err(CloakError::UnsupportedAddressVersion(bytes[0]));
    }

    // 6. Extract pubkey
    let pubkey: [u8; 32] = bytes[1..33].try_into().unwrap();

    // 7. Checksum verification (constant-time comparison)
    let expected = compute_checksum(&pubkey);
    let actual = &bytes[33..];
    use subtle::ConstantTimeEq;
    if expected.ct_eq(actual).unwrap_u8() != 1 {
        return Err(CloakError::AddressChecksum);
    }

    Ok(pubkey)
}

/// Validate a .cloak address string without returning the pubkey.
pub fn validate_address(addr: &str) -> CloakResult<()> {
    parse_address(addr).map(|_| ())
}

// ── CloakAddressRef ──────────────────────────────────────────────────────────

/// A lightweight reference wrapper that can derive a CloakAddress on demand
/// without allocating until needed.
pub struct CloakAddressRef<'a>(pub &'a [u8; 32]);

impl<'a> CloakAddressRef<'a> {
    pub fn to_address(&self) -> CloakAddress {
        derive_address(self.0)
    }
}

impl<'a> std::fmt::Display for CloakAddressRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_address())
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::Ed25519KeyPair;

    #[test]
    fn derive_and_parse_roundtrip() {
        let kp = Ed25519KeyPair::generate();
        let pk = kp.public_key_bytes();
        let addr = derive_address(&pk);
        let recovered = parse_address(addr.as_str()).unwrap();
        assert_eq!(pk, recovered);
    }

    #[test]
    fn address_ends_with_cloak() {
        let pk = [0x42u8; 32];
        let addr = derive_address(&pk);
        assert!(addr.as_str().ends_with(".cloak"));
    }

    #[test]
    fn missing_suffix_rejected() {
        let pk = [0x01u8; 32];
        let addr = derive_address(&pk);
        let without_suffix = addr.as_str().strip_suffix(".cloak").unwrap();
        assert!(parse_address(without_suffix).is_err());
    }

    #[test]
    fn tampered_checksum_rejected() {
        let pk = [0x01u8; 32];
        let addr = derive_address(&pk);
        // Flip a character in the middle of the base32 part to alter the payload
        let s = addr.as_str();
        let dot = s.rfind('.').unwrap();
        let mut chars: Vec<char> = s[..dot].chars().collect();
        let middle = chars.len() / 2;
        chars[middle] = if chars[middle] == 'a' { 'b' } else { 'a' };
        let tampered = format!("{}.cloak", chars.iter().collect::<String>());
        assert!(parse_address(&tampered).is_err());
    }

    #[test]
    fn wrong_pubkey_checksum_mismatch() {
        let pk1 = [0x01u8; 32];
        let pk2 = [0x02u8; 32];
        // Manually construct an address with pk2's data but pk1's checksum
        let checksum = compute_checksum(&pk1);
        let mut payload = [0u8; PAYLOAD_LEN];
        payload[0] = VERSION_BYTE;
        payload[1..33].copy_from_slice(&pk2);
        payload[33..].copy_from_slice(&checksum);
        let encoded = base32::encode(base32::Alphabet::RFC4648 { padding: false }, &payload)
            .to_lowercase();
        let bad_addr = format!("{}.cloak", encoded);
        assert!(parse_address(&bad_addr).is_err());
    }

    #[test]
    fn invalid_characters_rejected() {
        assert!(parse_address("abc!def.cloak").is_err());
        assert!(parse_address("abc def.cloak").is_err());
    }

    #[test]
    fn deterministic_across_calls() {
        let pk = [0x77u8; 32];
        assert_eq!(derive_address(&pk).as_str(), derive_address(&pk).as_str());
    }
}
