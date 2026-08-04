use core::fmt;

use crate::crypto::bip340::{self, Bip340Error};
use crate::types::public_key::VerifiedPublicKey;
use crate::wire::nip01::raw::{RawNip01Error, RawNip01Event, parse};
use crate::wire::nip01::tags::Nip01Tags;
use crate::wire::nip01::verify::{EventIdError, verify_declared_event_id};
use crate::wire::strict_json::StrictJsonError;
use crate::{EventId, RawEventBytes};

/// Immutable NIP-01 evidence whose shape, identifier, and signature are verified.
#[derive(Clone, PartialEq)]
pub struct VerifiedNip01Event {
    raw: RawEventBytes,
    id: EventId,
    author: VerifiedPublicKey,
    created_at: u64,
    kind: u16,
    tags: Nip01Tags,
    content: String,
}

impl VerifiedNip01Event {
    /// Performs the complete strict NIP-01 verification pipeline.
    pub fn verify(raw: RawEventBytes) -> Result<Self, Nip01VerificationError> {
        let parsed = parse(&raw).map_err(Nip01VerificationError::from_raw)?;
        verify_declared_event_id(&parsed).map_err(Nip01VerificationError::from_event_id)?;
        bip340::verify(parsed.pubkey, parsed.id, parsed.signature)
            .map_err(Nip01VerificationError::from_bip340)?;
        let RawNip01Event {
            id,
            pubkey: author,
            created_at,
            kind,
            tags,
            content,
            signature: _,
        } = parsed;
        Ok(Self {
            raw,
            id,
            author,
            created_at,
            kind,
            tags,
            content,
        })
    }

    /// Returns the verified event identifier.
    #[must_use]
    pub const fn event_id(&self) -> EventId {
        self.id
    }

    /// Returns the verified x-only author key bytes.
    #[must_use]
    pub const fn author_bytes(&self) -> &[u8; 32] {
        self.author.as_bytes()
    }

    /// Returns the signed timestamp, which is never used for protocol ordering.
    #[must_use]
    pub const fn created_at(&self) -> u64 {
        self.created_at
    }

    /// Returns the signed NIP-01 event kind.
    #[must_use]
    pub const fn kind(&self) -> u16 {
        self.kind
    }

    /// Returns exact validated tag arrays in signed order.
    #[must_use]
    pub fn tags(&self) -> &[Vec<String>] {
        self.tags.as_slice()
    }

    /// Returns exact signed content text.
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Returns the complete original signed event bytes.
    #[must_use]
    pub const fn raw(&self) -> &RawEventBytes {
        &self.raw
    }
}

impl fmt::Debug for VerifiedNip01Event {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VerifiedNip01Event")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .field("raw_length", &self.raw.as_bytes().len())
            .finish_non_exhaustive()
    }
}

/// Stable classification for strict NIP-01 verification failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nip01VerificationError {
    /// JSON is malformed or has trailing data.
    JsonSyntax,
    /// A top-level member name is duplicated.
    DuplicateMember,
    /// Required event or tag shape is invalid.
    Shape,
    /// An identifier or signature encoding is noncanonical.
    Identifier,
    /// Canonical NIP-01 serialization failed.
    Serialization,
    /// The declared EventId differs from the calculated identifier.
    EventIdMismatch,
    /// The x-only public key is not a secp256k1 point.
    InvalidPublicKey,
    /// BIP-340 verification failed.
    InvalidSignature,
}

impl Nip01VerificationError {
    fn from_raw(error: RawNip01Error) -> Self {
        match error {
            RawNip01Error::Json(StrictJsonError::Syntax) => Self::JsonSyntax,
            RawNip01Error::Json(StrictJsonError::DuplicateMember) => Self::DuplicateMember,
            RawNip01Error::Identifier(_) => Self::Identifier,
            RawNip01Error::Tags(_) | RawNip01Error::Shape => Self::Shape,
        }
    }

    fn from_event_id(error: EventIdError) -> Self {
        match error {
            EventIdError::Serialization(_) => Self::Serialization,
            EventIdError::Mismatch => Self::EventIdMismatch,
        }
    }

    fn from_bip340(error: Bip340Error) -> Self {
        match error {
            Bip340Error::InvalidPublicKey => Self::InvalidPublicKey,
            Bip340Error::InvalidSignature => Self::InvalidSignature,
        }
    }
}

impl fmt::Display for Nip01VerificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "NIP-01 verification failed: {self:?}")
    }
}

impl std::error::Error for Nip01VerificationError {}

#[cfg(test)]
mod tests {
    use super::VerifiedNip01Event;
    use crate::{ProtocolRevision, RawEventBytes};

    const VALID: &str = r#"{"id":"70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5","pubkey":"379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe","created_at":1612809991,"kind":1,"tags":[],"content":"test","sig":"273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502"}"#;

    #[test]
    #[allow(clippy::expect_used)]
    fn verifies_and_retains_complete_signed_event() {
        let raw = RawEventBytes::new(VALID.as_bytes(), ProtocolRevision::draft_v1())
            .expect("trusted fixture");
        let verified = VerifiedNip01Event::verify(raw).expect("known signed NIP-01 event");
        assert_eq!(
            verified.event_id().to_hex(),
            "70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5"
        );
        assert_eq!(verified.content(), "test");
        assert_eq!(verified.raw().as_str(), VALID);
    }
}
