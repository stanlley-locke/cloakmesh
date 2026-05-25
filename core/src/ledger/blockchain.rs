use std::collections::HashMap;
use tokio::sync::RwLock;
use crate::proto::v1::{Transaction, Utxo};
use crate::errors::{CloakError, CloakResult};
use sha2::{Sha256, Digest};
use ed25519_dalek::{VerifyingKey, Signature, Verifier};

/// In-memory UTXO set and transaction pool
pub struct BlockchainState {
    pub utxos: RwLock<HashMap<String, Utxo>>, // Key: txid:vout
    pub mempool: RwLock<HashMap<String, Transaction>>,
}

impl BlockchainState {
    pub fn new() -> Self {
        Self {
            utxos: RwLock::new(HashMap::new()),
            mempool: RwLock::new(HashMap::new()),
        }
    }

    /// Validates a transaction and adds it to the mempool
    pub async fn add_transaction(&self, tx: Transaction) -> CloakResult<()> {
        let mut hasher = Sha256::new();
        hasher.update(&tx.timestamp.to_le_bytes());
        for input in &tx.inputs {
            hasher.update(input.txid.as_bytes());
            hasher.update(&input.vout.to_le_bytes());
        }
        for output in &tx.outputs {
            hasher.update(&output.amount.to_le_bytes());
            hasher.update(output.address.as_bytes());
        }
        let expected_hash = hex::encode(hasher.finalize());
        if expected_hash != tx.id {
            return Err(CloakError::Protocol("Invalid transaction hash".to_string()));
        }

        // Verify signatures and input amounts
        let mut input_sum = 0;
        let mut output_sum = 0;
        
        let utxo_guard = self.utxos.read().await;
        
        for input in &tx.inputs {
            let utxo_key = format!("{}:{}", input.txid, input.vout);
            let utxo = utxo_guard.get(&utxo_key)
                .ok_or_else(|| CloakError::Protocol("UTXO not found or already spent".to_string()))?;
                
            input_sum += utxo.amount;
            
            // Verify signature
            let pubkey = VerifyingKey::from_bytes(
                &input.pubkey.as_slice().try_into().map_err(|_| CloakError::Protocol("Invalid pubkey length".to_string()))?
            ).map_err(|_| CloakError::Protocol("Invalid public key".to_string()))?;
            
            let sig = Signature::from_slice(&input.signature)
                .map_err(|_| CloakError::Protocol("Invalid signature format".to_string()))?;
                
            // The signed message is the transaction hash
            pubkey.verify(tx.id.as_bytes(), &sig)
                .map_err(|_| CloakError::Protocol("Invalid transaction signature".to_string()))?;
        }
        
        for output in &tx.outputs {
            output_sum += output.amount;
        }
        
        // Allow genesis transactions (no inputs) for testing/mining
        if !tx.inputs.is_empty() && input_sum < output_sum {
            return Err(CloakError::Protocol("Insufficient input amount".to_string()));
        }
        
        // Add to mempool
        self.mempool.write().await.insert(tx.id.clone(), tx);
        Ok(())
    }
    
    /// Process mempool and commit to UTXO set (simulating a block being mined)
    pub async fn process_mempool(&self) {
        let mut mempool = self.mempool.write().await;
        let mut utxos = self.utxos.write().await;
        
        for (txid, tx) in mempool.drain() {
            // Remove spent inputs
            for input in &tx.inputs {
                let utxo_key = format!("{}:{}", input.txid, input.vout);
                utxos.remove(&utxo_key);
            }
            // Add new outputs
            for (vout, output) in tx.outputs.iter().enumerate() {
                let utxo_key = format!("{}:{}", txid, vout);
                utxos.insert(utxo_key, Utxo {
                    txid: txid.clone(),
                    vout: vout as u32,
                    amount: output.amount,
                    address: output.address.clone(),
                });
            }
        }
    }
}
