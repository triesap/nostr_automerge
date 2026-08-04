use super::ActorState;
use crate::automerge_adapter::document::Document;

/// A deterministic UTF-16 Automerge document kept behind the authoring boundary.
pub struct AuthoringDocument {
    pub(crate) document: Document,
    pub(crate) actor_state: ActorState,
}

/// Why an accepted document and explicit actor state cannot initialize authoring.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthoringDocumentError {
    /// Empty initialization was requested with a nonempty accepted frontier.
    NonemptyGenesis,
    /// Accepted Automerge bytes failed strict UTF-16 loading.
    Load,
    /// Loaded Automerge heads do not equal the actor state's accepted frontier.
    Heads,
}

impl AuthoringDocument {
    /// Initializes an empty UTF-16 document and installs the explicit actor.
    pub fn empty(actor_state: ActorState) -> Result<Self, AuthoringDocumentError> {
        if !actor_state.accepted_heads().is_empty() {
            return Err(AuthoringDocumentError::NonemptyGenesis);
        }
        let mut document = Document::new_utf16();
        document.replace_unused_actor(actor_state.actor_id().as_bytes());
        Ok(Self {
            document,
            actor_state,
        })
    }

    /// Loads exact accepted state, verifies its heads, and installs the explicit actor.
    pub fn from_accepted(
        canonical_bytes: &[u8],
        actor_state: ActorState,
    ) -> Result<Self, AuthoringDocumentError> {
        let mut document =
            Document::load_utf16(canonical_bytes).map_err(|_| AuthoringDocumentError::Load)?;
        let heads = document
            .semantic_heads()
            .map_err(|_| AuthoringDocumentError::Load)?;
        if heads != *actor_state.accepted_heads() {
            return Err(AuthoringDocumentError::Heads);
        }
        document.replace_unused_actor(actor_state.actor_id().as_bytes());
        Ok(Self {
            document,
            actor_state,
        })
    }

    /// Returns the caller-owned actor state bound to this document.
    #[must_use]
    pub const fn actor_state(&self) -> &ActorState {
        &self.actor_state
    }

    /// Returns non-compressed state bytes for caller-owned durable continuation.
    ///
    /// These bytes are not a normative protocol digest or cross-language identity.
    #[must_use]
    pub fn accepted_state_bytes(&self) -> Vec<u8> {
        self.document.canonical_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::{AuthoringDocument, AuthoringDocumentError};
    use crate::authoring::ActorState;
    use crate::{ActorId, ChangeHash};
    use std::collections::BTreeSet;

    #[test]
    fn initialize_authoring_document_deterministically() {
        let actor = ActorId::from_bytes([0x42; 32]);
        let first = AuthoringDocument::empty(ActorState::initial(actor, BTreeSet::new()));
        let second = AuthoringDocument::empty(ActorState::initial(actor, BTreeSet::new()));
        assert!(first.is_ok() && second.is_ok());
        let (Ok(first), Ok(second)) = (first, second) else {
            return;
        };
        assert_eq!(first.document.actor_bytes(), actor.as_bytes());
        assert_eq!(
            first.document.canonical_bytes(),
            second.document.canonical_bytes()
        );
        let loaded = AuthoringDocument::from_accepted(
            &first.document.canonical_bytes(),
            ActorState::initial(actor, BTreeSet::new()),
        );
        assert!(loaded.is_ok());
        assert_eq!(
            AuthoringDocument::empty(ActorState::initial(
                actor,
                BTreeSet::from([ChangeHash::from_bytes([1; 32])])
            ))
            .err(),
            Some(AuthoringDocumentError::NonemptyGenesis)
        );
    }
}
