use std::collections::BTreeSet;

use crate::{ActorId, ChangeHash};

/// Durable caller-owned state required to author the next change for one actor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActorState {
    actor_id: ActorId,
    next_sequence: u64,
    next_operation: u64,
    accepted_heads: BTreeSet<ChangeHash>,
    last_authored_change: Option<ChangeHash>,
}

/// Why caller-provided actor state cannot safely author another change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActorStateError {
    /// Automerge actor sequences and operation counters are one-based.
    ZeroCounter,
    /// Restored noninitial state omitted the previous authored change identity.
    MissingPreviousChange,
    /// A checked transition would exceed a counter's range.
    CounterOverflow,
}

impl ActorState {
    /// Creates fresh state for an actor against an explicit accepted frontier.
    #[must_use]
    pub fn initial(actor_id: ActorId, accepted_heads: BTreeSet<ChangeHash>) -> Self {
        Self {
            actor_id,
            next_sequence: 1,
            next_operation: 1,
            accepted_heads,
            last_authored_change: None,
        }
    }

    /// Restores checked durable state previously returned by authoring.
    pub fn restore(
        actor_id: ActorId,
        next_sequence: u64,
        next_operation: u64,
        accepted_heads: BTreeSet<ChangeHash>,
        last_authored_change: Option<ChangeHash>,
    ) -> Result<Self, ActorStateError> {
        if next_sequence == 0 || next_operation == 0 {
            return Err(ActorStateError::ZeroCounter);
        }
        if next_sequence > 1 && last_authored_change.is_none() {
            return Err(ActorStateError::MissingPreviousChange);
        }
        next_sequence
            .checked_add(1)
            .ok_or(ActorStateError::CounterOverflow)?;
        Ok(Self {
            actor_id,
            next_sequence,
            next_operation,
            accepted_heads,
            last_authored_change,
        })
    }

    /// Returns the derived actor identity.
    #[must_use]
    pub const fn actor_id(&self) -> ActorId {
        self.actor_id
    }
    /// Returns the sequence required on the next authored change.
    #[must_use]
    pub const fn next_sequence(&self) -> u64 {
        self.next_sequence
    }
    /// Returns the first operation counter required by the next nonempty change.
    #[must_use]
    pub const fn next_operation(&self) -> u64 {
        self.next_operation
    }
    /// Returns the accepted frontier bound to this state.
    #[must_use]
    pub fn accepted_heads(&self) -> &BTreeSet<ChangeHash> {
        &self.accepted_heads
    }
    /// Returns the previous authored change, when the actor is not fresh.
    #[must_use]
    pub const fn last_authored_change(&self) -> Option<ChangeHash> {
        self.last_authored_change
    }

    pub(crate) fn transition(
        &self,
        change_hash: ChangeHash,
        operation_count: u64,
    ) -> Result<Self, ActorStateError> {
        Ok(Self {
            actor_id: self.actor_id,
            next_sequence: self
                .next_sequence
                .checked_add(1)
                .ok_or(ActorStateError::CounterOverflow)?,
            next_operation: if operation_count == 0 {
                self.next_operation
            } else {
                self.next_operation
                    .checked_add(operation_count)
                    .ok_or(ActorStateError::CounterOverflow)?
            },
            accepted_heads: BTreeSet::from([change_hash]),
            last_authored_change: Some(change_hash),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{ActorState, ActorStateError};
    use crate::{ActorId, ChangeHash};
    use std::collections::BTreeSet;

    #[test]
    fn add_explicit_actorstate_value() {
        let actor = ActorId::from_bytes([1; 32]);
        let head = ChangeHash::from_bytes([2; 32]);
        let initial = ActorState::initial(actor, BTreeSet::from([head]));
        assert_eq!((initial.next_sequence(), initial.next_operation()), (1, 1));
        assert_eq!(initial.accepted_heads(), &BTreeSet::from([head]));
        let restored = ActorState::restore(actor, 2, 4, BTreeSet::from([head]), Some(head));
        assert_eq!(
            restored.as_ref().map(|state| state.last_authored_change()),
            Ok(Some(head))
        );
        assert_eq!(
            ActorState::restore(actor, 0, 1, BTreeSet::new(), None),
            Err(ActorStateError::ZeroCounter)
        );
        assert_eq!(
            ActorState::restore(actor, 2, 1, BTreeSet::new(), None),
            Err(ActorStateError::MissingPreviousChange)
        );
        assert_eq!(
            ActorState::restore(actor, u64::MAX, 1, BTreeSet::new(), Some(head)),
            Err(ActorStateError::CounterOverflow)
        );
    }
}
