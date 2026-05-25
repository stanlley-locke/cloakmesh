use crate::proto::v1::FileShard;
use sha2::{Sha256, Digest};

/// Hashes data using SHA-256 and returns hex string
pub fn hash_data(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Converts raw erasure shards into Protobuf FileShard messages
pub fn create_file_shards(file_hash: &str, raw_shards: Vec<Vec<u8>>, data_shards: usize) -> Vec<FileShard> {
    raw_shards.into_iter().enumerate().map(|(i, data)| {
        FileShard {
            file_hash: file_hash.to_string(),
            shard_index: i as u32,
            is_parity: i >= data_shards,
            data,
        }
    }).collect()
}
