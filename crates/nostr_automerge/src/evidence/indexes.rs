use std::collections::{BTreeMap, BTreeSet};

use crate::carrier::VerifiedCarrier;
use crate::evidence::event::EventEvidence;
use crate::{ActorId, ChangeHash, EventId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ControlIndexRecord {
    pub(crate) event_id: EventId,
    pub(crate) parent: Option<EventId>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ControlIndexes {
    pub(crate) controls_by_id: BTreeMap<EventId, ControlIndexRecord>,
    pub(crate) genesis: BTreeSet<EventId>,
    pub(crate) children_by_parent: BTreeMap<EventId, BTreeSet<EventId>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ChangeIndexRecord {
    pub(crate) event_id: EventId,
    pub(crate) change_hash: ChangeHash,
    pub(crate) control_id: EventId,
    pub(crate) actor: ActorId,
    pub(crate) dependencies: BTreeSet<ChangeHash>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ChangeIndexes {
    pub(crate) carriers_by_hash: BTreeMap<ChangeHash, BTreeSet<EventId>>,
    pub(crate) preferred_carrier: BTreeMap<ChangeHash, EventId>,
    pub(crate) hashes_by_control: BTreeMap<EventId, BTreeSet<ChangeHash>>,
    pub(crate) hashes_by_actor: BTreeMap<ActorId, BTreeSet<ChangeHash>>,
    pub(crate) dependencies_by_hash: BTreeMap<ChangeHash, BTreeSet<ChangeHash>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct TrustedIndexes {
    pub(crate) controls: ControlIndexes,
    pub(crate) changes: ChangeIndexes,
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
            VerifiedCarrier::Manifest(_)
            | VerifiedCarrier::CheckpointDescriptor(_)
            | VerifiedCarrier::CheckpointChunk(_)
            | VerifiedCarrier::UnsupportedRevision { .. } => {}
        }
    }
    indexes
}

fn index_control(
    indexes: &mut ControlIndexes,
    control: &crate::carrier::control::ValidatedControlCarrier,
) {
    let record = ControlIndexRecord {
        event_id: control.event_id(),
        parent: control.parent(),
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

fn index_change(indexes: &mut ChangeIndexes, change: &crate::carrier::change::ChangeCarrier) {
    let event_id = change.event_id();
    let change_hash = change.change_hash();
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
        let control = ValidatedControlCarrier::synthetic(
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
