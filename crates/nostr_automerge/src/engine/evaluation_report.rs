use core::fmt;
use std::collections::BTreeSet;

use crate::{
    ChangeHash, CheckpointVerificationResult, Completion, DispositionsDigest, DocumentCoordinate,
    EventId, EvidenceRecord, HistoryDigest, IntegrityAlert, ProtocolDisposition, ProtocolRevision,
    ResolvedManifestAvailability,
};

/// Stable category explaining why an evaluation did not complete.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EvaluationFailure {
    /// Retained evidence violated a known-revision semantic rule.
    InvalidEvidence,
    /// Dependency-graph construction, closure, or scheduling failed.
    Graph,
    /// Automerge change decoding or qualification failed.
    Decode,
    /// Automerge application or document loading failed.
    Apply,
    /// A typed caller-selected work counter was exhausted.
    BudgetExhausted,
    /// The caller requested cooperative cancellation.
    Cancelled,
    /// A repository-owned report or state invariant failed.
    InvariantViolation,
}

/// A noncanonical implementation failure returned outside protocol reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EvaluationError {
    /// Dependency-graph construction, closure, or scheduling failed internally.
    Graph,
    /// Automerge change decoding failed after carrier qualification.
    Decode,
    /// Exact-state Automerge application failed unexpectedly.
    Apply,
    /// Canonical report or accepted-state invariants were inconsistent.
    ReportInvariant,
    /// Immutable materialized document projection failed.
    Projection,
}

impl fmt::Display for EvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Graph => "internal dependency graph failure",
            Self::Decode => "internal Automerge decode failure",
            Self::Apply => "internal Automerge application failure",
            Self::ReportInvariant => "internal canonical report invariant failure",
            Self::Projection => "internal materialized projection failure",
        })
    }
}

impl std::error::Error for EvaluationError {}

use crate::automerge_adapter::materialized_view::MaterializedDocumentView;

pub(crate) const REPORT_INVARIANT_ITEMS: u64 = 8;

/// A canonical identifier whose namespace prevents collisions between protocol item kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ProtocolItemIdentifier {
    /// A signed control event identified by its NIP-01 event identifier.
    ControlEvent(EventId),
    /// An Automerge change identified by its canonical change hash.
    ChangeHash(ChangeHash),
    /// A signed protocol event not represented by a more specific namespace.
    Event(EventId),
}

impl ProtocolItemIdentifier {
    /// Constructs a control-event identifier.
    #[must_use]
    pub const fn control_event(identifier: EventId) -> Self {
        Self::ControlEvent(identifier)
    }

    /// Constructs a generic signed-event identifier.
    #[must_use]
    pub const fn event(identifier: EventId) -> Self {
        Self::Event(identifier)
    }

    /// Returns the underlying canonical 32-byte identifier.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        match self {
            Self::ControlEvent(identifier) | Self::Event(identifier) => identifier.as_bytes(),
            Self::ChangeHash(identifier) => identifier.as_bytes(),
        }
    }
}

impl From<ChangeHash> for ProtocolItemIdentifier {
    fn from(identifier: ChangeHash) -> Self {
        Self::ChangeHash(identifier)
    }
}

/// One canonical dynamic protocol outcome with optional explanatory detail.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DispositionRecord {
    identifier: ProtocolItemIdentifier,
    disposition: ProtocolDisposition,
    diagnostic: Option<crate::DiagnosticCode>,
}

impl DispositionRecord {
    /// Constructs a namespaced disposition record for canonical serialization.
    #[must_use]
    pub const fn new(
        identifier: ProtocolItemIdentifier,
        disposition: ProtocolDisposition,
        diagnostic: Option<crate::DiagnosticCode>,
    ) -> Self {
        Self {
            identifier,
            disposition,
            diagnostic,
        }
    }

    /// Returns the namespaced protocol item identifier.
    #[must_use]
    pub const fn identifier(&self) -> ProtocolItemIdentifier {
        self.identifier
    }

    /// Returns the canonical protocol disposition.
    #[must_use]
    pub const fn disposition(&self) -> ProtocolDisposition {
        self.disposition
    }

    /// Returns an optional stable diagnostic that does not affect digest identity.
    #[must_use]
    pub const fn diagnostic(&self) -> Option<crate::DiagnosticCode> {
        self.diagnostic
    }
}

fn disposition_records_are_canonical(records: &[DispositionRecord]) -> bool {
    records
        .windows(2)
        .all(|pair| pair[0].identifier < pair[1].identifier)
}

