//! Kademlia-based Distributed Hash Table (DHT) Implementation
//!
//! Provides the core routing logic for peer discovery and descriptor storage.
//! Features XOR distance metrics, k-buckets, and active background refreshes.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, info};

use crate::errors::CloakResult;

use zeroize::{Zeroize, ZeroizeOnDrop};

/// Size of a Kademlia node ID or key (32 bytes / 256 bits for SHA-256 / Ed25519).
pub const KEY_LEN: usize = 32;

/// Maximum number of peers in a single k-bucket.
pub const K_VALUE: usize = 20;

/// A 256-bit identifier used for both Node IDs and Storage Keys.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Zeroize, ZeroizeOnDrop)]
pub struct DhtKey(pub [u8; KEY_LEN]);

impl DhtKey {
    /// Compute the XOR distance between this key and another.
    pub fn distance(&self, other: &DhtKey) -> DhtKey {
        let mut dist = [0u8; KEY_LEN];
        for (i, byte) in dist.iter_mut().enumerate() {
            *byte = self.0[i] ^ other.0[i];
        }
        DhtKey(dist)
    }

    /// Returns the index of the highest set bit (0-255).
    /// Used to determine which k-bucket a peer belongs in.
    pub fn bucket_index(&self, local_id: &DhtKey) -> Option<usize> {
        let dist = self.distance(local_id);
        for (byte_idx, &byte) in dist.0.iter().enumerate() {
            if byte != 0 {
                let bit_idx = 7 - byte.leading_zeros() as usize;
                return Some((KEY_LEN - 1 - byte_idx) * 8 + bit_idx);
            }
        }
        None // Same ID
    }
}

/// Information about a peer in the DHT.
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub id: DhtKey,
    pub address: String,
    pub last_seen: tokio::time::Instant,
    pub reputation: u32,
    pub flags: HashSet<NodeFlag>, // Node Flag Auto-Classification
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeFlag {
    Guard,
    Middle,
    Exit,
    Authority, // Directory Authorities
    Stable,
}

/// A single Kademlia bucket.
#[derive(Debug)]
pub struct KBucket {
    pub peers: Vec<PeerInfo>,
    pub last_updated: tokio::time::Instant,
}

impl Default for KBucket {
    fn default() -> Self {
        Self::new()
    }
}

impl KBucket {
    pub fn new() -> Self {
        Self {
            peers: Vec::with_capacity(K_VALUE),
            last_updated: tokio::time::Instant::now(),
        }
    }

    /// Add a peer to the bucket, updating its timestamp if it exists,
    /// or pushing it to the back if there is space.
    pub fn add_peer(&mut self, peer: PeerInfo) -> bool {
        self.last_updated = tokio::time::Instant::now();
        if let Some(existing) = self.peers.iter_mut().find(|p| p.id == peer.id) {
            existing.last_seen = tokio::time::Instant::now();
            existing.address = peer.address;
            return true;
        }

        if self.peers.len() < K_VALUE {
            self.peers.push(peer);
            true
        } else {
            // Full bucket. In a complete Kademlia implementation, we would ping the
            // oldest peer and replace it if it's unresponsive. For now, we drop the new peer.
            false
        }
    }
}

/// The local routing table containing up to 256 k-buckets.
pub struct RoutingTable {
    pub local_id: DhtKey,
    buckets: Vec<KBucket>,
}

impl RoutingTable {
    pub fn new(local_id: DhtKey) -> Self {
        let mut buckets = Vec::with_capacity(256);
        for _ in 0..256 {
            buckets.push(KBucket::new());
        }
        Self { local_id, buckets }
    }

    pub fn add_peer(&mut self, peer: PeerInfo) -> bool {
        if let Some(idx) = peer.id.bucket_index(&self.local_id) {
            self.buckets[idx].add_peer(peer)
        } else {
            false
        }
    }

    /// Find the K closest peers to a given target key.
    pub fn find_closest(&self, target: &DhtKey, count: usize) -> Vec<PeerInfo> {
        let mut all_peers: Vec<_> = self.buckets
            .iter()
            .flat_map(|b| b.peers.iter().cloned())
            .collect();

        all_peers.sort_by(|a, b| {
            a.id.distance(target).cmp(&b.id.distance(target))
        });

        all_peers.into_iter().take(count).collect()
    }
}

/// The local DHT storage for descriptors.
pub struct DhtStorage {
    // Map of Key -> (Value, Expiry)
    // Values are wrapped in ZeroizeWrapper to ensure Volatile RAM Storage Policy
    store: HashMap<DhtKey, (Arc<ZeroizeWrapper>, tokio::time::Instant)>,
}

