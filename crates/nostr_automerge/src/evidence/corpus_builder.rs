use std::collections::BTreeMap;

use crate::carrier::VerifiedCarrier;
use crate::carrier::classify::classify;
use crate::evidence::event::{EventEvidence, RawChecksum};
use crate::{DiagnosticCode, EventId, Nip01VerificationError, RawEventBytes, VerifiedNip01Event};

#[derive(Default)]
pub(crate) struct CorpusBuilder {
    events: BTreeMap<EventId, EventEvidence>,
    invalid: BTreeMap<RawChecksum, EventEvidence>,
    duplicates: Vec<EventEvidence>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BuiltCorpus {
    pub(crate) events: BTreeMap<EventId, EventEvidence>,
    pub(crate) invalid: BTreeMap<RawChecksum, EventEvidence>,
    pub(crate) duplicates: Vec<EventEvidence>,
}

impl CorpusBuilder {
    pub(crate) fn ingest(&mut self, raw: RawEventBytes) {
        let checksum = RawChecksum::of(&raw);
        match VerifiedNip01Event::verify(raw.clone()) {
            Ok(event) => self.ingest_verified(event, checksum),
            Err(error) => {
                self.invalid
                    .entry(checksum)
                    .or_insert_with(|| EventEvidence::InvalidEvent {
                        raw,
                        raw_checksum: checksum,
                        diagnostic: verification_diagnostic(error),
                    });
            }
        }
    }

    fn ingest_verified(&mut self, event: VerifiedNip01Event, checksum: RawChecksum) {
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
            return;
        }
        let evidence = match classify(event.clone()) {
            Some(carrier @ VerifiedCarrier::UnsupportedRevision { .. }) => {
                EventEvidence::UnsupportedRevision {
                    carrier,
                    raw_checksum: checksum,
                    diagnostic: DiagnosticCode::registered("carrier.revision"),
                }
            }
            Some(carrier) => EventEvidence::VerifiedCarrier {
                carrier,
                raw_checksum: checksum,
            },
            None => EventEvidence::IrrelevantEvent {
                event,
                raw_checksum: checksum,
            },
        };
        self.events.insert(event_id, evidence);
    }

    pub(crate) fn finish(self) -> BuiltCorpus {
        BuiltCorpus {
            events: self.events,
            invalid: self.invalid,
            duplicates: self.duplicates,
        }
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
    use super::CorpusBuilder;
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
        first.ingest(second_invalid.clone());
        first.ingest(valid.clone());
        first.ingest(first_invalid.clone());
        first.ingest(valid.clone());
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
