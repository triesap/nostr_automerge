use core::fmt;

use sha2::{Digest, Sha256};

use crate::carrier::VerifiedCarrier;
use crate::{DiagnosticCode, EventId, RawEventBytes, VerifiedNip01Event};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct RawChecksum([u8; 32]);

impl RawChecksum {
    pub(crate) fn of(raw: &RawEventBytes) -> Self {
        Self(Sha256::digest(raw.as_bytes()).into())
    }

    pub(crate) const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    #[cfg(test)]
    pub(crate) const fn test_only(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

#[derive(Clone, PartialEq)]
pub(crate) enum EventEvidence {
    VerifiedCarrier {
        carrier: VerifiedCarrier,
        raw_checksum: RawChecksum,
    },
    InvalidEvent {
        raw: RawEventBytes,
        raw_checksum: RawChecksum,
        diagnostic: DiagnosticCode,
    },
    InvalidCarrier {
        event: VerifiedNip01Event,
        raw_checksum: RawChecksum,
        diagnostic: DiagnosticCode,
    },
    UnsupportedRevision {
        carrier: VerifiedCarrier,
        raw_checksum: RawChecksum,
        diagnostic: DiagnosticCode,
    },
    IrrelevantEvent {
        event: VerifiedNip01Event,
        raw_checksum: RawChecksum,
    },
    DuplicateEvent {
        event_id: EventId,
        raw_checksum: RawChecksum,
    },
}

impl fmt::Debug for EventEvidence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (variant, checksum, diagnostic) = match self {
            Self::VerifiedCarrier { raw_checksum, .. } => ("verified_carrier", raw_checksum, None),
            Self::InvalidEvent {
                raw_checksum,
                diagnostic,
                ..
            } => ("invalid_event", raw_checksum, Some(*diagnostic)),
            Self::InvalidCarrier {
                raw_checksum,
                diagnostic,
                ..
            } => ("invalid_carrier", raw_checksum, Some(*diagnostic)),
            Self::UnsupportedRevision {
                raw_checksum,
                diagnostic,
                ..
            } => ("unsupported_revision", raw_checksum, Some(*diagnostic)),
            Self::IrrelevantEvent { raw_checksum, .. } => ("irrelevant_event", raw_checksum, None),
            Self::DuplicateEvent { raw_checksum, .. } => ("duplicate_event", raw_checksum, None),
        };
        formatter
            .debug_struct("EventEvidence")
            .field("variant", &variant)
            .field("raw_checksum_prefix", &&checksum.as_bytes()[..4])
            .field("diagnostic", &diagnostic.map(DiagnosticCode::as_str))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::{EventEvidence, RawChecksum};
    use crate::{DiagnosticCode, EventId, ProtocolRevision, RawEventBytes};

    #[test]
    fn add_immutable_event_evidence_records() {
        let raw = RawEventBytes::new(b"private content", ProtocolRevision::draft_v1());
        assert!(raw.is_ok());
        let raw = match raw {
            Ok(value) => value,
            Err(_) => return,
        };
        let checksum = RawChecksum::of(&raw);
        let code = DiagnosticCode::lookup("nip01.shape");
        assert!(code.is_some());
        let code = match code {
            Some(value) => value,
            None => return,
        };
        let evidence = EventEvidence::InvalidEvent {
            raw: raw.clone(),
            raw_checksum: checksum,
            diagnostic: code,
        };
        let debug = format!("{evidence:?}");
        assert!(debug.contains("invalid_event"));
        assert!(debug.contains("nip01.shape"));
        assert!(!debug.contains("private content"));
        assert!(matches!(
            evidence,
            EventEvidence::InvalidEvent { raw: retained, raw_checksum, diagnostic }
                if retained.as_bytes() == b"private content"
                    && raw_checksum == checksum
                    && diagnostic == code
        ));

        let duplicate = EventEvidence::DuplicateEvent {
            event_id: EventId::from_bytes([7; 32]),
            raw_checksum: checksum,
        };
        assert!(!format!("{duplicate:?}").contains(&"07".repeat(32)));
    }
}
