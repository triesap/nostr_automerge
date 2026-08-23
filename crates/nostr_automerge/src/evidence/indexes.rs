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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IndexedParentEvidence {
    ValidatedControl,
    UnsupportedRevision,
    StaticInvalidControl,
    WrongKind,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ControlIndexes {
    pub(crate) controls_by_id: BTreeMap<EventId, ControlIndexRecord>,
    pub(crate) genesis: BTreeSet<EventId>,
    pub(crate) children_by_parent: BTreeMap<EventId, BTreeSet<EventId>>,
    pub(crate) parent_evidence: BTreeMap<EventId, IndexedParentEvidence>,
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
    pub(crate) carriers_by_coordinate_hash:
        BTreeMap<(DocumentCoordinate, ChangeHash), BTreeSet<EventId>>,
    pub(crate) preferred_carrier: BTreeMap<ChangeHash, EventId>,
    pub(crate) hashes_by_control: BTreeMap<EventId, BTreeSet<ChangeHash>>,
    pub(crate) hashes_by_coordinate_control:
        BTreeMap<(DocumentCoordinate, EventId), BTreeSet<ChangeHash>>,
    pub(crate) hashes_by_actor: BTreeMap<ActorId, BTreeSet<ChangeHash>>,
    pub(crate) dependencies_by_hash: BTreeMap<ChangeHash, BTreeSet<ChangeHash>>,
    pub(crate) raw_changes_by_coordinate_hash: BTreeMap<(DocumentCoordinate, ChangeHash), Vec<u8>>,
    pub(crate) inconsistent_raw_changes: BTreeSet<(DocumentCoordinate, ChangeHash)>,
    pub(crate) prior_claims_by_coordinate:
        BTreeMap<DocumentCoordinate, BTreeMap<ChangeHash, BTreeSet<EventId>>>,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IndexedDescriptorEvidence {
    ValidatedDescriptor,
    UnsupportedRevision,
    StaticInvalidDescriptor,
    WrongKind,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct CheckpointIndexes {
    pub(crate) descriptors_by_id: BTreeMap<EventId, CheckpointDescriptorIndexRecord>,
    pub(crate) descriptors_by_coordinate: BTreeMap<DocumentCoordinate, BTreeSet<EventId>>,
    pub(crate) chunks_by_id: BTreeMap<EventId, CheckpointChunkIndexRecord>,
    pub(crate) chunks_by_descriptor: BTreeMap<EventId, BTreeSet<EventId>>,
    pub(crate) chunks_by_coordinate_descriptor:
        BTreeMap<(DocumentCoordinate, EventId), BTreeSet<EventId>>,
    pub(crate) descriptor_evidence: BTreeMap<EventId, IndexedDescriptorEvidence>,
    pub(crate) pending_descriptors: BTreeSet<EventId>,
    pub(crate) pending_chunks: BTreeSet<EventId>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct CoordinateEvidenceIndexes {
    pub(crate) events: BTreeMap<DocumentCoordinate, BTreeSet<EventId>>,
    pub(crate) controls: BTreeMap<DocumentCoordinate, BTreeSet<EventId>>,
    pub(crate) control_children_by_coordinate_parent:
        BTreeMap<DocumentCoordinate, BTreeMap<Option<EventId>, BTreeSet<EventId>>>,
    pub(crate) change_hashes: BTreeMap<DocumentCoordinate, BTreeSet<ChangeHash>>,
    pub(crate) manifests: BTreeMap<DocumentCoordinate, BTreeSet<EventId>>,
    pub(crate) lifecycle_support: BTreeMap<DocumentCoordinate, BTreeSet<EventId>>,
    pub(crate) duplicates: BTreeMap<DocumentCoordinate, Vec<usize>>,
    pub(crate) work: BTreeMap<DocumentCoordinate, CoordinateWorkMetadata>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct CoordinateWorkMetadata {
    pub(crate) control_count: usize,
    pub(crate) control_relationship_count: usize,
    pub(crate) change_hash_count: usize,
    pub(crate) evaluation_event_count: usize,
    pub(crate) carrier_evidence_count: usize,
    pub(crate) reportable_event_count: usize,
    pub(crate) change_carrier_event_count: usize,
    pub(crate) other_event_count: usize,
    pub(crate) evidence_record_count: usize,
    pub(crate) checkpoint_descriptor_count: usize,
    pub(crate) checkpoint_chunk_count: usize,
    pub(crate) decode_work_bytes: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct TrustedIndexes {
    pub(crate) controls: ControlIndexes,
    pub(crate) changes: ChangeIndexes,
    pub(crate) checkpoints: CheckpointIndexes,
    pub(crate) coordinates: CoordinateEvidenceIndexes,
}

pub(crate) fn derive_trusted_indexes(
    events: &BTreeMap<EventId, EventEvidence>,
    duplicates: &[EventEvidence],
) -> TrustedIndexes {
    let mut indexes = TrustedIndexes::default();
    for (event_id, evidence) in events {
        if let Some(coordinate) = evidence_coordinate(evidence) {
            indexes
                .coordinates
                .events
                .entry(coordinate)
                .or_default()
                .insert(*event_id);
            if is_manifest_candidate(evidence) {
                indexes
                    .coordinates
                    .manifests
                    .entry(coordinate)
                    .or_default()
                    .insert(*event_id);
            }
        }
        let EventEvidence::VerifiedCarrier { carrier, .. } = evidence else {
            continue;
        };
        match carrier {
            VerifiedCarrier::Control(control) => {
                indexes
                    .coordinates
                    .control_children_by_coordinate_parent
                    .entry(control.coordinate())
                    .or_default()
                    .entry(control.parent())
                    .or_default()
                    .insert(control.event_id());
                indexes
                    .coordinates
                    .controls
                    .entry(control.coordinate())
                    .or_default()
                    .insert(control.event_id());
                index_control(&mut indexes.controls, control);
            }
            VerifiedCarrier::Change(change) => {
                indexes
                    .coordinates
                    .change_hashes
                    .entry(change.coordinate())
                    .or_default()
                    .insert(change.change_hash());
                index_change(&mut indexes.changes, change);
            }
            VerifiedCarrier::CheckpointDescriptor(descriptor) => {
                index_checkpoint_descriptor(&mut indexes.checkpoints, descriptor);
            }
            VerifiedCarrier::CheckpointChunk(chunk) => {
                index_checkpoint_chunk(&mut indexes.checkpoints, chunk);
            }
            VerifiedCarrier::Manifest(manifest) => {
                indexes
                    .coordinates
                    .manifests
                    .entry(manifest.coordinate())
                    .or_default()
                    .insert(manifest.event_id);
            }
            VerifiedCarrier::UnsupportedRevision { .. } => {}
        }
    }
    for (index, duplicate) in duplicates.iter().enumerate() {
        let EventEvidence::DuplicateEvent { event_id, .. } = duplicate else {
            continue;
        };
        if let Some(coordinate) = events.get(event_id).and_then(evidence_coordinate) {
            indexes
                .coordinates
                .duplicates
                .entry(coordinate)
                .or_default()
                .push(index);
        }
    }
    derive_parent_evidence(&mut indexes, events);
    derive_pending_controls(&mut indexes);
    derive_descriptor_evidence(&mut indexes, events);
    derive_pending_checkpoints(&mut indexes);
    derive_lifecycle_support(&mut indexes, events);
    derive_coordinate_work_metadata(&mut indexes, events);
    indexes
}

fn is_manifest_candidate(evidence: &EventEvidence) -> bool {
    match evidence {
        EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::Manifest(_),
            ..
        } => true,
        EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::UnsupportedRevision { event, .. },
            ..
        }
        | EventEvidence::UnsupportedRevision {
            carrier: VerifiedCarrier::UnsupportedRevision { event, .. },
            ..
        }
        | EventEvidence::InvalidCarrier { event, .. }
        | EventEvidence::IrrelevantEvent { event, .. } => event.kind() == 31_624,
        EventEvidence::VerifiedCarrier { .. }
        | EventEvidence::UnsupportedRevision { .. }
        | EventEvidence::InvalidEvent { .. }
        | EventEvidence::DuplicateEvent { .. } => false,
    }
}

fn derive_coordinate_work_metadata(
    indexes: &mut TrustedIndexes,
    events: &BTreeMap<EventId, EventEvidence>,
) {
    for (coordinate, reportable) in &indexes.coordinates.events {
        let support = indexes.coordinates.lifecycle_support.get(coordinate);
        let duplicates = indexes
            .coordinates
            .duplicates
            .get(coordinate)
            .map_or(0, Vec::len);
        let evaluation_event_count = reportable
            .len()
            .saturating_add(support.map_or(0, BTreeSet::len))
            .saturating_add(duplicates);
        let carrier_evidence_count = reportable
            .iter()
            .chain(support.into_iter().flatten())
            .filter(|event_id| {
                matches!(
                    events.get(event_id),
                    Some(
                        EventEvidence::VerifiedCarrier { .. }
                            | EventEvidence::InvalidCarrier { .. }
                            | EventEvidence::UnsupportedRevision { .. }
                    )
                )
            })
            .count();
        let control_count = indexes
            .coordinates
            .controls
            .get(coordinate)
            .map_or(0, BTreeSet::len);
        let change_carrier_event_count = reportable
            .iter()
            .filter(|event_id| is_attributable_change_carrier(events.get(event_id)))
            .count();
        let other_event_count = reportable
            .len()
            .checked_sub(control_count)
            .and_then(|count| count.checked_sub(change_carrier_event_count))
            .unwrap_or(usize::MAX);
        let mut seen = BTreeSet::<ChangeHash>::new();
        let decode_work_bytes = reportable.iter().try_fold(0_u64, |total, event_id| {
            let Some(EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::Change(change),
                ..
            }) = events.get(event_id)
            else {
                return Some(total);
            };
            if !seen.insert(change.change_hash()) {
                return Some(total);
            }
            total.checked_add(change.decode_work_bytes()?)
        });
        indexes.coordinates.work.insert(
            *coordinate,
            CoordinateWorkMetadata {
                control_count,
                control_relationship_count: indexes
                    .coordinates
                    .control_children_by_coordinate_parent
                    .get(coordinate)
                    .into_iter()
                    .flat_map(BTreeMap::values)
                    .try_fold(0_usize, |total, children| total.checked_add(children.len()))
                    .unwrap_or(usize::MAX),
                change_hash_count: indexes
                    .coordinates
                    .change_hashes
                    .get(coordinate)
                    .map_or(0, BTreeSet::len),
                evaluation_event_count,
                carrier_evidence_count,
                reportable_event_count: reportable.len(),
                change_carrier_event_count,
                other_event_count,
                evidence_record_count: reportable.len().saturating_add(duplicates),
                checkpoint_descriptor_count: indexes
                    .checkpoints
                    .descriptors_by_coordinate
                    .get(coordinate)
                    .map_or(0, BTreeSet::len),
                checkpoint_chunk_count: indexes
                    .checkpoints
                    .chunks_by_coordinate_descriptor
                    .range(
                        (*coordinate, EventId::from_bytes([0; 32]))
                            ..=(*coordinate, EventId::from_bytes([u8::MAX; 32])),
                    )
                    .try_fold(0_usize, |total, (_, chunks)| {
                        total.checked_add(chunks.len())
                    })
                    .unwrap_or(usize::MAX),
                decode_work_bytes,
            },
        );
    }
}

fn is_attributable_change_carrier(evidence: Option<&EventEvidence>) -> bool {
    match evidence {
        Some(EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::Change(_),
            ..
        }) => true,
        Some(EventEvidence::InvalidCarrier { event, .. })
        | Some(EventEvidence::UnsupportedRevision {
            carrier: VerifiedCarrier::UnsupportedRevision { event, .. },
            ..
        })
        | Some(EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::UnsupportedRevision { event, .. },
            ..
        }) => event.kind() == 1_624,
        Some(
            EventEvidence::VerifiedCarrier { .. }
            | EventEvidence::UnsupportedRevision { .. }
            | EventEvidence::InvalidEvent { .. }
            | EventEvidence::IrrelevantEvent { .. }
            | EventEvidence::DuplicateEvent { .. },
        )
        | None => false,
    }
}

fn derive_lifecycle_support(
    indexes: &mut TrustedIndexes,
    events: &BTreeMap<EventId, EventEvidence>,
) {
    for (coordinate, control_ids) in &indexes.coordinates.controls {
        for event_id in control_ids {
            let Some(EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::Control(control),
                ..
            }) = events.get(event_id)
            else {
                continue;
            };
            if control.parent().is_none()
                && let Some(link) = control.predecessor()
                && !indexes
                    .coordinates
                    .events
                    .get(coordinate)
                    .is_some_and(|ids| ids.contains(&link.terminal_control))
                && events.contains_key(&link.terminal_control)
            {
                indexes
                    .coordinates
                    .lifecycle_support
                    .entry(*coordinate)
                    .or_default()
                    .insert(link.terminal_control);
            }
        }
    }
}

