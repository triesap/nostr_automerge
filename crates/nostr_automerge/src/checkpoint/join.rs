use crate::carrier::checkpoint_chunk::ValidatedCheckpointChunkCarrier;
use crate::carrier::checkpoint_descriptor::ValidatedCheckpointDescriptorCarrier;

use super::CheckpointChunk;
use crate::{CancellationCheck, WorkBudget};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum JoinError {
    Author,
    Coordinate,
    Descriptor,
    Count,
    DuplicateIndex,
    MissingIndex,
    Size,
    Budget,
    Cancelled,
}

pub(crate) fn join_chunks<'a>(
    descriptor: &ValidatedCheckpointDescriptorCarrier,
    chunks: impl IntoIterator<Item = &'a ValidatedCheckpointChunkCarrier>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<Vec<CheckpointChunk>, JoinError> {
    let commitments = descriptor.descriptor();
    let count = usize::try_from(commitments.chunk_count).map_err(|_| JoinError::Count)?;
    let mut by_index = Vec::new();
    for _ in 0..count {
        charge_join_item(budget, cancellation)?;
        by_index.push(None);
    }
    for carrier in chunks {
        charge_join_item(budget, cancellation)?;
        if carrier.author() != descriptor.author() {
            return Err(JoinError::Author);
        }
        if carrier.coordinate() != descriptor.coordinate() {
            return Err(JoinError::Coordinate);
        }
        if carrier.descriptor_id() != descriptor.event_id() {
            return Err(JoinError::Descriptor);
        }
        let chunk = carrier.chunk();
        if chunk.count != commitments.chunk_count {
            return Err(JoinError::Count);
        }
        let expected_size = if chunk.index + 1 == chunk.count {
            commitments
                .raw_size
                .checked_sub(
                    u64::from(commitments.chunk_size)
                        .checked_mul(u64::from(commitments.chunk_count - 1))
                        .ok_or(JoinError::Size)?,
                )
                .ok_or(JoinError::Size)?
        } else {
            u64::from(commitments.chunk_size)
        };
        if u64::try_from(chunk.data.len()).map_err(|_| JoinError::Size)? != expected_size {
            return Err(JoinError::Size);
        }
        let mut proof = Vec::new();
        for step in &chunk.proof {
            charge_join_item(budget, cancellation)?;
            proof.push(*step);
        }
        if cancellation.is_cancelled() {
            return Err(JoinError::Cancelled);
        }
        budget
            .charge_checkpoint_bytes(u64::try_from(chunk.data.len()).map_err(|_| JoinError::Size)?)
            .map_err(|_| JoinError::Budget)?;
        let slot = by_index
            .get_mut(usize::try_from(chunk.index).map_err(|_| JoinError::Count)?)
            .ok_or(JoinError::MissingIndex)?;
        if slot.is_some() {
            return Err(JoinError::DuplicateIndex);
        }
        *slot = Some(CheckpointChunk {
            index: chunk.index,
            count: chunk.count,
            data: chunk.data.clone(),
            proof,
        });
    }
    let mut ordered = Vec::new();
    for chunk in by_index {
        charge_join_item(budget, cancellation)?;
        let Some(chunk) = chunk else {
            return Err(JoinError::MissingIndex);
        };
        ordered.push(chunk);
    }
    Ok(ordered)
}

