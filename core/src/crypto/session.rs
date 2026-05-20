use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Nonce,
};
use std::time::{Duration, Instant};
use zeroize::Zeroizing;

use crate::errors::{CloakError, CloakResult};
use crate::types::CellNonce;

// ── Session Key ──────────────────────────────────────────────────────────────

/// A single-use symmetric key for AEAD encryption of cells.
pub struct SessionKey {
    pub raw: Zeroizing<[u8; 32]>,
    pub cipher: ChaCha20Poly1305,
    pub nonce: CellNonce,
    pub policy: RotationPolicy,
    pub created_at: Instant,
}

impl SessionKey {
    pub fn new(raw: [u8; 32], circuit_prefix: [u8; 4], policy: RotationPolicy) -> CloakResult<Self> {
        let cipher = ChaCha20Poly1305::new_from_slice(&raw)
            .map_err(|_| CloakError::InvalidKeyMaterial("AEAD init failed".into()))?;
        Ok(Self {
            raw: Zeroizing::new(raw),
            cipher,
            nonce: CellNonce::new(circuit_prefix),
            policy,
            created_at: Instant::now(),
        })
    }

    pub fn encrypt(&mut self, plaintext: &[u8], aad: &[u8]) -> CloakResult<Vec<u8>> {
        let nonce_bytes = self.nonce.next()
            .ok_or_else(|| CloakError::NonceExhausted)?;
        
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = self.cipher
            .encrypt(nonce, Payload { msg: plaintext, aad })
            .map_err(|_| CloakError::AeadEncrypt)?;

        Ok(ciphertext)
    }

    pub fn decrypt(&self, nonce_bytes: &[u8; 12], ciphertext: &[u8], aad: &[u8]) -> CloakResult<Vec<u8>> {
        let nonce = Nonce::from_slice(nonce_bytes);
        self.cipher
            .decrypt(nonce, Payload { msg: ciphertext, aad })
            .map_err(|_| CloakError::AeadDecrypt)
    }

    pub fn needs_rotation(&self) -> bool {
        self.nonce.current_count() >= self.policy.max_cells || self.created_at.elapsed() >= self.policy.max_age
    }

    pub fn rotate(&self, circuit_prefix: [u8; 4]) -> CloakResult<Self> {
        let mut next_raw = [0u8; 32];
        for i in 0..32 { next_raw[i] = self.raw[i] ^ 0xFF; }
        Self::new(next_raw, circuit_prefix, self.policy.clone())
    }
}

// ── Rotation Policy ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RotationPolicy {
    pub max_cells: u64,
    pub max_age: Duration,
}

impl Default for RotationPolicy {
    fn default() -> Self {
        Self {
            max_cells: 100_000,
            max_age: Duration::from_secs(3600),
        }
    }
}

// ── Bidirectional Session ────────────────────────────────────────────────────

/// A pair of session keys: one for sending, one for receiving.
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
        self.send.needs_rotation() || self.recv.needs_rotation()
    }

    pub fn generate_mock() -> Self {
        let policy = RotationPolicy::default();
        Self::new([0u8; 32], [1u8; 32], [0u8; 4], policy).unwrap()
    }

    pub fn rotate(&self, circuit_prefix: [u8; 4]) -> CloakResult<Self> {
        Ok(Self {
            send: self.send.rotate(circuit_prefix)?,
            recv: self.recv.rotate(circuit_prefix)?,
        })
    }

    pub fn encrypt(&mut self, plaintext: &[u8], aad: &[u8]) -> CloakResult<Vec<u8>> {
        self.send.encrypt(plaintext, aad)
    }

    pub fn decrypt(&self, nonce_bytes: &[u8; 12], ciphertext: &[u8], aad: &[u8]) -> CloakResult<Vec<u8>> {
        self.recv.decrypt(nonce_bytes, ciphertext, aad)
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
        let ct = key.encrypt(plaintext, aad).unwrap();
        // Since we don't return the nonce in encrypt for simplicity here (demo),
        // we'd need it for decryption. In real cells, it's in the header.
        // For the test, we know it's prefix + counter=1
        let mut nonce = [0u8; 12];
        nonce[..4].copy_from_slice(&[0x01, 0x02, 0x03, 0x04]);
        nonce[4] = 1; // Counter starts at 1, little-endian
        let pt = key.decrypt(&nonce, &ct, aad).unwrap();
        assert_eq!(pt, plaintext);
    }
}
