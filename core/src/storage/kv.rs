use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::errors::{CloakError, CloakResult};

// ── Configuration ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct KvConfig {
    /// Maximum number of entries. Oldest entries are evicted when full.
    pub max_entries: usize,
    /// Maximum value size in bytes. Larger values are rejected.
    pub max_value_bytes: usize,
}

impl Default for KvConfig {
    fn default() -> Self {
        Self {
            max_entries: 10_000,
            max_value_bytes: 64 * 1024, // 64 KiB
        }
    }
}

// ── Entry ────────────────────────────────────────────────────────────────────

struct Entry {
    value: Vec<u8>,
    expires_at: Option<Instant>,
    inserted_at: Instant,
}

impl Entry {
    fn is_expired(&self) -> bool {
        self.expires_at.map(|t| Instant::now() >= t).unwrap_or(false)
    }
}

// ── KV Store ─────────────────────────────────────────────────────────────────

/// In-memory key-value store with TTL expiry, size limits, and LRU-style eviction.
pub struct KvStore {
    data: HashMap<Vec<u8>, Entry>,
    config: KvConfig,
    hits: u64,
    misses: u64,
    evictions: u64,
}

impl KvStore {
    pub fn new(config: KvConfig) -> Self {
        Self {
            data: HashMap::with_capacity(config.max_entries),
            config,
            hits: 0,
            misses: 0,
            evictions: 0,
        }
    }

    /// Insert or update a key. Returns an error if the value exceeds the size limit.
    pub fn set(&mut self, key: Vec<u8>, value: Vec<u8>, ttl: Option<Duration>) -> CloakResult<()> {
        if value.len() > self.config.max_value_bytes {
            return Err(CloakError::StorageSerde(format!(
                "value {} bytes exceeds max {} bytes",
                value.len(),
                self.config.max_value_bytes
            )));
        }
        // Evict expired entries first to make room
        if self.data.len() >= self.config.max_entries && !self.data.contains_key(&key) {
            self.evict_expired();
            // If still full, evict the oldest entry
            if self.data.len() >= self.config.max_entries {
                self.evict_oldest();
            }
        }
        self.data.insert(
            key,
            Entry {
                value,
                expires_at: ttl.map(|d| Instant::now() + d),
                inserted_at: Instant::now(),
            },
        );
        Ok(())
    }

    /// Retrieve a value. Returns `None` if the key is absent or expired.
    pub fn get(&mut self, key: &[u8]) -> Option<&[u8]> {
        // Check expiry first without holding a borrow
        let expired = self.data.get(key).map(|e| e.is_expired()).unwrap_or(false);
        if expired {
            self.data.remove(key);
            self.misses += 1;
            return None;
        }
        match self.data.get(key) {
            Some(e) => {
                self.hits += 1;
                Some(e.value.as_slice())
            }
            None => {
                self.misses += 1;
                None
            }
        }
    }

    /// Check existence without updating hit/miss counters.
    pub fn contains(&self, key: &[u8]) -> bool {
        self.data.get(key).map(|e| !e.is_expired()).unwrap_or(false)
    }

    pub fn delete(&mut self, key: &[u8]) -> bool {
        self.data.remove(key).is_some()
    }

    /// Remove all expired entries. Returns the number of entries removed.
    pub fn evict_expired(&mut self) -> usize {
        let before = self.data.len();
        self.data.retain(|_, e| !e.is_expired());
        let removed = before - self.data.len();
        self.evictions += removed as u64;
        removed
    }

    /// Remove the single oldest entry (by insertion time).
    fn evict_oldest(&mut self) {
        if let Some(oldest_key) = self
            .data
            .iter()
            .min_by_key(|(_, e)| e.inserted_at)
            .map(|(k, _)| k.clone())
        {
            self.data.remove(&oldest_key);
            self.evictions += 1;
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn stats(&self) -> KvStats {
        KvStats {
            entries: self.data.len(),
            hits: self.hits,
            misses: self.misses,
            evictions: self.evictions,
            hit_rate: if self.hits + self.misses > 0 {
                self.hits as f64 / (self.hits + self.misses) as f64
            } else {
                0.0
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct KvStats {
    pub entries: usize,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub hit_rate: f64,
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> KvStore {
        KvStore::new(KvConfig::default())
    }

    #[test]
    fn set_and_get() {
        let mut s = store();
        s.set(b"key".to_vec(), b"value".to_vec(), None).unwrap();
        assert_eq!(s.get(b"key"), Some(b"value".as_slice()));
    }

    #[test]
    fn missing_key_returns_none() {
        let mut s = store();
        assert_eq!(s.get(b"missing"), None);
    }

    #[test]
    fn expired_entry_returns_none() {
        let mut s = store();
        s.set(b"k".to_vec(), b"v".to_vec(), Some(Duration::from_millis(1))).unwrap();
        std::thread::sleep(Duration::from_millis(5));
        assert_eq!(s.get(b"k"), None);
    }

    #[test]
    fn delete_removes_entry() {
        let mut s = store();
        s.set(b"k".to_vec(), b"v".to_vec(), None).unwrap();
        assert!(s.delete(b"k"));
        assert_eq!(s.get(b"k"), None);
    }

    #[test]
    fn evict_expired_cleans_up() {
        let mut s = store();
        s.set(b"a".to_vec(), b"1".to_vec(), Some(Duration::from_millis(1))).unwrap();
        s.set(b"b".to_vec(), b"2".to_vec(), None).unwrap();
        std::thread::sleep(Duration::from_millis(5));
        let removed = s.evict_expired();
        assert_eq!(removed, 1);
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn oversized_value_rejected() {
        let mut s = KvStore::new(KvConfig { max_entries: 100, max_value_bytes: 10 });
        let result = s.set(b"k".to_vec(), vec![0u8; 11], None);
        assert!(result.is_err());
    }

    #[test]
    fn max_entries_evicts_oldest() {
        let mut s = KvStore::new(KvConfig { max_entries: 3, max_value_bytes: 1024 });
        s.set(b"a".to_vec(), b"1".to_vec(), None).unwrap();
        s.set(b"b".to_vec(), b"2".to_vec(), None).unwrap();
        s.set(b"c".to_vec(), b"3".to_vec(), None).unwrap();
        // Adding a 4th entry should evict the oldest
        s.set(b"d".to_vec(), b"4".to_vec(), None).unwrap();
        assert_eq!(s.len(), 3);
    }

    #[test]
    fn stats_track_hits_and_misses() {
        let mut s = store();
        s.set(b"k".to_vec(), b"v".to_vec(), None).unwrap();
        s.get(b"k");
        s.get(b"k");
        s.get(b"missing");
        let stats = s.stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
    }
}
