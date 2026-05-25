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
use crate::storage::db::{StorageBackend, VolatileStorage, PersistentStorage};

use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::proto::v1::cloak_mesh_node_client::CloakMeshNodeClient;
use crate::proto::v1::{FindValueRequest, StoreValueRequest};

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

pub struct DhtNode {
    #[allow(dead_code)]
    routing: Arc<RwLock<RoutingTable>>,
    storage: Arc<RwLock<Box<dyn StorageBackend>>>,
    authorities: Vec<String>, // Directory Authorities
}

impl DhtNode {
    pub fn new(local_id: DhtKey, mine_atk: bool, bootstrap_peers: Vec<String>, data_dir: String) -> Self {
        let storage: Box<dyn StorageBackend> = if mine_atk {
            match PersistentStorage::new(&format!("{}/sled_db", data_dir)) {
                Ok(db) => {
                    tracing::info!("[200] Persistent Sled storage initialized for ATK mining");
                    Box::new(db)
                },
                Err(e) => {
                    tracing::error!("Failed to init Sled DB: {}. Falling back to volatile storage.", e);
                    Box::new(VolatileStorage::new())
                }
            }
        } else {
            Box::new(VolatileStorage::new())
        };

        Self {
            routing: Arc::new(RwLock::new(RoutingTable::new(local_id))),
            storage: Arc::new(RwLock::new(storage)),
            authorities: bootstrap_peers,
        }
    }