/// Canonical owned result of one deterministic reference evaluation.
#[derive(Clone, PartialEq, Eq)]
pub struct EvaluationReport {
    coordinate: DocumentCoordinate,
    revision: ProtocolRevision,
    canonical_controls: Vec<EventId>,
    disposition_records: Vec<DispositionRecord>,
    control_dispositions: Vec<(EventId, ProtocolDisposition)>,
    dispositions: Vec<(ChangeHash, ProtocolDisposition)>,
    accepted_changes: Vec<ChangeHash>,
    pending_changes: Vec<ChangeHash>,
    excluded_changes: Vec<ChangeHash>,
    invalid_changes: Vec<ChangeHash>,
    heads: Vec<ChangeHash>,
    evidence: Vec<EvidenceRecord>,
    checkpoints: Vec<CheckpointVerificationResult>,
    history_digest: HistoryDigest,
    dispositions_digest: DispositionsDigest,
    integrity_alerts: Vec<IntegrityAlert>,
    manifest: ResolvedManifestAvailability,
    completion: Completion,
    failure: Option<EvaluationFailure>,
    document: Option<MaterializedDocumentView>,
}

#[derive(Clone)]
pub(crate) struct EvaluationReportParts {
    pub(crate) coordinate: DocumentCoordinate,
    pub(crate) revision: ProtocolRevision,
    pub(crate) canonical_controls: Vec<EventId>,
    pub(crate) disposition_records: Vec<DispositionRecord>,
    pub(crate) control_dispositions: Vec<(EventId, ProtocolDisposition)>,
    pub(crate) dispositions: Vec<(ChangeHash, ProtocolDisposition)>,
    pub(crate) change_carrier_dispositions: Vec<(EventId, ChangeHash, ProtocolDisposition)>,
    pub(crate) accepted_changes: Vec<ChangeHash>,
    pub(crate) pending_changes: Vec<ChangeHash>,
    pub(crate) excluded_changes: Vec<ChangeHash>,
    pub(crate) invalid_changes: Vec<ChangeHash>,
    pub(crate) heads: Vec<ChangeHash>,
    pub(crate) evidence: Vec<EvidenceRecord>,
    pub(crate) checkpoints: Vec<CheckpointVerificationResult>,
    pub(crate) history_digest: HistoryDigest,
    pub(crate) dispositions_digest: DispositionsDigest,
    pub(crate) integrity_alerts: Vec<IntegrityAlert>,
    pub(crate) manifest: ResolvedManifestAvailability,
    pub(crate) completion: Completion,
    pub(crate) failure: Option<EvaluationFailure>,
    pub(crate) document: Option<MaterializedDocumentView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReportConstructionPath {
    Complete,
    InterruptedBatch,
    NoProgress,
}

impl ReportConstructionPath {
    pub(crate) const ALL: [Self; 3] = [Self::Complete, Self::InterruptedBatch, Self::NoProgress];

    pub(crate) const fn identifier(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::InterruptedBatch => "interrupted_batch",
            Self::NoProgress => "no_progress",
        }
    }
}

impl EvaluationReport {
    pub(crate) fn from_complete_parts(
        parts: EvaluationReportParts,
    ) -> Result<Self, EvaluationError> {
        Self::from_parts(ReportConstructionPath::Complete, parts)
    }

    pub(crate) fn from_interrupted_batch_parts(
        parts: EvaluationReportParts,
    ) -> Result<Self, EvaluationError> {
        Self::from_parts(ReportConstructionPath::InterruptedBatch, parts)
    }

    pub(crate) fn from_no_progress_parts(
        parts: EvaluationReportParts,
    ) -> Result<Self, EvaluationError> {
        Self::from_parts(ReportConstructionPath::NoProgress, parts)
    }

