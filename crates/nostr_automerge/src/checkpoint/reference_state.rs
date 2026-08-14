use std::collections::BTreeMap;

use crate::ProtocolDisposition;
use crate::carrier::VerifiedCarrier;
use crate::carrier::checkpoint_descriptor::ValidatedCheckpointDescriptorCarrier;
use crate::evidence::corpus_builder::EvidenceCorpus;
use crate::evidence::event::EventEvidence;
use crate::{DocumentCoordinate, EventId};

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

pub(crate) fn resolve_referenced_descriptor<'a>(
    corpus: &'a EvidenceCorpus,
    event_id: EventId,
    coordinate: DocumentCoordinate,
    dispositions: &BTreeMap<EventId, ProtocolDisposition>,
) -> ReferencedDescriptorState<'a> {
    let Some(evidence) = corpus.events.get(&event_id) else {
        return ReferencedDescriptorState::Missing;
    };
    let descriptor = match evidence {
        EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::CheckpointDescriptor(descriptor),
            ..
        } => descriptor.as_ref(),
        EventEvidence::UnsupportedRevision { .. }
        | EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::UnsupportedRevision { .. },
            ..
        } => return ReferencedDescriptorState::UnsupportedRevision,
        EventEvidence::InvalidCarrier { event, .. } if event.kind() == 1_626 => {
            return ReferencedDescriptorState::StaticInvalid;
        }
        EventEvidence::VerifiedCarrier { .. }
        | EventEvidence::InvalidCarrier { .. }
        | EventEvidence::InvalidEvent { .. }
        | EventEvidence::IrrelevantEvent { .. }
        | EventEvidence::DuplicateEvent { .. } => return ReferencedDescriptorState::WrongKind,
    };
    if descriptor.coordinate() != coordinate {
        return ReferencedDescriptorState::WrongCoordinate;
    }
    match dispositions.get(&event_id) {
        Some(ProtocolDisposition::Accepted) => {
            ReferencedDescriptorState::VerifiedTarget(descriptor)
        }
        Some(ProtocolDisposition::Pending) | None => ReferencedDescriptorState::Pending(descriptor),
        Some(
            ProtocolDisposition::Excluded
            | ProtocolDisposition::Invalid
            | ProtocolDisposition::UnsupportedRevision,
        ) => ReferencedDescriptorState::DynamicInvalid,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{ReferencedDescriptorState, resolve_referenced_descriptor};
    use crate::carrier::checkpoint_descriptor::ValidatedCheckpointDescriptorCarrier;
    use crate::checkpoint::CheckpointDescriptor;
    use crate::{
        ChangeHash, ControllerPublicKey, CorpusBuilder, DevicePublicKey, DocumentCoordinate,
        DocumentId, EventId, ProtocolDisposition, SnapshotHash,
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

    #[test]
    fn absent_descriptor_evidence_resolves_as_missing() {
        let coordinate = DocumentCoordinate::new(
            ControllerPublicKey::from_bytes([10; 32]),
            DocumentId::from_bytes([11; 32]),
        );
        let corpus = CorpusBuilder::new().finish();
        assert_eq!(
            resolve_referenced_descriptor(
                &corpus,
                EventId::from_bytes([12; 32]),
                coordinate,
                &BTreeMap::new(),
            ),
            ReferencedDescriptorState::Missing
        );
    }

    #[test]
    fn pending_descriptor_keeps_dependent_chunk_pending() {
        let descriptor = descriptor();
        assert_eq!(
            ReferencedDescriptorState::Pending(&descriptor).dependent_disposition(),
            Some(ProtocolDisposition::Pending)
        );
    }

    #[test]
    fn wrong_kind_descriptor_invalidates_dependent_chunk() {
        assert_eq!(
            ReferencedDescriptorState::WrongKind.dependent_disposition(),
            Some(ProtocolDisposition::Invalid)
        );
    }

    #[test]
    fn wrong_coordinate_descriptor_invalidates_dependent_chunk() {
        assert_eq!(
            ReferencedDescriptorState::WrongCoordinate.dependent_disposition(),
            Some(ProtocolDisposition::Invalid)
        );
    }
}
