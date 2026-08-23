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
    match verify_proof_with(index, count, leaf, proof, root, &mut || {
        Ok::<(), core::convert::Infallible>(())
    }) {
        Ok(result) => result,
        Err(error) => match error {},
    }
}

pub(crate) fn verify_proof_metered(
    index: u32,
    count: u32,
    leaf: [u8; 32],
    proof: &[ProofStep],
    root: [u8; 32],
    budget: &mut crate::WorkBudget,
    cancellation: &impl crate::CancellationCheck,
) -> Result<Result<(), MerkleError>, crate::Completion> {
    verify_proof_with(index, count, leaf, proof, root, &mut || {
        if cancellation.is_cancelled() {
            return Err(crate::Completion::Cancelled);
        }
        budget
            .charge_checkpoint_items(1)
            .map_err(|_| crate::Completion::BudgetExhausted)
    })
}

fn verify_proof_with<E>(
    index: u32,
    count: u32,
    leaf: [u8; 32],
    proof: &[ProofStep],
    root: [u8; 32],
    visit: &mut impl FnMut() -> Result<(), E>,
) -> Result<Result<(), MerkleError>, E> {
    fn sides<E>(
        index: usize,
        count: usize,
        output: &mut Vec<Side>,
        visit: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<(), E> {
        if count == 1 {
            return Ok(());
        }
        visit()?;
        let split = count.next_power_of_two() / 2;
        if index < split {
            sides(index, split, output, visit)?;
            visit()?;
            output.push(Side::Right);
        } else {
            sides(index - split, count - split, output, visit)?;
            visit()?;
            output.push(Side::Left);
        }
        Ok(())
    }
    if count == 0 {
        return Ok(Err(MerkleError::Proof));
    }
    if count > super::MAX_CHUNK_COUNT {
        return Ok(Err(MerkleError::Proof));
    }
    if index >= count {
        return Ok(Err(MerkleError::Proof));
    }
    let mut expected = Vec::new();
    sides(index as usize, count as usize, &mut expected, visit)?;
    if expected.len() != proof.len() {
        return Ok(Err(MerkleError::Proof));
    }
    for (side, step) in expected.iter().zip(proof) {
        visit()?;
        if *side != step.side {
            return Ok(Err(MerkleError::Proof));
        }
    }
    let mut actual = leaf;
    for step in proof {
        visit()?;
        actual = match step.side {
            Side::Left => node(step.hash, actual),
            Side::Right => node(actual, step.hash),
        };
    }
    if actual == root {
        Ok(Ok(()))
    } else {
        Ok(Err(MerkleError::Proof))
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

        let maximum = vec![[7_u8; 32]; super::super::MAX_CHUNK_COUNT as usize];
        assert!(super::merkle_root(&maximum).is_ok());
        let over_limit = vec![[7_u8; 32]; super::super::MAX_CHUNK_COUNT as usize + 1];
        assert_eq!(
            super::merkle_root(&over_limit),
            Err(super::MerkleError::Count)
        );

        let leaves = [[1_u8; 32], [2_u8; 32]];
        let mut expected = Sha256::new();
        expected.update([1]);
        expected.update(super::super::MERKLE_DOMAIN);
        expected.update([0]);
        expected.update(leaves[0]);
        expected.update(leaves[1]);
        assert_eq!(
            super::merkle_root(&leaves),
            Ok(<[u8; 32]>::from(expected.finalize()))
        );
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

        assert_eq!(
            super::verify_proof(0, 3, leaves[0], &[super::ProofStep::right(leaves[1])], root),
            Err(super::MerkleError::Proof)
        );
        assert_eq!(
            super::verify_proof(
                0,
                3,
                leaves[0],
                &[
                    super::ProofStep::left(leaves[1]),
                    super::ProofStep::right(leaves[2]),
                ],
                root,
            ),
            Err(super::MerkleError::Proof)
        );

        for (index, count) in [(0, 0), (3, 3), (0, super::super::MAX_CHUNK_COUNT + 1)] {
            assert_eq!(
                super::verify_proof(index, count, [0; 32], &[], [0; 32]),
                Err(super::MerkleError::Proof)
            );
        }

        let six = [[1_u8; 32], [2; 32], [3; 32], [4; 32], [5; 32], [6; 32]];
        let first_pair = super::node(six[0], six[1]);
        let second_pair = super::node(six[2], six[3]);
        let first_four = super::node(first_pair, second_pair);
        let last_pair = super::node(six[4], six[5]);
        let six_root = super::node(first_four, last_pair);
        assert_eq!(
            super::verify_proof(
                4,
                6,
                six[4],
                &[
                    super::ProofStep::right(six[5]),
                    super::ProofStep::left(first_four),
                ],
                six_root,
            ),
            Ok(())
        );
        assert_eq!(
            super::verify_proof(
                5,
                6,
                six[5],
                &[
                    super::ProofStep::left(six[4]),
                    super::ProofStep::left(first_four),
                ],
                six_root,
            ),
            Ok(())
        );

        let maximum_leaf = [9_u8; 32];
        let maximum_proof = (0_u8..12)
            .map(|value| super::ProofStep::right([value; 32]))
            .collect::<Vec<_>>();
        let maximum_root = maximum_proof
            .iter()
            .fold(maximum_leaf, |value, step| super::node(value, step.hash));
        assert_eq!(
            super::verify_proof(
                0,
                super::super::MAX_CHUNK_COUNT,
                maximum_leaf,
                &maximum_proof,
                maximum_root,
            ),
            Ok(())
        );
    }

    #[test]
    fn metered_proof_visits_stop_at_the_exact_boundary() {
        let leaves = [[1; 32], [2; 32], [3; 32]];
        let root = super::merkle_root(&leaves).unwrap_or([0; 32]);
        let proof = [
            super::ProofStep::right(leaves[1]),
            super::ProofStep::right(leaves[2]),
        ];
        let mut exact = crate::WorkBudget::new(0, 8);
        assert_eq!(
            super::verify_proof_metered(
                0,
                3,
                leaves[0],
                &proof,
                root,
                &mut exact,
                &crate::NeverCancelled,
            ),
            Ok(Ok(()))
        );
        assert_eq!(exact.consumed().get(crate::WorkCounter::CheckpointItem), 8);

        let mut one_short = crate::WorkBudget::new(0, 7);
        assert_eq!(
            super::verify_proof_metered(
                0,
                3,
                leaves[0],
                &proof,
                root,
                &mut one_short,
                &crate::NeverCancelled,
            ),
            Err(crate::Completion::BudgetExhausted)
        );
        assert_eq!(
            one_short.consumed().get(crate::WorkCounter::CheckpointItem),
            7
        );

        let mut cancelled = crate::WorkBudget::new(0, 8);
        assert_eq!(
            super::verify_proof_metered(0, 3, leaves[0], &proof, root, &mut cancelled, &|| true,),
            Err(crate::Completion::Cancelled)
        );
        assert_eq!(
            cancelled.consumed().get(crate::WorkCounter::CheckpointItem),
            0
        );
    }
}
