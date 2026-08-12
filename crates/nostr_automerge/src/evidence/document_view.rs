use std::collections::BTreeSet;

use crate::carrier::VerifiedCarrier;
use crate::evidence::corpus_builder::{EvidenceCorpus, EvidenceRecord, ManifestSelection};
use crate::evidence::event::EventEvidence;
use crate::{ChangeHash, DocumentCoordinate, EventId};

/// Immutable evidence boundary for evaluating exactly one document coordinate.
pub(crate) struct DocumentEvidenceView<'a> {
    corpus: &'a EvidenceCorpus,
    coordinate: DocumentCoordinate,
    reportable_event_ids: BTreeSet<EventId>,
    support_event_ids: BTreeSet<EventId>,
}

impl<'a> DocumentEvidenceView<'a> {
    pub(crate) fn derive(corpus: &'a EvidenceCorpus, coordinate: DocumentCoordinate) -> Self {
        let reportable_event_ids = corpus
            .events
            .iter()
            .filter_map(|(event_id, evidence)| {
                (evidence_coordinate(evidence) == Some(coordinate)).then_some(*event_id)
            })
            .collect::<BTreeSet<_>>();
        let support_event_ids = reportable_event_ids
            .iter()
            .filter_map(|event_id| match corpus.events.get(event_id) {
                Some(EventEvidence::VerifiedCarrier {
                    carrier: VerifiedCarrier::Control(control),
                    ..
                }) if control.parent().is_none() => {
                    control.predecessor().map(|link| link.terminal_control)
                }
                _ => None,
            })
            .filter(|event_id| {
                !reportable_event_ids.contains(event_id) && corpus.events.contains_key(event_id)
            })
            .collect();
        Self {
            corpus,
            coordinate,
            reportable_event_ids,
            support_event_ids,
        }
    }

    pub(crate) const fn corpus(&self) -> &'a EvidenceCorpus {
        self.corpus
    }

    pub(crate) const fn coordinate(&self) -> DocumentCoordinate {
        self.coordinate
    }

    pub(crate) fn reportable_event_ids(&self) -> &BTreeSet<EventId> {
        &self.reportable_event_ids
    }

    pub(crate) fn contains_reportable(&self, event_id: &EventId) -> bool {
        self.reportable_event_ids.contains(event_id)
    }

    pub(crate) fn contains_input(&self, event_id: &EventId) -> bool {
        self.reportable_event_ids.contains(event_id) || self.support_event_ids.contains(event_id)
    }

    pub(crate) fn input_event_ids(&self) -> impl Iterator<Item = EventId> + '_ {
        self.reportable_event_ids
            .iter()
            .chain(&self.support_event_ids)
            .copied()
    }

    pub(crate) fn change_hashes(&self) -> impl Iterator<Item = ChangeHash> + '_ {
        self.corpus
            .indexes
            .changes
            .claims_by_hash
            .iter()
            .filter(|(_, claims)| {
                claims
                    .keys()
                    .any(|event_id| self.reportable_event_ids.contains(event_id))
            })
            .map(|(hash, _)| *hash)
    }

    pub(crate) fn change_claim_event_ids(
        &self,
        hash: ChangeHash,
    ) -> impl Iterator<Item = EventId> + '_ {
        self.corpus
            .indexes
            .changes
            .claims_by_hash
            .get(&hash)
            .into_iter()
            .flat_map(|claims| claims.keys())
            .filter(|event_id| self.reportable_event_ids.contains(event_id))
            .copied()
    }

    pub(crate) fn evaluation_event_count(&self) -> usize {
        self.input_event_ids().count().saturating_add(
            self.corpus
                .duplicates
                .iter()
                .filter(|evidence| match evidence {
                    EventEvidence::DuplicateEvent { event_id, .. } => self.contains_input(event_id),
                    _ => false,
                })
                .count(),
        )
    }

    pub(crate) fn carrier_evidence_count(&self) -> usize {
        self.input_event_ids()
            .filter(|event_id| {
                matches!(
                    self.corpus.events.get(event_id),
                    Some(
                        EventEvidence::VerifiedCarrier { .. }
                            | EventEvidence::InvalidCarrier { .. }
                            | EventEvidence::UnsupportedRevision { .. }
                    )
                )
            })
            .count()
    }

    pub(crate) fn decode_work_bytes(&self) -> Option<u64> {
        let mut seen = BTreeSet::<ChangeHash>::new();
        self.reportable_event_ids
            .iter()
            .try_fold(0_u64, |total, event_id| {
                let Some(EventEvidence::VerifiedCarrier {
                    carrier: VerifiedCarrier::Change(change),
                    ..
                }) = self.corpus.events.get(event_id)
                else {
                    return Some(total);
                };
                if !seen.insert(change.change_hash()) {
                    return Some(total);
                }
                total.checked_add(change.decode_work_bytes()?)
            })
    }

    pub(crate) fn selected_manifest(&self) -> Option<ManifestSelection> {
        self.corpus
            .selected_manifest_selection_in(self.coordinate, &self.reportable_event_ids)
    }

    pub(crate) fn records(&self) -> impl Iterator<Item = EvidenceRecord> + '_ {
        self.reportable_event_ids
            .iter()
            .filter_map(|event_id| self.corpus.record_for_event(event_id))
            .chain(self.corpus.duplicates.iter().filter_map(|evidence| {
                let EventEvidence::DuplicateEvent { event_id, .. } = evidence else {
                    return None;
                };
                self.contains_reportable(event_id)
                    .then(|| self.corpus.record_for_duplicate(evidence))
                    .flatten()
            }))
    }
}

fn evidence_coordinate(evidence: &EventEvidence) -> Option<DocumentCoordinate> {
    match evidence {
        EventEvidence::VerifiedCarrier { carrier, .. } => match carrier {
            VerifiedCarrier::Manifest(value) => Some(value.coordinate()),
            VerifiedCarrier::Control(value) => Some(value.coordinate()),
            VerifiedCarrier::Change(value) => Some(value.coordinate()),
            VerifiedCarrier::CheckpointDescriptor(value) => Some(value.coordinate()),
            VerifiedCarrier::CheckpointChunk(value) => Some(value.coordinate()),
            VerifiedCarrier::UnsupportedRevision { event, .. } => signed_coordinate(event),
        },
        EventEvidence::InvalidCarrier { event, .. }
        | EventEvidence::IrrelevantEvent { event, .. } => signed_coordinate(event),
        EventEvidence::UnsupportedRevision {
            carrier: VerifiedCarrier::UnsupportedRevision { event, .. },
            ..
        } => signed_coordinate(event),
        EventEvidence::UnsupportedRevision { .. }
        | EventEvidence::InvalidEvent { .. }
        | EventEvidence::DuplicateEvent { .. } => None,
    }
}

fn signed_coordinate(event: &crate::VerifiedNip01Event) -> Option<DocumentCoordinate> {
    if event.kind() == 31_624 {
        return super::corpus_builder::manifest_coordinate(event);
    }
    if !matches!(event.kind(), 1_624..=1_627) {
        return None;
    }
    distinct_valid_tag_value(event.tags(), "a")
}

fn distinct_valid_tag_value(tags: &[Vec<String>], name: &str) -> Option<DocumentCoordinate> {
    let values = tags
        .iter()
        .filter(|tag| tag.first().is_some_and(|value| value == name))
        .filter_map(|tag| tag.get(1)?.parse().ok())
        .collect::<BTreeSet<_>>();
    (values.len() == 1)
        .then(|| values.into_iter().next())
        .flatten()
}