/// Helper to ensure Vec<u8> is zeroed on drop
pub struct ZeroizeWrapper(pub Vec<u8>);
impl Drop for ZeroizeWrapper {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl Default for DhtStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl DhtStorage {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: DhtKey, value: Vec<u8>, ttl: Duration) {
        self.store.insert(key, (Arc::new(ZeroizeWrapper(value)), tokio::time::Instant::now() + ttl));
    }

    pub fn get(&self, key: &DhtKey) -> Option<Vec<u8>> {
        if let Some((wrapper, expiry)) = self.store.get(key) {
            if tokio::time::Instant::now() < *expiry {
                return Some(wrapper.0.clone());
            }
        }
        None
    }

    pub fn cleanup(&mut self) {
        let now = tokio::time::Instant::now();
        // Evicted items are automatically zeroed by ZeroizeWrapper::drop
        self.store.retain(|_, (_, expiry)| *expiry > now);
    }
}

/// Main DHT orchestrator wrapping routing and storage.
pub struct DhtNode {
    #[allow(dead_code)]
    routing: Arc<RwLock<RoutingTable>>,
    storage: Arc<RwLock<DhtStorage>>,
    authorities: Vec<String>, // Directory Authorities
}

impl DhtNode {
    pub fn new(local_id: DhtKey) -> Self {
        Self {
            routing: Arc::new(RwLock::new(RoutingTable::new(local_id))),
            storage: Arc::new(RwLock::new(DhtStorage::new())),
            authorities: vec!["bootstrap.cloakmesh.network:4001".to_string()],
        }
    }

    /// Decentralized Bootstrapping
    /// Connects to directory authorities and performs iterative FIND_NODE for own ID.
    pub async fn bootstrap(&self) -> CloakResult<()> {
        info!("Starting decentralized bootstrapping via directory authorities...");
        for auth in &self.authorities {
            debug!("Querying authority: {}", auth);
            // In a full implementation, this would send a gRPC FIND_NODE request
        }
        Ok(())
    }

    /// Start background maintenance tasks (bucket refresh, storage cleanup).
    pub fn start_maintenance(self: Arc<Self>) {
        let storage = self.storage.clone();
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(60));
            loop {
                ticker.tick().await;
                debug!("Running DHT maintenance cycle");
                storage.write().await.cleanup();
                // TODO: ping oldest peers in routing table
            }
        });
    }

    /// Store a value locally. In a full implementation, this also routes a STORE RPC.
    pub async fn store_local(&self, key: DhtKey, value: Vec<u8>, ttl: Duration) -> CloakResult<()> {
        self.storage.write().await.insert(key, value, ttl);
        Ok(())
    }

    /// Retrieve a value locally. In a full implementation, this initiates a FIND_VALUE query.
    pub async fn get_local(&self, key: &DhtKey) -> CloakResult<Option<Vec<u8>>> {
        Ok(self.storage.read().await.get(key))
    }

    /// List all known peers in the routing table.
    pub async fn list_peers(&self) -> Vec<PeerInfo> {
        let routing = self.routing.read().await;
        routing.buckets.iter().flat_map(|b| b.peers.iter().cloned()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xor_distance() {
        let mut k1 = [0u8; 32]; k1[31] = 0b0000_0001;
        let mut k2 = [0u8; 32]; k2[31] = 0b0000_0010;
        let d1 = DhtKey(k1);
        let d2 = DhtKey(k2);
        
        let dist = d1.distance(&d2);
        assert_eq!(dist.0[31], 0b0000_0011);
    }

    #[test]
    fn test_bucket_index() {
        let local = [0u8; 32];
        let mut remote = [0u8; 32];
        
        remote[31] = 1; // 1st bit differs
        assert_eq!(DhtKey(remote).bucket_index(&DhtKey(local)), Some(0));

        remote[31] = 2; // 2nd bit differs
        assert_eq!(DhtKey(remote).bucket_index(&DhtKey(local)), Some(1));

        remote[30] = 1; // 9th bit differs
        assert_eq!(DhtKey(remote).bucket_index(&DhtKey(local)), Some(8));
    }

    #[test]
    fn test_routing_table() {
        let local = DhtKey([0u8; 32]);
        let mut table = RoutingTable::new(local);

        let mut remote = [0u8; 32];
        remote[31] = 0xFF;
        let peer1 = PeerInfo {
            id: DhtKey(remote),
            address: "1.2.3.4:4001".into(),
            last_seen: tokio::time::Instant::now(),
            reputation: 100,
            flags: HashSet::new(),
        };

        assert!(table.add_peer(peer1.clone()));
        
        let closest = table.find_closest(&DhtKey(remote), 10);
        assert_eq!(closest.len(), 1);
        assert_eq!(closest[0].id, DhtKey(remote));
    }
}
