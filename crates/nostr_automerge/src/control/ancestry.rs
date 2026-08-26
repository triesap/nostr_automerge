use std::sync::Arc;

use crate::carrier::control::ValidatedControlContent;
use crate::control::transition::TransitionError;
use crate::control::validate::ControlEnvelope;
use crate::{Completion, EventId};

#[derive(Clone, Default)]
pub(crate) struct ControlAncestry {
    tail: Option<Arc<ControlAncestryNode>>,
}

struct ControlAncestryNode {
    envelope: Arc<ControlEnvelope>,
    parent: Option<Arc<ControlAncestryNode>>,
}

impl ControlAncestry {
    pub(crate) fn is_empty(&self) -> bool {
        self.tail.is_none()
    }

    pub(crate) fn from_ordered(
        ancestry: impl IntoIterator<Item = ControlEnvelope>,
    ) -> Result<Self, ()> {
        let mut result = Self::default();
        for envelope in ancestry {
            result = result.push_checked(envelope)?;
        }
        Ok(result)
    }

    pub(crate) fn push_checked(&self, envelope: ControlEnvelope) -> Result<Self, ()> {
        let expected_parent = self.tail.as_ref().map(|node| node.envelope.event_id());
        if envelope.parent() != expected_parent {
            return Err(());
        }
        if let Some(parent) = &self.tail
            && (parent.envelope.coordinate != envelope.coordinate
                || parent
                    .envelope
                    .sequence()
                    .checked_add(1)
                    .is_none_or(|sequence| sequence != envelope.sequence()))
        {
            return Err(());
        }
        Ok(Self {
            tail: Some(Arc::new(ControlAncestryNode {
                envelope: Arc::new(envelope),
                parent: self.tail.clone(),
            })),
        })
    }

    pub(crate) fn last_event_id(&self) -> Option<EventId> {
        self.tail.as_ref().map(|node| node.envelope.event_id())
    }

    pub(crate) fn no_reintroduction_metered(
        &self,
        child: &ValidatedControlContent,
        visit: &mut impl FnMut() -> Result<(), Completion>,
    ) -> Result<Result<(), TransitionError>, Completion> {
        let mut grant_index = 0;
        while grant_index < child.members.len() {
            visit()?;
            let grant = &child.members[grant_index];
            grant_index += 1;
            let mut absent_after_presence = false;
            let mut cursor = self.tail.as_ref();
            while cursor.is_some() {
                visit()?;
                let Some(node) = cursor else {
                    break;
                };
                let mut present = false;
                let members = &node.envelope.content().members;
                let mut member_index = 0;
                while member_index < members.len() {
                    visit()?;
                    let historical = &members[member_index];
                    member_index += 1;
                    visit()?;
                    if historical.device == grant.device {
                        present = true;
                        break;
                    }
                }
                if present {
                    if absent_after_presence {
                        return Ok(Err(TransitionError::DeviceReintroduced));
                    }
                } else {
                    absent_after_presence = true;
                }
                if node.parent.is_some() {
                    visit()?;
                }
                cursor = node.parent.as_ref();
            }
        }
        Ok(Ok(()))
    }

    #[cfg(test)]
    fn parent_shares_tail_with(&self, parent: &Self) -> bool {
        match (&self.tail, &parent.tail) {
            (Some(child), Some(parent)) => child
                .parent
                .as_ref()
                .is_some_and(|retained| Arc::ptr_eq(retained, parent)),
            (Some(child), None) => child.parent.is_none(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::ControlAncestry;
    use crate::control::transition::TransitionError;
    use crate::control::validate::tests::genesis;
    use crate::{Completion, EventId};

    #[test]
    fn persistent_chain_retains_only_the_checked_parent_handle() {
        let root = genesis();
        let ancestry = ControlAncestry::from_ordered([root.clone()]);
        assert!(ancestry.is_ok());
        let Ok(mut ancestry) = ancestry else {
            return;
        };
        for sequence in 1_u64..=256 {
            let parent = ancestry.clone();
            let mut envelope = root.clone();
            let mut event_id = [0_u8; 32];
            event_id[..8].copy_from_slice(&sequence.to_be_bytes());
            envelope.event_id = EventId::from_bytes(event_id);
            envelope.parent = parent.last_event_id();
            envelope.content.sequence = sequence;
            let next = ancestry.push_checked(envelope);
            assert!(next.is_ok());
            let Ok(next) = next else {
                return;
            };
            ancestry = next;
            assert!(ancestry.parent_shares_tail_with(&parent));
        }
        assert_eq!(
            ancestry
                .last_event_id()
                .map(EventId::to_hex)
                .map(|id| id.len()),
            Some(64)
        );
    }

    #[test]
    fn wide_fork_children_share_one_checked_parent_tail() {
        let root = genesis();
        let ancestry = ControlAncestry::from_ordered([root.clone()]);
        assert!(ancestry.is_ok());
        let Ok(ancestry) = ancestry else {
            return;
        };
        for suffix in 1_u16..=256 {
            let mut child = root.clone();
            let mut event_id = [0_u8; 32];
            event_id[..2].copy_from_slice(&suffix.to_be_bytes());
            child.event_id = EventId::from_bytes(event_id);
            child.parent = ancestry.last_event_id();
            child.content.sequence = 1;
            let branch = ancestry.push_checked(child);
            assert!(branch.is_ok());
            let Ok(branch) = branch else {
                return;
            };
            assert!(branch.parent_shares_tail_with(&ancestry));
        }
    }

    #[test]
    fn ancestry_member_traversal_stops_at_every_prefix() {
        let root = genesis();
        let ancestry = ControlAncestry::from_ordered([root.clone()]);
        assert!(ancestry.is_ok());
        let Ok(ancestry) = ancestry else {
            return;
        };
        let visits = Cell::new(0_u64);
        let mut count = || {
            visits.set(visits.get() + 1);
            Ok(())
        };
        assert_eq!(
            ancestry.no_reintroduction_metered(&root.content, &mut count),
            Ok(Ok(()))
        );
        let exact = visits.get();
        assert!(exact > 0);
        for boundary in 0..exact {
            let observed = Cell::new(0_u64);
            let mut stop = || {
                if observed.get() == boundary {
                    return Err(Completion::Cancelled);
                }
                observed.set(observed.get() + 1);
                Ok(())
            };
            assert_eq!(
                ancestry.no_reintroduction_metered(&root.content, &mut stop),
                Err(Completion::Cancelled)
            );
            assert_eq!(observed.get(), boundary);
        }

        let mut removed = root.content.clone();
        removed.members.clear();
        let mut removed_envelope = root.clone();
        removed_envelope.event_id = EventId::from_bytes([8; 32]);
        removed_envelope.parent = Some(root.event_id);
        removed_envelope.content = removed;
        removed_envelope.content.sequence = 1;
        let removed_ancestry = ancestry.push_checked(removed_envelope);
        assert!(removed_ancestry.is_ok());
        let Ok(removed_ancestry) = removed_ancestry else {
            return;
        };
        let mut unlimited = || Ok(());
        assert_eq!(
            removed_ancestry.no_reintroduction_metered(&root.content, &mut unlimited),
            Ok(Err(TransitionError::DeviceReintroduced))
        );
    }
}
