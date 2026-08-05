use core::fmt;
use std::collections::BTreeSet;

use crate::{
    ChangeHash, CheckpointVerificationResult, Completion, DispositionsDigest, DocumentCoordinate,
    EventId, EvidenceRecord, HistoryDigest, IntegrityAlert, ProtocolDisposition,
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

/// Immutable materialized document state owned by the reference engine.
#[derive(Clone, PartialEq, Eq)]
pub struct MaterializedDocumentView {
    canonical_bytes: Vec<u8>,
}

impl MaterializedDocumentView {
    pub(crate) fn from_canonical_bytes(canonical_bytes: Vec<u8>) -> Self {
        Self { canonical_bytes }
    }

    /// Returns the size of the materialized canonical Automerge state.
    #[must_use]
    pub fn byte_len(&self) -> usize {
        self.canonical_bytes.len()
    }

    /// Returns true when the materialized state has an empty byte encoding.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.canonical_bytes.is_empty()
    }
}

impl fmt::Debug for MaterializedDocumentView {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MaterializedDocumentView")
            .field("byte_len", &self.byte_len())
            .finish()
    }
}

/// Canonical owned result of one deterministic reference evaluation.
#[derive(Clone, PartialEq, Eq)]
pub struct EvaluationReport {
    coordinate: DocumentCoordinate,
    canonical_controls: Vec<EventId>,
    dispositions: Vec<(ChangeHash, ProtocolDisposition)>,
    accepted_changes: Vec<ChangeHash>,
    pending_changes: Vec<ChangeHash>,
    excluded_changes: Vec<ChangeHash>,
    heads: Vec<ChangeHash>,
    evidence: Vec<EvidenceRecord>,
    checkpoints: Vec<CheckpointVerificationResult>,
    history_digest: HistoryDigest,
    dispositions_digest: DispositionsDigest,
    integrity_alerts: Vec<IntegrityAlert>,
    completion: Completion,
    failure: Option<EvaluationFailure>,
    document: Option<MaterializedDocumentView>,
}