pub(crate) fn evidence_coordinate(evidence: &EventEvidence) -> Option<DocumentCoordinate> {
    match evidence {
        EventEvidence::VerifiedCarrier { carrier, .. } => match carrier {
            VerifiedCarrier::Manifest(value) => Some(value.coordinate()),
            VerifiedCarrier::Control(value) => Some(value.coordinate()),
            VerifiedCarrier::Change(value) => Some(value.coordinate()),
            VerifiedCarrier::CheckpointDescriptor(value) => Some(value.coordinate()),
            VerifiedCarrier::CheckpointChunk(value) => Some(value.coordinate()),
            VerifiedCarrier::UnsupportedRevision { event, .. } => signed_coordinate(event),
        },
        EventEvidence::InvalidCarrier { event, .. }
        | EventEvidence::IrrelevantEvent { event, .. } => signed_coordinate(event),
        EventEvidence::UnsupportedRevision {
            carrier: VerifiedCarrier::UnsupportedRevision { event, .. },
            ..
        } => signed_coordinate(event),
        EventEvidence::UnsupportedRevision { .. }
        | EventEvidence::InvalidEvent { .. }
        | EventEvidence::DuplicateEvent { .. } => None,
    }
}

fn signed_coordinate(event: &crate::VerifiedNip01Event) -> Option<DocumentCoordinate> {
    if event.kind() == 31_624 {
        return super::corpus_builder::manifest_coordinate(event);
    }
    if !matches!(event.kind(), 1_624..=1_627) {
        return None;
    }
    let values = event
        .tags()
        .iter()
        .filter(|tag| tag.first().is_some_and(|value| value == "a"))
        .filter_map(|tag| tag.get(1)?.parse().ok())
        .collect::<BTreeSet<_>>();
    (values.len() == 1)
        .then(|| values.into_iter().next())
        .flatten()
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
    indexes
        .chunks_by_coordinate_descriptor
        .entry((record.coordinate, record.descriptor_id))
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
            .descriptor_evidence
            .contains_key(&chunk.descriptor_id)
        {
            indexes.checkpoints.pending_chunks.insert(chunk.event_id);
        }
    }
}

