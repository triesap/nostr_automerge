use core::fmt;

use crate::checkpoint::{CheckpointChunk, ChunkError};
use crate::wire::tags;
use crate::{ChunkHash, DevicePublicKey, DocumentCoordinate, EventId, VerifiedNip01Event};

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct ValidatedCheckpointChunkCarrier {
    event_id: EventId,
    author: DevicePublicKey,
    coordinate: DocumentCoordinate,
    descriptor_id: EventId,
    chunk_hash: ChunkHash,
    chunk: CheckpointChunk,
}

impl ValidatedCheckpointChunkCarrier {
    pub(crate) const fn event_id(&self) -> EventId {
        self.event_id
    }

    pub(crate) const fn author(&self) -> DevicePublicKey {
        self.author
    }

    pub(crate) const fn coordinate(&self) -> DocumentCoordinate {
        self.coordinate
    }

    pub(crate) const fn descriptor_id(&self) -> EventId {
        self.descriptor_id
    }

    pub(crate) const fn chunk_hash(&self) -> ChunkHash {
        self.chunk_hash
    }

    pub(crate) const fn chunk(&self) -> &CheckpointChunk {
        &self.chunk
    }
}

impl fmt::Debug for ValidatedCheckpointChunkCarrier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ValidatedCheckpointChunkCarrier")
            .field("event_id", &self.event_id)
            .field("author", &self.author)
            .field("coordinate", &self.coordinate)
            .field("descriptor_id", &self.descriptor_id)
            .field("chunk_hash", &self.chunk_hash)
            .field("chunk", &"[REDACTED]")
            .finish()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CheckpointChunkCarrierError {
    Kind,
    Tags,
    Coordinate,
    Descriptor,
    Hash,
    Part,
    Chunk(ChunkError),
}

pub(crate) fn validate(
    event: &VerifiedNip01Event,
) -> Result<ValidatedCheckpointChunkCarrier, CheckpointChunkCarrierError> {
    if event.kind() != crate::checkpoint::CHUNK_KIND {
        return Err(CheckpointChunkCarrierError::Kind);
    }
    let coordinate: DocumentCoordinate = tags::required_tag(event.tags(), "a", 2)
        .map_err(|_| CheckpointChunkCarrierError::Tags)?[1]
        .parse()
        .map_err(|_| CheckpointChunkCarrierError::Coordinate)?;
    let descriptor_id = tags::required_tag(event.tags(), "e", 2)
        .map_err(|_| CheckpointChunkCarrierError::Tags)?[1]
        .parse()
        .map_err(|_| CheckpointChunkCarrierError::Descriptor)?;
    let chunk_hash: ChunkHash = tags::required_tag(event.tags(), "x", 2)
        .map_err(|_| CheckpointChunkCarrierError::Tags)?[1]
        .parse()
        .map_err(|_| CheckpointChunkCarrierError::Hash)?;
    let part = tags::required_tag(event.tags(), "part", 3)
        .map_err(|_| CheckpointChunkCarrierError::Tags)?;
    let index = part[1]
        .parse()
        .map_err(|_| CheckpointChunkCarrierError::Part)?;
    let count = part[2]
        .parse()
        .map_err(|_| CheckpointChunkCarrierError::Part)?;
    let chunk =
        CheckpointChunk::parse_content(event.content(), index, count, *chunk_hash.as_bytes())
            .map_err(CheckpointChunkCarrierError::Chunk)?;
    Ok(ValidatedCheckpointChunkCarrier {
        event_id: event.event_id(),
        author: DevicePublicKey::from_bytes(*event.author_bytes()),
        coordinate,
        descriptor_id,
        chunk_hash,
        chunk,
    })
}
