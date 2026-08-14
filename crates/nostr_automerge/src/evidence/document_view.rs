use std::collections::BTreeSet;

use crate::evidence::corpus_builder::{EvidenceCorpus, EvidenceRecord, ManifestSelection};
use crate::evidence::indexes::CoordinateWorkMetadata;
use crate::{ChangeHash, DocumentCoordinate, EventId};

/// Immutable evidence boundary for evaluating exactly one document coordinate.
pub(crate) struct DocumentEvidenceView<'a> {
    corpus: &'a EvidenceCorpus,
    coordinate: DocumentCoordinate,
    reportable_event_ids: Option<&'a BTreeSet<EventId>>,
    support_event_ids: Option<&'a BTreeSet<EventId>>,
    work: Option<&'a CoordinateWorkMetadata>,
}

impl<'a> DocumentEvidenceView<'a> {
    pub(crate) fn derive(corpus: &'a EvidenceCorpus, coordinate: DocumentCoordinate) -> Self {
        let reportable_event_ids = corpus.indexes.coordinates.events.get(&coordinate);
        let support_event_ids = corpus
            .indexes
            .coordinates
            .lifecycle_support
            .get(&coordinate);
        let work = corpus.indexes.coordinates.work.get(&coordinate);
        Self {
            corpus,
            coordinate,
            reportable_event_ids,
            support_event_ids,
            work,
        }
    }

    pub(crate) const fn corpus(&self) -> &'a EvidenceCorpus {
        self.corpus
    }

    pub(crate) const fn coordinate(&self) -> DocumentCoordinate {
        self.coordinate
    }

    pub(crate) fn reportable_event_ids(&self) -> impl Iterator<Item = &EventId> {
        self.reportable_event_ids.into_iter().flatten()
    }

    pub(crate) fn contains_reportable(&self, event_id: &EventId) -> bool {
        self.reportable_event_ids
            .is_some_and(|events| events.contains(event_id))
    }

    pub(crate) fn contains_input(&self, event_id: &EventId) -> bool {
        self.reportable_event_ids
            .is_some_and(|events| events.contains(event_id))
            || self
                .support_event_ids
                .is_some_and(|events| events.contains(event_id))
    }

    pub(crate) fn input_event_ids(&self) -> impl Iterator<Item = EventId> + '_ {
        self.reportable_event_ids
            .into_iter()
            .flatten()
            .chain(self.support_event_ids.into_iter().flatten())
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
            .prior_claims_by_coordinate
            .get(&self.coordinate)
            .into_iter()
            .flat_map(move |claims| claims.get(&hash))
            .flatten()
            .copied()
    }

    pub(crate) fn control_count(&self) -> usize {
        self.work.map_or(0, |work| work.control_count)
    }

    pub(crate) fn change_hash_count(&self) -> usize {
        self.work.map_or(0, |work| work.change_hash_count)
    }

    pub(crate) fn evaluation_event_count(&self) -> usize {
        self.work.map_or(0, |work| work.evaluation_event_count)
    }

    pub(crate) fn carrier_evidence_count(&self) -> usize {
        self.work.map_or(0, |work| work.carrier_evidence_count)
    }

    pub(crate) fn decode_work_bytes(&self) -> Option<u64> {
        self.work.map_or(Some(0), |work| work.decode_work_bytes)
    }

    pub(crate) fn selected_manifest(&self) -> Option<ManifestSelection> {
        self.corpus
            .indexes
            .coordinates
            .manifests
            .get(&self.coordinate)
            .and_then(|event_ids| {
                self.corpus
                    .selected_manifest_selection_in(self.coordinate, event_ids)
            })
    }

    pub(crate) fn records(&self) -> impl Iterator<Item = EvidenceRecord> + '_ {
        self.reportable_event_ids
            .into_iter()
            .flatten()
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
