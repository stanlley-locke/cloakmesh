use blake2::{Blake2s256, Digest as _};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use zeroize::Zeroizing;

use crate::errors::{CloakError, CloakResult};

type HmacSha256 = Hmac<Sha256>;

// ── HKDF-SHA256 ──────────────────────────────────────────────────────────────

/// HKDF-Extract: `PRK = HMAC-SHA256(salt, ikm)`
pub fn hkdf_extract(salt: &[u8], ikm: &[u8]) -> Zeroizing<[u8; 32]> {
    let mut mac = HmacSha256::new_from_slice(salt)
        .expect("HMAC accepts any key length");
    mac.update(ikm);
    let result = mac.finalize().into_bytes();
    let mut out = Zeroizing::new([0u8; 32]);
    out.copy_from_slice(&result);
    out
}

/// HKDF-Expand: derive `length` bytes from `prk` with `info` label.
/// Returns exactly 32 bytes (one HMAC block).
pub fn hkdf_expand(prk: &[u8], info: &[u8], context: &[u8]) -> CloakResult<Zeroizing<[u8; 32]>> {
    // T(1) = HMAC-SHA256(PRK, info || context || 0x01)
    let mut mac = HmacSha256::new_from_slice(prk)
        .map_err(|e| CloakError::Kdf(e.to_string()))?;
    mac.update(info);
    mac.update(context);
    mac.update(&[0x01u8]);
    let result = mac.finalize().into_bytes();
    let mut out = Zeroizing::new([0u8; 32]);
    out.copy_from_slice(&result);
    Ok(out)
}

/// Full HKDF: extract then expand.
pub fn derive_key(salt: &[u8], ikm: &[u8], info: &[u8]) -> CloakResult<Zeroizing<[u8; 32]>> {
    let prk = hkdf_extract(salt, ikm);
    hkdf_expand(&*prk, info, b"")
}

/// Derive two independent 32-byte keys from one shared secret (e.g. for
/// send/receive key split in a session).
pub fn derive_key_pair(
    shared_secret: &[u8],
    info_a: &[u8],
    info_b: &[u8],
) -> CloakResult<(Zeroizing<[u8; 32]>, Zeroizing<[u8; 32]>)> {
    let salt = b"cloakmesh-kdf-v1";
    let prk = hkdf_extract(salt, shared_secret);
    let ka = hkdf_expand(&*prk, info_a, b"")?;
    let kb = hkdf_expand(&*prk, info_b, b"")?;
    Ok((ka, kb))
}

// ── BLAKE2s-256 (used in Noise handshake) ────────────────────────────────────

/// Hash data with BLAKE2s-256, returning 32 bytes.
pub fn blake2s_hash(data: &[u8]) -> [u8; 32] {
    let mut h = Blake2s256::new();
    h.update(data);
    h.finalize().into()
}

/// BLAKE2s-based HMAC-like MixHash for Noise protocol state.
pub fn noise_mix_hash(state: &[u8; 32], data: &[u8]) -> [u8; 32] {
    let mut h = Blake2s256::new();
    h.update(state);
    h.update(data);
    h.finalize().into()
}

/// Noise MixKey: derive new chaining key and temp key from DH output.
/// Returns `(new_ck, temp_k)` per the Noise spec.
pub fn noise_mix_key(
    chaining_key: &[u8; 32],
    dh_output: &[u8],
) -> CloakResult<([u8; 32], Zeroizing<[u8; 32]>)> {
    // HKDF(ck, dh_output) → (new_ck, temp_k)
    let prk = hkdf_extract(chaining_key, dh_output);
    let new_ck_z = hkdf_expand(&*prk, b"", &[0x01])?;
    let temp_k_z = hkdf_expand(&*prk, &*new_ck_z, &[0x02])?;
    let mut new_ck = [0u8; 32];
    new_ck.copy_from_slice(&*new_ck_z);
    Ok((new_ck, temp_k_z))
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hkdf_deterministic() {
        let k1 = derive_key(b"salt", b"ikm", b"info").unwrap();
        let k2 = derive_key(b"salt", b"ikm", b"info").unwrap();
        assert_eq!(*k1, *k2);
    }

    #[test]
    fn hkdf_different_info_produces_different_keys() {
        let k1 = derive_key(b"salt", b"ikm", b"info-a").unwrap();
        let k2 = derive_key(b"salt", b"ikm", b"info-b").unwrap();
        assert_ne!(*k1, *k2);
    }

    #[test]
    fn key_pair_keys_are_independent() {
        let (ka, kb) = derive_key_pair(b"shared", b"send", b"recv").unwrap();
        assert_ne!(*ka, *kb);
    }

    #[test]
    fn blake2s_hash_deterministic() {
        assert_eq!(blake2s_hash(b"test"), blake2s_hash(b"test"));
        assert_ne!(blake2s_hash(b"test"), blake2s_hash(b"other"));
    }

    #[test]
    fn noise_mix_key_produces_two_distinct_outputs() {
        let ck = [1u8; 32];
        let (new_ck, temp_k) = noise_mix_key(&ck, b"dh_output").unwrap();
        assert_ne!(new_ck, *temp_k);
        assert_ne!(new_ck, ck);
    }
}
