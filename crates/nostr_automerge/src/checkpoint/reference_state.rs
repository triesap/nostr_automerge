use crate::ProtocolDisposition;
use crate::carrier::checkpoint_descriptor::ValidatedCheckpointDescriptorCarrier;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReferencedDescriptorState<'a> {
    VerifiedTarget(&'a ValidatedCheckpointDescriptorCarrier),
    Pending(&'a ValidatedCheckpointDescriptorCarrier),
    Missing,
    WrongKind,
    WrongCoordinate,
    StaticInvalid,
    DynamicInvalid,
    UnsupportedRevision,
}

impl ReferencedDescriptorState<'_> {
    pub(crate) const fn dependent_disposition(self) -> Option<ProtocolDisposition> {
        match self {
            Self::VerifiedTarget(_) => None,
            Self::Pending(_) | Self::Missing => Some(ProtocolDisposition::Pending),
            Self::WrongKind
            | Self::WrongCoordinate
            | Self::StaticInvalid
            | Self::DynamicInvalid
            | Self::UnsupportedRevision => Some(ProtocolDisposition::Invalid),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::ReferencedDescriptorState;
    use crate::carrier::checkpoint_descriptor::ValidatedCheckpointDescriptorCarrier;
    use crate::checkpoint::CheckpointDescriptor;
    use crate::{
        ChangeHash, ControllerPublicKey, DevicePublicKey, DocumentCoordinate, DocumentId, EventId,
        ProtocolDisposition, SnapshotHash,
    };

    fn descriptor() -> ValidatedCheckpointDescriptorCarrier {
        ValidatedCheckpointDescriptorCarrier::for_test(
            EventId::from_bytes([1; 32]),
            DevicePublicKey::from_bytes([2; 32]),
            DocumentCoordinate::new(
                ControllerPublicKey::from_bytes([3; 32]),
                DocumentId::from_bytes([4; 32]),
            ),
            EventId::from_bytes([5; 32]),
            CheckpointDescriptor {
                snapshot_hash: SnapshotHash::from_bytes([6; 32]),
                heads: BTreeSet::from([ChangeHash::from_bytes([7; 32])]),
                raw_size: 1,
                chunk_size: 1,
                chunk_count: 1,
                chunk_root: [8; 32],
                change_count: 1,
                change_set_hash: [9; 32],
                dependency_edges: 0,
                total_ops: 1,
            },
        )
    }

    #[test]
    fn every_descriptor_reference_state_has_one_dependent_outcome() {
        let descriptor = descriptor();
        let cases = [
            (ReferencedDescriptorState::VerifiedTarget(&descriptor), None),
            (
                ReferencedDescriptorState::Pending(&descriptor),
                Some(ProtocolDisposition::Pending),
            ),
            (
                ReferencedDescriptorState::Missing,
                Some(ProtocolDisposition::Pending),
            ),
            (
                ReferencedDescriptorState::WrongKind,
                Some(ProtocolDisposition::Invalid),
            ),
            (
                ReferencedDescriptorState::WrongCoordinate,
                Some(ProtocolDisposition::Invalid),
            ),
            (
                ReferencedDescriptorState::StaticInvalid,
                Some(ProtocolDisposition::Invalid),
            ),
            (
                ReferencedDescriptorState::DynamicInvalid,
                Some(ProtocolDisposition::Invalid),
            ),
            (
                ReferencedDescriptorState::UnsupportedRevision,
                Some(ProtocolDisposition::Invalid),
            ),
        ];
        for (state, expected) in cases {
            assert_eq!(state.dependent_disposition(), expected);
        }
    }
}
