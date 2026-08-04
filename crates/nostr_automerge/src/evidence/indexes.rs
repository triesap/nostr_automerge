use std::collections::{BTreeMap, BTreeSet};

use crate::{ActorId, ChangeHash, EventId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IndexValidity {
    Valid,
    Pending,
    Invalid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ControlIndexRecord {
    pub(crate) event_id: EventId,
    pub(crate) parent: Option<EventId>,
    pub(crate) validity: IndexValidity,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ControlIndexes {
    pub(crate) controls_by_id: BTreeMap<EventId, ControlIndexRecord>,
    pub(crate) genesis: BTreeSet<EventId>,
    pub(crate) children_by_parent: BTreeMap<EventId, BTreeSet<EventId>>,
    pub(crate) pending: BTreeSet<EventId>,
    pub(crate) invalid: BTreeSet<EventId>,
}

pub(crate) fn index_controls(
    records: impl IntoIterator<Item = ControlIndexRecord>,
) -> ControlIndexes {
    let mut indexes = ControlIndexes::default();
    for record in records {
        match record.validity {
            IndexValidity::Valid => {
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
            IndexValidity::Pending => {
                indexes.pending.insert(record.event_id);
            }
            IndexValidity::Invalid => {
                indexes.invalid.insert(record.event_id);
            }
        }
    }
    indexes
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ChangeIndexRecord {
    pub(crate) event_id: EventId,
    pub(crate) change_hash: ChangeHash,
    pub(crate) control_id: EventId,
    pub(crate) actor: ActorId,
    pub(crate) dependencies: Vec<ChangeHash>,
    pub(crate) validity: IndexValidity,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ChangeIndexes {
    pub(crate) valid_carriers_by_hash: BTreeMap<ChangeHash, BTreeSet<EventId>>,
    pub(crate) invalid_carriers_by_hash: BTreeMap<ChangeHash, BTreeSet<EventId>>,
    pub(crate) hashes_by_control: BTreeMap<EventId, BTreeSet<ChangeHash>>,
    pub(crate) hashes_by_actor: BTreeMap<ActorId, BTreeSet<ChangeHash>>,
    pub(crate) dependencies_by_hash: BTreeMap<ChangeHash, BTreeSet<ChangeHash>>,
    pub(crate) pending: BTreeSet<EventId>,
}

pub(crate) fn index_changes(records: impl IntoIterator<Item = ChangeIndexRecord>) -> ChangeIndexes {
    let mut indexes = ChangeIndexes::default();
    for record in records {
        match record.validity {
            IndexValidity::Valid => {
                indexes
                    .valid_carriers_by_hash
                    .entry(record.change_hash)
                    .or_default()
                    .insert(record.event_id);
                indexes
                    .hashes_by_control
                    .entry(record.control_id)
                    .or_default()
                    .insert(record.change_hash);
                indexes
                    .hashes_by_actor
                    .entry(record.actor)
                    .or_default()
                    .insert(record.change_hash);
                indexes
                    .dependencies_by_hash
                    .entry(record.change_hash)
                    .or_default()
                    .extend(record.dependencies);
            }
            IndexValidity::Pending => {
                indexes.pending.insert(record.event_id);
            }
            IndexValidity::Invalid => {
                indexes
                    .invalid_carriers_by_hash
                    .entry(record.change_hash)
                    .or_default()
                    .insert(record.event_id);
            }
        }
    }
    indexes
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{
        ChangeIndexRecord, ControlIndexRecord, IndexValidity, index_changes, index_controls,
    };
    use crate::{ActorId, ChangeHash, EventId};

    #[test]
    fn index_controls_deterministically() {
        let parent = EventId::from_bytes([1; 32]);
        let child = EventId::from_bytes([2; 32]);
        let invalid = EventId::from_bytes([3; 32]);
        let pending = EventId::from_bytes([4; 32]);
        let records = vec![
            ControlIndexRecord {
                event_id: child,
                parent: Some(parent),
                validity: IndexValidity::Valid,
            },
            ControlIndexRecord {
                event_id: invalid,
                parent: None,
                validity: IndexValidity::Invalid,
            },
            ControlIndexRecord {
                event_id: parent,
                parent: None,
                validity: IndexValidity::Valid,
            },
            ControlIndexRecord {
                event_id: pending,
                parent: Some(parent),
                validity: IndexValidity::Pending,
            },
        ];
        let mut reversed = records.clone();
        reversed.reverse();
        let first = index_controls(records);
        assert_eq!(first, index_controls(reversed));
        assert_eq!(
            first.genesis.iter().copied().collect::<Vec<_>>(),
            vec![parent]
        );
        assert_eq!(
            first
                .children_by_parent
                .get(&parent)
                .and_then(|set| set.first())
                .copied(),
            Some(child)
        );
        assert!(first.invalid.contains(&invalid));
        assert!(first.pending.contains(&pending));
        assert!(!first.controls_by_id.contains_key(&invalid));
    }

    #[test]
    fn index_change_carriers_by_changehash() {
        let hash = ChangeHash::from_bytes([1; 32]);
        let dependency = ChangeHash::from_bytes([2; 32]);
        let control = EventId::from_bytes([3; 32]);
        let actor = ActorId::from_bytes([4; 32]);
        let valid_a = EventId::from_bytes([5; 32]);
        let valid_b = EventId::from_bytes([6; 32]);
        let invalid = EventId::from_bytes([7; 32]);
        let records = vec![
            ChangeIndexRecord {
                event_id: valid_b,
                change_hash: hash,
                control_id: control,
                actor,
                dependencies: vec![dependency],
                validity: IndexValidity::Valid,
            },
            ChangeIndexRecord {
                event_id: invalid,
                change_hash: hash,
                control_id: control,
                actor,
                dependencies: vec![ChangeHash::from_bytes([9; 32])],
                validity: IndexValidity::Invalid,
            },
            ChangeIndexRecord {
                event_id: valid_a,
                change_hash: hash,
                control_id: control,
                actor,
                dependencies: vec![dependency],
                validity: IndexValidity::Valid,
            },
        ];
        let mut reversed = records.clone();
        reversed.reverse();
        let indexes = index_changes(records);
        assert_eq!(indexes, index_changes(reversed));
        assert_eq!(
            indexes.valid_carriers_by_hash.get(&hash),
            Some(&BTreeSet::from([valid_a, valid_b]))
        );
        assert_eq!(
            indexes.invalid_carriers_by_hash.get(&hash),
            Some(&BTreeSet::from([invalid]))
        );
        assert_eq!(
            indexes.dependencies_by_hash.get(&hash),
            Some(&BTreeSet::from([dependency]))
        );
        assert_eq!(
            indexes.hashes_by_control.get(&control),
            Some(&BTreeSet::from([hash]))
        );
        assert_eq!(
            indexes.hashes_by_actor.get(&actor),
            Some(&BTreeSet::from([hash]))
        );
    }
}
