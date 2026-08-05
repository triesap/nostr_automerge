use core::fmt;

use crate::checkpoint::{CheckpointDescriptor, DescriptorError};
use crate::wire::tags;
use crate::{DevicePublicKey, DocumentCoordinate, EventId, SnapshotHash, VerifiedNip01Event};

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct ValidatedCheckpointDescriptorCarrier {
    event_id: EventId,
    author: DevicePublicKey,
    coordinate: DocumentCoordinate,
    control_id: EventId,
    snapshot_hash: SnapshotHash,
    descriptor: CheckpointDescriptor,
}

impl ValidatedCheckpointDescriptorCarrier {
    pub(crate) const fn event_id(&self) -> EventId {
        self.event_id
    }

    pub(crate) const fn author(&self) -> DevicePublicKey {
        self.author
    }

    pub(crate) const fn coordinate(&self) -> DocumentCoordinate {
        self.coordinate
    }

    pub(crate) const fn control_id(&self) -> EventId {
        self.control_id
    }

    pub(crate) const fn snapshot_hash(&self) -> SnapshotHash {
        self.snapshot_hash
    }

    pub(crate) const fn descriptor(&self) -> &CheckpointDescriptor {
        &self.descriptor
    }

    #[cfg(test)]
    pub(crate) fn for_test(
        event_id: EventId,
        author: DevicePublicKey,
        coordinate: DocumentCoordinate,
        control_id: EventId,
        descriptor: CheckpointDescriptor,
    ) -> Self {
        Self {
            event_id,
            author,
            coordinate,
            control_id,
            snapshot_hash: descriptor.snapshot_hash,
            descriptor,
        }
    }
}

impl fmt::Debug for ValidatedCheckpointDescriptorCarrier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ValidatedCheckpointDescriptorCarrier")
            .field("event_id", &self.event_id)
            .field("author", &self.author)
            .field("coordinate", &self.coordinate)
            .field("control_id", &self.control_id)
            .field("snapshot_hash", &self.snapshot_hash)
            .field("descriptor", &"[REDACTED]")
            .finish()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CheckpointDescriptorCarrierError {
    Kind,
    Tags(tags::TagError),
    Coordinate,
    Control,
    Snapshot,
    Descriptor(DescriptorError),
}

pub(crate) fn validate(
    event: &VerifiedNip01Event,
) -> Result<ValidatedCheckpointDescriptorCarrier, CheckpointDescriptorCarrierError> {
    if event.kind() != crate::checkpoint::DESCRIPTOR_KIND {
        return Err(CheckpointDescriptorCarrierError::Kind);
    }
    tags::require_absent(event.tags(), "d").map_err(CheckpointDescriptorCarrierError::Tags)?;
    tags::require_durable_tags(event.tags()).map_err(CheckpointDescriptorCarrierError::Tags)?;
    if event.tags().len() != 3
        || event.tags().iter().any(|tag| {
            tag.first()
                .is_none_or(|name| name != "a" && name != "e" && name != "x")
        })
    {
        return Err(CheckpointDescriptorCarrierError::Tags(
            tags::TagError::Forbidden,
        ));
    }
    let coordinate: DocumentCoordinate = tags::required_tag(event.tags(), "a", 2)
        .map_err(CheckpointDescriptorCarrierError::Tags)?[1]
        .parse()
        .map_err(|_| CheckpointDescriptorCarrierError::Coordinate)?;
    let control_id = tags::required_tag(event.tags(), "e", 2)
        .map_err(CheckpointDescriptorCarrierError::Tags)?[1]
        .parse()
        .map_err(|_| CheckpointDescriptorCarrierError::Control)?;
    let snapshot_hash = tags::required_tag(event.tags(), "x", 2)
        .map_err(CheckpointDescriptorCarrierError::Tags)?[1]
        .parse()
        .map_err(|_| CheckpointDescriptorCarrierError::Snapshot)?;
    let descriptor = CheckpointDescriptor::parse_content(event.content(), snapshot_hash)
        .map_err(CheckpointDescriptorCarrierError::Descriptor)?;
    descriptor
        .validate_arithmetic()
        .map_err(CheckpointDescriptorCarrierError::Descriptor)?;
    Ok(ValidatedCheckpointDescriptorCarrier {
        event_id: event.event_id(),
        author: DevicePublicKey::from_bytes(*event.author_bytes()),
        coordinate,
        control_id,
        snapshot_hash,
        descriptor,
    })
}
