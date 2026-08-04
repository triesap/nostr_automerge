use sha2::{Digest, Sha256};

/// One sibling in an ordered Merkle proof.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProofStep {
    pub(crate) hash: [u8; 32],
    pub(crate) side: Side,
}
impl ProofStep {
    /// Constructs a left sibling step.
    #[must_use]
    pub const fn left(hash: [u8; 32]) -> Self {
        Self {
            hash,
            side: Side::Left,
        }
    }
    /// Constructs a right sibling step.
    #[must_use]
    pub const fn right(hash: [u8; 32]) -> Self {
        Self {
            hash,
            side: Side::Right,
        }
    }
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
    hash.update([0]);
    hash.update(super::MERKLE_DOMAIN);
    hash.update([0]);
    hash.update(index.to_be_bytes());
    hash.update(count.to_be_bytes());
    hash.update(chunk_hash);
    hash.finalize().into()
}

fn node(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update([1]);
    hash.update(super::MERKLE_DOMAIN);
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
    index: u32,
    count: u32,
    leaf: [u8; 32],
    proof: &[ProofStep],
    root: [u8; 32],
) -> Result<(), MerkleError> {
    fn sides(index: usize, count: usize, output: &mut Vec<Side>) {
        if count == 1 {
            return;
        }
        let split = count.next_power_of_two() / 2;
        if index < split {
            sides(index, split, output);
            output.push(Side::Right);
        } else {
            sides(index - split, count - split, output);
            output.push(Side::Left);
        }
    }
    if count == 0 || count > super::MAX_CHUNK_COUNT || index >= count {
        return Err(MerkleError::Proof);
    }
    let mut expected = Vec::new();
    sides(index as usize, count as usize, &mut expected);
    if expected.len() != proof.len()
        || expected
            .iter()
            .zip(proof)
            .any(|(side, step)| *side != step.side)
    {
        return Err(MerkleError::Proof);
    }
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

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};
    #[test]
    fn implement_ordered_merkle_leaf_hashing() {
        let chunk_hash: [u8; 32] = Sha256::digest(b"chunk").into();
        let actual = super::leaf_hash(2, 5, chunk_hash);
        let mut manual = Sha256::new();
        manual.update([0]);
        manual.update(super::super::MERKLE_DOMAIN);
        manual.update([0]);
        manual.update(2_u32.to_be_bytes());
        manual.update(5_u32.to_be_bytes());
        manual.update(chunk_hash);
        assert_eq!(actual, <[u8; 32]>::from(manual.finalize()));
    }
    #[test]
    fn implement_ordered_unpadded_merkle_root() {
        for count in [1_usize, 2, 3, 5, 8] {
            let leaves = (0..count)
                .map(|i| super::leaf_hash(i as u32, count as u32, [i as u8; 32]))
                .collect::<Vec<_>>();
            let root = super::merkle_root(&leaves);
            assert!(root.is_ok());
            assert_eq!(root, super::merkle_root(&leaves));
        }
        assert_eq!(super::merkle_root(&[]), Err(super::MerkleError::Count));
    }
    #[test]
    fn verify_ordered_merkle_proofs() {
        let leaves = [[1; 32], [2; 32], [3; 32]];
        let root = super::merkle_root(&leaves).unwrap_or([0; 32]);
        let left_pair = super::node(leaves[0], leaves[1]);
        assert_eq!(
            super::verify_proof(
                0,
                3,
                leaves[0],
                &[
                    super::ProofStep::right(leaves[1]),
                    super::ProofStep::right(leaves[2])
                ],
                root
            ),
            Ok(())
        );
        assert_eq!(
            super::verify_proof(2, 3, leaves[2], &[super::ProofStep::left(left_pair)], root),
            Ok(())
        );
        assert_eq!(
            super::verify_proof(0, 3, leaves[0], &[super::ProofStep::left(leaves[1])], root),
            Err(super::MerkleError::Proof)
        );
    }
}
