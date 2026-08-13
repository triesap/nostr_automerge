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
            .indexes
            .coordinates
            .events
            .get(&coordinate)
            .cloned()
            .unwrap_or_default();
        let support_event_ids = corpus
            .indexes
            .coordinates
            .lifecycle_support
            .get(&coordinate)
            .cloned()
            .unwrap_or_default();
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
            .coordinates
            .change_hashes
            .get(&self.coordinate)
            .into_iter()
            .flatten()
            .copied()
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

    pub(crate) fn control_count(&self) -> usize {
        self.reportable_event_ids
            .iter()
            .filter(|event_id| {
                matches!(
                    self.corpus.events.get(event_id),
                    Some(EventEvidence::VerifiedCarrier {
                        carrier: VerifiedCarrier::Control(_),
                        ..
                    })
                )
            })
            .count()
    }

    pub(crate) fn change_hash_count(&self) -> usize {
        self.change_hashes().count()
    }

    pub(crate) fn evaluation_event_count(&self) -> usize {
        self.input_event_ids().count().saturating_add(
            self.corpus
                .indexes
                .coordinates
                .duplicates
                .get(&self.coordinate)
                .map_or(0, Vec::len),
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
            .chain(
                self.corpus
                    .indexes
                    .coordinates
                    .duplicates
                    .get(&self.coordinate)
                    .into_iter()
                    .flatten()
                    .filter_map(|index| self.corpus.duplicates.get(*index))
                    .filter_map(|evidence| self.corpus.record_for_duplicate(evidence)),
            )
    }
}