    fn from_parts(
        construction: ReportConstructionPath,
        parts: EvaluationReportParts,
    ) -> Result<Self, EvaluationError> {
        match construction {
            ReportConstructionPath::Complete
            | ReportConstructionPath::InterruptedBatch
            | ReportConstructionPath::NoProgress => {}
        }
        if parts
            .canonical_controls
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            != parts.canonical_controls.len()
            || !strictly_sorted(&parts.accepted_changes)
            || !strictly_sorted(&parts.pending_changes)
            || !strictly_sorted(&parts.excluded_changes)
            || !strictly_sorted(&parts.invalid_changes)
            || !strictly_sorted(&parts.heads)
            || !parts
                .control_dispositions
                .windows(2)
                .all(|pair| pair[0].0 < pair[1].0)
            || !disposition_records_are_canonical(&parts.disposition_records)
            || !parts
                .dispositions
                .windows(2)
                .all(|pair| pair[0].0 < pair[1].0)
            || !parts
                .change_carrier_dispositions
                .windows(2)
                .all(|pair| pair[0].0 < pair[1].0)
            || !parts
                .checkpoints
                .windows(2)
                .all(|pair| pair[0].descriptor_event() < pair[1].descriptor_event())
        {
            return Err(EvaluationError::ReportInvariant);
        }
        if !change_carrier_dispositions_are_consistent(&parts) {
            return Err(EvaluationError::ReportInvariant);
        }
        let accepted = parts
            .accepted_changes
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let pending = parts
            .pending_changes
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let excluded = parts
            .excluded_changes
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let invalid = parts
            .invalid_changes
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        if !accepted.is_disjoint(&pending)
            || !accepted.is_disjoint(&excluded)
            || !accepted.is_disjoint(&invalid)
            || !pending.is_disjoint(&excluded)
            || !pending.is_disjoint(&invalid)
            || !excluded.is_disjoint(&invalid)
            || !parts.heads.iter().all(|head| accepted.contains(head))
        {
            return Err(EvaluationError::ReportInvariant);
        }
        if parts.canonical_controls.iter().any(|control| {
            match parts
                .control_dispositions
                .binary_search_by_key(control, |(event_id, _)| *event_id)
            {
                Ok(index) => {
                    parts.control_dispositions[index].1 != crate::ProtocolDisposition::Accepted
                }
                Err(_) => true,
            }
        }) {
            return Err(EvaluationError::ReportInvariant);
        }
        let event_outcome = |event_id| {
            let identifier = ProtocolItemIdentifier::event(event_id);
            parts
                .disposition_records
                .binary_search_by_key(&identifier, DispositionRecord::identifier)
                .ok()
                .map(|index| {
                    let record = parts.disposition_records[index];
                    (record.disposition(), record.diagnostic())
                })
        };
        let manifest_consistent = match &parts.manifest {
            ResolvedManifestAvailability::Missing => true,
            ResolvedManifestAvailability::Available { hints, .. } => {
                event_outcome(hints.event_id()).map(|(disposition, _)| disposition)
                    == Some(ProtocolDisposition::Accepted)
            }
            ResolvedManifestAvailability::Pending { hints, .. } => {
                event_outcome(hints.event_id()).map(|(disposition, _)| disposition)
                    == Some(ProtocolDisposition::Pending)
            }
            ResolvedManifestAvailability::Unavailable {
                event_id,
                diagnostic,
                ..
            } => {
                event_outcome(*event_id).map(|(disposition, _)| disposition)
                    == Some(if diagnostic.as_str() == "carrier.revision" {
                        ProtocolDisposition::UnsupportedRevision
                    } else {
                        ProtocolDisposition::Invalid
                    })
            }
        };
        let checkpoints_consistent = parts.checkpoints.iter().all(|checkpoint| {
            let expected = checkpoint.status().event_outcome();
            event_outcome(checkpoint.descriptor_event()) == Some(expected)
                && checkpoint
                    .chunk_events()
                    .iter()
                    .all(|event_id| event_outcome(*event_id) == Some(expected))
        });
        if !manifest_consistent || !checkpoints_consistent {
            return Err(EvaluationError::ReportInvariant);
        }
        let completion_matches_failure = matches!(
            (parts.completion, parts.failure),
            (Completion::Complete, None)
                | (
                    Completion::BudgetExhausted,
                    Some(EvaluationFailure::BudgetExhausted)
                )
                | (Completion::Cancelled, Some(EvaluationFailure::Cancelled))
        );
        if !completion_matches_failure {
            return Err(EvaluationError::ReportInvariant);
        }
        if (parts.completion == Completion::Complete) != parts.document.is_some() {
            return Err(EvaluationError::ReportInvariant);
        }
        Ok(Self {
            coordinate: parts.coordinate,
            revision: parts.revision,
            canonical_controls: parts.canonical_controls,
            disposition_records: parts.disposition_records,
            control_dispositions: parts.control_dispositions,
            dispositions: parts.dispositions,
            accepted_changes: parts.accepted_changes,
            pending_changes: parts.pending_changes,
            excluded_changes: parts.excluded_changes,
            invalid_changes: parts.invalid_changes,
            heads: parts.heads,
            evidence: parts.evidence,
            checkpoints: parts.checkpoints,
            history_digest: parts.history_digest,
            dispositions_digest: parts.dispositions_digest,
            integrity_alerts: parts.integrity_alerts,
            manifest: parts.manifest,
            completion: parts.completion,
            failure: parts.failure,
            document: parts.document,
        })
    }

