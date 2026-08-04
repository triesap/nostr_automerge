//! Pure deterministic construction of protocol content and unsigned drafts.
//!
//! This boundary accepts explicit semantic inputs and returns values for a
//! caller to persist, sign, and publish. It owns no keys, clocks, storage,
//! network clients, signing services, outboxes, or background work.

mod actor_state;
mod change;
mod commit;
mod document;

pub use actor_state::{ActorState, ActorStateError};
pub use change::{AuthoredChange, AuthoringError, Operation};
pub use commit::{CommitMetadata, CommitMetadataError};
pub use document::{AuthoringDocument, AuthoringDocumentError};

#[cfg(test)]
mod tests {
    #[test]
    fn define_authoring_api_boundary() {
        let source = include_str!("mod.rs");
        assert!(source.contains("Pure deterministic construction"));
        let manifest = include_str!("../../Cargo.toml");
        assert!(!manifest.lines().any(|line| line.starts_with("tokio =")));
        assert!(!manifest.lines().any(|line| line.starts_with("reqwest =")));
    }
}
