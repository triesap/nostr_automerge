use crate::ChangeHash;
use crate::automerge_adapter::document::{AdapterAuthoringError, AuthoringOperation};

use super::{ActorState, AuthoringDocument};

/// One caller-selected local operation in an explicit authored batch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operation {
    /// Assigns a root-map string property.
    PutString {
        /// Root-map property name.
        key: String,
        /// Assigned scalar value.
        value: String,
    },
    /// Creates a root-map list populated with string values.
    CreateList {
        /// Root-map property name.
        key: String,
        /// String elements inserted in order.
        values: Vec<String>,
    },
    /// Creates a root-map UTF-16 text object.
    CreateText {
        /// Root-map property name.
        key: String,
        /// Complete initial UTF-16 text value.
        value: String,
    },
    /// Creates and optionally increments a root-map counter.
    CreateCounter {
        /// Root-map property name.
        key: String,
        /// Initial counter value.
        value: i64,
        /// Increment applied in the same transaction.
        increment: i64,
    },
}

/// Canonical raw Automerge change bytes and their semantic hash.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthoredChange {
    raw: Vec<u8>,
    change_hash: ChangeHash,
    previous_state: ActorState,
    new_state: ActorState,
}

impl AuthoredChange {
    pub(crate) fn from_adapter(
        change: crate::automerge_adapter::document::AdapterAuthoredChange,
        previous_state: ActorState,
        new_state: ActorState,
    ) -> Self {
        Self {
            raw: change.raw,
            change_hash: change.hash,
            previous_state,
            new_state,
        }
    }
    /// Returns the canonical uncompressed change bytes.
    #[must_use]
    pub fn raw(&self) -> &[u8] {
        &self.raw
    }
    /// Returns the full Automerge change hash.
    #[must_use]
    pub const fn change_hash(&self) -> ChangeHash {
        self.change_hash
    }
    /// Returns the durable state consumed by this atomic transition.
    #[must_use]
    pub const fn previous_state(&self) -> &ActorState {
        &self.previous_state
    }
    /// Returns the durable state that must be used for the next change.
    #[must_use]
    pub const fn new_state(&self) -> &ActorState {
        &self.new_state
    }
}

/// Why a requested local operation batch was not authored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthoringError {
    /// No operation would produce a change.
    Empty,
    /// Automerge rejected an otherwise bounded operation.
    Operation,
    /// A sealed byte, dependency, or operation limit would be exceeded.
    Limit,
    /// Explicit actor counters could not advance without overflow.
    State,
    /// The document no longer matches the frontier bound into actor state.
    Stale,
}

impl AuthoringDocument {
    /// Applies all operations atomically and commits exactly one canonical change.
    pub fn author_change(
        &mut self,
        operations: &[Operation],
    ) -> Result<AuthoredChange, AuthoringError> {
        let operations = operations
            .iter()
            .cloned()
            .map(Into::into)
            .collect::<Vec<_>>();
        let previous_state = self.actor_state().clone();
        if self
            .document
            .semantic_heads()
            .map_err(|_| AuthoringError::Stale)?
            != *previous_state.accepted_heads()
        {
            return Err(AuthoringError::Stale);
        }
        previous_state
            .next_sequence()
            .checked_add(1)
            .ok_or(AuthoringError::State)?;
        let mut staged = self.document.clone();
        let authored = staged
            .author_operations(&operations)
            .map_err(|error| match error {
                AdapterAuthoringError::Empty => AuthoringError::Empty,
                AdapterAuthoringError::Limit => AuthoringError::Limit,
                AdapterAuthoringError::Operation
                | AdapterAuthoringError::Missing
                | AdapterAuthoringError::Hash => AuthoringError::Operation,
            })?;
        let new_state = previous_state
            .transition(authored.hash, authored.operation_count)
            .map_err(|_| AuthoringError::State)?;
        self.document = staged;
        self.actor_state = new_state.clone();
        Ok(AuthoredChange::from_adapter(
            authored,
            previous_state,
            new_state,
        ))
    }
}