    /// Returns the evaluated document coordinate.
    #[must_use]
    pub const fn coordinate(&self) -> DocumentCoordinate {
        self.coordinate
    }

    /// Returns the sealed protocol revision used for this evaluation.
    #[must_use]
    pub const fn revision(&self) -> ProtocolRevision {
        self.revision
    }

    /// Returns the canonical control chain in evaluation order.
    #[must_use]
    pub fn canonical_controls(&self) -> &[EventId] {
        &self.canonical_controls
    }

    /// Returns all canonical protocol outcomes in namespace and identifier order.
    #[must_use]
    pub fn disposition_records(&self) -> &[DispositionRecord] {
        &self.disposition_records
    }

    /// Returns stateful control dispositions ordered by event identifier.
    #[must_use]
    pub fn control_dispositions(&self) -> &[(EventId, ProtocolDisposition)] {
        &self.control_dispositions
    }

    /// Returns canonical change dispositions ordered by change hash.
    #[must_use]
    pub fn dispositions(&self) -> &[(ChangeHash, ProtocolDisposition)] {
        &self.dispositions
    }

    /// Returns accepted change hashes in canonical byte order.
    #[must_use]
    pub fn accepted_changes(&self) -> &[ChangeHash] {
        &self.accepted_changes
    }

    /// Returns pending change hashes in canonical byte order.
    #[must_use]
    pub fn pending_changes(&self) -> &[ChangeHash] {
        &self.pending_changes
    }

    /// Returns excluded change hashes in canonical byte order.
    #[must_use]
    pub fn excluded_changes(&self) -> &[ChangeHash] {
        &self.excluded_changes
    }

    /// Returns invalid change hashes in canonical byte order.
    #[must_use]
    pub fn invalid_changes(&self) -> &[ChangeHash] {
        &self.invalid_changes
    }

    /// Returns materialized Automerge heads in canonical byte order.
    #[must_use]
    pub fn heads(&self) -> &[ChangeHash] {
        &self.heads
    }

    /// Returns immutable content-free evidence outcomes.
    #[must_use]
    pub fn evidence(&self) -> &[EvidenceRecord] {
        &self.evidence
    }

    /// Returns checkpoint verification results ordered by descriptor event ID.
    #[must_use]
    pub fn checkpoints(&self) -> &[CheckpointVerificationResult] {
        &self.checkpoints
    }

    /// Returns replacement-first advisory manifest availability after dynamic control resolution.
    #[must_use]
    pub const fn manifest(&self) -> &ResolvedManifestAvailability {
        &self.manifest
    }

    /// Returns the normative history digest.
    #[must_use]
    pub const fn history_digest(&self) -> HistoryDigest {
        self.history_digest
    }

    /// Returns the normative dispositions digest.
    #[must_use]
    pub const fn dispositions_digest(&self) -> DispositionsDigest {
        self.dispositions_digest
    }

    /// Returns canonical integrity alerts.
    #[must_use]
    pub fn integrity_alerts(&self) -> &[IntegrityAlert] {
        &self.integrity_alerts
    }

    pub(crate) fn push_integrity_alert(&mut self, alert: IntegrityAlert) {
        if !self.integrity_alerts.contains(&alert) {
            self.integrity_alerts.push(alert);
        }
    }

    /// Returns local evaluation completion without changing protocol dispositions.
    #[must_use]
    pub const fn completion(&self) -> Completion {
        self.completion
    }

    /// Returns the typed reason evaluation did not complete.
    #[must_use]
    pub const fn failure(&self) -> Option<EvaluationFailure> {
        self.failure
    }

    /// Returns the immutable materialized document when evaluation produced one.
    #[must_use]
    pub const fn document(&self) -> Option<&MaterializedDocumentView> {
        self.document.as_ref()
    }
}

