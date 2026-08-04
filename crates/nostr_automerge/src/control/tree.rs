use std::collections::{BTreeMap, BTreeSet};

use super::validate::{ControlEnvelope, ControlValidationError};
use crate::EventId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ChildStructure {
    Valid,
    PendingParent,
    Invalid(ControlValidationError),
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ControlTree {
    accepted: BTreeMap<EventId, ControlEnvelope>,
    pending: BTreeSet<EventId>,
}

impl ControlTree {
    pub(crate) fn insert_accepted(&mut self, control: ControlEnvelope) {
        self.pending.remove(&control.event_id);
        self.accepted.insert(control.event_id, control);
    }

    pub(crate) fn mark_pending(&mut self, event_id: EventId) {
        if !self.accepted.contains_key(&event_id) {
            self.pending.insert(event_id);
        }
    }

    pub(crate) fn validate_child(&self, child: &ControlEnvelope) -> ChildStructure {
        let Some(parent_id) = child.parent else {
            return ChildStructure::Invalid(ControlValidationError::Parent);
        };
        let Some(parent) = self.accepted.get(&parent_id) else {
            return ChildStructure::PendingParent;
        };
        if child.coordinate != parent.coordinate || child.author != child.coordinate.controller() {
            return ChildStructure::Invalid(ControlValidationError::Author);
        }
        let Some(expected_sequence) = parent.content.sequence.checked_add(1) else {
            return ChildStructure::Invalid(ControlValidationError::Sequence);
        };
        if child.content.sequence != expected_sequence {
            return ChildStructure::Invalid(ControlValidationError::Sequence);
        }
        ChildStructure::Valid
    }
}

#[cfg(test)]
mod tests {
    use super::{ChildStructure, ControlTree};
    use crate::EventId;
    use crate::control::validate::ControlValidationError;
    use crate::control::validate::tests::genesis;

    #[test]
    fn implement_child_parent_and_sequence_validation() {
        let parent = genesis();
        let mut child = parent.clone();
        child.event_id = EventId::from_bytes([5; 32]);
        child.parent = Some(parent.event_id);
        child.content.sequence = 1;
        let mut tree = ControlTree::default();
        tree.insert_accepted(parent.clone());
        assert_eq!(tree.validate_child(&child), ChildStructure::Valid);

        let mut no_parent = child.clone();
        no_parent.parent = None;
        assert_eq!(
            tree.validate_child(&no_parent),
            ChildStructure::Invalid(ControlValidationError::Parent)
        );
        let mut unknown = child.clone();
        unknown.parent = Some(EventId::from_bytes([6; 32]));
        assert_eq!(tree.validate_child(&unknown), ChildStructure::PendingParent);
        tree.mark_pending(EventId::from_bytes([6; 32]));
        assert_eq!(tree.validate_child(&unknown), ChildStructure::PendingParent);
        let mut gap = child.clone();
        gap.content.sequence = 2;
        assert_eq!(
            tree.validate_child(&gap),
            ChildStructure::Invalid(ControlValidationError::Sequence)
        );

        let mut maximum = parent;
        maximum.event_id = EventId::from_bytes([7; 32]);
        maximum.content.sequence = u64::MAX;
        let maximum_id = maximum.event_id;
        tree.insert_accepted(maximum);
        let mut overflow = child;
        overflow.parent = Some(maximum_id);
        assert_eq!(
            tree.validate_child(&overflow),
            ChildStructure::Invalid(ControlValidationError::Sequence)
        );
    }
}
