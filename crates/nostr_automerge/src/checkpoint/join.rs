use std::collections::BTreeMap;

use crate::carrier::checkpoint_chunk::ValidatedCheckpointChunkCarrier;
use crate::carrier::checkpoint_descriptor::ValidatedCheckpointDescriptorCarrier;

use super::CheckpointChunk;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum JoinError {
    Author,
    Coordinate,
    Descriptor,
    Count,
    DuplicateIndex,
    MissingIndex,
    Size,
}

pub(crate) fn join_chunks<'a>(
    descriptor: &ValidatedCheckpointDescriptorCarrier,
    chunks: impl IntoIterator<Item = &'a ValidatedCheckpointChunkCarrier>,
) -> Result<Vec<CheckpointChunk>, JoinError> {
    let commitments = descriptor.descriptor();
    let mut by_index = BTreeMap::new();
    for carrier in chunks {
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
        if by_index.insert(chunk.index, chunk.clone()).is_some() {
            return Err(JoinError::DuplicateIndex);
        }
    }
    if by_index.len() != usize::try_from(commitments.chunk_count).map_err(|_| JoinError::Count)?
        || by_index.keys().copied().ne(0..commitments.chunk_count)
    {
        return Err(JoinError::MissingIndex);
    }
    Ok(by_index.into_values().collect())
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
            join_chunks(&descriptor, [&second, &first]).map(|chunks| chunks
                .into_iter()
                .map(|chunk| chunk.index)
                .collect::<Vec<_>>()),
            Ok(vec![0, 1])
        );
        assert_eq!(
            join_chunks(&descriptor, [&first]),
            Err(JoinError::MissingIndex)
        );
        assert_eq!(
            join_chunks(&descriptor, [&first, &first]),
            Err(JoinError::DuplicateIndex)
        );
        let wrong_size = make(12, 1, vec![3, 4]);
        assert_eq!(
            join_chunks(&descriptor, [&first, &wrong_size]),
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
            join_chunks(&descriptor, [&first, &wrong_author]),
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
            join_chunks(&descriptor, [&first, &wrong_coordinate]),
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
            join_chunks(&descriptor, [&first, &wrong_descriptor]),
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
            join_chunks(&descriptor, [&first, &wrong_count]),
            Err(JoinError::Count)
        );
        let out_of_range = make(20, 2, vec![3, 4]);
        assert_eq!(
            join_chunks(&descriptor, [&first, &out_of_range]),
            Err(JoinError::MissingIndex)
        );
    }
}