fn change_carrier_dispositions_are_consistent(parts: &EvaluationReportParts) -> bool {
    let carrier_records_match =
        parts
            .change_carrier_dispositions
            .iter()
            .all(|(event_id, hash, carrier_disposition)| {
                let event_identifier = ProtocolItemIdentifier::event(*event_id);
                let event_matches = parts
                    .disposition_records
                    .binary_search_by_key(&event_identifier, DispositionRecord::identifier)
                    .is_ok_and(|index| {
                        parts.disposition_records[index].disposition() == *carrier_disposition
                    });
                let hash_matches = parts
                    .dispositions
                    .binary_search_by_key(hash, |(candidate, _)| *candidate)
                    .is_ok_and(|index| {
                        *carrier_disposition != ProtocolDisposition::Accepted
                            || parts.dispositions[index].1 == ProtocolDisposition::Accepted
                    });
                event_matches && hash_matches
            });
    if !carrier_records_match {
        return false;
    }
    parts.completion != Completion::Complete
        || parts
            .dispositions
            .iter()
            .filter(|(_, disposition)| *disposition == ProtocolDisposition::Accepted)
            .all(|(hash, _)| {
                parts
                    .change_carrier_dispositions
                    .iter()
                    .any(|(_, carrier_hash, disposition)| {
                        carrier_hash == hash && *disposition == ProtocolDisposition::Accepted
                    })
            })
}

impl fmt::Debug for EvaluationReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EvaluationReport")
            .field("canonical_control_count", &self.canonical_controls.len())
            .field(
                "control_disposition_count",
                &self.control_dispositions.len(),
            )
            .field("disposition_record_count", &self.disposition_records.len())
            .field("accepted_change_count", &self.accepted_changes.len())
            .field("pending_change_count", &self.pending_changes.len())
            .field("excluded_change_count", &self.excluded_changes.len())
            .field("invalid_change_count", &self.invalid_changes.len())
            .field("head_count", &self.heads.len())
            .field("evidence_count", &self.evidence.len())
            .field("checkpoint_count", &self.checkpoints.len())
            .field("alert_count", &self.integrity_alerts.len())
            .field("manifest", &self.manifest)
            .field("completion", &self.completion)
            .field("failure", &self.failure)
            .field("has_document", &self.document.is_some())
            .finish()
    }
}

fn strictly_sorted<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

#[cfg(test)]
mod tests {
    use super::{
        DispositionRecord, EvaluationReport, EvaluationReportParts, ProtocolItemIdentifier,
        ReportConstructionPath, disposition_records_are_canonical,
    };

    #[test]
    fn report_construction_inventory_is_closed_and_ordered() {
        let identifiers = ReportConstructionPath::ALL.map(ReportConstructionPath::identifier);
        assert_eq!(
            identifiers,
            ["complete", "interrupted_batch", "no_progress"]
        );

        for invalid in [
            &["complete", "interrupted_batch"][..],
            &["complete", "interrupted_batch", "no_progress", "alternate"][..],
            &["interrupted_batch", "complete", "no_progress"][..],
            &["complete", "stale", "no_progress"][..],
        ] {
            assert_ne!(invalid, identifiers);
        }
    }
    use crate::{
        ChangeHash, CheckpointVerificationResult, CheckpointVerificationStatus, Completion,
        DispositionsDigest, DocumentCoordinate, EventId, HistoryDigest, SnapshotHash,
    };

    #[test]
    fn evaluation_report_api_enforces_ordering_and_redacts_document() {
        let coordinate =
            format!("31624:{}:{}", "11".repeat(32), "22".repeat(32)).parse::<DocumentCoordinate>();
        assert!(coordinate.is_ok());
        let Ok(coordinate) = coordinate else { return };
        let parts = || EvaluationReportParts {
            coordinate,
            revision: crate::ProtocolRevision::draft_v1(),
            canonical_controls: vec![EventId::from_bytes([1; 32])],
            disposition_records: vec![],
            control_dispositions: vec![(
                EventId::from_bytes([1; 32]),
                crate::ProtocolDisposition::Accepted,
            )],
            dispositions: vec![],
            change_carrier_dispositions: vec![],
            accepted_changes: vec![ChangeHash::from_bytes([2; 32])],
            pending_changes: vec![],
            excluded_changes: vec![],
            invalid_changes: vec![],
            heads: vec![ChangeHash::from_bytes([2; 32])],
            evidence: vec![],
            checkpoints: vec![],
            history_digest: HistoryDigest::from_bytes([3; 32]),
            dispositions_digest: DispositionsDigest::from_bytes([4; 32]),
            integrity_alerts: vec![],
            manifest: crate::ResolvedManifestAvailability::Missing,
            completion: Completion::Complete,
            failure: None,
            document: None,
        };
        let report = EvaluationReport::from_parts(ReportConstructionPath::Complete, parts());
        assert!(report.is_err());

        let mut invalid = parts();
        invalid.accepted_changes = vec![
            ChangeHash::from_bytes([2; 32]),
            ChangeHash::from_bytes([1; 32]),
        ];
        assert!(EvaluationReport::from_parts(ReportConstructionPath::Complete, invalid).is_err());

        let mut incomplete_with_document = parts();
        incomplete_with_document.completion = Completion::Cancelled;
        incomplete_with_document.failure = Some(super::EvaluationFailure::Cancelled);
        assert!(
            EvaluationReport::from_parts(
                ReportConstructionPath::InterruptedBatch,
                incomplete_with_document,
            )
            .is_ok()
        );
    }

