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

    pub(crate) fn change_hashes_for_control(
        &self,
        control_id: EventId,
    ) -> Option<&'a BTreeSet<ChangeHash>> {
        self.corpus
            .indexes
            .changes
            .hashes_by_coordinate_control
            .get(&(self.coordinate, control_id))
    }

    pub(crate) fn change_carrier_event_ids(
        &self,
        hash: ChangeHash,
    ) -> Option<&'a BTreeSet<EventId>> {
        self.corpus
            .indexes
            .changes
            .carriers_by_coordinate_hash
            .get(&(self.coordinate, hash))
    }

    pub(crate) fn checkpoint_descriptor_event_ids(&self) -> Option<&'a BTreeSet<EventId>> {
        self.corpus
            .indexes
            .checkpoints
            .descriptors_by_coordinate
            .get(&self.coordinate)
    }

    pub(crate) fn checkpoint_chunk_event_ids(
        &self,
        descriptor_id: EventId,
    ) -> Option<&'a BTreeSet<EventId>> {
        self.corpus
            .indexes
            .checkpoints
            .chunks_by_coordinate_descriptor
            .get(&(self.coordinate, descriptor_id))
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

    pub(crate) fn checkpoint_descriptor_count(&self) -> usize {
        self.work.map_or(0, |work| work.checkpoint_descriptor_count)
    }

    pub(crate) fn checkpoint_chunk_count(&self) -> usize {
        self.work.map_or(0, |work| work.checkpoint_chunk_count)
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

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::DocumentEvidenceView;
    use crate::evidence::corpus_builder::EvidenceCorpus;
    use crate::evidence::indexes::{CoordinateWorkMetadata, TrustedIndexes};
    use crate::{ChangeHash, ControllerPublicKey, DocumentCoordinate, DocumentId, EventId};

    #[test]
    fn exposes_borrowed_coordinate_dependent_membership() {
        let coordinate = DocumentCoordinate::new(
            ControllerPublicKey::from_bytes([1; 32]),
            DocumentId::from_bytes([2; 32]),
        );
        let control_id = EventId::from_bytes([3; 32]);
        let descriptor_id = EventId::from_bytes([4; 32]);
        let chunk_id = EventId::from_bytes([5; 32]);
        let hash = ChangeHash::from_bytes([6; 32]);
        let mut indexes = TrustedIndexes::default();
        indexes
            .changes
            .hashes_by_coordinate_control
            .insert((coordinate, control_id), BTreeSet::from([hash]));
        indexes.changes.carriers_by_coordinate_hash.insert(
            (coordinate, hash),
            BTreeSet::from([EventId::from_bytes([7; 32])]),
        );
        indexes
            .checkpoints
            .descriptors_by_coordinate
            .insert(coordinate, BTreeSet::from([descriptor_id]));
        indexes
            .checkpoints
            .chunks_by_coordinate_descriptor
            .insert((coordinate, descriptor_id), BTreeSet::from([chunk_id]));
        indexes.coordinates.work.insert(
            coordinate,
            CoordinateWorkMetadata {
                checkpoint_descriptor_count: 1,
                checkpoint_chunk_count: 1,
                ..CoordinateWorkMetadata::default()
            },
        );
        let corpus = EvidenceCorpus {
            events: BTreeMap::new(),
            invalid: BTreeMap::new(),
            duplicates: Vec::new(),
            indexes,
        };
        let view = DocumentEvidenceView::derive(&corpus, coordinate);
        assert_eq!(
            view.change_hashes_for_control(control_id),
            Some(&BTreeSet::from([hash]))
        );
        assert_eq!(
            view.change_carrier_event_ids(hash),
            Some(&BTreeSet::from([EventId::from_bytes([7; 32])]))
        );
        assert_eq!(
            view.checkpoint_descriptor_event_ids(),
            Some(&BTreeSet::from([descriptor_id]))
        );
        assert_eq!(
            view.checkpoint_chunk_event_ids(descriptor_id),
            Some(&BTreeSet::from([chunk_id]))
        );
        assert_eq!(view.checkpoint_descriptor_count(), 1);
        assert_eq!(view.checkpoint_chunk_count(), 1);
    }
}
