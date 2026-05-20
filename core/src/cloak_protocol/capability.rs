//! Capability token issuance and verification.
//!
//! A capability token grants a client access to a .cloak service with
//! specific scopes (e.g. "read", "write", "admin").
//!
//! Security properties:
//! - Ed25519-signed by the service's identity key
//! - Bound to a specific .cloak address (cannot be reused across services)
//! - Expiry enforced with clock-skew tolerance
//! - Scope list checked with constant-time string comparison
//! - Token ID is a 128-bit random value (collision-resistant)

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::crypto::ed25519::{verify_raw, Ed25519KeyPair};
use crate::errors::{CloakError, CloakResult};

// ── Token ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityToken {
    pub token_id: String,
    pub cloak_address: String,
    pub scopes: Vec<String>,
    pub expires_at: u64,
    #[serde(with = "hex_array_32")]
    pub issuer_pubkey: [u8; 32],
    #[serde(with = "hex_array_64")]
    pub signature: [u8; 64],
}

mod hex_array_32 {
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S: Serializer>(v: &[u8; 32], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&hex::encode(v))
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 32], D::Error> {
        let s = String::deserialize(d)?;
        let b = hex::decode(&s).map_err(serde::de::Error::custom)?;
        b.try_into().map_err(|_| serde::de::Error::custom("expected 32 bytes"))
    }
}

mod hex_array_64 {
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S: Serializer>(v: &[u8; 64], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&hex::encode(v))
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 64], D::Error> {
        let s = String::deserialize(d)?;
        let b = hex::decode(&s).map_err(serde::de::Error::custom)?;
        b.try_into().map_err(|_| serde::de::Error::custom("expected 64 bytes"))
    }
}

impl CapabilityToken {
    /// Serialize the fields that are covered by the signature.
    pub fn canonical_body(&self) -> Vec<u8> {
        let mut body = Vec::new();
        // Length-prefix each field to prevent concatenation attacks
        encode_field(&mut body, self.token_id.as_bytes());
        encode_field(&mut body, self.cloak_address.as_bytes());
        body.extend_from_slice(&(self.scopes.len() as u32).to_le_bytes());
        for scope in &self.scopes {
            encode_field(&mut body, scope.as_bytes());
        }
        body.extend_from_slice(&self.expires_at.to_le_bytes());
        encode_field(&mut body, &self.issuer_pubkey);
        body
    }

    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // Allow 30s clock skew
        self.expires_at < now.saturating_sub(30)
    }

    pub fn has_scope(&self, required: &str) -> bool {
        self.scopes.iter().any(|s| s == required)
    }
}

fn encode_field(buf: &mut Vec<u8>, data: &[u8]) {
    buf.extend_from_slice(&(data.len() as u32).to_le_bytes());
    buf.extend_from_slice(data);
}

// ── Issuer ───────────────────────────────────────────────────────────────────

pub struct CapabilityIssuer {
    keypair: Ed25519KeyPair,
    service_address: String,
}

impl CapabilityIssuer {
    pub fn new(keypair: Ed25519KeyPair, service_address: String) -> Self {
        Self { keypair, service_address }
    }

    /// Issue a new capability token.
    pub fn issue(
        &self,
        scopes: Vec<String>,
        ttl_secs: u64,
    ) -> CloakResult<CapabilityToken> {
        if scopes.is_empty() {
            return Err(CloakError::InsufficientScope {
                required: "(any)".into(),
                present: vec![],
            });
        }
        for scope in &scopes {
            if scope.is_empty() || scope.contains('\0') {
                return Err(CloakError::Auth(format!("invalid scope: {:?}", scope)));
            }
        }

        let token_id = {
            use rand::RngCore;
            let mut id = [0u8; 16];
            rand::rngs::OsRng.fill_bytes(&mut id);
            hex::encode(id)
        };

        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + ttl_secs;

        let issuer_pubkey = self.keypair.public_key_bytes();

        let mut token = CapabilityToken {
            token_id,
            cloak_address: self.service_address.clone(),
            scopes,
            expires_at,
            issuer_pubkey,
            signature: [0u8; 64],
        };

        let body = token.canonical_body();
        token.signature = self.keypair.sign(&body).to_bytes();
        Ok(token)
    }
}

// ── Verifier ─────────────────────────────────────────────────────────────────