impl From<Operation> for AuthoringOperation {
    fn from(value: Operation) -> Self {
        match value {
            Operation::PutString { key, value } => Self::PutString { key, value },
            Operation::CreateList { key, values } => Self::CreateList { key, values },
            Operation::CreateText { key, value } => Self::CreateText { key, value },
            Operation::CreateCounter {
                key,
                value,
                increment,
            } => Self::CreateCounter {
                key,
                value,
                increment,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AuthoringError, Operation};
    use crate::authoring::{ActorState, AuthoringDocument};
    use crate::automerge_adapter::decode::decode_change;
    use crate::{ActorId, ChangeHash, ProtocolRevision};
    use std::collections::BTreeSet;

    #[test]
    fn create_canonical_operation_bearing_changes() {
        let create = || {
            AuthoringDocument::empty(ActorState::initial(
                ActorId::from_bytes([1; 32]),
                BTreeSet::new(),
            ))
        };
        let first = create();
        let second = create();
        assert!(first.is_ok() && second.is_ok());
        let (Ok(mut first), Ok(mut second)) = (first, second) else {
            return;
        };
        let operations = vec![
            Operation::PutString {
                key: "map".to_owned(),
                value: "value".to_owned(),
            },
            Operation::CreateList {
                key: "list".to_owned(),
                values: vec!["a".to_owned(), "b".to_owned()],
            },
            Operation::CreateText {
                key: "text".to_owned(),
                value: "🙂".to_owned(),
            },
            Operation::CreateCounter {
                key: "counter".to_owned(),
                value: 1,
                increment: 2,
            },
        ];
        let left = first.author_change(&operations);
        let right = second.author_change(&operations);
        assert_eq!(left, right);
        let Ok(left) = left else { return };
        let decoded = decode_change(left.raw(), ProtocolRevision::draft_v1());
        assert!(matches!(decoded, Ok(change) if !change.operations.is_empty()));
        assert_eq!(first.author_change(&[]), Err(AuthoringError::Empty));
        let limits = ProtocolRevision::draft_v1().limits();
        let too_many = vec![
            Operation::PutString {
                key: "k".to_owned(),
                value: "v".to_owned()
            };
            usize::try_from(limits.change_operations.get())
                .unwrap_or(0)
                .saturating_add(1)
        ];
        assert_eq!(first.author_change(&too_many), Err(AuthoringError::Limit));
    }

    #[test]
    fn return_checked_actor_state_transitions() {
        let actor = ActorId::from_bytes([9; 32]);
        let state = ActorState::initial(actor, BTreeSet::new());
        let document = AuthoringDocument::empty(state.clone());
        assert!(document.is_ok());
        let Ok(mut document) = document else { return };
        let result = document.author_change(&[Operation::PutString {
            key: "k".to_owned(),
            value: "v".to_owned(),
        }]);
        assert!(result.is_ok());
        let Ok(result) = result else { return };
        assert_eq!(result.previous_state(), &state);
        assert_eq!(result.new_state(), document.actor_state());
        assert_eq!(result.new_state().next_sequence(), 2);
        assert!(result.new_state().next_operation() > 1);
        assert_eq!(
            result.new_state().accepted_heads(),
            &BTreeSet::from([result.change_hash()])
        );

        let before = document.actor_state().clone();
        assert_eq!(document.author_change(&[]), Err(AuthoringError::Empty));
        assert_eq!(document.actor_state(), &before);

        let overflow = ActorState::restore(
            actor,
            u64::MAX - 1,
            1,
            BTreeSet::new(),
            Some(result.change_hash()),
        );
        assert!(overflow.is_ok());
        let Ok(overflow) = overflow else { return };
        let mut overflow_document = AuthoringDocument::empty(overflow);
        assert!(overflow_document.is_ok());
        let Ok(ref mut overflow_document) = overflow_document else {
            return;
        };
        assert!(
            overflow_document
                .author_change(&[Operation::PutString {
                    key: "first".into(),
                    value: "v".into()
                }])
                .is_ok()
        );
        let before = overflow_document.actor_state().clone();
        assert_eq!(
            overflow_document.author_change(&[Operation::PutString {
                key: "second".into(),
                value: "v".into()
            }]),
            Err(AuthoringError::State)
        );
        assert_eq!(overflow_document.actor_state(), &before);
    }

    #[test]
    fn guard_against_stale_out_of_order_actor_state() {
        let actor = ActorId::from_bytes([7; 32]);
        let original = ActorState::initial(actor, BTreeSet::new());
        let document = AuthoringDocument::empty(original.clone());
        assert!(document.is_ok());
        let Ok(mut document) = document else { return };
        let authored = document.author_change(&[Operation::PutString {
            key: "k".into(),
            value: "v".into(),
        }]);
        assert!(authored.is_ok());
        let Ok(authored) = authored else { return };
        let bytes = document.accepted_state_bytes();

        assert_eq!(
            AuthoringDocument::from_accepted(&bytes, original).err(),
            Some(crate::authoring::AuthoringDocumentError::Heads)
        );
        let changed_heads = ActorState::restore(
            actor,
            authored.new_state().next_sequence(),
            authored.new_state().next_operation(),
            BTreeSet::from([ChangeHash::from_bytes([8; 32])]),
            Some(authored.change_hash()),
        );
        assert!(changed_heads.is_ok());
        let Ok(changed_heads) = changed_heads else {
            return;
        };
        assert_eq!(
            AuthoringDocument::from_accepted(&bytes, changed_heads).err(),
            Some(crate::authoring::AuthoringDocumentError::Heads)
        );
        assert!(AuthoringDocument::from_accepted(&bytes, authored.new_state().clone()).is_ok());
    }
}
