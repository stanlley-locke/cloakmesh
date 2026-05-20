//! INTRODUCE1/2 message construction and validation.
//!
//! Act 7: Client sends INTRODUCE1 to an introduction point.
//! Act 8: Introduction point forwards to service; service sends INTRODUCE2 to RP.
//!
//! The INTRODUCE1 payload is encrypted for the service using the hybrid PQ
//! public key from the descriptor, so the introduction point learns nothing
//! about the rendezvous details.

use serde::{Deserialize, Serialize};

use crate::cloak_protocol::capability::CapabilityToken;
use crate::crypto::x25519::HybridKeyPair;
use crate::errors::{CloakError, CloakResult};
use crate::types::{CircuitId, RendezvousCookie};

// ── INTRODUCE1 ───────────────────────────────────────────────────────────────

/// The plaintext payload inside an INTRODUCE1 message (encrypted for the service).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Introduce1Payload {
    /// Rendezvous point address (host:port)
    pub rp_address: String,
    /// Rendezvous cookie (32 bytes)
    pub cookie: RendezvousCookie,
    /// Client's ephemeral X25519 public key for the RP-side session
    pub client_ephemeral_pubkey: [u8; 32],
    /// Optional capability token
    pub capability_token: Option<CapabilityToken>,
    /// Client-chosen circuit ID for the rendezvous leg
    pub circuit_id: CircuitId,
}

/// INTRODUCE1 message sent from client to introduction point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Introduce1Message {
    /// ID of the introduction point being used
    pub intro_point_id: String,
    /// Encrypted payload (for the service, opaque to the IP)
    pub encrypted_payload: Vec<u8>,
    /// Ephemeral public key used for the hybrid encryption (X25519 part)
    pub sender_ephemeral_pubkey: Vec<u8>,
}

impl Introduce1Message {
    /// Construct and encrypt an INTRODUCE1 message.
    pub fn build(
        intro_point_id: String,
        payload: Introduce1Payload,
        service_hybrid_pubkey: &[u8],
    ) -> CloakResult<Self> {
        // Serialize the payload
        let payload_bytes = serde_json::to_vec(&payload)
            .map_err(|e| CloakError::Protocol(format!("serialize introduce1 payload: {e}")))?;

        // Encrypt using hybrid PQ encapsulation
        let (ciphertext, shared_secret) = HybridKeyPair::encapsulate(service_hybrid_pubkey)?;

        // Derive an encryption key from the shared secret
        let enc_key = crate::crypto::kdf::derive_key(
            b"cloakmesh-introduce1-v1",
            &*shared_secret,
            b"payload-enc",
        )?;

        // Encrypt the payload with ChaCha20-Poly1305
        use chacha20poly1305::{aead::{Aead, KeyInit, Payload as AeadPayload}, ChaCha20Poly1305, Nonce};
        let cipher = ChaCha20Poly1305::new_from_slice(&*enc_key)
            .map_err(|_| CloakError::AeadEncrypt)?;
        let nonce = Nonce::from_slice(&[0u8; 12]); // single-use nonce (key is ephemeral)
        let encrypted_payload = cipher
            .encrypt(nonce, AeadPayload { msg: &payload_bytes, aad: intro_point_id.as_bytes() })
            .map_err(|_| CloakError::AeadEncrypt)?;

        Ok(Self {
            intro_point_id,
            encrypted_payload,
            sender_ephemeral_pubkey: ciphertext,
        })
    }

    /// Decrypt and deserialize the payload using the service's hybrid key pair.
    pub fn decrypt_payload(
        &self,
        service_hybrid_keypair: &HybridKeyPair,
    ) -> CloakResult<Introduce1Payload> {
        let shared_secret = service_hybrid_keypair.decapsulate(&self.sender_ephemeral_pubkey)?;

        let enc_key = crate::crypto::kdf::derive_key(
            b"cloakmesh-introduce1-v1",
            &*shared_secret,
            b"payload-enc",
        )?;

        use chacha20poly1305::{aead::{Aead, KeyInit, Payload as AeadPayload}, ChaCha20Poly1305, Nonce};
        let cipher = ChaCha20Poly1305::new_from_slice(&*enc_key)
            .map_err(|_| CloakError::AeadDecrypt)?;
        let nonce = Nonce::from_slice(&[0u8; 12]);
        let plaintext = cipher
            .decrypt(nonce, AeadPayload {
                msg: &self.encrypted_payload,
                aad: self.intro_point_id.as_bytes(),
            })
            .map_err(|_| CloakError::AeadDecrypt)?;

        serde_json::from_slice(&plaintext)
            .map_err(|e| CloakError::Protocol(format!("deserialize introduce1 payload: {e}")))
    }
}

// ── INTRODUCE2 ───────────────────────────────────────────────────────────────

/// INTRODUCE2 message sent from service to rendezvous point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Introduce2Message {
    /// Rendezvous cookie (must match what the client sent to the RP)
    pub cookie: RendezvousCookie,
    /// Service's ephemeral X25519 public key for the session key exchange
    pub service_ephemeral_pubkey: [u8; 32],
    /// Circuit ID for the service-side leg
    pub circuit_id: CircuitId,
    /// Encrypted session key material for the client
    pub session_key_material: Vec<u8>,
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::x25519::HybridKeyPair;
    use crate::types::{CircuitId, RendezvousCookie};

    #[test]
    fn introduce1_encrypt_decrypt_roundtrip() {
        let service_hybrid = HybridKeyPair::generate();
        let service_pk = service_hybrid.combined_public_bytes();

        let cookie = RendezvousCookie::generate();
        let circuit_id = CircuitId::generate();

        let payload = Introduce1Payload {
            rp_address: "127.0.0.1:5001".into(),
            cookie: cookie.clone(),
            client_ephemeral_pubkey: [0x42u8; 32],
            capability_token: None,
            circuit_id: circuit_id.clone(),
        };

        let msg = Introduce1Message::build(
            "intro-point-1".into(),
            payload,
            &service_pk,
        ).unwrap();

        let recovered = msg.decrypt_payload(&service_hybrid).unwrap();
        assert_eq!(recovered.rp_address, "127.0.0.1:5001");
        assert!(recovered.cookie.ct_eq(&cookie));
        assert_eq!(recovered.circuit_id, circuit_id);
    }

    #[test]
    fn tampered_ciphertext_fails_decryption() {
        let service_hybrid = HybridKeyPair::generate();
        let service_pk = service_hybrid.combined_public_bytes();

        let payload = Introduce1Payload {
            rp_address: "127.0.0.1:5001".into(),
            cookie: RendezvousCookie::generate(),
            client_ephemeral_pubkey: [0u8; 32],
            capability_token: None,
            circuit_id: CircuitId::generate(),
        };

        let mut msg = Introduce1Message::build("ip-1".into(), payload, &service_pk).unwrap();
        // Flip a byte in the encrypted payload
        let last = msg.encrypted_payload.len() - 1;
        msg.encrypted_payload[last] ^= 0xFF;

        assert!(msg.decrypt_payload(&service_hybrid).is_err());
    }
}