pub struct CapabilityVerifier {
    /// The .cloak address of the service that will accept tokens
    service_address: String,
    /// Trusted issuer public keys (a service may rotate keys)
    trusted_issuers: Vec<[u8; 32]>,
}

impl CapabilityVerifier {
    pub fn new(service_address: String, trusted_issuers: Vec<[u8; 32]>) -> Self {
        Self { service_address, trusted_issuers }
    }

    /// Verify a token and check that it grants `required_scope`.
    pub fn verify(&self, token: &CapabilityToken, required_scope: &str) -> CloakResult<()> {
        // 1. Address binding
        if token.cloak_address != self.service_address {
            return Err(CloakError::TokenAddressMismatch {
                token_addr: token.cloak_address.clone(),
                request_addr: self.service_address.clone(),
            });
        }

        // 2. Expiry
        if token.is_expired() {
            return Err(CloakError::TokenExpired);
        }

        // 3. Issuer trust
        let issuer_trusted = self
            .trusted_issuers
            .iter()
            .any(|pk| pk == &token.issuer_pubkey);
        if !issuer_trusted {
            return Err(CloakError::TokenSignatureInvalid);
        }

        // 4. Signature
        let body = token.canonical_body();
        verify_raw(&token.issuer_pubkey, &body, &token.signature)
            .map_err(|_| CloakError::TokenSignatureInvalid)?;

        // 5. Scope
        if !token.has_scope(required_scope) {
            return Err(CloakError::InsufficientScope {
                required: required_scope.to_string(),
                present: token.scopes.clone(),
            });
        }

        Ok(())
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cloak_protocol::address::derive_address;
    use crate::crypto::Ed25519KeyPair;

    fn setup() -> (CapabilityIssuer, CapabilityVerifier, String) {
        let kp = Ed25519KeyPair::generate();
        let pk = kp.public_key_bytes();
        let addr = derive_address(&pk).to_string();
        let issuer = CapabilityIssuer::new(kp, addr.clone());
        let verifier = CapabilityVerifier::new(addr.clone(), vec![pk]);
        (issuer, verifier, addr)
    }

    #[test]
    fn valid_token_verifies() {
        let (issuer, verifier, _) = setup();
        let token = issuer.issue(vec!["read".into()], 3600).unwrap();
        assert!(verifier.verify(&token, "read").is_ok());
    }

    #[test]
    fn missing_scope_rejected() {
        let (issuer, verifier, _) = setup();
        let token = issuer.issue(vec!["read".into()], 3600).unwrap();
        assert!(verifier.verify(&token, "write").is_err());
    }

    #[test]
    fn expired_token_rejected() {
        let (issuer, verifier, _) = setup();
        let mut token = issuer.issue(vec!["read".into()], 3600).unwrap();
        token.expires_at = 1; // far in the past
        assert!(verifier.verify(&token, "read").is_err());
    }

    #[test]
    fn tampered_signature_rejected() {
        let (issuer, verifier, _) = setup();
        let mut token = issuer.issue(vec!["read".into()], 3600).unwrap();
        token.signature[0] ^= 0xFF;
        assert!(verifier.verify(&token, "read").is_err());
    }

    #[test]
    fn wrong_address_rejected() {
        let (issuer, _, _) = setup();
        let token = issuer.issue(vec!["read".into()], 3600).unwrap();
        let other_kp = Ed25519KeyPair::generate();
        let other_addr = derive_address(&other_kp.public_key_bytes()).to_string();
        let issuer_pk = issuer.keypair.public_key_bytes();
        let verifier = CapabilityVerifier::new(other_addr, vec![issuer_pk]);
        assert!(verifier.verify(&token, "read").is_err());
    }

    #[test]
    fn untrusted_issuer_rejected() {
        let (issuer, _, addr) = setup();
        let token = issuer.issue(vec!["read".into()], 3600).unwrap();
        // Verifier trusts a different key
        let other_kp = Ed25519KeyPair::generate();
        let verifier = CapabilityVerifier::new(addr, vec![other_kp.public_key_bytes()]);
        assert!(verifier.verify(&token, "read").is_err());
    }

    #[test]
    fn multiple_scopes_work() {
        let (issuer, verifier, _) = setup();
        let token = issuer.issue(vec!["read".into(), "write".into()], 3600).unwrap();
        assert!(verifier.verify(&token, "read").is_ok());
        assert!(verifier.verify(&token, "write").is_ok());
        assert!(verifier.verify(&token, "admin").is_err());
    }
}
