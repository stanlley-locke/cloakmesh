//! Noise_XX_25519_ChaChaPoly_BLAKE2s-inspired handshake implementation.
//!
//! Handshake pattern:
//!   -> e
//!   <- e, enc(s), ee, es
//!   -> enc(s), se
//!
//! After completion both parties hold matching session keys derived from
//! the chaining key via HKDF.

use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Nonce,
};
use zeroize::Zeroizing;

use crate::crypto::{
    kdf::{hkdf_expand, noise_mix_hash, noise_mix_key},
    x25519::X25519KeyPair,
};
use crate::errors::{CloakError, CloakResult};

const PROTOCOL_NAME: &[u8] = b"Noise_XX_25519_ChaChaPoly_BLAKE2s";
const TAG_LEN: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseState {
    InitiatorSendMsg1,
    InitiatorWaitMsg2,
    InitiatorSendMsg3,
    ResponderWaitMsg1,
    ResponderSendMsg2,
    ResponderWaitMsg3,
    Complete,
    Failed,
}

pub struct HandshakeResult {
    pub send_key: Zeroizing<[u8; 32]>,
    pub recv_key: Zeroizing<[u8; 32]>,
    pub remote_static_pubkey: [u8; 32],
}

pub struct NoiseHandshake {
    state: NoiseState,
    is_initiator: bool,
    chaining_key: [u8; 32],
    hash: [u8; 32],
    tk: Option<Zeroizing<[u8; 32]>>,
    local_ephemeral: Option<X25519KeyPair>,
    local_static: X25519KeyPair,
    remote_ephemeral_pub: Option<[u8; 32]>,
    remote_static_pub: Option<[u8; 32]>,
    result: Option<HandshakeResult>,
}

impl NoiseHandshake {
    fn initialize(is_initiator: bool, local_static: X25519KeyPair) -> Self {
        let hash = crate::crypto::kdf::blake2s_hash(PROTOCOL_NAME);
        let chaining_key: [u8; 32] = hash;
        Self {
            state: if is_initiator { NoiseState::InitiatorSendMsg1 } else { NoiseState::ResponderWaitMsg1 },
            is_initiator,
            chaining_key,
            hash,
            tk: None,
            local_ephemeral: None,
            local_static,
            remote_ephemeral_pub: None,
            remote_static_pub: None,
            result: None,
        }
    }

    pub fn new_initiator(local_static: X25519KeyPair) -> Self { Self::initialize(true, local_static) }
    pub fn new_responder(local_static: X25519KeyPair) -> Self { Self::initialize(false, local_static) }
    pub fn state(&self) -> NoiseState { self.state }
    pub fn is_complete(&self) -> bool { self.state == NoiseState::Complete }
    pub fn take_result(&mut self) -> HandshakeResult { self.result.take().expect("handshake not complete") }

    fn mix_hash(&mut self, data: &[u8]) {
        self.hash = noise_mix_hash(&self.hash, data);
    }

    fn mix_key(&mut self, dh_output: &[u8]) -> CloakResult<()> {
        let (new_ck, temp_k) = noise_mix_key(&self.chaining_key, dh_output)?;
        self.chaining_key = new_ck;
        self.tk = Some(temp_k);
        Ok(())
    }

    fn get_encryption_key(&self) -> CloakResult<Zeroizing<[u8; 32]>> {
        // Use the current tk if available, else derive from chaining_key
        if let Some(ref tk) = self.tk {
            Ok(tk.clone())
        } else {
            hkdf_expand(&self.chaining_key, b"cloakmesh-encryption", b"")
        }
    }

    fn encrypt_and_hash(&mut self, plaintext: &[u8]) -> CloakResult<Vec<u8>> {
        let k = self.get_encryption_key()?;
        let cipher = ChaCha20Poly1305::new_from_slice(&*k)
            .map_err(|_| CloakError::NoiseHandshake { step: 0, reason: "cipher init failed".into() })?;
        let nonce = Nonce::from_slice(&[0u8; 12]);
        let ct = cipher
            .encrypt(nonce, Payload { msg: plaintext, aad: &self.hash })
            .map_err(|_| CloakError::AeadEncrypt)?;
        self.mix_hash(&ct);
        Ok(ct)
    }

    fn decrypt_and_hash(&mut self, ciphertext: &[u8]) -> CloakResult<Vec<u8>> {
        let k = self.get_encryption_key()?;
        let cipher = ChaCha20Poly1305::new_from_slice(&*k)
            .map_err(|_| CloakError::NoiseHandshake { step: 0, reason: "cipher init failed".into() })?;
        let nonce = Nonce::from_slice(&[0u8; 12]);
        let hash_before = self.hash;
        self.mix_hash(ciphertext);
        cipher
            .decrypt(nonce, Payload { msg: ciphertext, aad: &hash_before })
            .map_err(|_| CloakError::AeadDecrypt)
    }