fn derive_descriptor_evidence(
    indexes: &mut TrustedIndexes,
    events: &BTreeMap<EventId, EventEvidence>,
) {
    for descriptor_id in indexes.checkpoints.chunks_by_descriptor.keys().copied() {
        let Some(evidence) = events.get(&descriptor_id) else {
            continue;
        };
        let state = match evidence {
            EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::CheckpointDescriptor(_),
                ..
            } => IndexedDescriptorEvidence::ValidatedDescriptor,
            EventEvidence::UnsupportedRevision { .. }
            | EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::UnsupportedRevision { .. },
                ..
            } => IndexedDescriptorEvidence::UnsupportedRevision,
            EventEvidence::InvalidCarrier { event, .. } if event.kind() == 1_626 => {
                IndexedDescriptorEvidence::StaticInvalidDescriptor
            }
            EventEvidence::VerifiedCarrier { .. }
            | EventEvidence::InvalidCarrier { .. }
            | EventEvidence::InvalidEvent { .. }
            | EventEvidence::IrrelevantEvent { .. }
            | EventEvidence::DuplicateEvent { .. } => IndexedDescriptorEvidence::WrongKind,
        };
        indexes
            .checkpoints
            .descriptor_evidence
            .insert(descriptor_id, state);
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
            .is_some_and(|parent| !indexes.controls.parent_evidence.contains_key(&parent));
        let frontier_missing = record
            .base_heads
            .iter()
            .any(|head| !indexes.changes.carriers_by_hash.contains_key(head));
        if parent_missing || frontier_missing {
            indexes.controls.pending.insert(record.event_id);
        }
    }
}

