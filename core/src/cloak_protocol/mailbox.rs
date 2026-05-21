//! Cryptographic Asynchronous Mailboxes
//!
//! Provides a mechanism for storing offline messages on the DHT.
//! Messages are encrypted using the Double Ratchet protocol and can be
//! retrieved by the recipient once they come online.

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::errors::{CloakError, CloakResult};
use crate::routing::dht::{DhtNode, DhtKey};

/// A local or remote mailbox for storing messages.
pub struct Mailbox {
    pub owner_address: String,
    storage: Arc<Mutex<VecDeque<Vec<u8>>>>,
    dht: Arc<DhtNode>,
}

impl Mailbox {
    pub fn new(owner_address: String, dht: Arc<DhtNode>) -> Self {
        Self {
            owner_address,
            storage: Arc::new(Mutex::new(VecDeque::with_capacity(1000))),
            dht,
        }
    }

    /// Push a message into the mailbox.
    /// In a production scenario, this would involve DHT replication.
    pub async fn store_message(&self, recipient_pk: [u8; 32], encrypted_payload: Vec<u8>) -> CloakResult<()> {
        let key = DhtKey(recipient_pk);
        // Store locally if we are the designated mailbox node for this recipient
        let mut guard = self.storage.lock().await;
        if guard.len() >= 1000 {
            guard.pop_front(); // Evict oldest if full
        }
        guard.push_back(encrypted_payload.clone());
        
        // Also replicate to the DHT
        self.dht.store_local(key, encrypted_payload, std::time::Duration::from_secs(86400 * 7)).await?;
        Ok(())
    }

    /// Retrieve all messages from the mailbox.
    pub async fn retrieve_messages(&self) -> CloakResult<Vec<Vec<u8>>> {
        let mut guard = self.storage.lock().await;
        let messages = guard.drain(..).collect();
        Ok(messages)
    }
}
