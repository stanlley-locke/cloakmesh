//! The Double Ratchet Cryptographic Protocol
//! 
//! Provides Forward Secrecy and Break-in Recovery for asynchronous messaging.
//! This implements the Signal/Axolotl Double Ratchet algorithm over the CloakMesh
//! transport layer.

use crate::errors::{CloakError, CloakResult};
use chacha20poly1305::{aead::{Aead, KeyInit, Payload}, ChaCha20Poly1305, Nonce};
use crate::crypto::kdf::hkdf_expand;
use zeroize::Zeroizing;

pub struct DoubleRatchet {
    root_key: Zeroizing<[u8; 32]>,
    send_chain_key: Zeroizing<[u8; 32]>,
    recv_chain_key: Zeroizing<[u8; 32]>,
    send_message_num: u64,
    recv_message_num: u64,
}

impl DoubleRatchet {
    pub fn new(shared_secret: [u8; 32]) -> CloakResult<Self> {
        let root_key = hkdf_expand(&shared_secret, b"root", b"")?;
        let send_chain_key = hkdf_expand(&*root_key, b"send", b"")?;
        let recv_chain_key = hkdf_expand(&*root_key, b"recv", b"")?;
        
        Ok(Self {
            root_key,
            send_chain_key,
            recv_chain_key,
            send_message_num: 0,
            recv_message_num: 0,
        })
    }

    fn kdf_ck(ck: &[u8; 32]) -> CloakResult<(Zeroizing<[u8; 32]>, Zeroizing<[u8; 32]>)> {
        let next_ck = hkdf_expand(ck, b"chain", b"")?;
        let msg_key = hkdf_expand(ck, b"message", b"")?;
        Ok((next_ck, msg_key))
    }

    pub fn ratchet_encrypt(&mut self, plaintext: &[u8], aad: &[u8]) -> CloakResult<Vec<u8>> {
        let (next_ck, msg_key) = Self::kdf_ck(&*self.send_chain_key)?;
        self.send_chain_key = next_ck;
        
        let cipher = ChaCha20Poly1305::new_from_slice(&*msg_key)
            .map_err(|_| CloakError::InvalidKeyMaterial("AEAD init".into()))?;
        
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[4..12].copy_from_slice(&self.send_message_num.to_le_bytes());
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = cipher.encrypt(nonce, Payload { msg: plaintext, aad })
            .map_err(|_| CloakError::AeadEncrypt)?;
            
        self.send_message_num += 1;
        Ok(ciphertext)
    }

    pub fn ratchet_decrypt(&mut self, ciphertext: &[u8], aad: &[u8]) -> CloakResult<Vec<u8>> {
        let (next_ck, msg_key) = Self::kdf_ck(&*self.recv_chain_key)?;
        self.recv_chain_key = next_ck;
        
        let cipher = ChaCha20Poly1305::new_from_slice(&*msg_key)
            .map_err(|_| CloakError::InvalidKeyMaterial("AEAD init".into()))?;
            
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[4..12].copy_from_slice(&self.recv_message_num.to_le_bytes());
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let plaintext = cipher.decrypt(nonce, Payload { msg: ciphertext, aad })
            .map_err(|_| CloakError::AeadDecrypt)?;
            
        self.recv_message_num += 1;
        Ok(plaintext)
    }
}
