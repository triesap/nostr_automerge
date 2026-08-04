use sha2::{Digest, Sha256};

/// One sibling in an ordered Merkle proof.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProofStep {
    pub(crate) hash: [u8; 32],
    pub(crate) side: Side,
}
/// Side occupied by the proof sibling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// Sibling precedes the current node.
    Left,
    /// Sibling follows the current node.
    Right,
}
/// Why an ordered Merkle input or proof was rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MerkleError {
    /// Input collection was empty or over limit.
    Count,
    /// Proof did not reconstruct the expected root.
    Proof,
}

/// Hashes one position-bound chunk identity.
#[must_use]
pub fn leaf_hash(index: u32, count: u32, chunk_hash: [u8; 32]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(super::LEAF_DOMAIN);
    hash.update([0]);
    hash.update(index.to_be_bytes());
    hash.update(count.to_be_bytes());
    hash.update(chunk_hash);
    hash.finalize().into()
}

fn node(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(super::NODE_DOMAIN);
    hash.update([0]);
    hash.update(left);
    hash.update(right);
    hash.finalize().into()
}

/// Computes the recursive ordered, unpadded Merkle root.
pub fn merkle_root(leaves: &[[u8; 32]]) -> Result<[u8; 32], MerkleError> {
    if leaves.is_empty() || leaves.len() > super::MAX_CHUNK_COUNT as usize {
        return Err(MerkleError::Count);
    }
    if leaves.len() == 1 {
        return Ok(leaves[0]);
    }
    let split = leaves.len().next_power_of_two() / 2;
    Ok(node(
        merkle_root(&leaves[..split])?,
        merkle_root(&leaves[split..])?,
    ))
}

/// Verifies a caller-provided ordered proof.
pub fn verify_proof(
    leaf: [u8; 32],
    proof: &[ProofStep],
    root: [u8; 32],
) -> Result<(), MerkleError> {
    let actual = proof.iter().fold(leaf, |value, step| match step.side {
        Side::Left => node(step.hash, value),
        Side::Right => node(value, step.hash),
    });
    if actual == root {
        Ok(())
    } else {
        Err(MerkleError::Proof)
    }
}
