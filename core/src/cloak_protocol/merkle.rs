//! Merkle tree implementation for descriptor storage proofs.
//!
//! Uses SHA-256 with domain separation:
//!   leaf hash:   SHA-256(0x00 || data)
//!   parent hash: SHA-256(0x01 || left || right)
//!
//! Domain separation prevents second-preimage attacks where an attacker
//! could substitute an internal node for a leaf.

use sha2::{Digest, Sha256};

use crate::errors::{CloakError, CloakResult};

// ── Domain separation prefixes ───────────────────────────────────────────────

const LEAF_PREFIX: u8 = 0x00;
const NODE_PREFIX: u8 = 0x01;

// ── Hash helpers ─────────────────────────────────────────────────────────────

fn hash_leaf(data: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update([LEAF_PREFIX]);
    h.update(data);
    h.finalize().into()
}

fn hash_node(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update([NODE_PREFIX]);
    h.update(left);
    h.update(right);
    h.finalize().into()
}

// ── Merkle root ──────────────────────────────────────────────────────────────

/// Compute the Merkle root of a list of leaf data items.
/// Returns `[0u8; 32]` for an empty list.
pub fn merkle_root(leaves: &[Vec<u8>]) -> [u8; 32] {
    if leaves.is_empty() {
        return [0u8; 32];
    }
    let mut layer: Vec<[u8; 32]> = leaves.iter().map(|l| hash_leaf(l)).collect();
    while layer.len() > 1 {
        // Duplicate the last node if the layer has odd length (standard approach)
        if layer.len() % 2 != 0 {
            layer.push(*layer.last().unwrap());
        }
        layer = layer
            .chunks_exact(2)
            .map(|pair| hash_node(&pair[0], &pair[1]))
            .collect();
    }
    layer[0]
}

// ── Merkle proof ─────────────────────────────────────────────────────────────

/// A Merkle inclusion proof for a single leaf.
#[derive(Debug, Clone)]
pub struct MerkleProof {
    /// Index of the leaf in the original list.
    pub leaf_index: usize,
    /// Sibling hashes from leaf to root (bottom-up).
    pub siblings: Vec<[u8; 32]>,
}

impl MerkleProof {
    /// Generate an inclusion proof for the leaf at `index`.
    pub fn generate(leaves: &[Vec<u8>], index: usize) -> CloakResult<Self> {
        if leaves.is_empty() {
            return Err(CloakError::MerkleProofInvalid);
        }
        if index >= leaves.len() {
            return Err(CloakError::MerkleProofInvalid);
        }

        let mut layer: Vec<[u8; 32]> = leaves.iter().map(|l| hash_leaf(l)).collect();
        let mut siblings = Vec::new();
        let mut idx = index;

        while layer.len() > 1 {
            if layer.len() % 2 != 0 {
                layer.push(*layer.last().unwrap());
            }
            // The sibling is the node at the paired index
            let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
            siblings.push(layer[sibling_idx]);
            idx /= 2;
            layer = layer
                .chunks_exact(2)
                .map(|pair| hash_node(&pair[0], &pair[1]))
                .collect();
        }

        Ok(MerkleProof { leaf_index: index, siblings })
    }

    /// Verify this proof against a known root and leaf data.
    pub fn verify(&self, root: &[u8; 32], leaf_data: &[u8]) -> CloakResult<()> {
        let mut current = hash_leaf(leaf_data);
        let mut idx = self.leaf_index;

        for sibling in &self.siblings {
            current = if idx % 2 == 0 {
                hash_node(&current, sibling)
            } else {
                hash_node(sibling, &current)
            };
            idx /= 2;
        }

        use subtle::ConstantTimeEq;
        if current.ct_eq(root).unwrap_u8() == 1 {
            Ok(())
        } else {
            Err(CloakError::MerkleProofInvalid)
        }
    }

    /// Serialize the proof to bytes: 8-byte LE index + 32 bytes per sibling.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(8 + self.siblings.len() * 32);
        out.extend_from_slice(&(self.leaf_index as u64).to_le_bytes());
        for s in &self.siblings {
            out.extend_from_slice(s);
        }
        out
    }

    /// Deserialize from bytes.
    pub fn from_bytes(bytes: &[u8]) -> CloakResult<Self> {
        if bytes.len() < 8 || (bytes.len() - 8) % 32 != 0 {
            return Err(CloakError::MerkleProofInvalid);
        }
        let leaf_index = u64::from_le_bytes(bytes[..8].try_into().unwrap()) as usize;
        let siblings = bytes[8..]
            .chunks_exact(32)
            .map(|c| c.try_into().unwrap())
            .collect();
        Ok(MerkleProof { leaf_index, siblings })
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn leaves(n: usize) -> Vec<Vec<u8>> {
        (0..n).map(|i| format!("leaf-{}", i).into_bytes()).collect()
    }

    #[test]
    fn root_is_deterministic() {
        let l = leaves(4);
        assert_eq!(merkle_root(&l), merkle_root(&l));
    }

    #[test]
    fn single_leaf_root_equals_leaf_hash() {
        let l = vec![b"only".to_vec()];
        let root = merkle_root(&l);
        assert_eq!(root, hash_leaf(b"only"));
    }

    #[test]
    fn different_leaves_different_root() {
        let l1 = leaves(4);
        let mut l2 = leaves(4);
        l2[2] = b"tampered".to_vec();
        assert_ne!(merkle_root(&l1), merkle_root(&l2));
    }

    #[test]
    fn proof_verifies_for_all_indices() {
        for n in [1, 2, 3, 4, 5, 8, 9] {
            let l = leaves(n);
            let root = merkle_root(&l);
            for i in 0..n {
                let proof = MerkleProof::generate(&l, i).unwrap();
                assert!(proof.verify(&root, &l[i]).is_ok(), "failed for n={n}, i={i}");
            }
        }
    }

    #[test]
    fn proof_fails_for_wrong_leaf() {
        let l = leaves(4);
        let root = merkle_root(&l);
        let proof = MerkleProof::generate(&l, 0).unwrap();
        assert!(proof.verify(&root, b"wrong data").is_err());
    }

    #[test]
    fn proof_fails_for_wrong_root() {
        let l = leaves(4);
        let proof = MerkleProof::generate(&l, 0).unwrap();
        let wrong_root = [0xFF_u8; 32];
        assert!(proof.verify(&wrong_root, &l[0]).is_err());
    }

    #[test]
    fn proof_serialization_roundtrip() {
        let l = leaves(5);
        let proof = MerkleProof::generate(&l, 3).unwrap();
        let bytes = proof.to_bytes();
        let recovered = MerkleProof::from_bytes(&bytes).unwrap();
        assert_eq!(proof.leaf_index, recovered.leaf_index);
        assert_eq!(proof.siblings, recovered.siblings);
    }

    #[test]
    fn domain_separation_prevents_second_preimage() {
        // A leaf hash must not equal a node hash for the same input bytes
        let data = b"test";
        let leaf_h = hash_leaf(data);
        // Construct what a node hash would look like with the same 32 bytes
        let node_h = hash_node(&leaf_h, &leaf_h);
        assert_ne!(leaf_h, node_h);
    }
}