    /// Decentralized Bootstrapping
    /// Connects to directory authorities, adds them to the routing table, then
    /// performs a FindNode RPC for our own ID to discover all peers the authority knows.
    pub async fn bootstrap(&self) -> CloakResult<()> {
        info!("[201] Starting decentralized bootstrapping via {} directory authorities...", self.authorities.len());
        
        let local_id_hex = {
            let rt = self.routing.read().await;
            hex::encode(&rt.local_id.0)
        };

        for auth in &self.authorities {
            debug!("Querying authority: {}", auth);
            let addr = format!("http://{}", auth);
            if let Ok(mut client) = CloakMeshNodeClient::connect(addr.clone()).await {
                // Step 1: Ping to add the authority itself to our routing table
                let req = tonic::Request::new(crate::proto::v1::Ping {
                    nonce: 0,
                    sent_at: Some(prost_types::Timestamp {
                        seconds: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64,
                        nanos: 0,
                    }),
                });
                if let Ok(resp) = client.keep_alive(req).await {
                    let pong = resp.into_inner();
                    if let Ok(bytes) = hex::decode(&pong.node_id) {
                        if bytes.len() == 32 {
                            let mut id = [0u8; 32];
                            id.copy_from_slice(&bytes);
                            let peer = PeerInfo {
                                id: DhtKey(id),
                                address: auth.clone(),
                                last_seen: tokio::time::Instant::now(),
                                reputation: 100,
                                flags: std::collections::HashSet::new(),
                            };
                            let added = self.routing.write().await.add_peer(peer);
                            if added {
                                info!("[201] Added bootstrap authority {} (id: {})", auth, hex::encode(&id[0..4]));
                            } else {
                                // Same-ID node or full bucket — still usable via authorities list
                                info!("[201] Bootstrap authority {} reachable (same identity or full bucket)", auth);
                            }
                        }
                    }
                }

                // Step 2: FindNode for our own ID to discover peers the authority knows
                // This is the core of Kademlia bootstrapping: "who else is near me?"
                let find_req = tonic::Request::new(crate::proto::v1::FindNodeRequest {
                    target_id: local_id_hex.clone(),
                });
                if let Ok(resp) = client.find_node(find_req).await {
                    let discovered = resp.into_inner().nodes;
                    info!("[201] Peer discovery via FindNode returned {} candidates", discovered.len());
                    for contact in discovered {
                        if let Ok(bytes) = hex::decode(&contact.node_id) {
                            if bytes.len() == 32 && !contact.address.is_empty() {
                                let mut id = [0u8; 32];
                                id.copy_from_slice(&bytes);
                                let peer = PeerInfo {
                                    id: DhtKey(id),
                                    address: contact.address.clone(),
                                    last_seen: tokio::time::Instant::now(),
                                    reputation: 80,
                                    flags: std::collections::HashSet::new(),
                                };
                                if self.routing.write().await.add_peer(peer) {
                                    info!("[201] Discovered peer {} at {}", hex::encode(&id[0..4]), contact.address);
                                }
                            }
                        }
                    }
                }
            } else {
                tracing::warn!("Failed to connect to bootstrap peer {}", auth);
            }
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
        self.storage.write().await.insert(key, value, ttl)
    }

    /// Retrieve a value locally. In a full implementation, this initiates a FIND_VALUE query.
    pub async fn get_local(&self, key: &DhtKey) -> CloakResult<Option<Vec<u8>>> {
        self.storage.read().await.get(key)
    }

    /// List all known peers in the routing table.
    pub async fn list_peers(&self) -> Vec<PeerInfo> {
        let routing = self.routing.read().await;
        routing.buckets.iter().flat_map(|b| b.peers.iter().cloned()).collect()
    }

    /// Retrieve the K closest peers to a given key
    pub async fn get_closest_peers(&self, key: &DhtKey, count: usize) -> Vec<PeerInfo> {
        let routing = self.routing.read().await;
        routing.find_closest(key, count)
    }

    /// True Kademlia Iterative Find Value.
    /// Falls back to directly querying bootstrap/authority peers if the routing
    /// table is empty (e.g. all nodes share the same identity key during dev).
    pub async fn find_value_network(&self, key: &DhtKey) -> CloakResult<Option<Vec<u8>>> {
        // 1. Check local storage first
        if let Some(val) = self.get_local(key).await? {
            return Ok(Some(val));
        }

        // 2. Build initial candidate list from routing table (closest peers to key)
        let mut to_query: Vec<String> = {
            let routing = self.routing.read().await;
            routing.find_closest(key, 3).into_iter().map(|p| p.address).collect()
        };

        // 3. If routing table has no peers (e.g. same-ID nodes can't add each other),
        //    fall back to querying bootstrap/authority peers directly.
        if to_query.is_empty() {
            debug!("Routing table empty — querying {} bootstrap peers directly", self.authorities.len());
            to_query.extend(self.authorities.iter().cloned());
        }

        // 4. Iterative Kademlia lookup
        let mut queried = std::collections::HashSet::new();
        while let Some(addr) = to_query.pop() {
            if !queried.insert(addr.clone()) {
                continue;
            }

            let connect_addr = format!("http://{}", addr);
            if let Ok(mut client) = CloakMeshNodeClient::connect(connect_addr).await {
                let req = tonic::Request::new(FindValueRequest {
                    target_key: hex::encode(&key.0),
                });

                if let Ok(resp) = client.find_value(req).await {
                    match resp.into_inner().result {
                        Some(crate::proto::v1::find_value_response::Result::Value(val)) => {
                            info!("Found value at peer {}", addr);
                            return Ok(Some(val));
                        }
                        Some(crate::proto::v1::find_value_response::Result::Closest(closest_resp)) => {
                            // Peer doesn't have it — add its suggested closer nodes
                            for node in closest_resp.nodes {
                                if !queried.contains(&node.address) {
                                    to_query.push(node.address);
                                }
                            }
                        }
                        None => {}
                    }
                }
            }
        }

        Ok(None)
    }

    /// True Kademlia Replicated Store.
    /// Stores locally, then replicates to the K closest peers in the routing table.
    /// Falls back to replicating directly to bootstrap/authority peers when the
    /// routing table is empty.
    pub async fn store_value_network(&self, key: &DhtKey, value: Vec<u8>, ttl: Duration) -> CloakResult<()> {
        // 1. Store locally
        self.store_local(key.clone(), value.clone(), ttl).await?;

        // 2. Collect replication targets from routing table
        let mut targets: Vec<String> = {
            let routing = self.routing.read().await;
            routing.find_closest(key, 3).into_iter().map(|p| p.address).collect()
        };

        // 3. Fall back to bootstrap peers if routing table is empty
        if targets.is_empty() {
            debug!("Routing table empty — replicating to {} bootstrap peers directly", self.authorities.len());
            targets.extend(self.authorities.iter().cloned());
        }

        // 4. Push value to each target
        for addr in targets {
            let connect_addr = format!("http://{}", addr);
            if let Ok(mut client) = CloakMeshNodeClient::connect(connect_addr).await {
                let req = tonic::Request::new(StoreValueRequest {
                    key: hex::encode(&key.0),
                    value: value.clone(),
                    ttl_seconds: ttl.as_secs(),
                });
                match client.store_value_network(req).await {
                    Ok(_) => info!("Replicated DHT value to peer {}", addr),
                    Err(e) => debug!("Failed to replicate to {}: {}", addr, e),
                }
            }
        }
        Ok(())
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
