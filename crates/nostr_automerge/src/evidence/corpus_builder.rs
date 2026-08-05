use std::collections::BTreeMap;

use crate::carrier::VerifiedCarrier;
use crate::carrier::classify::classify;
use crate::evidence::event::{EventEvidence, RawChecksum};
use crate::evidence::source::AcquiredRawEvent;
use crate::{DiagnosticCode, EventId, Nip01VerificationError, RawEventBytes, VerifiedNip01Event};

/// The stable result of ingesting one raw event into an evidence corpus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IngestOutcome {
    /// A supported, verified protocol carrier was retained.
    Accepted {
        /// The verified carrier event identifier.
        event_id: EventId,
    },
    /// The event identifier was already represented in the corpus.
    Duplicate {
        /// The already-retained event identifier.
        event_id: EventId,
    },
    /// The raw event failed strict NIP-01 verification.
    Invalid {
        /// The stable strict-verification failure code.
        diagnostic: DiagnosticCode,
    },
    /// The signed event is valid but is not a draft-v1 carrier kind.
    Irrelevant {
        /// The valid non-carrier event identifier.
        event_id: EventId,
    },
    /// The carrier declares an unsupported protocol revision.
    UnsupportedRevision {
        /// The verified carrier event identifier.
        event_id: EventId,
        /// The stable unsupported-revision code.
        diagnostic: DiagnosticCode,
    },
}

#[derive(Default)]
/// A deterministic single-use builder for retained raw-event evidence.
pub struct CorpusBuilder {
    events: BTreeMap<EventId, EventEvidence>,
    invalid: BTreeMap<RawChecksum, EventEvidence>,
    duplicates: Vec<EventEvidence>,
}

#[derive(Clone, Debug, PartialEq)]
/// Immutable retained ingress evidence produced by [`CorpusBuilder`].
pub struct IngressCorpus {
    pub(crate) events: BTreeMap<EventId, EventEvidence>,
    pub(crate) invalid: BTreeMap<RawChecksum, EventEvidence>,
    pub(crate) duplicates: Vec<EventEvidence>,
}

impl CorpusBuilder {
    /// Creates an empty deterministic evidence-corpus builder.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            events: BTreeMap::new(),
            invalid: BTreeMap::new(),
            duplicates: Vec::new(),
        }
    }

    /// Validates and ingests exact raw signed-event bytes under draft-v1.
    pub fn ingest_bytes(&mut self, bytes: &[u8]) -> IngestOutcome {
        match RawEventBytes::new(bytes, crate::ProtocolRevision::draft_v1()) {
            Ok(raw) => self.ingest(raw),
            Err(error) => IngestOutcome::Invalid {
                diagnostic: error.diagnostic(),
            },
        }
    }

    pub(crate) fn ingest_acquired(&mut self, acquired: AcquiredRawEvent) -> IngestOutcome {
        self.ingest(acquired.into_raw())
    }

    /// Strictly verifies and deterministically retains one bounded raw event.
    pub fn ingest(&mut self, raw: RawEventBytes) -> IngestOutcome {
        let checksum = RawChecksum::of(&raw);
        match VerifiedNip01Event::verify(raw.clone()) {
            Ok(event) => self.ingest_verified(event, checksum),
            Err(error) => {
                let diagnostic = verification_diagnostic(error);
                self.invalid
                    .entry(checksum)
                    .or_insert_with(|| EventEvidence::InvalidEvent {
                        raw,
                        raw_checksum: checksum,
                        diagnostic,
                    });
                IngestOutcome::Invalid { diagnostic }
            }
        }
    }

    fn ingest_verified(
        &mut self,
        event: VerifiedNip01Event,
        checksum: RawChecksum,
    ) -> IngestOutcome {
        let event_id = event.event_id();
        if let Some(existing) = self.events.get(&event_id) {
            let existing_checksum = evidence_checksum(existing);
            self.duplicates.push(EventEvidence::DuplicateEvent {
                event_id,
                raw_checksum: checksum,
            });
            if existing_checksum != checksum {
                self.duplicates.sort_by_key(evidence_checksum);
            }
            return IngestOutcome::Duplicate { event_id };
        }
        let (evidence, outcome) = match classify(event.clone()) {
            Some(carrier @ VerifiedCarrier::UnsupportedRevision { .. }) => {
                let diagnostic = DiagnosticCode::registered("carrier.revision");
                (
                    EventEvidence::UnsupportedRevision {
                        carrier,
                        raw_checksum: checksum,
                        diagnostic,
                    },
                    IngestOutcome::UnsupportedRevision {
                        event_id,
                        diagnostic,
                    },
                )
            }
            Some(carrier) => (
                EventEvidence::VerifiedCarrier {
                    carrier,
                    raw_checksum: checksum,
                },
                IngestOutcome::Accepted { event_id },
            ),
            None => (
                EventEvidence::IrrelevantEvent {
                    event,
                    raw_checksum: checksum,
                },
                IngestOutcome::Irrelevant { event_id },
            ),
        };
        self.events.insert(event_id, evidence);
        outcome
    }

    /// Consumes the builder and returns immutable retained ingress evidence.
    #[must_use]
    pub fn finish(self) -> IngressCorpus {
        IngressCorpus {
            events: self.events,
            invalid: self.invalid,
            duplicates: self.duplicates,
        }
    }
}