    fn split(&self) -> CloakResult<(Zeroizing<[u8; 32]>, Zeroizing<[u8; 32]>)> {
        let k1 = hkdf_expand(&self.chaining_key, b"cloakmesh-noise-send", b"")?;
        let k2 = hkdf_expand(&self.chaining_key, b"cloakmesh-noise-recv", b"")?;
        if self.is_initiator { Ok((k1, k2)) } else { Ok((k2, k1)) }
    }

    // ── Message 1: initiator -> e ────────────────────────────────────────────

    pub fn write_message1(&mut self) -> CloakResult<Vec<u8>> {
        if self.state != NoiseState::InitiatorSendMsg1 {
            return Err(CloakError::NoiseHandshake { step: 1, reason: format!("wrong state: {:?}", self.state) });
        }
        let ephemeral = X25519KeyPair::generate();
        let e_pub = ephemeral.public_bytes();
        self.mix_hash(&e_pub);
        self.local_ephemeral = Some(ephemeral);
        self.state = NoiseState::InitiatorWaitMsg2;
        Ok(e_pub.to_vec())
    }

    pub fn read_message1(&mut self, msg: &[u8]) -> CloakResult<()> {
        if self.state != NoiseState::ResponderWaitMsg1 {
            return Err(CloakError::NoiseHandshake { step: 1, reason: format!("wrong state: {:?}", self.state) });
        }
        if msg.len() < 32 {
            return Err(CloakError::NoiseHandshake { step: 1, reason: "message too short".into() });
        }
        let re_pub: [u8; 32] = msg[..32].try_into().unwrap();
        self.mix_hash(&re_pub);
        self.remote_ephemeral_pub = Some(re_pub);
        self.state = NoiseState::ResponderSendMsg2;
        Ok(())
    }

    // ── Message 2: responder <- e, ee, s, es ────────────────────────────────

    pub fn write_message2(&mut self) -> CloakResult<Vec<u8>> {
        if self.state != NoiseState::ResponderSendMsg2 {
            return Err(CloakError::NoiseHandshake { step: 2, reason: format!("wrong state: {:?}", self.state) });
        }
        let re_pub = self.remote_ephemeral_pub.unwrap();

        let ephemeral = X25519KeyPair::generate();
        let e_pub = ephemeral.public_bytes();
        self.mix_hash(&e_pub);

        // ee: DH(e, re)
        let ee_ss_bytes = *ephemeral.diffie_hellman(&re_pub);
        self.mix_key(&ee_ss_bytes)?;

        // s: encrypt our static public key
        let s_pub = self.local_static.public_bytes();
        let enc_s = self.encrypt_and_hash(&s_pub)?;

        // es: DH(s, re)
        let es_ss_bytes = *self.local_static.diffie_hellman(&re_pub);
        self.mix_key(&es_ss_bytes)?;

        self.local_ephemeral = Some(ephemeral);
        self.state = NoiseState::ResponderWaitMsg3;

        let mut out = Vec::with_capacity(32 + enc_s.len());
        out.extend_from_slice(&e_pub);
        out.extend_from_slice(&enc_s);
        Ok(out)
    }

    pub fn read_message2(&mut self, msg: &[u8]) -> CloakResult<()> {
        if self.state != NoiseState::InitiatorWaitMsg2 {
            return Err(CloakError::NoiseHandshake { step: 2, reason: format!("wrong state: {:?}", self.state) });
        }
        let min_len = 32 + 32 + TAG_LEN;
        if msg.len() < min_len {
            return Err(CloakError::NoiseHandshake {
                step: 2,
                reason: format!("message too short: {} < {}", msg.len(), min_len),
            });
        }

        let re_pub: [u8; 32] = msg[..32].try_into().unwrap();
        self.mix_hash(&re_pub);
        self.remote_ephemeral_pub = Some(re_pub);

        // ee: DH(e, re)
        let ee_ss_bytes = *self.local_ephemeral.as_ref().unwrap().diffie_hellman(&re_pub);
        self.mix_key(&ee_ss_bytes)?;

        // Decrypt remote static key
        let rs_pub_bytes = self.decrypt_and_hash(&msg[32..])?;
        if rs_pub_bytes.len() != 32 {
            return Err(CloakError::NoiseHandshake { step: 2, reason: "decrypted static key wrong length".into() });
        }
        let rs_pub: [u8; 32] = rs_pub_bytes.try_into().unwrap();
        self.remote_static_pub = Some(rs_pub);

        // es: DH(e, rs) (initiator's ephemeral, responder's static)
        let es_ss_bytes = *self.local_ephemeral.as_ref().unwrap().diffie_hellman(&rs_pub);
        self.mix_key(&es_ss_bytes)?;

        self.state = NoiseState::InitiatorSendMsg3;
        Ok(())
    }