fn derive_parent_evidence(indexes: &mut TrustedIndexes, events: &BTreeMap<EventId, EventEvidence>) {
    for parent in indexes.controls.children_by_parent.keys().copied() {
        let Some(evidence) = events.get(&parent) else {
            continue;
        };
        let state = match evidence {
            EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::Control(_),
                ..
            } => IndexedParentEvidence::ValidatedControl,
            EventEvidence::UnsupportedRevision { .. }
            | EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::UnsupportedRevision { .. },
                ..
            } => IndexedParentEvidence::UnsupportedRevision,
            EventEvidence::InvalidCarrier { event, .. } if event.kind() == 1_625 => {
                IndexedParentEvidence::StaticInvalidControl
            }
            EventEvidence::VerifiedCarrier { .. }
            | EventEvidence::InvalidCarrier { .. }
            | EventEvidence::InvalidEvent { .. }
            | EventEvidence::IrrelevantEvent { .. }
            | EventEvidence::DuplicateEvent { .. } => IndexedParentEvidence::WrongKind,
        };
        indexes.controls.parent_evidence.insert(parent, state);
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
    index_canonical_raw_change(
        indexes,
        change.coordinate(),
        change_hash,
        change.canonical_raw_bytes(),
    );
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
    indexes
        .prior_claims_by_coordinate
        .entry(change.coordinate())
        .or_default()
        .entry(change_hash)
        .or_default()
        .insert(event_id);
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
        .carriers_by_coordinate_hash
        .entry((change.coordinate(), change_hash))
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
        .hashes_by_coordinate_control
        .entry((change.coordinate(), change.control_id()))
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