    #[test]
    fn report_invariant_mutations_return_typed_errors() {
        let coordinate =
            format!("31624:{}:{}", "31".repeat(32), "32".repeat(32)).parse::<DocumentCoordinate>();
        assert!(coordinate.is_ok());
        let Ok(coordinate) = coordinate else { return };
        let hash = ChangeHash::from_bytes([2; 32]);
        let parts = || EvaluationReportParts {
            coordinate,
            revision: crate::ProtocolRevision::draft_v1(),
            canonical_controls: vec![EventId::from_bytes([1; 32])],
            disposition_records: vec![],
            control_dispositions: vec![(
                EventId::from_bytes([1; 32]),
                crate::ProtocolDisposition::Accepted,
            )],
            dispositions: vec![(hash, crate::ProtocolDisposition::Accepted)],
            change_carrier_dispositions: vec![],
            accepted_changes: vec![hash],
            pending_changes: vec![],
            excluded_changes: vec![],
            invalid_changes: vec![],
            heads: vec![hash],
            evidence: vec![],
            checkpoints: vec![],
            history_digest: HistoryDigest::from_bytes([3; 32]),
            dispositions_digest: DispositionsDigest::from_bytes([4; 32]),
            integrity_alerts: vec![],
            manifest: crate::ResolvedManifestAvailability::Missing,
            completion: Completion::Cancelled,
            failure: Some(super::EvaluationFailure::Cancelled),
            document: None,
        };
        assert!(
            EvaluationReport::from_parts(ReportConstructionPath::InterruptedBatch, parts()).is_ok()
        );

        let carrier_id = EventId::from_bytes([9; 32]);
        let mut carrier_consistent = parts();
        carrier_consistent.disposition_records = vec![
            DispositionRecord::new(
                ProtocolItemIdentifier::from(hash),
                crate::ProtocolDisposition::Accepted,
                None,
            ),
            DispositionRecord::new(
                ProtocolItemIdentifier::event(carrier_id),
                crate::ProtocolDisposition::Accepted,
                None,
            ),
        ];
        carrier_consistent.change_carrier_dispositions =
            vec![(carrier_id, hash, crate::ProtocolDisposition::Accepted)];
        assert!(
            EvaluationReport::from_parts(
                ReportConstructionPath::InterruptedBatch,
                carrier_consistent.clone(),
            )
            .is_ok()
        );

        let mut missing_carrier_record = carrier_consistent.clone();
        missing_carrier_record.disposition_records.pop();
        assert!(
            EvaluationReport::from_parts(
                ReportConstructionPath::InterruptedBatch,
                missing_carrier_record,
            )
            .is_err()
        );

        let mut duplicate_carrier = carrier_consistent.clone();
        duplicate_carrier.change_carrier_dispositions.push((
            carrier_id,
            hash,
            crate::ProtocolDisposition::Accepted,
        ));
        assert!(
            EvaluationReport::from_parts(
                ReportConstructionPath::InterruptedBatch,
                duplicate_carrier,
            )
            .is_err()
        );

        let mut wrong_carrier_hash = carrier_consistent;
        wrong_carrier_hash.change_carrier_dispositions[0].1 = ChangeHash::from_bytes([8; 32]);
        assert!(
            EvaluationReport::from_parts(
                ReportConstructionPath::InterruptedBatch,
                wrong_carrier_hash,
            )
            .is_err()
        );

        let assert_invariant = |parts| {
            assert_eq!(
                EvaluationReport::from_parts(ReportConstructionPath::InterruptedBatch, parts),
                Err(super::EvaluationError::ReportInvariant)
            );
        };
        let mut duplicate_control = parts();
        duplicate_control
            .canonical_controls
            .push(EventId::from_bytes([1; 32]));
        assert_invariant(duplicate_control);

        let mut duplicate_control_outcome = parts();
        duplicate_control_outcome.control_dispositions.push((
            EventId::from_bytes([1; 32]),
            crate::ProtocolDisposition::Excluded,
        ));
        assert_invariant(duplicate_control_outcome);

        let mut missing_canonical_outcome = parts();
        missing_canonical_outcome.control_dispositions.clear();
        assert_invariant(missing_canonical_outcome);

        let mut contradictory_canonical_outcome = parts();
        contradictory_canonical_outcome.control_dispositions[0].1 =
            crate::ProtocolDisposition::Excluded;
        assert_invariant(contradictory_canonical_outcome);

        let record = DispositionRecord::new(
            ProtocolItemIdentifier::from(hash),
            crate::ProtocolDisposition::Accepted,
            None,
        );
        let mut duplicate_record = parts();
        duplicate_record.disposition_records = vec![record, record];
        assert_invariant(duplicate_record);

        let mut duplicate_disposition = parts();
        duplicate_disposition
            .dispositions
            .push((hash, crate::ProtocolDisposition::Excluded));
        assert_invariant(duplicate_disposition);

        let mut overlapping = parts();
        overlapping.pending_changes.push(hash);
        assert_invariant(overlapping);

        let mut foreign_head = parts();
        foreign_head.heads = vec![ChangeHash::from_bytes([9; 32])];
        assert_invariant(foreign_head);

        let mut completion_mismatch = parts();
        completion_mismatch.failure = None;
        assert_invariant(completion_mismatch);

        let mut document_mismatch = parts();
        document_mismatch.completion = Completion::Complete;
        document_mismatch.failure = None;
        assert_invariant(document_mismatch);

        let descriptor = EventId::from_bytes([10; 32]);
        let chunk = EventId::from_bytes([11; 32]);
        let checkpoint = CheckpointVerificationResult::new(
            descriptor,
            vec![chunk],
            SnapshotHash::from_bytes([12; 32]),
            vec![],
            0,
            [13; 32],
            vec![],
            vec![],
            CheckpointVerificationStatus::Verified,
        );
        let mut inconsistent_checkpoint = parts();
        inconsistent_checkpoint.checkpoints.push(checkpoint.clone());
        inconsistent_checkpoint.disposition_records = vec![
            DispositionRecord::new(
                ProtocolItemIdentifier::event(descriptor),
                crate::ProtocolDisposition::Accepted,
                None,
            ),
            DispositionRecord::new(
                ProtocolItemIdentifier::event(chunk),
                crate::ProtocolDisposition::Pending,
                None,
            ),
        ];
        assert_invariant(inconsistent_checkpoint);

        let mut consistent_checkpoint = parts();
        consistent_checkpoint.checkpoints.push(checkpoint);
        consistent_checkpoint.disposition_records = vec![
            DispositionRecord::new(
                ProtocolItemIdentifier::event(descriptor),
                crate::ProtocolDisposition::Accepted,
                None,
            ),
            DispositionRecord::new(
                ProtocolItemIdentifier::event(chunk),
                crate::ProtocolDisposition::Accepted,
                None,
            ),
        ];
        assert!(
            EvaluationReport::from_parts(
                ReportConstructionPath::InterruptedBatch,
                consistent_checkpoint,
            )
            .is_ok()
        );

        let refused_checkpoint = CheckpointVerificationResult::new(
            descriptor,
            vec![chunk],
            SnapshotHash::from_bytes([12; 32]),
            vec![],
            0,
            [13; 32],
            vec![],
            vec![],
            CheckpointVerificationStatus::Unauthorized,
        );
        let history_diagnostic = crate::DiagnosticCode::lookup("checkpoint.history");
        assert!(history_diagnostic.is_some());
        let mut consistent_refusal = parts();
        consistent_refusal.checkpoints.push(refused_checkpoint);
        consistent_refusal.disposition_records = vec![
            DispositionRecord::new(
                ProtocolItemIdentifier::event(descriptor),
                crate::ProtocolDisposition::Invalid,
                history_diagnostic,
            ),
            DispositionRecord::new(
                ProtocolItemIdentifier::event(chunk),
                crate::ProtocolDisposition::Invalid,
                history_diagnostic,
            ),
        ];
        assert!(
            EvaluationReport::from_parts(
                ReportConstructionPath::InterruptedBatch,
                consistent_refusal.clone(),
            )
            .is_ok()
        );

        let mut missing_descriptor_diagnostic = consistent_refusal.clone();
        missing_descriptor_diagnostic.disposition_records[0] = DispositionRecord::new(
            ProtocolItemIdentifier::event(descriptor),
            crate::ProtocolDisposition::Invalid,
            None,
        );
        assert_invariant(missing_descriptor_diagnostic);

        let mut wrong_chunk_diagnostic = consistent_refusal;
        wrong_chunk_diagnostic.disposition_records[1] = DispositionRecord::new(
            ProtocolItemIdentifier::event(chunk),
            crate::ProtocolDisposition::Invalid,
            crate::DiagnosticCode::lookup("checkpoint.chunk"),
        );
        assert_invariant(wrong_chunk_diagnostic);
    }