pub(crate) struct EvaluationReportParts {
    pub(crate) coordinate: DocumentCoordinate,
    pub(crate) canonical_controls: Vec<EventId>,
    pub(crate) dispositions: Vec<(ChangeHash, ProtocolDisposition)>,
    pub(crate) accepted_changes: Vec<ChangeHash>,
    pub(crate) pending_changes: Vec<ChangeHash>,
    pub(crate) excluded_changes: Vec<ChangeHash>,
    pub(crate) heads: Vec<ChangeHash>,
    pub(crate) evidence: Vec<EvidenceRecord>,
    pub(crate) checkpoints: Vec<CheckpointVerificationResult>,
    pub(crate) history_digest: HistoryDigest,
    pub(crate) dispositions_digest: DispositionsDigest,
    pub(crate) integrity_alerts: Vec<IntegrityAlert>,
    pub(crate) completion: Completion,
    pub(crate) failure: Option<EvaluationFailure>,
    pub(crate) document: Option<MaterializedDocumentView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EvaluationReportInvariant;

impl EvaluationReport {
    pub(crate) fn from_parts(
        parts: EvaluationReportParts,
    ) -> Result<Self, EvaluationReportInvariant> {
        if parts
            .canonical_controls
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            != parts.canonical_controls.len()
            || !strictly_sorted(&parts.accepted_changes)
            || !strictly_sorted(&parts.pending_changes)
            || !strictly_sorted(&parts.excluded_changes)
            || !strictly_sorted(&parts.heads)
            || !parts
                .dispositions
                .windows(2)
                .all(|pair| pair[0].0 < pair[1].0)
            || !parts
                .checkpoints
                .windows(2)
                .all(|pair| pair[0].descriptor_event() < pair[1].descriptor_event())
        {
            return Err(EvaluationReportInvariant);
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
        if !accepted.is_disjoint(&pending)
            || !accepted.is_disjoint(&excluded)
            || !pending.is_disjoint(&excluded)
            || !parts.heads.iter().all(|head| accepted.contains(head))
        {
            return Err(EvaluationReportInvariant);
        }
        let completion_matches_failure = matches!(
            (parts.completion, parts.failure),
            (Completion::Complete, None)
                | (
                    Completion::BudgetExhausted,
                    Some(EvaluationFailure::BudgetExhausted)
                )
                | (Completion::Cancelled, Some(EvaluationFailure::Cancelled))
                | (
                    Completion::Failed,
                    Some(
                        EvaluationFailure::InvalidEvidence
                            | EvaluationFailure::Graph
                            | EvaluationFailure::Decode
                            | EvaluationFailure::Apply
                            | EvaluationFailure::InvariantViolation
                    )
                )
        );
        if !completion_matches_failure {
            return Err(EvaluationReportInvariant);
        }
        if (parts.completion == Completion::Complete) != parts.document.is_some() {
            return Err(EvaluationReportInvariant);
        }
        Ok(Self {
            coordinate: parts.coordinate,
            canonical_controls: parts.canonical_controls,
            dispositions: parts.dispositions,
            accepted_changes: parts.accepted_changes,
            pending_changes: parts.pending_changes,
            excluded_changes: parts.excluded_changes,
            heads: parts.heads,
            evidence: parts.evidence,
            checkpoints: parts.checkpoints,
            history_digest: parts.history_digest,
            dispositions_digest: parts.dispositions_digest,
            integrity_alerts: parts.integrity_alerts,
            completion: parts.completion,
            failure: parts.failure,
            document: parts.document,
        })
    }

    pub(crate) fn from_canonical_parts(parts: EvaluationReportParts) -> Self {
        match Self::from_parts(parts) {
            Ok(report) => report,
            Err(_) => unreachable!("reference evaluator produced a non-canonical report"),
        }
    }

    /// Returns the evaluated document coordinate.
    #[must_use]
    pub const fn coordinate(&self) -> DocumentCoordinate {
        self.coordinate
    }

    /// Returns the canonical control chain in evaluation order.
    #[must_use]
    pub fn canonical_controls(&self) -> &[EventId] {
        &self.canonical_controls
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

impl fmt::Debug for EvaluationReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EvaluationReport")
            .field("canonical_control_count", &self.canonical_controls.len())
            .field("accepted_change_count", &self.accepted_changes.len())
            .field("pending_change_count", &self.pending_changes.len())
            .field("excluded_change_count", &self.excluded_changes.len())
            .field("head_count", &self.heads.len())
            .field("evidence_count", &self.evidence.len())
            .field("checkpoint_count", &self.checkpoints.len())
            .field("alert_count", &self.integrity_alerts.len())
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
    use super::{EvaluationReport, EvaluationReportParts, MaterializedDocumentView};
    use crate::{
        ChangeHash, Completion, DispositionsDigest, DocumentCoordinate, EventId, HistoryDigest,
    };

    #[test]
    fn evaluation_report_api_enforces_ordering_and_redacts_document() {
        let coordinate =
            format!("31624:{}:{}", "11".repeat(32), "22".repeat(32)).parse::<DocumentCoordinate>();
        assert!(coordinate.is_ok());
        let Ok(coordinate) = coordinate else { return };
        let parts = || EvaluationReportParts {
            coordinate,
            canonical_controls: vec![EventId::from_bytes([1; 32])],
            dispositions: vec![],
            accepted_changes: vec![ChangeHash::from_bytes([2; 32])],
            pending_changes: vec![],
            excluded_changes: vec![],
            heads: vec![ChangeHash::from_bytes([2; 32])],
            evidence: vec![],
            checkpoints: vec![],
            history_digest: HistoryDigest::from_bytes([3; 32]),
            dispositions_digest: DispositionsDigest::from_bytes([4; 32]),
            integrity_alerts: vec![],
            completion: Completion::Complete,
            failure: None,
            document: Some(MaterializedDocumentView::from_canonical_bytes(vec![
                9, 8, 7,
            ])),
        };
        let report = EvaluationReport::from_parts(parts());
        assert!(report.is_ok());
        let Ok(report) = report else { return };
        assert_eq!(report.accepted_changes(), report.heads());
        let debug = format!("{report:?}");
        assert!(debug.contains("has_document: true"));
        assert!(!debug.contains("9, 8, 7"));

        let mut invalid = parts();
        invalid.accepted_changes = vec![
            ChangeHash::from_bytes([2; 32]),
            ChangeHash::from_bytes([1; 32]),
        ];
        assert!(EvaluationReport::from_parts(invalid).is_err());

        let mut missing_document = parts();
        missing_document.document = None;
        assert!(EvaluationReport::from_parts(missing_document).is_err());

        let mut incomplete_with_document = parts();
        incomplete_with_document.completion = Completion::Cancelled;
        incomplete_with_document.failure = Some(super::EvaluationFailure::Cancelled);
        assert!(EvaluationReport::from_parts(incomplete_with_document).is_err());
    }
}