fn charge_join_item(
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<(), JoinError> {
    if cancellation.is_cancelled() {
        return Err(JoinError::Cancelled);
    }
    budget
        .charge_checkpoint_items(1)
        .map_err(|_| JoinError::Budget)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{JoinError, join_chunks};
    use crate::carrier::checkpoint_chunk::ValidatedCheckpointChunkCarrier;
    use crate::carrier::checkpoint_descriptor::ValidatedCheckpointDescriptorCarrier;
    use crate::checkpoint::{CheckpointChunk, CheckpointDescriptor};
    use crate::{
        ChangeHash, ChunkHash, ControllerPublicKey, DevicePublicKey, DocumentCoordinate,
        DocumentId, EventId, SnapshotHash,
    };

    fn join<'a>(
        descriptor: &ValidatedCheckpointDescriptorCarrier,
        chunks: impl IntoIterator<Item = &'a ValidatedCheckpointChunkCarrier>,
    ) -> Result<Vec<CheckpointChunk>, JoinError> {
        join_chunks(
            descriptor,
            chunks,
            &mut crate::WorkBudget::new(u64::MAX, u64::MAX),
            &crate::NeverCancelled,
        )
    }

    #[test]
    fn checkpoint_join_requires_one_coherent_signed_set() {
        let author = DevicePublicKey::from_bytes([1; 32]);
        let coordinate = DocumentCoordinate::new(
            ControllerPublicKey::from_bytes([2; 32]),
            DocumentId::from_bytes([3; 32]),
        );
        let descriptor_id = EventId::from_bytes([4; 32]);
        let descriptor = ValidatedCheckpointDescriptorCarrier::for_test(
            descriptor_id,
            author,
            coordinate,
            EventId::from_bytes([5; 32]),
            CheckpointDescriptor {
                snapshot_hash: SnapshotHash::from_bytes([6; 32]),
                heads: BTreeSet::from([ChangeHash::from_bytes([7; 32])]),
                raw_size: 3,
                chunk_size: 2,
                chunk_count: 2,
                chunk_root: [8; 32],
                change_count: 1,
                change_set_hash: [9; 32],
                dependency_edges: 0,
                total_ops: 1,
            },
        );
        let make = |event: u8, index: u32, data: Vec<u8>| {
            ValidatedCheckpointChunkCarrier::for_test(
                EventId::from_bytes([event; 32]),
                author,
                coordinate,
                descriptor_id,
                ChunkHash::from_bytes([event; 32]),
                CheckpointChunk {
                    index,
                    count: 2,
                    data,
                    proof: Vec::new(),
                },
            )
        };
        let first = make(10, 0, vec![1, 2]);
        let second = make(11, 1, vec![3]);
        assert_eq!(
            join(&descriptor, [&second, &first]).map(|chunks| chunks
                .into_iter()
                .map(|chunk| chunk.index)
                .collect::<Vec<_>>()),
            Ok(vec![0, 1])
        );
        let mut exact = crate::WorkBudget::new(3, 6);
        assert_eq!(
            join_chunks(
                &descriptor,
                [&second, &first],
                &mut exact,
                &crate::NeverCancelled,
            )
            .map(|chunks| chunks.len()),
            Ok(2)
        );
        assert_eq!(exact.consumed().get(crate::WorkCounter::CheckpointByte), 3);
        assert_eq!(exact.consumed().get(crate::WorkCounter::CheckpointItem), 6);
        let mut one_short = crate::WorkBudget::new(3, 5);
        assert_eq!(
            join_chunks(
                &descriptor,
                [&second, &first],
                &mut one_short,
                &crate::NeverCancelled,
            ),
            Err(JoinError::Budget)
        );
        assert_eq!(
            one_short.consumed().get(crate::WorkCounter::CheckpointItem),
            5
        );
        assert_eq!(join(&descriptor, [&first]), Err(JoinError::MissingIndex));
        assert_eq!(
            join(&descriptor, [&first, &first]),
            Err(JoinError::DuplicateIndex)
        );
        let wrong_size = make(12, 1, vec![3, 4]);
        assert_eq!(
            join(&descriptor, [&first, &wrong_size]),
            Err(JoinError::Size)
        );
        let wrong_author = ValidatedCheckpointChunkCarrier::for_test(
            EventId::from_bytes([13; 32]),
            DevicePublicKey::from_bytes([14; 32]),
            coordinate,
            descriptor_id,
            ChunkHash::from_bytes([13; 32]),
            CheckpointChunk {
                index: 1,
                count: 2,
                data: vec![3],
                proof: Vec::new(),
            },
        );
        assert_eq!(
            join(&descriptor, [&first, &wrong_author]),
            Err(JoinError::Author)
        );
        let wrong_coordinate = ValidatedCheckpointChunkCarrier::for_test(
            EventId::from_bytes([15; 32]),
            author,
            DocumentCoordinate::new(
                ControllerPublicKey::from_bytes([2; 32]),
                DocumentId::from_bytes([16; 32]),
            ),
            descriptor_id,
            ChunkHash::from_bytes([15; 32]),
            CheckpointChunk {
                index: 1,
                count: 2,
                data: vec![3],
                proof: Vec::new(),
            },
        );
        assert_eq!(
            join(&descriptor, [&first, &wrong_coordinate]),
            Err(JoinError::Coordinate)
        );
        let wrong_descriptor = ValidatedCheckpointChunkCarrier::for_test(
            EventId::from_bytes([17; 32]),
            author,
            coordinate,
            EventId::from_bytes([18; 32]),
            ChunkHash::from_bytes([17; 32]),
            CheckpointChunk {
                index: 1,
                count: 2,
                data: vec![3],
                proof: Vec::new(),
            },
        );
        assert_eq!(
            join(&descriptor, [&first, &wrong_descriptor]),
            Err(JoinError::Descriptor)
        );
        let wrong_count = ValidatedCheckpointChunkCarrier::for_test(
            EventId::from_bytes([19; 32]),
            author,
            coordinate,
            descriptor_id,
            ChunkHash::from_bytes([19; 32]),
            CheckpointChunk {
                index: 1,
                count: 3,
                data: vec![3],
                proof: Vec::new(),
            },
        );
        assert_eq!(
            join(&descriptor, [&first, &wrong_count]),
            Err(JoinError::Count)
        );
        let out_of_range = make(20, 2, vec![3, 4]);
        assert_eq!(
            join(&descriptor, [&first, &out_of_range]),
            Err(JoinError::MissingIndex)
        );
    }
}
