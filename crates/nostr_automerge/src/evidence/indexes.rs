use std::collections::{BTreeMap, BTreeSet};

use crate::{ChangeHash, EventId};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ChangeIndexRecord {
    pub(crate) event_id: EventId,
    pub(crate) change_hash: ChangeHash,
    pub(crate) control_id: EventId,
    pub(crate) validity: IndexValidity,
}

#[cfg(test)]
mod tests {
    use super::{ControlIndexRecord, IndexValidity, index_controls};
    use crate::EventId;

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
}
