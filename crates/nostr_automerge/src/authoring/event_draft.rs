use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{DevicePublicKey, EventId};

const SAFE_INTEGER_MAX: u64 = 9_007_199_254_740_991;

/// Caller-timestamped protocol content without key custody or signature policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsignedEventDraft {
    created_at: u64,
    kind: u16,
    tags: Vec<Vec<String>>,
    content: String,
}

/// A canonical NIP-01 signing payload and its already-computed event identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedEvent {
    public_key: DevicePublicKey,
    event_id: EventId,
    preimage: Vec<u8>,
    draft: UnsignedEventDraft,
}

/// Why an unsigned draft or its NIP-01 preimage could not be constructed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnsignedEventDraftError {
    /// NIP-01 safe-integer or tag shape requirements were violated.
    Shape,
    /// Canonical JSON serialization failed.
    Serialization,
}

impl UnsignedEventDraft {
    /// Records exact caller-selected carrier fields without signing or clock access.
    pub fn new(
        created_at: u64,
        kind: u16,
        tags: Vec<Vec<String>>,
        content: String,
    ) -> Result<Self, UnsignedEventDraftError> {
        if created_at > SAFE_INTEGER_MAX || tags.iter().any(Vec::is_empty) {
            return Err(UnsignedEventDraftError::Shape);
        }
        Ok(Self {
            created_at,
            kind,
            tags,
            content,
        })
    }

    /// Prepares the exact NIP-01 array to hash and sign for an explicit public key.
    pub fn prepare(
        self,
        public_key: DevicePublicKey,
    ) -> Result<PreparedEvent, UnsignedEventDraftError> {
        let public_key_hex = public_key.to_hex();
        let tuple = (
            0_u8,
            public_key_hex.as_str(),
            self.created_at,
            self.kind,
            self.tags.as_slice(),
            self.content.as_str(),
        );
        let mut preimage = Vec::new();
        tuple
            .serialize(&mut serde_json::Serializer::new(&mut preimage))
            .map_err(|_| UnsignedEventDraftError::Serialization)?;
        let event_id = EventId::from_bytes(Sha256::digest(&preimage).into());
        Ok(PreparedEvent {
            public_key,
            event_id,
            preimage,
            draft: self,
        })
    }
}

impl PreparedEvent {
    /// Returns the event ID the signature must cover.
    #[must_use]
    pub const fn event_id(&self) -> EventId {
        self.event_id
    }
    /// Returns the exact canonical NIP-01 hashing preimage.
    #[must_use]
    pub fn preimage(&self) -> &[u8] {
        &self.preimage
    }
    /// Returns the explicit signing public key.
    #[must_use]
    pub const fn public_key(&self) -> DevicePublicKey {
        self.public_key
    }
    /// Returns the caller-selected timestamp.
    #[must_use]
    pub const fn created_at(&self) -> u64 {
        self.draft.created_at
    }
    /// Returns the carrier kind.
    #[must_use]
    pub const fn kind(&self) -> u16 {
        self.draft.kind
    }
    /// Returns exact signed-order tags.
    #[must_use]
    pub fn tags(&self) -> &[Vec<String>] {
        &self.draft.tags
    }
    /// Returns exact content.
    #[must_use]
    pub fn content(&self) -> &str {
        &self.draft.content
    }
}

#[cfg(test)]
mod tests {
    use super::{UnsignedEventDraft, UnsignedEventDraftError};
    use crate::DevicePublicKey;

    #[test]
    fn create_unsigned_nip_01_carrier_drafts() {
        let draft = UnsignedEventDraft::new(
            7,
            1_624,
            vec![vec!["a".to_owned(), "coordinate".to_owned()]],
            "line\n☃".to_owned(),
        );
        assert!(draft.is_ok());
        let Ok(draft) = draft else { return };
        let prepared = draft.prepare(DevicePublicKey::from_bytes([0x11; 32]));
        assert!(prepared.is_ok());
        let Ok(prepared) = prepared else { return };
        let expected = format!(
            r#"[0,"{}",7,1624,[["a","coordinate"]],"line\n☃"]"#,
            "11".repeat(32)
        );
        assert_eq!(prepared.preimage(), expected.as_bytes());
        assert_eq!(
            UnsignedEventDraft::new(9_007_199_254_740_992, 1, vec![], String::new()),
            Err(UnsignedEventDraftError::Shape)
        );
    }
}
