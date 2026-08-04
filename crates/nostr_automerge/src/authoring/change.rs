use crate::ChangeHash;
use crate::automerge_adapter::document::{AdapterAuthoringError, AuthoringOperation};

use super::AuthoringDocument;

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
}

impl AuthoredChange {
    pub(crate) fn from_adapter(
        change: crate::automerge_adapter::document::AdapterAuthoredChange,
    ) -> Self {
        Self {
            raw: change.raw,
            change_hash: change.hash,
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
        self.document
            .author_operations(&operations)
            .map(AuthoredChange::from_adapter)
            .map_err(|error| match error {
                AdapterAuthoringError::Empty => AuthoringError::Empty,
                AdapterAuthoringError::Limit => AuthoringError::Limit,
                AdapterAuthoringError::Operation
                | AdapterAuthoringError::Missing
                | AdapterAuthoringError::Hash => AuthoringError::Operation,
            })
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
    use crate::{ActorId, ProtocolRevision};
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
}
