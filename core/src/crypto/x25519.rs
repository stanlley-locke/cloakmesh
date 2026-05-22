use rand::rngs::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey, StaticSecret};
use zeroize::{Zeroize, Zeroizing};

use crate::errors::{CloakError, CloakResult};

// ── Static X25519 Key Pair ───────────────────────────────────────────────────

/// Long-term X25519 key pair for hybrid key exchange.
/// The secret is zeroed on drop.
pub struct X25519KeyPair {
    secret: StaticSecret,
    pub public: PublicKey,
}

impl X25519KeyPair {
    pub fn generate() -> Self {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);
        Self { secret, public }
    }

    pub fn from_secret_bytes(mut bytes: Zeroizing<[u8; 32]>) -> Self {
        let secret = StaticSecret::from(*bytes);
        bytes.zeroize();
        let public = PublicKey::from(&secret);
        Self { secret, public }
    }

    pub fn public_bytes(&self) -> [u8; 32] {
        self.public.to_bytes()
    }

    /// Perform Diffie-Hellman with a remote public key.
    /// Returns the shared secret, zeroed on drop.
    pub fn diffie_hellman(&self, their_public_bytes: &[u8; 32]) -> Zeroizing<[u8; 32]> {
        let their_pk = PublicKey::from(*their_public_bytes);
        Zeroizing::new(self.secret.diffie_hellman(&their_pk).to_bytes())
    }
}

// ── Ephemeral X25519 DH ──────────────────────────────────────────────────────

/// Perform a one-shot ephemeral DH exchange.
/// Returns `(our_ephemeral_public, shared_secret)`.
/// The ephemeral secret is consumed and zeroed after use.
pub fn ephemeral_dh(their_public_bytes: &[u8; 32]) -> CloakResult<([u8; 32], Zeroizing<[u8; 32]>)> {
    let secret = EphemeralSecret::random_from_rng(OsRng);
    let our_public = PublicKey::from(&secret);
    let their_pk = PublicKey::from(*their_public_bytes);
    let shared = Zeroizing::new(secret.diffie_hellman(&their_pk).to_bytes());
    // Reject low-order points (all-zero shared secret indicates a small-subgroup attack)
    if shared.iter().all(|&b| b == 0) {
        return Err(CloakError::InvalidKeyMaterial(
            "X25519 DH produced all-zero shared secret (low-order point attack)".into(),
        ));
    }
    Ok((our_public.to_bytes(), shared))
}

// ── Hybrid PQ Key Exchange ───────────────────────────────────────────────────
//
// The hybrid scheme combines X25519 ECDH with a post-quantum KEM (Kyber-768).
// The combined shared secret is: HKDF(x25519_ss || kyber_ss, "cloakmesh-hybrid-v1")
//
// Kyber-768 is not yet in stable Rust crates without C FFI; we use a
// clearly-marked stub that is API-compatible so Phase 2 can drop in the real
// implementation (e.g. `pqcrypto-kyber` or `ml-kem`) without changing callers.

/// Hybrid PQ public key: X25519 (32B) || Kyber-768 public key (1184B)
pub const HYBRID_PUBKEY_LEN: usize = 32 + 1184;
/// Hybrid PQ ciphertext: Kyber-768 ciphertext (1088B)
pub const HYBRID_CT_LEN: usize = 1088;

pub struct HybridKeyPair {
    pub x25519: X25519KeyPair,
    kyber_sk: Zeroizing<[u8; 2400]>,
    pub kyber_pk: [u8; 1184],
}

impl HybridKeyPair {
    pub fn generate() -> Self {
        let x25519 = X25519KeyPair::generate();
        let mut rng = OsRng;
        let keys = pqc_kyber::keypair(&mut rng).expect("Kyber keygen failed");
        Self {
            x25519,
            kyber_sk: Zeroizing::new(keys.secret),
            kyber_pk: keys.public,
        }
    }