impl IngressCorpus {
    /// Returns the number of uniquely identified verified signed events.
    #[must_use]
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Returns the number of unique invalid raw-event byte strings.
    #[must_use]
    pub fn invalid_count(&self) -> usize {
        self.invalid.len()
    }

    /// Returns the number of duplicate event observations.
    #[must_use]
    pub fn duplicate_count(&self) -> usize {
        self.duplicates.len()
    }

    /// Returns true when no evidence of any class was retained.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty() && self.invalid.is_empty() && self.duplicates.is_empty()
    }
}

fn evidence_checksum(evidence: &EventEvidence) -> RawChecksum {
    match evidence {
        EventEvidence::VerifiedCarrier { raw_checksum, .. }
        | EventEvidence::InvalidEvent { raw_checksum, .. }
        | EventEvidence::UnsupportedRevision { raw_checksum, .. }
        | EventEvidence::IrrelevantEvent { raw_checksum, .. }
        | EventEvidence::DuplicateEvent { raw_checksum, .. } => *raw_checksum,
    }
}

const fn verification_diagnostic(error: Nip01VerificationError) -> DiagnosticCode {
    match error {
        Nip01VerificationError::JsonSyntax => DiagnosticCode::registered("json.syntax"),
        Nip01VerificationError::DuplicateMember => {
            DiagnosticCode::registered("json.duplicate_member")
        }
        Nip01VerificationError::Shape => DiagnosticCode::registered("nip01.shape"),
        Nip01VerificationError::Identifier => DiagnosticCode::registered("nip01.identifier"),
        Nip01VerificationError::Serialization | Nip01VerificationError::EventIdMismatch => {
            DiagnosticCode::registered("nip01.event_id")
        }
        Nip01VerificationError::InvalidPublicKey | Nip01VerificationError::InvalidSignature => {
            DiagnosticCode::registered("nip01.signature")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CorpusBuilder, IngestOutcome};
    use crate::{ProtocolRevision, RawEventBytes};

    fn raw(value: &[u8]) -> Option<RawEventBytes> {
        RawEventBytes::new(value, ProtocolRevision::draft_v1()).ok()
    }

    #[test]
    fn implement_corpusbuilder_idempotent_ingestion() {
        let valid = include_bytes!("../../../../fixtures/v1_draft/nip01/valid_event.json");
        let valid = raw(valid);
        let first_invalid = raw(b"{}");
        let second_invalid = raw(b"[]");
        assert!(valid.is_some() && first_invalid.is_some() && second_invalid.is_some());
        let (valid, first_invalid, second_invalid) = match (valid, first_invalid, second_invalid) {
            (Some(valid), Some(first), Some(second)) => (valid, first, second),
            _ => return,
        };

        let mut first = CorpusBuilder::default();
        assert!(matches!(
            first.ingest(second_invalid.clone()),
            IngestOutcome::Invalid { .. }
        ));
        assert!(matches!(
            first.ingest(valid.clone()),
            IngestOutcome::Irrelevant { .. }
        ));
        assert!(matches!(
            first.ingest(first_invalid.clone()),
            IngestOutcome::Invalid { .. }
        ));
        assert!(matches!(
            first.ingest(valid.clone()),
            IngestOutcome::Duplicate { .. }
        ));
        let first = first.finish();
        assert_eq!(first.events.len(), 1);
        assert_eq!(first.invalid.len(), 2);
        assert_eq!(first.duplicates.len(), 1);

        let mut second = CorpusBuilder::default();
        second.ingest(first_invalid);
        second.ingest(valid.clone());
        second.ingest(second_invalid);
        second.ingest(valid);
        assert_eq!(first, second.finish());
    }
}