    // ── Message 3: initiator -> s, se ────────────────────────────────────────

    pub fn write_message3(&mut self) -> CloakResult<Vec<u8>> {
        if self.state != NoiseState::InitiatorSendMsg3 {
            return Err(CloakError::NoiseHandshake { step: 3, reason: format!("wrong state: {:?}", self.state) });
        }
        let re_pub = self.remote_ephemeral_pub.unwrap();

        // s: encrypt our static public key
        let s_pub = self.local_static.public_bytes();
        let enc_s = self.encrypt_and_hash(&s_pub)?;

        // se: DH(s, re) (initiator's static, responder's ephemeral)
        let se_ss_bytes = *self.local_static.diffie_hellman(&re_pub);
        self.mix_key(&se_ss_bytes)?;

        let (send_key, recv_key) = self.split()?;
        let remote_static_pubkey = self.remote_static_pub.unwrap();
        self.result = Some(HandshakeResult { send_key, recv_key, remote_static_pubkey });
        self.state = NoiseState::Complete;
        Ok(enc_s)
    }

    pub fn read_message3(&mut self, msg: &[u8]) -> CloakResult<()> {
        if self.state != NoiseState::ResponderWaitMsg3 {
            return Err(CloakError::NoiseHandshake { step: 3, reason: format!("wrong state: {:?}", self.state) });
        }

        // Decrypt initiator's static key
        let rs_pub_bytes = self.decrypt_and_hash(msg)?;
        if rs_pub_bytes.len() != 32 {
            return Err(CloakError::NoiseHandshake { step: 3, reason: "decrypted static key wrong length".into() });
        }
        let rs_pub: [u8; 32] = rs_pub_bytes.try_into().unwrap();
        self.remote_static_pub = Some(rs_pub);

        // se: DH(e, rs) (responder's ephemeral, initiator's static)
        let se_ss_bytes = *self.local_ephemeral.as_ref().unwrap().diffie_hellman(&rs_pub);
        self.mix_key(&se_ss_bytes)?;

        let (send_key, recv_key) = self.split()?;
        let remote_static_pubkey = self.remote_static_pub.unwrap();
        self.result = Some(HandshakeResult { send_key, recv_key, remote_static_pubkey });
        self.state = NoiseState::Complete;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pair() -> (NoiseHandshake, NoiseHandshake) {
        (
            NoiseHandshake::new_initiator(X25519KeyPair::generate()),
            NoiseHandshake::new_responder(X25519KeyPair::generate()),
        )
    }

    #[test]
    fn full_xx_handshake_completes() {
        let (mut i, mut r) = make_pair();
        let m1 = i.write_message1().unwrap();
        r.read_message1(&m1).unwrap();
        let m2 = r.write_message2().unwrap();
        i.read_message2(&m2).unwrap();
        let m3 = i.write_message3().unwrap();
        r.read_message3(&m3).unwrap();
        assert!(i.is_complete());
        assert!(r.is_complete());
    }

    #[test]
    fn transport_keys_match_after_handshake() {
        let (mut i, mut r) = make_pair();
        let m1 = i.write_message1().unwrap();
        r.read_message1(&m1).unwrap();
        let m2 = r.write_message2().unwrap();
        i.read_message2(&m2).unwrap();
        let m3 = i.write_message3().unwrap();
        r.read_message3(&m3).unwrap();
        let ir = i.take_result();
        let rr = r.take_result();
        assert_eq!(*ir.send_key, *rr.recv_key);
        assert_eq!(*ir.recv_key, *rr.send_key);
        assert_eq!(ir.remote_static_pubkey, r.local_static.public_bytes());
        assert_eq!(rr.remote_static_pubkey, i.local_static.public_bytes());
    }

    #[test]
    fn tampered_message2_fails() {
        let (mut i, mut r) = make_pair();
        let m1 = i.write_message1().unwrap();
        r.read_message1(&m1).unwrap();
        let mut m2 = r.write_message2().unwrap();
        let last = m2.len() - 1;
        m2[last] ^= 0xFF;
        assert!(i.read_message2(&m2).is_err());
    }
}
