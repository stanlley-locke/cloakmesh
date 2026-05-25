use reed_solomon_erasure::galois_8::ReedSolomon;
use crate::errors::{CloakError, CloakResult};

/// Encodes a file's data into data shards and parity shards
pub fn encode_file(data: &[u8], data_shards: usize, parity_shards: usize) -> CloakResult<Vec<Vec<u8>>> {
    let r = ReedSolomon::new(data_shards, parity_shards)
        .map_err(|e| CloakError::Other(anyhow::anyhow!("RS Init Error: {}", e)))?;
        
    // Calculate shard size (must be divisible by data_shards)
    let chunk_size = (data.len() + data_shards - 1) / data_shards;
    
    let mut shards: Vec<Vec<u8>> = vec![vec![0; chunk_size]; data_shards + parity_shards];
    
    // Copy data into shards
    for i in 0..data_shards {
        let start = i * chunk_size;
        let end = std::cmp::min(start + chunk_size, data.len());
        let slice = &data[start..end];
        shards[i][..slice.len()].copy_from_slice(slice);
    }
    
    // Construct parity shards
    r.encode(&mut shards)
        .map_err(|e| CloakError::Other(anyhow::anyhow!("RS Encode Error: {}", e)))?;
        
    Ok(shards)
}

/// Reconstructs the original data from available shards
pub fn reconstruct_file(mut shards: Vec<Option<Vec<u8>>>, data_shards: usize, parity_shards: usize, original_size: usize) -> CloakResult<Vec<u8>> {
    let r = ReedSolomon::new(data_shards, parity_shards)
        .map_err(|e| CloakError::Other(anyhow::anyhow!("RS Init Error: {}", e)))?;
        
    r.reconstruct(&mut shards)
        .map_err(|e| CloakError::Other(anyhow::anyhow!("RS Reconstruct Error: {}", e)))?;
        
    let mut result = Vec::with_capacity(original_size);
    for i in 0..data_shards {
        if let Some(shard) = &shards[i] {
            result.extend_from_slice(shard);
        }
    }
    result.truncate(original_size);
    
    Ok(result)
}
