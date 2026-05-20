use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Nonce,
};
use std::time::{Duration, Instant};
use zeroize::Zeroizing;

use crate::errors::{CloakError, CloakResult};
use crate::types::CellNonce;

// ── Rotation policy ──────────────────────────────────────────────────────────

/// Session key rotation triggers: whichever threshold is hit first.
#[derive(Debug, Clone)]
pub struct RotationPolicy {
    /// Rotate after this many cells encrypted.
    pub max_cells: u64,
    /// Rotate after this duration.
    pub max_age: Duration,
}

impl Default for RotationPolicy {
    fn default() -> Self {
        Self {
            max_cells: 10_000,
            max_age: Duration::from_secs(300),
        }
    }
}

// ── Session Key ──────────────────────────────────────────────────────────────

/// A ChaCha20-Poly1305 session key with integrated nonce management and
/// automatic rotation signalling.
///
/// Security properties:
/// - Nonces are never reused: a 64-bit counter with a 32-bit circuit prefix
///   guarantees uniqueness for the lifetime of the key.
/// - The key material is zeroed on drop.
/// - Rotation is signalled (not automatic) so the caller can coordinate
///   key exchange before the old key is discarded.
pub struct SessionKey {
    cipher: ChaCha20Poly1305,
    nonce_gen: CellNonce,
    cells_encrypted: u64,
    cells_decrypted: u64,
    created_at: Instant,
    policy: RotationPolicy,
    /// Raw key bytes kept for re-derivation during rotation.
    raw: Zeroizing<[u8; 32]>,
}

impl SessionKey {
    /// Create a new session key from 32 raw bytes.
    /// `circuit_prefix` is the first 4 bytes of the circuit ID, used to
    /// namespace nonces so that two sessions with the same key (impossible in
    /// practice, but defensive) cannot share nonces.
    pub fn new(
        key_bytes: [u8; 32],
        circuit_prefix: [u8; 4],
        policy: RotationPolicy,
    ) -> CloakResult<Self> {
        let cipher = ChaCha20Poly1305::new_from_slice(&key_bytes)
            .map_err(|_| CloakError::InvalidKeyMaterial("invalid ChaCha20 key length".into()))?;
        Ok(Self {
            cipher,
            nonce_gen: CellNonce::new(circuit_prefix),
            cells_encrypted: 0,
            cells_decrypted: 0,
            created_at: Instant::now(),
            policy,
            raw: Zeroizing::new(key_bytes),
        })
    }

    /// Encrypt `plaintext` with the next nonce. `aad` is additional
    /// authenticated data (e.g. circuit ID + sequence number).
    ///
    /// Returns `(nonce_bytes, ciphertext_with_tag)`.
    pub fn encrypt(&mut self, plaintext: &[u8], aad: &[u8]) -> CloakResult<([u8; 12], Vec<u8>)> {
        let nonce_bytes = self
            .nonce_gen
            .next()
            .ok_or(CloakError::NonceExhausted)?;
        let ct = self
            .cipher
            .encrypt(
                Nonce::from_slice(&nonce_bytes),
                Payload { msg: plaintext, aad },
            )
            .map_err(|_| CloakError::AeadEncrypt)?;
        self.cells_encrypted += 1;
        Ok((nonce_bytes, ct))
    }

    /// Decrypt `ciphertext` (which includes the 16-byte Poly1305 tag).
    /// `nonce` must be the exact nonce used during encryption.
    /// `aad` must match what was passed to `encrypt`.
    pub fn decrypt(&mut self, nonce: &[u8; 12], ciphertext: &[u8], aad: &[u8]) -> CloakResult<Vec<u8>> {
        let pt = self
            .cipher
            .decrypt(
                Nonce::from_slice(nonce),
                Payload { msg: ciphertext, aad },
            )
            .map_err(|_| CloakError::AeadDecrypt)?;
        self.cells_decrypted += 1;
        Ok(pt)
    }

    /// Returns true if this key should be rotated.
    pub fn needs_rotation(&self) -> bool {
        self.cells_encrypted >= self.policy.max_cells
            || self.created_at.elapsed() >= self.policy.max_age
    }

