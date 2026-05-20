use wasm_bindgen::prelude::*;
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use x25519_dalek::{StaticSecret, PublicKey as X25519PublicKey};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

#[wasm_bindgen]
pub struct KeyPair {
    signing_key: SigningKey,
}

#[wasm_bindgen]
impl KeyPair {
    #[wasm_bindgen(constructor)]
    pub fn generate() -> Self {
        Self { signing_key: SigningKey::generate(&mut OsRng) }
    }

    pub fn public_key(&self) -> Vec<u8> {
        self.signing_key.verifying_key().to_bytes().to_vec()
    }

    pub fn sign(&self, msg: &[u8]) -> Vec<u8> {
        self.signing_key.sign(msg).to_bytes().to_vec()
    }

    pub fn verify(pubkey: &[u8], msg: &[u8], sig_bytes: &[u8]) -> bool {
        if pubkey.len() != 32 || sig_bytes.len() != 64 { return false; }
        let pk_bytes: [u8; 32] = pubkey.try_into().unwrap();
        let pk = match VerifyingKey::from_bytes(&pk_bytes) {
            Ok(k) => k,
            Err(_) => return false,
        };
        let sig_arr: [u8; 64] = sig_bytes.try_into().unwrap();
        let sig = Signature::from_bytes(&sig_arr);
        pk.verify(msg, &sig).is_ok()
    }
}

#[wasm_bindgen]
pub struct X25519Exchange {
    secret: StaticSecret,
}

#[wasm_bindgen]
impl X25519Exchange {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { secret: StaticSecret::random_from_rng(OsRng) }
    }

    pub fn public_key(&self) -> Vec<u8> {
        X25519PublicKey::from(&self.secret).as_bytes().to_vec()
    }

    pub fn diffie_hellman(&self, remote_pubkey: &[u8]) -> Result<Vec<u8>, JsValue> {
        if remote_pubkey.len() != 32 {
            return Err(JsValue::from_str("remote pubkey must be 32 bytes"));
        }
        let remote: [u8; 32] = remote_pubkey.try_into().unwrap();
        let remote_pk = X25519PublicKey::from(remote);
        let shared = self.secret.diffie_hellman(&remote_pk);
        Ok(shared.as_bytes().to_vec())
    }
}

#[wasm_bindgen]
pub fn cloak_address_from_pubkey(pubkey: &[u8]) -> Result<String, JsValue> {
    if pubkey.len() != 32 {
        return Err(JsValue::from_str("pubkey must be 32 bytes"));
    }
    let pk: [u8; 32] = pubkey.try_into().unwrap();
    let version = b"\x01";
    
    // Parity with core: double SHA256 checksum
    let mut hasher = Sha256::new();
    hasher.update(version);
    hasher.update(&pk);
    let first = hasher.finalize();
    
    let second = Sha256::digest(first);
    let checksum = &second[..4];

    let mut payload = vec![0x01u8];
    payload.extend_from_slice(&pk);
    payload.extend_from_slice(checksum);
    
    let encoded = base32::encode(base32::Alphabet::RFC4648 { padding: false }, &payload)
        .to_lowercase();
    Ok(format!("{}.cloak", encoded))
}
