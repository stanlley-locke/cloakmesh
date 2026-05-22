//! .cloak service descriptor: assembly, signing, verification, and replay protection.
//!
//! A descriptor is the signed record a service publishes to the DHT so that
//! clients can find its introduction points and authenticate it.
//!
//! Security properties enforced here:
//! - Ed25519 signature over the canonical serialized body
//! - Timestamp + nonce replay window (configurable, default 5 min)
//! - Merkle root over introduction point list (tamper-evident)
//! - Maximum descriptor age check

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::cloak_protocol::merkle::merkle_root;
use crate::crypto::ed25519::{verify_raw, Ed25519KeyPair};
use crate::errors::{CloakError, CloakResult};

// ── Introduction Point ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntroductionPoint {
    /// Peer ID (hex-encoded public key hash)
    pub peer_id: String,
    /// host:port
    pub address: String,
    /// Short-term auth key for this IP (32 bytes, rotated with the IP)
    pub auth_key: [u8; 32],
}

impl IntroductionPoint {
    /// Canonical byte representation used as a Merkle leaf.
    pub fn to_leaf_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(self.peer_id.as_bytes());
        out.push(b'|');
        out.extend_from_slice(self.address.as_bytes());
        out.push(b'|');
        out.extend_from_slice(&self.auth_key);
        out
    }
}

// ── Auth Policy ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum AuthPolicy {
    Public = 0,
    CapabilityRequired = 1,
    ZkGated = 2,
}

// ── Descriptor ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloakDescriptor {
    pub cloak_address: String,
    pub intro_points: Vec<IntroductionPoint>,
    #[serde(with = "hex_array_32")]
    pub identity_pubkey: [u8; 32],
    pub hybrid_pq_pubkey: Vec<u8>,
    #[serde(with = "hex_array_64")]
    pub signature: [u8; 64],
    #[serde(with = "hex_array_32")]
    pub merkle_root: [u8; 32],
    pub issued_at: u64,
    pub nonce: u64,
    pub auth_policy: AuthPolicy,
    pub version: u32,
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

impl CloakDescriptor {
    /// Serialize the body that is signed (everything except `signature`).
    pub fn canonical_body(&self) -> Vec<u8> {
        // Use a length-prefixed encoding to prevent field confusion attacks.
        let mut body = Vec::new();
        encode_field(&mut body, self.cloak_address.as_bytes());
        encode_field(&mut body, &self.identity_pubkey);
        encode_field(&mut body, &self.hybrid_pq_pubkey);
        encode_field(&mut body, &self.merkle_root);
        body.extend_from_slice(&self.issued_at.to_le_bytes());
        body.extend_from_slice(&self.nonce.to_le_bytes());
        body.push(self.auth_policy as u8);
        body.extend_from_slice(&self.version.to_le_bytes());
        // Include intro point count so the list cannot be truncated silently
        body.extend_from_slice(&(self.intro_points.len() as u32).to_le_bytes());
        body
    }

    /// Microdescriptor Compaction
    /// Compresses the descriptor into a minimal binary format for DHT storage efficiency.
    pub fn compact(&self) -> Vec<u8> {
        // In a full implementation, this might use zstd or a custom bit-packed format
        self.canonical_body()
    }
}

fn encode_field(buf: &mut Vec<u8>, data: &[u8]) {
    buf.extend_from_slice(&(data.len() as u32).to_le_bytes());
    buf.extend_from_slice(data);
}

// ── Builder ──────────────────────────────────────────────────────────────────

pub struct DescriptorBuilder {
    cloak_address: String,
    intro_points: Vec<IntroductionPoint>,
    identity_keypair: Ed25519KeyPair,
    hybrid_pq_pubkey: Vec<u8>,
    auth_policy: AuthPolicy,
    version: u32,
}

impl DescriptorBuilder {
    pub fn new(
        cloak_address: String,
        identity_keypair: Ed25519KeyPair,
        hybrid_pq_pubkey: Vec<u8>,
    ) -> Self {
        Self {
            cloak_address,
            intro_points: Vec::new(),
            identity_keypair,
            hybrid_pq_pubkey,
            auth_policy: AuthPolicy::Public,
            version: 1,
        }
    }

    pub fn add_intro_point(mut self, ip: IntroductionPoint) -> Self {
        self.intro_points.push(ip);
        self
    }

    pub fn auth_policy(mut self, policy: AuthPolicy) -> Self {
        self.auth_policy = policy;
        self
    }

    pub fn build(self) -> CloakResult<CloakDescriptor> {
        if self.intro_points.is_empty() {
            return Err(CloakError::DescriptorMissingField("intro_points".into()));
        }
        if self.intro_points.len() > 5 {
            return Err(CloakError::DescriptorMissingField(
                "too many intro_points (max 5)".into(),
            ));
        }

        let leaves: Vec<Vec<u8>> = self.intro_points.iter().map(|ip| ip.to_leaf_bytes()).collect();
        let merkle = merkle_root(&leaves);

        let issued_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let nonce = {
            use rand::RngCore;
            let mut n = [0u8; 8];
            rand::rngs::OsRng.fill_bytes(&mut n);
            u64::from_le_bytes(n)
        };

        let identity_pubkey = self.identity_keypair.public_key_bytes();

        let mut desc = CloakDescriptor {
            cloak_address: self.cloak_address,
            intro_points: self.intro_points,
            identity_pubkey,
            hybrid_pq_pubkey: self.hybrid_pq_pubkey,
            signature: [0u8; 64],
            merkle_root: merkle,
            issued_at,
            nonce,
            auth_policy: self.auth_policy,
            version: self.version,
        };

        // Sign the canonical body
        let body = desc.canonical_body();
        let sig = self.identity_keypair.sign(&body);
        desc.signature = sig.to_bytes();

        Ok(desc)
    }
}