    pub fn cells_encrypted(&self) -> u64 {
        self.cells_encrypted
    }

    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Derive the next session key from this one using HKDF.
    /// The old key's raw bytes are used as IKM so forward secrecy is maintained
    /// (the new key cannot be derived without the old one, and the old one is
    /// zeroed after rotation).
    pub fn rotate(&self, circuit_prefix: [u8; 4]) -> CloakResult<SessionKey> {
        let new_key = crate::crypto::kdf::derive_key(
            b"cloakmesh-session-rotation-v1",
            &*self.raw,
            b"next-session-key",
        )?;
        SessionKey::new(*new_key, circuit_prefix, self.policy.clone())
    }
}

// ── Bidirectional session ────────────────────────────────────────────────────

/// A pair of session keys: one for sending, one for receiving.
/// This models the split transport state after a Noise handshake.
pub struct BidirectionalSession {
    pub send: SessionKey,
    pub recv: SessionKey,
}

impl BidirectionalSession {
    pub fn new(
        send_key: [u8; 32],
        recv_key: [u8; 32],
        circuit_prefix: [u8; 4],
        policy: RotationPolicy,
    ) -> CloakResult<Self> {
        Ok(Self {
            send: SessionKey::new(send_key, circuit_prefix, policy.clone())?,
            recv: SessionKey::new(recv_key, circuit_prefix, policy)?,
        })
    }

    pub fn from_noise_result(
        result: crate::crypto::noise::HandshakeResult,
        circuit_prefix: [u8; 4],
        policy: RotationPolicy,
    ) -> CloakResult<Self> {
        Self::new(*result.send_key, *result.recv_key, circuit_prefix, policy)
    }

    pub fn needs_rotation(&self) -> bool {
        self.send_key.needs_rotation() || self.recv_key.needs_rotation()
    }

    /// Generates a mock session for testing and Phase 2 demonstration.
    pub fn generate_mock() -> Self {
        let policy = RotationPolicy::default();
        Self::new([0u8; 32], [1u8; 32], policy).unwrap()
    }
    }

    pub fn rotate(&self, circuit_prefix: [u8; 4]) -> CloakResult<Self> {
        Ok(Self {
            send: self.send.rotate(circuit_prefix)?,
            recv: self.recv.rotate(circuit_prefix)?,
        })
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_key() -> SessionKey {
        let key = [0x42u8; 32];
        let prefix = [0x01, 0x02, 0x03, 0x04];
        SessionKey::new(key, prefix, RotationPolicy::default()).unwrap()
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let mut key = make_key();
        let plaintext = b"hello cloakmesh";
        let aad = b"circuit-id-aad";
        let (nonce, ct) = key.encrypt(plaintext, aad).unwrap();
        let pt = key.decrypt(&nonce, &ct, aad).unwrap();
        assert_eq!(pt, plaintext);
    }

    #[test]
    fn wrong_aad_fails_decryption() {
        let mut key = make_key();
        let (nonce, ct) = key.encrypt(b"data", b"correct-aad").unwrap();
        assert!(key.decrypt(&nonce, &ct, b"wrong-aad").is_err());
    }

    #[test]
    fn tampered_ciphertext_fails_decryption() {
        let mut key = make_key();
        let (nonce, mut ct) = key.encrypt(b"data", b"aad").unwrap();
        ct[0] ^= 0xFF;
        assert!(key.decrypt(&nonce, &ct, b"aad").is_err());
    }

    #[test]
    fn nonces_are_unique_per_call() {
        let mut key = make_key();
        let (n1, _) = key.encrypt(b"a", b"").unwrap();
        let (n2, _) = key.encrypt(b"b", b"").unwrap();
        assert_ne!(n1, n2);
    }

    #[test]
    fn rotation_produces_different_key() {
        let key = make_key();
        let rotated = key.rotate([0x01, 0x02, 0x03, 0x04]).unwrap();
        assert_ne!(*key.raw, *rotated.raw);
    }

    #[test]
    fn rotation_triggered_by_cell_count() {
        let policy = RotationPolicy { max_cells: 3, max_age: Duration::from_secs(9999) };
        let mut key = SessionKey::new([0u8; 32], [0u8; 4], policy).unwrap();
        assert!(!key.needs_rotation());
        for _ in 0..3 {
            key.encrypt(b"x", b"").unwrap();
        }
        assert!(key.needs_rotation());
    }
}
