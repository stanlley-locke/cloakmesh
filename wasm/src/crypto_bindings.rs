use wasm_bindgen::prelude::*;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;

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
        use ed25519_dalek::Signer;
        self.signing_key.sign(msg).to_bytes().to_vec()
    }
}

#[wasm_bindgen]
pub fn cloak_address_from_pubkey(pubkey: &[u8]) -> Result<String, JsValue> {
    if pubkey.len() != 32 {
        return Err(JsValue::from_str("pubkey must be 32 bytes"));
    }
    let pk: [u8; 32] = pubkey.try_into().unwrap();
    use sha2::{Digest, Sha256};
    let checksum = &Sha256::digest([[0x01].as_slice(), &pk].concat())[..4];
    let mut payload = vec![0x01u8];
    payload.extend_from_slice(&pk);
    payload.extend_from_slice(checksum);
    let encoded = base32::encode(base32::Alphabet::RFC4648 { padding: false }, &payload)
        .to_lowercase();
    Ok(format!("{}.cloak", encoded))
}