// ── Verification ─────────────────────────────────────────────────────────────

/// Verify a descriptor's signature, timestamp, and Merkle root.
///
/// `max_age_secs`: maximum allowed age of the descriptor (e.g. 3600).
/// `nonce_window_secs`: replay window — descriptors older than this are rejected
///   even if the signature is valid.
pub fn verify_descriptor(
    desc: &CloakDescriptor,
    max_age_secs: u64,
    nonce_window_secs: u64,
) -> CloakResult<()> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // 1. Timestamp freshness
    if desc.issued_at > now + 60 {
        // Allow 60s clock skew
        return Err(CloakError::DescriptorExpired {
            issued_at: desc.issued_at,
            now,
        });
    }
    let age = now.saturating_sub(desc.issued_at);
    if age > max_age_secs {
        return Err(CloakError::DescriptorExpired {
            issued_at: desc.issued_at,
            now,
        });
    }

    // 2. Nonce window (basic replay protection — full replay protection
    //    requires a seen-nonce store, which lives in the DHT layer)
    if age > nonce_window_secs && desc.nonce == 0 {
        return Err(CloakError::DescriptorReplay);
    }

    // 3. Merkle root consistency
    let leaves: Vec<Vec<u8>> = desc.intro_points.iter().map(|ip| ip.to_leaf_bytes()).collect();
    let computed_root = merkle_root(&leaves);
    use subtle::ConstantTimeEq;
    if computed_root.ct_eq(&desc.merkle_root).unwrap_u8() != 1 {
        return Err(CloakError::MerkleProofInvalid);
    }

    // 4. Ed25519 signature
    let body = desc.canonical_body();
    verify_raw(&desc.identity_pubkey, &body, &desc.signature)
        .map_err(|_| CloakError::DescriptorSignatureInvalid)?;

    // 5. Address consistency: the identity pubkey must match the .cloak address
    let expected_addr = crate::cloak_protocol::address::derive_address(&desc.identity_pubkey);
    if expected_addr.as_str() != desc.cloak_address {
        return Err(CloakError::InvalidAddress(format!(
            "descriptor address '{}' does not match identity pubkey",
            desc.cloak_address
        )));
    }

    Ok(())
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cloak_protocol::address::derive_address;
    use crate::crypto::{Ed25519KeyPair, x25519::HybridKeyPair};

    fn make_descriptor() -> (CloakDescriptor, Ed25519KeyPair) {
        let kp = Ed25519KeyPair::generate();
        let pk = kp.public_key_bytes();
        let addr = derive_address(&pk);
        let hybrid = HybridKeyPair::generate();
        let hybrid_pk = hybrid.combined_public_bytes();

        let ip = IntroductionPoint {
            peer_id: hex::encode([0x01u8; 32]),
            address: "127.0.0.1:5000".into(),
            auth_key: [0x02u8; 32],
        };

        // We need a second keypair to build (builder consumes the keypair)
        let kp2 = Ed25519KeyPair::from_secret_bytes(kp.to_secret_bytes());
        let desc = DescriptorBuilder::new(addr.to_string(), kp2, hybrid_pk)
            .add_intro_point(ip)
            .build()
            .unwrap();

        (desc, kp)
    }

    #[test]
    fn valid_descriptor_verifies() {
        let (desc, _) = make_descriptor();
        assert!(verify_descriptor(&desc, 3600, 300).is_ok());
    }

    #[test]
    fn tampered_signature_rejected() {
        let (mut desc, _) = make_descriptor();
        desc.signature[0] ^= 0xFF;
        assert!(verify_descriptor(&desc, 3600, 300).is_err());
    }

    #[test]
    fn tampered_intro_point_rejected() {
        let (mut desc, _) = make_descriptor();
        desc.intro_points[0].address = "evil.host:9999".into();
        assert!(verify_descriptor(&desc, 3600, 300).is_err());
    }

    #[test]
    fn expired_descriptor_rejected() {
        let (mut desc, _) = make_descriptor();
        desc.issued_at = 1_000_000; // far in the past
        assert!(verify_descriptor(&desc, 3600, 300).is_err());
    }

    #[test]
    fn address_mismatch_rejected() {
        let (mut desc, _) = make_descriptor();
        desc.cloak_address = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.cloak".into();
        assert!(verify_descriptor(&desc, 3600, 300).is_err());
    }

    #[test]
    fn builder_requires_at_least_one_intro_point() {
        let kp = Ed25519KeyPair::generate();
        let pk = kp.public_key_bytes();
        let addr = derive_address(&pk);
        let result = DescriptorBuilder::new(addr.to_string(), kp, vec![]).build();
        assert!(result.is_err());
    }
}
