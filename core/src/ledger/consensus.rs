use crate::ledger::blockchain::BlockchainState;
use crate::proto::v1::{Transaction, TransactionOutput};
use crate::storage::proofs::{Proof, verify_proof};
use std::time::{SystemTime, UNIX_EPOCH};

/// Generates a coinbase transaction to reward a node for storage proofs
pub fn generate_coinbase(target_address: &str, amount: u64) -> Transaction {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    
    // A coinbase tx has no inputs
    let txid = format!("coinbase-{}", timestamp);
    
    Transaction {
        id: txid,
        inputs: vec![],
        outputs: vec![
            TransactionOutput {
                address: target_address.to_string(),
                amount,
            }
        ],
        timestamp,
    }
}

/// Verifies a Proof of Storage from a miner and issues an ATK reward via Coinbase transaction.
pub async fn verify_storage_proof_and_reward(
    proof: &Proof,
    expected_hash: &str,
    miner_address: &str,
    state: &BlockchainState
) -> Result<Transaction, String> {
    if verify_proof(proof, expected_hash) {
        // Valid proof! Issue a reward.
        let reward_tx = generate_coinbase(miner_address, 10); // 10 ATK reward
        
        // Add to mempool / ledger
        state.mempool.write().await.insert(reward_tx.id.clone(), reward_tx.clone());
        state.process_mempool().await;
        
        Ok(reward_tx)
    } else {
        Err("Invalid Proof of Storage".into())
    }
}