    #[test]
    fn disposition_identifier_orders_namespaces_and_redacts_debug() {
        let control = ProtocolItemIdentifier::control_event(EventId::from_bytes([3; 32]));
        let change = ProtocolItemIdentifier::from(ChangeHash::from_bytes([1; 32]));
        let event = ProtocolItemIdentifier::event(EventId::from_bytes([0; 32]));
        assert!(control < change);
        assert!(change < event);
        assert_eq!(change.as_bytes(), &[1; 32]);
        assert_eq!(
            change,
            ProtocolItemIdentifier::ChangeHash(ChangeHash::from_bytes([1; 32]))
        );
        let debug = format!("{control:?} {change:?} {event:?}");
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains("03030303"));
    }

    #[test]
    fn disposition_record_requires_canonical_unique_identifiers() {
        let control = ProtocolItemIdentifier::control_event(EventId::from_bytes([1; 32]));
        let change = ProtocolItemIdentifier::from(ChangeHash::from_bytes([2; 32]));
        let diagnostic = crate::DiagnosticCode::lookup("graph.cycle");
        let first =
            DispositionRecord::new(control, crate::ProtocolDisposition::Accepted, diagnostic);
        let second = DispositionRecord::new(change, crate::ProtocolDisposition::Excluded, None);
        assert_eq!(first.identifier(), control);
        assert_eq!(first.disposition(), crate::ProtocolDisposition::Accepted);
        assert_eq!(first.diagnostic(), diagnostic);
        assert!(disposition_records_are_canonical(&[first, second]));
        assert!(!disposition_records_are_canonical(&[second, first]));
        assert!(!disposition_records_are_canonical(&[first, first]));
    }

    #[test]
    #[ignore = "expected to fail until FINDING_081 closes"]
    #[allow(clippy::bool_assert_comparison)]
    fn finding_081_incomplete_report_rejects_canonical_cross_view_state() {
        let coordinate =
            format!("31624:{}:{}", "41".repeat(32), "42".repeat(32)).parse::<DocumentCoordinate>();
        assert!(coordinate.is_ok());
        let Ok(coordinate) = coordinate else { return };
        let control = EventId::from_bytes([1; 32]);
        let hash = ChangeHash::from_bytes([2; 32]);
        let report = EvaluationReport::from_parts(
            ReportConstructionPath::InterruptedBatch,
            EvaluationReportParts {
                coordinate,
                revision: crate::ProtocolRevision::draft_v1(),
                canonical_controls: vec![control],
                disposition_records: vec![],
                control_dispositions: vec![(control, crate::ProtocolDisposition::Accepted)],
                dispositions: vec![(hash, crate::ProtocolDisposition::Accepted)],
                change_carrier_dispositions: vec![],
                accepted_changes: vec![hash],
                pending_changes: vec![],
                excluded_changes: vec![],
                invalid_changes: vec![],
                heads: vec![hash],
                evidence: vec![],
                checkpoints: vec![],
                history_digest: HistoryDigest::from_bytes([3; 32]),
                dispositions_digest: DispositionsDigest::from_bytes([4; 32]),
                integrity_alerts: vec![],
                manifest: crate::ResolvedManifestAvailability::Missing,
                completion: Completion::Cancelled,
                failure: Some(super::EvaluationFailure::Cancelled),
                document: None,
            },
        );
        assert_eq!(
            report.is_err(),
            true,
            "FINDING_081 reproduced: incomplete report parts accept canonical state and arbitrary digests"
        );
    }
}
