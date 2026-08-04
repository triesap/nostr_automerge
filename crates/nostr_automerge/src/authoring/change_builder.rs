use super::{AuthoredChange, AuthoringDocument, AuthoringError, Operation};

/// An explicit caller-controlled operation coalescing boundary.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChangeBuilder {
    operations: Vec<Operation>,
}

impl ChangeBuilder {
    /// Starts an empty operation batch without timers or background policy.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    /// Appends one local operation to the pending batch.
    pub fn push(&mut self, operation: Operation) {
        self.operations.push(operation)
    }

    /// Returns the number of caller-selected operations awaiting commit.
    #[must_use]
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// Returns true when committing would be an accidental empty transaction.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Commits the complete batch as exactly one canonical Automerge change.
    pub fn commit(
        self,
        document: &mut AuthoringDocument,
    ) -> Result<AuthoredChange, AuthoringError> {
        document.author_change(&self.operations)
    }

    /// Discards the batch without touching the document or actor state.
    pub fn abort(self) {}
}

#[cfg(test)]
mod tests {
    use super::ChangeBuilder;
    use crate::authoring::{ActorState, AuthoringDocument, Operation};
    use crate::automerge_adapter::decode::decode_change;
    use crate::{ActorId, ProtocolRevision};
    use std::collections::BTreeSet;

    #[test]
    fn support_edit_coalescing_boundaries() {
        let state = ActorState::initial(ActorId::from_bytes([1; 32]), BTreeSet::new());
        let document = AuthoringDocument::empty(state.clone());
        assert!(document.is_ok());
        let Ok(mut document) = document else { return };
        let before = document.accepted_state_bytes();
        ChangeBuilder::new().abort();
        assert_eq!(document.actor_state(), &state);
        assert_eq!(document.accepted_state_bytes(), before);

        let mut builder = ChangeBuilder::new();
        builder.push(Operation::PutString {
            key: "first".to_owned(),
            value: "one".to_owned(),
        });
        builder.push(Operation::PutString {
            key: "second".to_owned(),
            value: "two".to_owned(),
        });
        assert_eq!(builder.len(), 2);
        let authored = builder.commit(&mut document);
        assert!(authored.is_ok());
        let Ok(authored) = authored else { return };
        let decoded = decode_change(authored.raw(), ProtocolRevision::draft_v1());
        assert!(matches!(decoded, Ok(change) if change.operations.len() == 2));
    }
}
