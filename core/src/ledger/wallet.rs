use crate::proto::v1::{Transaction, TransactionInput, TransactionOutput, Utxo};
use ed25519_dalek::{SigningKey, Signer, VerifyingKey};
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};

/// Derives a standard ATK address from a public key
pub fn derive_address(pubkey: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(pubkey);
    let hash = hasher.finalize();
    format!("atk1{}", hex::encode(&hash[..20])) // Simple prefix + 20 byte hash
}

pub struct Wallet {
    keypair: SigningKey,
    pub address: String,
}

impl Wallet {
    pub fn new(keypair: SigningKey) -> Self {
        let public_key = VerifyingKey::from(&keypair);
        let address = derive_address(public_key.as_bytes());
        Self { keypair, address }
    }

    /// Creates a signed transaction
    pub fn create_transaction(&self, to_address: String, amount: u64, available_utxos: Vec<Utxo>) -> Result<Transaction, String> {
        let mut inputs = Vec::new();
        let mut input_sum = 0;
        
        for utxo in available_utxos {
            input_sum += utxo.amount;
            inputs.push(TransactionInput {
                txid: utxo.txid,
                vout: utxo.vout,
                signature: vec![], // Signed later
                pubkey: VerifyingKey::from(&self.keypair).as_bytes().to_vec(),
            });
            if input_sum >= amount {
                break;
            }
        }
        
        if input_sum < amount {
            return Err("Insufficient balance".to_string());
        }
        
        let mut outputs = vec![
            TransactionOutput {
                address: to_address,
                amount,
            }
        ];
        
        // Change output
        if input_sum > amount {
            outputs.push(TransactionOutput {
                address: self.address.clone(),
                amount: input_sum - amount,
            });
        }
        
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        let mut hasher = Sha256::new();
        hasher.update(&timestamp.to_le_bytes());
        for input in &inputs {
            hasher.update(input.txid.as_bytes());
            hasher.update(&input.vout.to_le_bytes());
        }
        for output in &outputs {
            hasher.update(&output.amount.to_le_bytes());
            hasher.update(output.address.as_bytes());
        }
        let txid = hex::encode(hasher.finalize());
        
        // Sign inputs
        for input in &mut inputs {
            let sig = self.keypair.sign(txid.as_bytes());
            input.signature = sig.to_bytes().to_vec();
        }
        
        Ok(Transaction {
            id: txid,
            inputs,
            outputs,
            timestamp,
        })
    }
}