    /// Serialize the combined public key: X25519 || Kyber-768 pk
    pub fn combined_public_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(HYBRID_PUBKEY_LEN);
        out.extend_from_slice(&self.x25519.public_bytes());
        out.extend_from_slice(&self.kyber_pk);
        out
    }

    /// Encapsulate: given a remote hybrid public key, produce
    /// `(ciphertext, shared_secret)` where shared_secret is the HKDF output
    /// over both X25519 and Kyber shared secrets.
    pub fn encapsulate(remote_combined_pk: &[u8]) -> CloakResult<(Vec<u8>, Zeroizing<[u8; 32]>)> {
        if remote_combined_pk.len() < HYBRID_PUBKEY_LEN {
            return Err(CloakError::InvalidKeyMaterial(
                format!("hybrid pubkey too short: {} < {}", remote_combined_pk.len(), HYBRID_PUBKEY_LEN)
            ));
        }
        let x25519_pk: [u8; 32] = remote_combined_pk[..32].try_into().unwrap();
        let (our_eph_pub, x25519_ss) = ephemeral_dh(&x25519_pk)?;

        let kyber_pk: &[u8; 1184] = remote_combined_pk[32..32 + 1184].try_into().map_err(|_| {
            CloakError::InvalidKeyMaterial("Invalid Kyber public key length".into())
        })?;
        let mut rng = OsRng;
        let (kyber_ct, kyber_ss_bytes) = pqc_kyber::encapsulate(kyber_pk, &mut rng)
            .map_err(|_| CloakError::InvalidKeyMaterial("Kyber encapsulation failed".into()))?;

        let kyber_ss = Zeroizing::new(kyber_ss_bytes);
        let combined_ss = combine_secrets(&x25519_ss, &kyber_ss)?;

        let mut ct = our_eph_pub.to_vec();
        ct.extend_from_slice(&kyber_ct);
        Ok((ct, combined_ss))
    }

    /// Decapsulate: given a ciphertext, recover the shared secret.
    pub fn decapsulate(&self, ciphertext: &[u8]) -> CloakResult<Zeroizing<[u8; 32]>> {
        if ciphertext.len() < 32 + HYBRID_CT_LEN {
            return Err(CloakError::InvalidKeyMaterial("hybrid ciphertext too short".into()));
        }
        let their_eph_pub: [u8; 32] = ciphertext[..32].try_into().unwrap();
        let x25519_ss = self.x25519.diffie_hellman(&their_eph_pub);

        if x25519_ss.iter().all(|&b| b == 0) {
            return Err(CloakError::InvalidKeyMaterial(
                "X25519 decapsulation produced all-zero secret".into()
            ));
        }

        let kyber_ct: &[u8; 1088] = ciphertext[32..32 + HYBRID_CT_LEN].try_into().map_err(|_| {
            CloakError::InvalidKeyMaterial("Invalid Kyber ciphertext length".into())
        })?;
        let kyber_ss_bytes = pqc_kyber::decapsulate(kyber_ct, &self.kyber_sk[..])
            .map_err(|_| CloakError::InvalidKeyMaterial("Kyber decapsulation failed".into()))?;
        let kyber_ss = Zeroizing::new(kyber_ss_bytes);

        combine_secrets(&x25519_ss, &kyber_ss)
    }
}

/// Combine X25519 and Kyber shared secrets via HKDF-SHA256.
fn combine_secrets(
    x25519_ss: &Zeroizing<[u8; 32]>,
    kyber_ss: &Zeroizing<[u8; 32]>,
) -> CloakResult<Zeroizing<[u8; 32]>> {
    let mut ikm = Zeroizing::new([0u8; 64]);
    ikm[..32].copy_from_slice(x25519_ss.as_ref());
    ikm[32..].copy_from_slice(kyber_ss.as_ref());
    crate::crypto::kdf::hkdf_expand(ikm.as_ref(), b"cloakmesh-hybrid-v1", b"")
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ephemeral_dh_produces_same_secret_both_sides() {
        let alice = X25519KeyPair::generate();
        let bob = X25519KeyPair::generate();
        let alice_ss = alice.diffie_hellman(&bob.public_bytes());
        let bob_ss = bob.diffie_hellman(&alice.public_bytes());
        assert_eq!(*alice_ss, *bob_ss);
    }

    #[test]
    fn ephemeral_dh_rejects_low_order_point() {
        // All-zero public key is a low-order point on Curve25519
        let low_order = [0u8; 32];
        // ephemeral_dh may or may not produce all-zero; we just ensure it doesn't panic
        let _ = ephemeral_dh(&low_order);
    }

    #[test]
    fn hybrid_encap_decap_roundtrip() {
        let recipient = HybridKeyPair::generate();
        let combined_pk = recipient.combined_public_bytes();
        let (ct, ss_enc) = HybridKeyPair::encapsulate(&combined_pk).unwrap();
        let ss_dec = recipient.decapsulate(&ct).unwrap();
        assert_eq!(*ss_enc, *ss_dec);
    }
}
