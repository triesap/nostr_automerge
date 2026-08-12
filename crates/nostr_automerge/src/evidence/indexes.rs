use std::collections::{BTreeMap, BTreeSet};

use crate::carrier::VerifiedCarrier;
use crate::evidence::event::EventEvidence;
use crate::{ActorId, ChangeHash, DevicePublicKey, DocumentCoordinate, EventId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ControlIndexRecord {
    pub(crate) event_id: EventId,
    pub(crate) parent: Option<EventId>,
    pub(crate) base_heads: BTreeSet<ChangeHash>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ControlIndexes {
    pub(crate) controls_by_id: BTreeMap<EventId, ControlIndexRecord>,
    pub(crate) genesis: BTreeSet<EventId>,
    pub(crate) children_by_parent: BTreeMap<EventId, BTreeSet<EventId>>,
    pub(crate) pending: BTreeSet<EventId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ChangeIndexRecord {
    pub(crate) event_id: EventId,
    pub(crate) change_hash: ChangeHash,
    pub(crate) control_id: EventId,
    pub(crate) actor: ActorId,
    pub(crate) dependencies: BTreeSet<ChangeHash>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SemanticChangeRecord {
    pub(crate) actor: ActorId,
    pub(crate) sequence: u64,
    pub(crate) start_op: u64,
    pub(crate) operation_count: u64,
    pub(crate) dependencies: BTreeSet<ChangeHash>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ChangeCarrierClaim {
    pub(crate) event_id: EventId,
    pub(crate) coordinate: DocumentCoordinate,
    pub(crate) change_hash: ChangeHash,
    pub(crate) control_id: EventId,
    pub(crate) author: DevicePublicKey,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ChangeIndexes {
    pub(crate) semantic_by_hash: BTreeMap<ChangeHash, SemanticChangeRecord>,
    pub(crate) claims_by_hash: BTreeMap<ChangeHash, BTreeMap<EventId, ChangeCarrierClaim>>,
    pub(crate) claims_by_event: BTreeMap<EventId, ChangeCarrierClaim>,
    pub(crate) claims_by_control: BTreeMap<EventId, BTreeMap<ChangeHash, BTreeSet<EventId>>>,
    pub(crate) carriers_by_hash: BTreeMap<ChangeHash, BTreeSet<EventId>>,
    pub(crate) preferred_carrier: BTreeMap<ChangeHash, EventId>,
    pub(crate) hashes_by_control: BTreeMap<EventId, BTreeSet<ChangeHash>>,
    pub(crate) hashes_by_actor: BTreeMap<ActorId, BTreeSet<ChangeHash>>,
    pub(crate) dependencies_by_hash: BTreeMap<ChangeHash, BTreeSet<ChangeHash>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CheckpointDescriptorIndexRecord {
    pub(crate) event_id: EventId,
    pub(crate) coordinate: DocumentCoordinate,
    pub(crate) control_id: EventId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CheckpointChunkIndexRecord {
    pub(crate) event_id: EventId,
    pub(crate) coordinate: DocumentCoordinate,
    pub(crate) descriptor_id: EventId,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct CheckpointIndexes {
    pub(crate) descriptors_by_id: BTreeMap<EventId, CheckpointDescriptorIndexRecord>,
    pub(crate) descriptors_by_coordinate: BTreeMap<DocumentCoordinate, BTreeSet<EventId>>,
    pub(crate) chunks_by_id: BTreeMap<EventId, CheckpointChunkIndexRecord>,
    pub(crate) chunks_by_descriptor: BTreeMap<EventId, BTreeSet<EventId>>,
    pub(crate) pending_descriptors: BTreeSet<EventId>,
    pub(crate) pending_chunks: BTreeSet<EventId>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct TrustedIndexes {
    pub(crate) controls: ControlIndexes,
    pub(crate) changes: ChangeIndexes,
    pub(crate) checkpoints: CheckpointIndexes,
}

pub(crate) fn derive_trusted_indexes(events: &BTreeMap<EventId, EventEvidence>) -> TrustedIndexes {
    let mut indexes = TrustedIndexes::default();
    for evidence in events.values() {
        let EventEvidence::VerifiedCarrier { carrier, .. } = evidence else {
            continue;
        };
        match carrier {
            VerifiedCarrier::Control(control) => index_control(&mut indexes.controls, control),
            VerifiedCarrier::Change(change) => index_change(&mut indexes.changes, change),
            VerifiedCarrier::CheckpointDescriptor(descriptor) => {
                index_checkpoint_descriptor(&mut indexes.checkpoints, descriptor);
            }
            VerifiedCarrier::CheckpointChunk(chunk) => {
                index_checkpoint_chunk(&mut indexes.checkpoints, chunk);
            }
            VerifiedCarrier::Manifest(_) | VerifiedCarrier::UnsupportedRevision { .. } => {}
        }
    }
    derive_pending_controls(&mut indexes);
    derive_pending_checkpoints(&mut indexes);
    indexes
}

fn index_checkpoint_descriptor(
    indexes: &mut CheckpointIndexes,
    descriptor: &crate::carrier::checkpoint_descriptor::ValidatedCheckpointDescriptorCarrier,
) {
    let record = CheckpointDescriptorIndexRecord {
        event_id: descriptor.event_id(),
        coordinate: descriptor.coordinate(),
        control_id: descriptor.control_id(),
    };
    indexes
        .descriptors_by_coordinate
        .entry(record.coordinate)
        .or_default()
        .insert(record.event_id);
    indexes.descriptors_by_id.insert(record.event_id, record);
}

fn index_checkpoint_chunk(
    indexes: &mut CheckpointIndexes,
    chunk: &crate::carrier::checkpoint_chunk::ValidatedCheckpointChunkCarrier,
) {
    let record = CheckpointChunkIndexRecord {
        event_id: chunk.event_id(),
        coordinate: chunk.coordinate(),
        descriptor_id: chunk.descriptor_id(),
    };
    indexes
        .chunks_by_descriptor
        .entry(record.descriptor_id)
        .or_default()
        .insert(record.event_id);
    indexes.chunks_by_id.insert(record.event_id, record);
}

fn derive_pending_checkpoints(indexes: &mut TrustedIndexes) {
    for descriptor in indexes.checkpoints.descriptors_by_id.values() {
        if !indexes
            .controls
            .controls_by_id
            .contains_key(&descriptor.control_id)
        {
            indexes
                .checkpoints
                .pending_descriptors
                .insert(descriptor.event_id);
        }
    }
    for chunk in indexes.checkpoints.chunks_by_id.values() {
        if !indexes
            .checkpoints
            .descriptors_by_id
            .contains_key(&chunk.descriptor_id)
        {
            indexes.checkpoints.pending_chunks.insert(chunk.event_id);
        }
    }
}

fn index_control(
    indexes: &mut ControlIndexes,
    control: &crate::carrier::control::ValidatedControlCarrier,
) {
    let record = ControlIndexRecord {
        event_id: control.event_id(),
        parent: control.parent(),
        base_heads: control.base_heads().collect(),
    };
    if let Some(parent) = record.parent {
        indexes
            .children_by_parent
            .entry(parent)
            .or_default()
            .insert(record.event_id);
    } else {
        indexes.genesis.insert(record.event_id);
    }
    indexes.controls_by_id.insert(record.event_id, record);
}

fn derive_pending_controls(indexes: &mut TrustedIndexes) {
    for record in indexes.controls.controls_by_id.values() {
        let parent_missing = record
            .parent
            .is_some_and(|parent| !indexes.controls.controls_by_id.contains_key(&parent));
        let frontier_missing = record
            .base_heads
            .iter()
            .any(|head| !indexes.changes.carriers_by_hash.contains_key(head));
        if parent_missing || frontier_missing {
            indexes.controls.pending.insert(record.event_id);
        }
    }
}

fn index_change(indexes: &mut ChangeIndexes, change: &crate::carrier::change::ChangeCarrier) {
    let event_id = change.event_id();
    let change_hash = change.change_hash();
    let semantic = SemanticChangeRecord {
        actor: change.actor(),
        sequence: change.sequence(),
        start_op: change.start_op(),
        operation_count: change.operation_count(),
        dependencies: change.dependencies().collect(),
    };
    indexes
        .semantic_by_hash
        .entry(change_hash)
        .and_modify(|existing| debug_assert_eq!(existing, &semantic))
        .or_insert(semantic);
    let claim = ChangeCarrierClaim {
        event_id,
        coordinate: change.coordinate(),
        change_hash,
        control_id: change.control_id(),
        author: change.author_device(),
    };
    indexes
        .claims_by_hash
        .entry(change_hash)
        .or_default()
        .insert(event_id, claim.clone());
    indexes.claims_by_event.insert(event_id, claim);
    indexes
        .claims_by_control
        .entry(change.control_id())
        .or_default()
        .entry(change_hash)
        .or_default()
        .insert(event_id);
    indexes
        .carriers_by_hash
        .entry(change_hash)
        .or_default()
        .insert(event_id);
    indexes
        .preferred_carrier
        .entry(change_hash)
        .and_modify(|current| *current = (*current).min(event_id))
        .or_insert(event_id);
    indexes
        .hashes_by_control
        .entry(change.control_id())
        .or_default()
        .insert(change_hash);
    indexes
        .hashes_by_actor
        .entry(change.actor())
        .or_default()
        .insert(change_hash);
    indexes
        .dependencies_by_hash
        .entry(change_hash)
        .or_default()
        .extend(change.dependencies());
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::derive_trusted_indexes;
    use crate::carrier::VerifiedCarrier;
    use crate::carrier::control::{ValidatedControlCarrier, ValidatedControlContent};
    use crate::evidence::event::{EventEvidence, RawChecksum};
    use crate::{ControllerPublicKey, DocumentCoordinate, DocumentId, EventId};

    #[test]
    fn trusted_indexes_only_include_validated_carrier_evidence() {
        let controller = ControllerPublicKey::from_bytes([1; 32]);
        let coordinate = DocumentCoordinate::new(controller, DocumentId::from_bytes([2; 32]));
        let event_id = EventId::from_bytes([3; 32]);
        let control = ValidatedControlCarrier::for_test(
            event_id,
            controller,
            coordinate,
            None,
            ValidatedControlContent {
                base_heads: Vec::new(),
                members: Vec::new(),
                predecessor: None,
                sequence: 0,
                successor: None,
                terminal: true,
            },
        );
        let events = BTreeMap::from([(
            event_id,
            EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::Control(Box::new(control)),
                raw_checksum: RawChecksum::test_only([4; 32]),
            },
        )]);
        let indexes = derive_trusted_indexes(&events);
        assert_eq!(
            indexes.controls.genesis.into_iter().collect::<Vec<_>>(),
            vec![event_id]
        );
        assert!(indexes.changes.carriers_by_hash.is_empty());
    }
}
