use std::collections::BTreeMap;

use crate::evidence::corpus_builder::BuiltCorpus;
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

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EvidenceCorpus {
    pub(crate) ingress: BuiltCorpus,
    pub(crate) controls: ControlIndexes,
    pub(crate) changes: ChangeIndexes,
    pub(crate) invalid_carriers: BTreeMap<EventId, InvalidCarrierEvidence>,
}

impl EvidenceCorpus {
    pub(crate) fn build(
        ingress: BuiltCorpus,
        controls: impl IntoIterator<Item = ControlIndexRecord>,
        changes: impl IntoIterator<Item = ChangeIndexRecord>,
        invalid_carriers: impl IntoIterator<Item = InvalidCarrierEvidence>,
    ) -> Self {
        let invalid_carriers: BTreeMap<_, _> = invalid_carriers
            .into_iter()
            .map(|evidence| (evidence.event_id, evidence))
            .collect();
        let controls = controls.into_iter().map(|mut record| {
            if invalid_carriers.contains_key(&record.event_id) {
                record.validity = IndexValidity::Invalid;
            }
            record
        });
        let changes = changes.into_iter().map(|mut record| {
            if invalid_carriers.contains_key(&record.event_id) {
                record.validity = IndexValidity::Invalid;
            }
            record
        });
        Self {
            ingress,
            controls: index_controls(controls),
            changes: index_changes(changes),
            invalid_carriers,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{EvidenceCorpus, InvalidCarrierEvidence};
    use crate::evidence::corpus_builder::BuiltCorpus;
    use crate::evidence::indexes::{ChangeIndexRecord, ControlIndexRecord, IndexValidity};
    use crate::{ActorId, ChangeHash, DiagnosticCode, EventId};

    #[test]
    fn represent_invalid_evidence_without_state_poisoning() {
        let valid_control = EventId::from_bytes([1; 32]);
        let invalid_control = EventId::from_bytes([2; 32]);
        let valid_change = EventId::from_bytes([3; 32]);
        let invalid_change = EventId::from_bytes([4; 32]);
        let hash = ChangeHash::from_bytes([5; 32]);
        let ingress = BuiltCorpus {
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
}
