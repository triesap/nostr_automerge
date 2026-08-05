use super::{CheckpointChunk, CheckpointDescriptor, leaf_hash, verify_proof};
use crate::{CancellationCheck, WorkBudget};
use sha2::{Digest, Sha256};

/// Why a complete checkpoint chunk set could not be assembled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssemblyError {
    /// Chunk set, index, count, size, or proof differed.
    Chunks,
    /// Local deterministic budget was exhausted.
    Budget,
    /// Caller requested cancellation.
    Cancelled,
    /// Complete snapshot size or identity differed.
    Identity,
}

/// Orders, verifies, and joins a complete bounded chunk set.
pub fn assemble_chunks<C: CancellationCheck>(
    descriptor: &CheckpointDescriptor,
    chunks: &mut [CheckpointChunk],
    budget: &mut WorkBudget,
    cancellation: &C,
) -> Result<Vec<u8>, AssemblyError> {
    descriptor
        .validate_arithmetic()
        .map_err(|_| AssemblyError::Chunks)?;
    if chunks.len() != descriptor.chunk_count as usize {
        return Err(AssemblyError::Chunks);
    }
    chunks.sort_by_key(|chunk| chunk.index);
    let mut output = Vec::with_capacity(
        usize::try_from(descriptor.raw_size).map_err(|_| AssemblyError::Chunks)?,
    );
    for (expected, chunk) in chunks.iter().enumerate() {
        if cancellation.is_cancelled() {
            return Err(AssemblyError::Cancelled);
        }
        if chunk.index as usize != expected || chunk.count != descriptor.chunk_count {
            return Err(AssemblyError::Chunks);
        }
        let final_chunk = expected + 1 == chunks.len();
        if (!final_chunk && chunk.data.len() != descriptor.chunk_size as usize)
            || chunk.data.is_empty()
            || chunk.data.len() > descriptor.chunk_size as usize
        {
            return Err(AssemblyError::Chunks);
        }
        budget
            .charge_checkpoint_items(
                1_u64.saturating_add(u64::try_from(chunk.proof.len()).unwrap_or(u64::MAX)),
            )
            .map_err(|_| AssemblyError::Budget)?;
        budget
            .charge_checkpoint_bytes(chunk.data.len() as u64)
            .map_err(|_| AssemblyError::Budget)?;
        let raw_hash: [u8; 32] = Sha256::digest(&chunk.data).into();
        let leaf = leaf_hash(chunk.index, chunk.count, raw_hash);
        verify_proof(
            chunk.index,
            chunk.count,
            leaf,
            &chunk.proof,
            descriptor.chunk_root,
        )
        .map_err(|_| AssemblyError::Chunks)?;
        output.extend_from_slice(&chunk.data);
    }
    if output.len() as u64 != descriptor.raw_size
        || <[u8; 32]>::from(Sha256::digest(&output)) != *descriptor.snapshot_hash.as_bytes()
    {
        return Err(AssemblyError::Identity);
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ChangeHash, NeverCancelled, SnapshotHash};
    use std::collections::BTreeSet;
    fn descriptor(root: [u8; 32]) -> CheckpointDescriptor {
        CheckpointDescriptor {
            snapshot_hash: SnapshotHash::from_bytes(Sha256::digest(b"ab").into()),
            heads: BTreeSet::from([ChangeHash::from_bytes([8; 32])]),
            raw_size: 2,
            chunk_size: 1,
            chunk_count: 2,
            chunk_root: root,
            change_count: 1,
            change_set_hash: [7; 32],
            dependency_edges: 0,
            total_ops: 0,
        }
    }
    #[test]
    fn assemble_chunks_with_bounded_memory() {
        let l0 = leaf_hash(0, 2, Sha256::digest(b"a").into());
        let l1 = leaf_hash(1, 2, Sha256::digest(b"b").into());
        let root = super::super::merkle_root(&[l0, l1]).unwrap_or([0; 32]);
        let mut chunks = vec![
            CheckpointChunk {
                index: 1,
                count: 2,
                data: b"b".to_vec(),
                proof: vec![super::super::ProofStep::left(l0)],
            },
            CheckpointChunk {
                index: 0,
                count: 2,
                data: b"a".to_vec(),
                proof: vec![super::super::ProofStep::right(l1)],
            },
        ];
        let mut budget = WorkBudget::new(2, 4);
        assert_eq!(
            assemble_chunks(&descriptor(root), &mut chunks, &mut budget, &NeverCancelled),
            Ok(b"ab".to_vec())
        );
        assert_eq!(budget.consumed().get(crate::WorkCounter::CheckpointByte), 2);
        assert_eq!(budget.consumed().get(crate::WorkCounter::CheckpointItem), 4);
        assert_eq!(budget.consumed().get(crate::WorkCounter::DecodeByte), 0);
        assert_eq!(budget.consumed().get(crate::WorkCounter::GraphNode), 0);
        let mut budget = WorkBudget::new(1, 2);
        assert_eq!(
            assemble_chunks(&descriptor(root), &mut chunks, &mut budget, &NeverCancelled),
            Err(AssemblyError::Budget)
        );
    }
    #[test]
    fn verify_complete_snapshot_size_and_hash() {
        let leaf = leaf_hash(0, 1, Sha256::digest(b"x").into());
        let mut chunk = vec![CheckpointChunk {
            index: 0,
            count: 1,
            data: b"x".to_vec(),
            proof: vec![],
        }];
        let mut value = descriptor(leaf);
        value.raw_size = 1;
        value.chunk_count = 1;
        value.snapshot_hash = SnapshotHash::from_bytes([0; 32]);
        let mut budget = WorkBudget::new(1, 1);
        assert_eq!(
            assemble_chunks(&value, &mut chunk, &mut budget, &NeverCancelled),
            Err(AssemblyError::Identity)
        );
    }
}
