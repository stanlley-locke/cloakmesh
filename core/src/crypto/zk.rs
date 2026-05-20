//! Zero-knowledge proof interface.
//!
//! Phase 1 ships a `MockZkProver` that always succeeds.
//! Phase v0.2 will replace it with a real Groth16/PLONK implementation
//! (e.g. `bellman` or `halo2`) without changing any call sites.

use crate::errors::CloakResult;

// ── Trait ────────────────────────────────────────────────────────────────────

/// A ZK proof system for capability verification.
///
/// The witness is private data (e.g. a secret token seed).
/// The public input is what the verifier knows (e.g. a commitment hash).
pub trait ZkProver: Send + Sync {
    /// Generate a proof that the prover knows a witness satisfying the circuit.
    fn prove(&self, witness: &[u8], public_input: &[u8]) -> CloakResult<ZkProof>;

    /// Verify a proof against a public input.
    fn verify(&self, proof: &ZkProof, public_input: &[u8]) -> CloakResult<bool>;
}

// ── Proof type ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ZkProof {
    pub bytes: Vec<u8>,
    /// Identifies the proof system (e.g. "mock-v1", "groth16-bn254").
    pub system: &'static str,
}

// ── Mock implementation ──────────────────────────────────────────────────────

/// Mock ZK prover for Phase 1 testing.
/// Proof = BLAKE2s(witness || public_input) so it is deterministic and
/// verifiable without a real circuit.
pub struct MockZkProver;

impl ZkProver for MockZkProver {
    fn prove(&self, witness: &[u8], public_input: &[u8]) -> CloakResult<ZkProof> {
        let mut data = witness.to_vec();
        data.extend_from_slice(public_input);
        let hash = crate::crypto::kdf::blake2s_hash(&data);
        Ok(ZkProof { bytes: hash.to_vec(), system: "mock-v1" })
    }

    fn verify(&self, proof: &ZkProof, public_input: &[u8]) -> CloakResult<bool> {
        // In the mock, we cannot re-derive the witness, so we just check
        // that the proof is 32 bytes and non-zero (structural validity).
        if proof.system != "mock-v1" {
            return Ok(false);
        }
        if proof.bytes.len() != 32 {
            return Ok(false);
        }
        Ok(proof.bytes.iter().any(|&b| b != 0) || public_input.is_empty())
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_prove_and_verify() {
        let prover = MockZkProver;
        let witness = b"secret_seed";
        let public_input = b"commitment_hash";
        let proof = prover.prove(witness, public_input).unwrap();
        assert!(prover.verify(&proof, public_input).unwrap());
    }

    #[test]
    fn mock_proof_is_deterministic() {
        let prover = MockZkProver;
        let p1 = prover.prove(b"w", b"p").unwrap();
        let p2 = prover.prove(b"w", b"p").unwrap();
        assert_eq!(p1.bytes, p2.bytes);
    }

    #[test]
    fn different_witnesses_produce_different_proofs() {
        let prover = MockZkProver;
        let p1 = prover.prove(b"witness-a", b"pub").unwrap();
        let p2 = prover.prove(b"witness-b", b"pub").unwrap();
        assert_ne!(p1.bytes, p2.bytes);
    }
}
