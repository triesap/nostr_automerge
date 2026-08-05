use std::collections::BTreeMap;

use crate::evidence::corpus_builder::EvidenceCorpus as IngressCorpus;
use crate::evidence::event::EventEvidence;
use crate::evidence::indexes::{
    ChangeIndexRecord, ChangeIndexes, ControlIndexRecord, ControlIndexes, IndexValidity,
    index_changes, index_controls,
};
use crate::{DiagnosticCode, EventId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct InvalidCarrierEvidence {
    pub(crate) event_id: EventId,
    pub(crate) diagnostic: DiagnosticCode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UnsupportedRevisionEvidence {
    pub(crate) event_id: EventId,
    pub(crate) declared_version: Option<u64>,
    pub(crate) declared_profile: Option<String>,
    pub(crate) diagnostic: DiagnosticCode,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EvidenceCorpus {
    pub(crate) ingress: IngressCorpus,
    pub(crate) controls: ControlIndexes,
    pub(crate) changes: ChangeIndexes,
    pub(crate) invalid_carriers: BTreeMap<EventId, InvalidCarrierEvidence>,
    pub(crate) unsupported_revisions: BTreeMap<EventId, UnsupportedRevisionEvidence>,
}

impl EvidenceCorpus {
    pub(crate) fn build(
        ingress: IngressCorpus,
        controls: impl IntoIterator<Item = ControlIndexRecord>,
        changes: impl IntoIterator<Item = ChangeIndexRecord>,
        invalid_carriers: impl IntoIterator<Item = InvalidCarrierEvidence>,
    ) -> Self {
        let invalid_carriers: BTreeMap<_, _> = invalid_carriers
            .into_iter()
            .map(|evidence| (evidence.event_id, evidence))
            .collect();
        let unsupported_revisions = unsupported_revisions(&ingress);
        let controls = controls.into_iter().filter_map(|mut record| {
            if unsupported_revisions.contains_key(&record.event_id) {
                return None;
            }
            if invalid_carriers.contains_key(&record.event_id) {
                record.validity = IndexValidity::Invalid;
            }
            Some(record)
        });
        let changes = changes.into_iter().filter_map(|mut record| {
            if unsupported_revisions.contains_key(&record.event_id) {
                return None;
            }
            if invalid_carriers.contains_key(&record.event_id) {
                record.validity = IndexValidity::Invalid;
            }
            Some(record)
        });
        Self {
            ingress,
            controls: index_controls(controls),
            changes: index_changes(changes),
            invalid_carriers,
            unsupported_revisions,
        }
    }

    pub(crate) fn unsupported_report(&self) -> Vec<String> {
        self.unsupported_revisions
            .values()
            .map(|evidence| {
                format!(
                    "{}|v={}|profile={}|{}",
                    evidence.event_id.to_hex(),
                    evidence
                        .declared_version
                        .map_or_else(|| "none".to_owned(), |value| value.to_string()),
                    evidence.declared_profile.as_deref().unwrap_or("none"),
                    evidence.diagnostic.as_str(),
                )
            })
            .collect()
    }
}

fn unsupported_revisions(
    ingress: &IngressCorpus,
) -> BTreeMap<EventId, UnsupportedRevisionEvidence> {
    ingress
        .events
        .iter()
        .filter_map(|(event_id, evidence)| match evidence {
            EventEvidence::UnsupportedRevision {
                carrier:
                    crate::carrier::VerifiedCarrier::UnsupportedRevision {
                        declared_version,
                        declared_profile,
                        ..
                    },
                diagnostic,
                ..
            } => Some((
                *event_id,
                UnsupportedRevisionEvidence {
                    event_id: *event_id,
                    declared_version: *declared_version,
                    declared_profile: declared_profile.clone(),
                    diagnostic: *diagnostic,
                },
            )),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{EvidenceCorpus, InvalidCarrierEvidence};
    use crate::carrier::VerifiedCarrier;
    use crate::evidence::corpus_builder::EvidenceCorpus as IngressCorpus;
    use crate::evidence::event::{EventEvidence, RawChecksum};
    use crate::evidence::indexes::{ChangeIndexRecord, ControlIndexRecord, IndexValidity};
    use crate::{
        ActorId, ChangeHash, DiagnosticCode, EventId, ProtocolRevision, RawEventBytes,
        VerifiedNip01Event,
    };

    #[test]
    fn represent_invalid_evidence_without_state_poisoning() {
        let valid_control = EventId::from_bytes([1; 32]);
        let invalid_control = EventId::from_bytes([2; 32]);
        let valid_change = EventId::from_bytes([3; 32]);
        let invalid_change = EventId::from_bytes([4; 32]);
        let hash = ChangeHash::from_bytes([5; 32]);
        let ingress = IngressCorpus {
            events: BTreeMap::new(),
            invalid: BTreeMap::new(),
            duplicates: Vec::new(),
        };
        let corpus = EvidenceCorpus::build(
            ingress,
            [
                ControlIndexRecord {
                    event_id: valid_control,
                    parent: None,
                    validity: IndexValidity::Valid,
                },
                ControlIndexRecord {
                    event_id: invalid_control,
                    parent: Some(valid_control),
                    validity: IndexValidity::Valid,
                },
            ],
            [
                ChangeIndexRecord {
                    event_id: valid_change,
                    change_hash: hash,
                    control_id: valid_control,
                    actor: ActorId::from_bytes([6; 32]),
                    dependencies: Vec::new(),
                    validity: IndexValidity::Valid,
                },
                ChangeIndexRecord {
                    event_id: invalid_change,
                    change_hash: hash,
                    control_id: invalid_control,
                    actor: ActorId::from_bytes([7; 32]),
                    dependencies: Vec::new(),
                    validity: IndexValidity::Valid,
                },
            ],
            [
                InvalidCarrierEvidence {
                    event_id: invalid_control,
                    diagnostic: DiagnosticCode::registered("control.structure"),
                },
                InvalidCarrierEvidence {
                    event_id: invalid_change,
                    diagnostic: DiagnosticCode::registered("automerge.semantics"),
                },
            ],
        );

        assert_eq!(corpus.invalid_carriers.len(), 2);
        assert!(corpus.controls.invalid.contains(&invalid_control));
        assert!(
            !corpus
                .controls
                .controls_by_id
                .contains_key(&invalid_control)
        );
        assert_eq!(
            corpus.changes.valid_carriers_by_hash.get(&hash),
            Some(&std::collections::BTreeSet::from([valid_change]))
        );
        assert_eq!(
            corpus.changes.invalid_carriers_by_hash.get(&hash),
            Some(&std::collections::BTreeSet::from([invalid_change]))
        );
        assert!(
            !corpus
                .changes
                .hashes_by_control
                .contains_key(&invalid_control)
        );
    }

    #[test]
    fn represent_unsupported_revisions() {
        let raw = RawEventBytes::new(
            include_bytes!("../../../../fixtures/v1_draft/nip01/valid_event.json"),
            ProtocolRevision::draft_v1(),
        );
        assert!(raw.is_ok());
        let raw = match raw {
            Ok(raw) => raw,
            Err(_) => return,
        };
        let checksum = RawChecksum::of(&raw);
        let event = VerifiedNip01Event::verify(raw);
        assert!(event.is_ok());
        let event = match event {
            Ok(event) => event,
            Err(_) => return,
        };
        let event_id = event.event_id();
        let evidence = EventEvidence::UnsupportedRevision {
            carrier: VerifiedCarrier::UnsupportedRevision {
                event,
                declared_version: Some(2),
                declared_profile: Some("automerge-change-v2".to_owned()),
            },
            raw_checksum: checksum,
            diagnostic: DiagnosticCode::registered("carrier.revision"),
        };
        let ingress = IngressCorpus {
            events: BTreeMap::from([(event_id, evidence)]),
            invalid: BTreeMap::new(),
            duplicates: Vec::new(),
        };
        let corpus = EvidenceCorpus::build(
            ingress,
            [ControlIndexRecord {
                event_id,
                parent: None,
                validity: IndexValidity::Valid,
            }],
            [],
            [],
        );

        assert!(!corpus.controls.controls_by_id.contains_key(&event_id));
        assert_eq!(corpus.unsupported_revisions.len(), 1);
        assert_eq!(
            corpus.unsupported_report(),
            vec![format!(
                "{}|v=2|profile=automerge-change-v2|carrier.revision",
                event_id.to_hex()
            )]
        );
    }
}
