/// The only commit metadata admitted by the draft-v1 authoring profile.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CommitMetadata(());

/// Caller-supplied commit metadata differed from the sealed profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommitMetadataError;

impl CommitMetadata {
    /// Returns time zero, no message, and no extra-byte metadata.
    #[must_use]
    pub const fn canonical() -> Self {
        Self(())
    }

    /// Validates externally restored metadata without normalizing it.
    pub fn validate(
        time: i64,
        message: Option<&str>,
        extra_bytes: &[u8],
    ) -> Result<Self, CommitMetadataError> {
        if time != 0 || message.is_some() || !extra_bytes.is_empty() {
            return Err(CommitMetadataError);
        }
        Ok(Self::canonical())
    }

    /// Returns the fixed Automerge commit time.
    #[must_use]
    pub const fn time(self) -> i64 {
        0
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{CommitMetadata, CommitMetadataError};
    use crate::authoring::{ActorState, AuthoringDocument};
    use crate::automerge_adapter::decode::decode_change;
    use crate::{ActorId, ProtocolRevision};

    #[test]
    fn fix_commit_metadata() {
        assert_eq!(
            CommitMetadata::validate(0, None, &[]),
            Ok(CommitMetadata::canonical())
        );
        assert_eq!(
            CommitMetadata::validate(1, None, &[]),
            Err(CommitMetadataError)
        );
        assert_eq!(
            CommitMetadata::validate(0, Some("message"), &[]),
            Err(CommitMetadataError)
        );
        assert_eq!(
            CommitMetadata::validate(0, None, &[1]),
            Err(CommitMetadataError)
        );

        let document = AuthoringDocument::empty(ActorState::initial(
            ActorId::from_bytes([1; 32]),
            BTreeSet::new(),
        ));
        assert!(document.is_ok());
        let Ok(mut document) = document else { return };
        let raw = document.document.author_test_change();
        assert!(raw.is_some());
        let Some(raw) = raw else { return };
        let decoded = decode_change(&raw, ProtocolRevision::draft_v1());
        assert!(
            matches!(decoded, Ok(change) if change.time == 0 && change.message.is_none() && change.extra_bytes.is_empty())
        );
    }
}
