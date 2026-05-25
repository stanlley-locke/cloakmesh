use crate::proto::v1::FileManifest;
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};

/// Generates a FileManifest for a given file and its shard hashes
pub fn generate_manifest(
    file_data: &[u8],
    data_shards: u32,
    parity_shards: u32,
    shard_hashes: Vec<String>,
    owner_address: String,
) -> FileManifest {
    let file_hash = hex::encode(Sha256::digest(file_data));
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    FileManifest {
        file_hash,
        size_bytes: file_data.len() as u64,
        data_shards,
        parity_shards,
        shard_hashes,
        owner_address,
        timestamp,
    }
}