fn index_canonical_raw_change(
    indexes: &mut ChangeIndexes,
    coordinate: DocumentCoordinate,
    change_hash: ChangeHash,
    raw: &[u8],
) {
    let key = (coordinate, change_hash);
    if indexes.inconsistent_raw_changes.contains(&key) {
        return;
    }
    match indexes.raw_changes_by_coordinate_hash.entry(key) {
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(raw.to_vec());
        }
        std::collections::btree_map::Entry::Occupied(entry) if entry.get() != raw => {
            entry.remove();
            indexes.inconsistent_raw_changes.insert(key);
        }
        std::collections::btree_map::Entry::Occupied(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        ChangeIndexes, CheckpointIndexes, CoordinateWorkMetadata, IndexedParentEvidence,
        derive_trusted_indexes, index_canonical_raw_change, index_checkpoint_chunk,
    };
    use crate::carrier::VerifiedCarrier;
    use crate::carrier::checkpoint_chunk::ValidatedCheckpointChunkCarrier;
    use crate::carrier::control::{ValidatedControlCarrier, ValidatedControlContent};
    use crate::checkpoint::CheckpointChunk;
    use crate::evidence::event::{EventEvidence, RawChecksum};
    use crate::{
        ChangeHash, ChunkHash, ControllerPublicKey, DevicePublicKey, DocumentCoordinate,
        DocumentId, EventId,
    };

    #[test]
    fn dependent_checkpoint_index_is_coordinate_qualified() {
        let controller = ControllerPublicKey::from_bytes([1; 32]);
        let first_coordinate = DocumentCoordinate::new(controller, DocumentId::from_bytes([2; 32]));
        let second_coordinate =
            DocumentCoordinate::new(controller, DocumentId::from_bytes([3; 32]));
        let descriptor_id = EventId::from_bytes([4; 32]);
        let chunk = |coordinate, event| {
            ValidatedCheckpointChunkCarrier::for_test(
                EventId::from_bytes([event; 32]),
                DevicePublicKey::from_bytes([5; 32]),
                coordinate,
                descriptor_id,
                ChunkHash::from_bytes([6; 32]),
                CheckpointChunk {
                    index: 0,
                    count: 1,
                    data: vec![event],
                    proof: Vec::new(),
                },
            )
        };
        let mut indexes = CheckpointIndexes::default();
        index_checkpoint_chunk(&mut indexes, &chunk(first_coordinate, 7));
        index_checkpoint_chunk(&mut indexes, &chunk(second_coordinate, 8));
        assert_eq!(
            indexes
                .chunks_by_coordinate_descriptor
                .get(&(first_coordinate, descriptor_id)),
            Some(&std::collections::BTreeSet::from([EventId::from_bytes(
                [7; 32]
            )]))
        );
        assert_eq!(
            indexes
                .chunks_by_coordinate_descriptor
                .get(&(second_coordinate, descriptor_id)),
            Some(&std::collections::BTreeSet::from([EventId::from_bytes(
                [8; 32]
            )]))
        );
    }

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
        let indexes = derive_trusted_indexes(&events, &[]);
        assert_eq!(
            indexes.controls.genesis.into_iter().collect::<Vec<_>>(),
            vec![event_id]
        );
        assert!(indexes.changes.carriers_by_hash.is_empty());
        assert!(indexes.checkpoints.descriptor_evidence.is_empty());
        assert_eq!(
            indexes.coordinates.work.get(&coordinate),
            Some(&CoordinateWorkMetadata {
                control_count: 1,
                control_relationship_count: 1,
                change_hash_count: 0,
                evaluation_event_count: 1,
                carrier_evidence_count: 1,
                reportable_event_count: 1,
                change_carrier_event_count: 0,
                other_event_count: 0,
                evidence_record_count: 1,
                checkpoint_descriptor_count: 0,
                checkpoint_chunk_count: 0,
                decode_work_bytes: Some(0),
            })
        );
    }

    #[test]
    fn parent_index_retains_present_control_and_missing_parent_identities() {
        let controller = ControllerPublicKey::from_bytes([5; 32]);
        let coordinate = DocumentCoordinate::new(controller, DocumentId::from_bytes([6; 32]));
        let parent_id = EventId::from_bytes([7; 32]);
        let missing_id = EventId::from_bytes([8; 32]);
        let parent = ValidatedControlCarrier::for_test(
            parent_id,
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
        let child = |event_id, parent| {
            ValidatedControlCarrier::for_test(
                event_id,
                controller,
                coordinate,
                Some(parent),
                ValidatedControlContent {
                    base_heads: Vec::new(),
                    members: Vec::new(),
                    predecessor: None,
                    sequence: 1,
                    successor: None,
                    terminal: true,
                },
            )
        };
        let present_child_id = EventId::from_bytes([9; 32]);
        let missing_child_id = EventId::from_bytes([10; 32]);
        let checksum = RawChecksum::test_only([11; 32]);
        let events = BTreeMap::from([
            (
                parent_id,
                EventEvidence::VerifiedCarrier {
                    carrier: VerifiedCarrier::Control(Box::new(parent)),
                    raw_checksum: checksum,
                },
            ),
            (
                present_child_id,
                EventEvidence::VerifiedCarrier {
                    carrier: VerifiedCarrier::Control(Box::new(child(present_child_id, parent_id))),
                    raw_checksum: checksum,
                },
            ),
            (
                missing_child_id,
                EventEvidence::VerifiedCarrier {
                    carrier: VerifiedCarrier::Control(Box::new(child(
                        missing_child_id,
                        missing_id,
                    ))),
                    raw_checksum: checksum,
                },
            ),
        ]);
        let indexes = derive_trusted_indexes(&events, &[]);
        assert_eq!(
            indexes.controls.parent_evidence.get(&parent_id),
            Some(&IndexedParentEvidence::ValidatedControl)
        );
        assert!(!indexes.controls.parent_evidence.contains_key(&missing_id));
        assert!(indexes.controls.pending.contains(&missing_child_id));
        assert!(!indexes.controls.pending.contains(&present_child_id));
        let relationships = indexes
            .coordinates
            .control_children_by_coordinate_parent
            .get(&coordinate);
        assert_eq!(
            relationships.and_then(|children| children.get(&None)),
            Some(&std::collections::BTreeSet::from([parent_id]))
        );
        assert_eq!(
            relationships.and_then(|children| children.get(&Some(parent_id))),
            Some(&std::collections::BTreeSet::from([present_child_id]))
        );
        assert_eq!(
            relationships.and_then(|children| children.get(&Some(missing_id))),
            Some(&std::collections::BTreeSet::from([missing_child_id]))
        );
        assert_eq!(
            indexes
                .coordinates
                .work
                .get(&coordinate)
                .map(|work| work.control_relationship_count),
            Some(3)
        );
    }

    #[test]
    fn canonical_raw_change_index_rejects_inconsistent_duplicate_bytes() {
        let coordinate = DocumentCoordinate::new(
            ControllerPublicKey::from_bytes([12; 32]),
            DocumentId::from_bytes([13; 32]),
        );
        let hash = ChangeHash::from_bytes([14; 32]);
        let mut indexes = ChangeIndexes::default();
        index_canonical_raw_change(&mut indexes, coordinate, hash, b"canonical");
        index_canonical_raw_change(&mut indexes, coordinate, hash, b"canonical");
        assert_eq!(
            indexes
                .raw_changes_by_coordinate_hash
                .get(&(coordinate, hash))
                .map(Vec::as_slice),
            Some(b"canonical".as_slice())
        );
        index_canonical_raw_change(&mut indexes, coordinate, hash, b"inconsistent");
        assert!(
            !indexes
                .raw_changes_by_coordinate_hash
                .contains_key(&(coordinate, hash))
        );
        assert!(
            indexes
                .inconsistent_raw_changes
                .contains(&(coordinate, hash))
        );
        index_canonical_raw_change(&mut indexes, coordinate, hash, b"canonical");
        assert!(
            !indexes
                .raw_changes_by_coordinate_hash
                .contains_key(&(coordinate, hash))
        );
    }
}
