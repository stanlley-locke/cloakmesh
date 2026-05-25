use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::Duration;
use crate::routing::dht::{DhtKey, ZeroizeWrapper};
use crate::errors::CloakResult;

/// A unified trait for storing DHT items (volatile RAM or persistent DB)
pub trait StorageBackend: Send + Sync {
    fn insert(&mut self, key: DhtKey, value: Vec<u8>, ttl: Duration) -> CloakResult<()>;
    fn get(&self, key: &DhtKey) -> CloakResult<Option<Vec<u8>>>;
    fn cleanup(&mut self);
}

/// Strict Volatile RAM implementation (Zeroize on drop)
pub struct VolatileStorage {
    store: HashMap<DhtKey, (Arc<ZeroizeWrapper>, tokio::time::Instant)>,
}

impl VolatileStorage {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }
}

impl StorageBackend for VolatileStorage {
    fn insert(&mut self, key: DhtKey, value: Vec<u8>, ttl: Duration) -> CloakResult<()> {
        self.store.insert(key, (Arc::new(ZeroizeWrapper(value)), tokio::time::Instant::now() + ttl));
        Ok(())
    }

    fn get(&self, key: &DhtKey) -> CloakResult<Option<Vec<u8>>> {
        if let Some((wrapper, expiry)) = self.store.get(key) {
            if tokio::time::Instant::now() < *expiry {
                return Ok(Some(wrapper.0.clone()));
            }
        }
        Ok(None)
    }

    fn cleanup(&mut self) {
        let now = tokio::time::Instant::now();
        self.store.retain(|_, (_, expiry)| *expiry > now);
    }
}

/// Persistent Storage implementation for ATK Miners (Uses sled)
pub struct PersistentStorage {
    db: sled::Db,
}

impl PersistentStorage {
    pub fn new(path: &str) -> CloakResult<Self> {
        let db = sled::open(path).map_err(|e| crate::errors::CloakError::Other(anyhow::anyhow!("Sled err: {}", e)))?;
        Ok(Self { db })
    }
}

impl StorageBackend for PersistentStorage {
    fn insert(&mut self, key: DhtKey, value: Vec<u8>, _ttl: Duration) -> CloakResult<()> {
        // Sled persists to disk
        self.db.insert(key.0, value).map_err(|e| crate::errors::CloakError::Other(anyhow::anyhow!("Sled insert err: {}", e)))?;
        self.db.flush().map_err(|e| crate::errors::CloakError::Other(anyhow::anyhow!("Sled flush err: {}", e)))?;
        Ok(())
    }

    fn get(&self, key: &DhtKey) -> CloakResult<Option<Vec<u8>>> {
        let val = self.db.get(key.0).map_err(|e| crate::errors::CloakError::Other(anyhow::anyhow!("Sled get err: {}", e)))?;
        Ok(val.map(|ivec| ivec.to_vec()))
    }

    fn cleanup(&mut self) {
        // Persistent storage does not expire items via TTL strictly in this demo
        // In full production, a background thread would GC expired items from sled.
    }
}
