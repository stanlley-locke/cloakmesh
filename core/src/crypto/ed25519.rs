use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::path::Path;
use zeroize::{Zeroize, Zeroizing};

use crate::errors::{CloakError, CloakResult};

// ── Key Pair ─────────────────────────────────────────────────────────────────

/// Ed25519 signing key pair. The secret key is zeroed on drop.
pub struct Ed25519KeyPair {
    signing_key: SigningKey,
}

impl Drop for Ed25519KeyPair {
    fn drop(&mut self) {
        // ed25519-dalek's SigningKey implements Zeroize internally via ZeroizeOnDrop
    }
}

impl Ed25519KeyPair {
    /// Generate a fresh keypair using the OS CSPRNG.
    pub fn generate() -> Self {
        Self { signing_key: SigningKey::generate(&mut OsRng) }
    }

    /// Restore from a 32-byte secret seed. The seed is consumed and zeroed.
    pub fn from_secret_bytes(mut bytes: Zeroizing<[u8; 32]>) -> Self {
        let kp = Self { signing_key: SigningKey::from_bytes(&bytes) };
        bytes.zeroize();
        kp
    }

    /// Load from a raw 32-byte key file. File contents are zeroed after reading.
    pub fn load_from_file(path: &Path) -> CloakResult<Self> {
        let mut raw = std::fs::read(path)?;
        if raw.len() != 32 {
            raw.zeroize();
            return Err(CloakError::InvalidKeyMaterial(
                format!("identity key file must be 32 bytes, got {}", raw.len())
            ));
        }
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&raw);
        raw.zeroize();
        Ok(Self::from_secret_bytes(Zeroizing::new(seed)))
    }

    /// Save the secret seed to a file with restrictive permissions (0o600 on Unix).
    pub fn save_to_file(&self, path: &Path) -> CloakResult<()> {
        use std::io::Write;
        let seed = self.to_secret_bytes();
        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = {
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .mode(0o600)
                    .open(path)?
            }
            #[cfg(not(unix))]
            {
                std::fs::File::create(path)?
            }
        };
        file.write_all(&*seed)?;
        Ok(())
    }

    /// Load from file if it exists, otherwise generate and save a new keypair.
    pub fn load_or_generate(path: &Path) -> CloakResult<Self> {
        if path.exists() {
            Self::load_from_file(path)
        } else {
            let kp = Self::generate();
            kp.save_to_file(path)?;
            tracing::info!(path = %path.display(), "Generated new Ed25519 identity key");
            Ok(kp)
        }
    }

    /// Return the 32-byte public key.
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.signing_key.verifying_key().to_bytes()
    }

    /// Return the verifying key.
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// Sign a message. The message is pre-hashed internally by ed25519-dalek.
    pub fn sign(&self, msg: &[u8]) -> Ed25519Signature {
        Ed25519Signature(self.signing_key.sign(msg))
    }

    /// Export the secret seed bytes, wrapped in Zeroizing for safe handling.
    pub fn to_secret_bytes(&self) -> Zeroizing<[u8; 32]> {
        Zeroizing::new(self.signing_key.to_bytes())
    }
}

// ── Signature ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ed25519Signature(pub Signature);

impl Ed25519Signature {
    pub fn to_bytes(&self) -> [u8; 64] {
        self.0.to_bytes()
    }

    pub fn from_bytes(bytes: &[u8; 64]) -> Self {
        Self(Signature::from_bytes(bytes))
    }
}

// ── Standalone verification ──────────────────────────────────────────────────

/// Verify an Ed25519 signature. Uses constant-time operations internally.
pub fn verify(pubkey_bytes: &[u8; 32], msg: &[u8], sig_bytes: &[u8; 64]) -> CloakResult<()> {
    let vk = VerifyingKey::from_bytes(pubkey_bytes)
        .map_err(|e| CloakError::SignatureVerification(e.to_string()))?;
    let sig = Signature::from_bytes(sig_bytes);
    vk.verify(msg, &sig)
        .map_err(|e| CloakError::SignatureVerification(e.to_string()))
}

/// Verify a signature given raw bytes for both pubkey and signature.
pub fn verify_raw(pubkey: &[u8], msg: &[u8], sig: &[u8]) -> CloakResult<()> {
    let pk: &[u8; 32] = pubkey.try_into()
        .map_err(|_| CloakError::InvalidKeyMaterial("pubkey must be 32 bytes".into()))?;
    let s: &[u8; 64] = sig.try_into()
        .map_err(|_| CloakError::InvalidKeyMaterial("signature must be 64 bytes".into()))?;
    verify(pk, msg, s)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_and_verify_roundtrip() {
        let kp = Ed25519KeyPair::generate();
        let msg = b"cloakmesh protocol test message";
        let sig = kp.sign(msg);
        let pk = kp.public_key_bytes();
        assert!(verify(&pk, msg, &sig.to_bytes()).is_ok());
    }

    #[test]
    fn wrong_message_fails_verification() {
        let kp = Ed25519KeyPair::generate();
        let sig = kp.sign(b"correct message");
        let pk = kp.public_key_bytes();
        assert!(verify(&pk, b"tampered message", &sig.to_bytes()).is_err());
    }

    #[test]
    fn wrong_key_fails_verification() {
        let kp1 = Ed25519KeyPair::generate();
        let kp2 = Ed25519KeyPair::generate();
        let msg = b"test";
        let sig = kp1.sign(msg);
        let pk2 = kp2.public_key_bytes();
        assert!(verify(&pk2, msg, &sig.to_bytes()).is_err());
    }

    #[test]
    fn secret_bytes_roundtrip() {
        let kp = Ed25519KeyPair::generate();
        let pk_original = kp.public_key_bytes();
        let secret = kp.to_secret_bytes();
        let kp2 = Ed25519KeyPair::from_secret_bytes(secret);
        assert_eq!(pk_original, kp2.public_key_bytes());
    }

    #[test]
    fn save_and_load_key_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("identity.key");
        let kp = Ed25519KeyPair::generate();
        let pk = kp.public_key_bytes();
        kp.save_to_file(&path).unwrap();
        let kp2 = Ed25519KeyPair::load_from_file(&path).unwrap();
        assert_eq!(pk, kp2.public_key_bytes());
    }
}
