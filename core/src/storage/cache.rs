use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

// ── LRU Cache ────────────────────────────────────────────────────────────────

/// A fixed-capacity LRU cache with TTL support.
/// Used to cache fetched descriptors and session state.
pub struct LruCache<K, V> {
    capacity: usize,
    ttl: Option<Duration>,
    map: HashMap<K, (V, Instant)>,
    order: VecDeque<K>,
}

impl<K, V> LruCache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
{
    pub fn new(capacity: usize, ttl: Option<Duration>) -> Self {
        assert!(capacity > 0, "LruCache capacity must be > 0");
        Self {
            capacity,
            ttl,
            map: HashMap::with_capacity(capacity),
            order: VecDeque::with_capacity(capacity),
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        if self.map.contains_key(&key) {
            // Update existing: remove from order, re-insert at back
            self.order.retain(|k| k != &key);
        } else if self.map.len() >= self.capacity {
            // Evict LRU (front of deque)
            if let Some(lru_key) = self.order.pop_front() {
                self.map.remove(&lru_key);
            }
        }
        self.map.insert(key.clone(), (value, Instant::now()));
        self.order.push_back(key);
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        let entry = self.map.get(key)?;
        // TTL check
        if let Some(ttl) = self.ttl {
            if entry.1.elapsed() > ttl {
                self.map.remove(key);
                self.order.retain(|k| k != key);
                return None;
            }
        }
        // Move to back (most recently used)
        self.order.retain(|k| k != key);
        self.order.push_back(key.clone());
        self.map.get(key).map(|(v, _)| v)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.order.retain(|k| k != key);
        self.map.remove(key).map(|(v, _)| v)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn evict_expired(&mut self) -> usize {
        if self.ttl.is_none() {
            return 0;
        }
        let ttl = self.ttl.unwrap();
        let expired: Vec<K> = self
            .map
            .iter()
            .filter(|(_, (_, t))| t.elapsed() > ttl)
            .map(|(k, _)| k.clone())
            .collect();
        let count = expired.len();
        for k in &expired {
            self.map.remove(k);
            self.order.retain(|ok| ok != k);
        }
        count
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_insert_and_get() {
        let mut cache: LruCache<&str, u32> = LruCache::new(3, None);
        cache.insert("a", 1);
        cache.insert("b", 2);
        assert_eq!(cache.get(&"a"), Some(&1));
        assert_eq!(cache.get(&"b"), Some(&2));
        assert_eq!(cache.get(&"c"), None);
    }

    #[test]
    fn lru_eviction() {
        let mut cache: LruCache<u32, u32> = LruCache::new(3, None);
        cache.insert(1, 10);
        cache.insert(2, 20);
        cache.insert(3, 30);
        // Access 1 to make it recently used
        cache.get(&1);
        // Insert 4 — should evict 2 (LRU)
        cache.insert(4, 40);
        assert_eq!(cache.get(&1), Some(&10));
        assert_eq!(cache.get(&2), None); // evicted
        assert_eq!(cache.get(&3), Some(&30));
        assert_eq!(cache.get(&4), Some(&40));
    }

    #[test]
    fn ttl_expiry() {
        let mut cache: LruCache<&str, u32> = LruCache::new(10, Some(Duration::from_millis(5)));
        cache.insert("k", 42);
        assert_eq!(cache.get(&"k"), Some(&42));
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(cache.get(&"k"), None);
    }

    #[test]
    fn update_existing_key() {
        let mut cache: LruCache<&str, u32> = LruCache::new(3, None);
        cache.insert("k", 1);
        cache.insert("k", 2);
        assert_eq!(cache.get(&"k"), Some(&2));
        assert_eq!(cache.len(), 1);
    }
}
