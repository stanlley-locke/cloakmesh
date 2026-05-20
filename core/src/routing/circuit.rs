//! Adaptive Onion Routing Circuit Implementation
//!
//! Circuits are the backbone of metadata-resistant communication in CloakMesh.
//! They provide a fixed-path tunnel of 2-6 hops with layered encryption.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument};
use uuid::Uuid;

use crate::crypto::session::BidirectionalSession;
use crate::errors::{CloakError, CloakResult};

/// Represents a single hop in an onion circuit.
#[derive(Clone)]
pub struct CircuitHop {
    pub peer_id: String,
    pub address: String,
    /// The session used to encrypt/decrypt cells for this specific relay.
    pub session: Arc<RwLock<BidirectionalSession>>,
}

/// A pre-established onion routing circuit.
pub struct Circuit {
    pub id: String,
    pub hops: Vec<CircuitHop>,
    pub created_at: tokio::time::Instant,
    pub is_active: bool,
}

impl Circuit {
    pub fn new(hops: Vec<CircuitHop>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            hops,
            created_at: tokio::time::Instant::now(),
            is_active: true,
        }
    }

    /// Peel one layer of onion encryption (Incoming Cell).
    pub async fn decrypt_layer(&self, payload: Vec<u8>, hop_idx: usize, nonce: &[u8; 12]) -> CloakResult<Vec<u8>> {
        if hop_idx >= self.hops.len() {
            return Err(CloakError::Protocol("Hop index out of bounds".into()));
        }
        let session = self.hops[hop_idx].session.read().await;
        session.decrypt(nonce, &payload, b"circuit-layer")
    }

    /// Wrap payload in multiple layers of encryption (Outgoing Cell).
    pub async fn encrypt_all_layers(&self, mut payload: Vec<u8>) -> CloakResult<Vec<u8>> {
        // Encrypt from the last hop back to the first hop
        for hop in self.hops.iter().rev() {
            let mut session = hop.session.write().await;
            payload = session.encrypt(&payload, b"circuit-layer")?;
        }
        Ok(payload)
    }
}

/// Orchestrates the building and rotation of multiple circuits.
pub struct CircuitManager {
    circuits: Arc<RwLock<HashMap<String, Arc<Circuit>>>>,
}

impl CircuitManager {
    pub fn new() -> Self {
        Self {
            circuits: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Build a new circuit with the specified number of hops.
    #[instrument(skip(self, candidates))]
    pub async fn build_circuit(&self, hops_count: usize, candidates: Vec<(String, String)>) -> CloakResult<String> {
        if candidates.len() < hops_count {
            return Err(CloakError::InsufficientRelays { need: hops_count, have: candidates.len() });
        }

        info!(hops = hops_count, "Building new onion circuit");
        
        let mut hops = Vec::new();
        for i in 0..hops_count {
            let (peer_id, address) = &candidates[i];
            let session = BidirectionalSession::generate_mock(); 
            hops.push(CircuitHop {
                peer_id: peer_id.clone(),
                address: address.clone(),
                session: Arc::new(RwLock::new(session)),
            });
        }

        let circuit = Arc::new(Circuit::new(hops));
        let circuit_id = circuit.id.clone();
        self.circuits.write().await.insert(circuit_id.clone(), circuit);
        
        info!(id = %circuit_id, "Circuit established and ready");
        Ok(circuit_id)
    }

    pub async fn get_circuit(&self, id: &str) -> Option<Arc<Circuit>> {
        self.circuits.read().await.get(id).cloned()
    }

    /// Select a random active circuit for an outgoing request.
    pub async fn select_random_circuit(&self) -> Option<Arc<Circuit>> {
        let guard = self.circuits.read().await;
        let active: Vec<_> = guard.values().filter(|c| c.is_active).collect();
        if active.is_empty() {
            None
        } else {
            use rand::seq::SliceRandom;
            active.choose(&mut rand::thread_rng()).map(|&c| c.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_layer_encryption() {
        let manager = CircuitManager::new();
        let candidates = vec![
            ("node1".into(), "1.1.1.1:4001".into()),
            ("node2".into(), "2.2.2.2:4001".into()),
            ("node3".into(), "3.3.3.3:4001".into()),
        ];

        let id = manager.build_circuit(3, candidates).await.unwrap();
        let circuit = manager.get_circuit(&id).await.unwrap();

        let plaintext = b"Hello through the onion!".to_vec();
        let encrypted = circuit.encrypt_all_layers(plaintext.clone()).await.unwrap();
        
        assert_ne!(plaintext, encrypted);
        assert_eq!(encrypted.len(), plaintext.len() + (16 * 3)); // 3 hops, each adds 16B tag
    }
}
