use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};

#[cfg(test)]
use crate::control::reorganization::ControlChainSummary;
use crate::evidence::document_view::DocumentEvidenceView;
use crate::{
    CanonicalControlReorganizationAlert, ChangeHash, CheckpointVerificationResult, Completion,
    DispositionsDigest, DocumentCoordinate, EventId, EvidenceRecord, HistoryDigest, IntegrityAlert,
    ProtocolDisposition, ProtocolRevision, ResolvedManifestAvailability,
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
pub(crate) struct AttributableCarrierOutcome {
    event_id: EventId,
    change_hash: Option<ChangeHash>,
    disposition: ProtocolDisposition,
    diagnostic: Option<crate::DiagnosticCode>,
}

impl AttributableCarrierOutcome {
    pub(crate) const fn verified_change(
        event_id: EventId,
        change_hash: ChangeHash,
        disposition: ProtocolDisposition,
        diagnostic: Option<crate::DiagnosticCode>,
    ) -> Self {
        Self {
            event_id,
            change_hash: Some(change_hash),
            disposition,
            diagnostic,
        }
    }

    pub(crate) const fn event_only(
        event_id: EventId,
        disposition: ProtocolDisposition,
        diagnostic: Option<crate::DiagnosticCode>,
    ) -> Self {
        Self {
            event_id,
            change_hash: None,
            disposition,
            diagnostic,
        }
    }

    pub(crate) const fn event_id(self) -> EventId {
        self.event_id
    }

    pub(crate) const fn change_hash(self) -> Option<ChangeHash> {
        self.change_hash
    }

    pub(crate) const fn disposition(self) -> ProtocolDisposition {
        self.disposition
    }

    pub(crate) const fn diagnostic(self) -> Option<crate::DiagnosticCode> {
        self.diagnostic
    }
}

#[derive(Clone, Copy)]
pub(crate) struct CompleteReportWitness<'a> {
    control_children: Option<&'a BTreeMap<Option<EventId>, BTreeSet<EventId>>>,
    semantic_dispositions: &'a BTreeMap<ChangeHash, ProtocolDisposition>,
    carrier_outcomes: &'a BTreeMap<EventId, AttributableCarrierOutcome>,
    accepted_changes: Option<&'a BTreeSet<ChangeHash>>,
    heads: Option<&'a BTreeSet<ChangeHash>>,
    source_authority: CompleteReportSourceAuthority<'a>,
    field_authority: CompleteReportFieldAuthority,
}

impl<'a> CompleteReportWitness<'a> {
    pub(crate) const fn new(
        control_children: Option<&'a BTreeMap<Option<EventId>, BTreeSet<EventId>>>,
        semantic_dispositions: &'a BTreeMap<ChangeHash, ProtocolDisposition>,
        carrier_outcomes: &'a BTreeMap<EventId, AttributableCarrierOutcome>,
        accepted_changes: Option<&'a BTreeSet<ChangeHash>>,
        heads: Option<&'a BTreeSet<ChangeHash>>,
        source_authority: CompleteReportSourceAuthority<'a>,
        field_authority: CompleteReportFieldAuthority,
    ) -> Self {
        Self {
            control_children,
            semantic_dispositions,
            carrier_outcomes,
            accepted_changes,
            heads,
            source_authority,
            field_authority,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum CompleteReportSourceAuthority<'a> {
    Engine(&'a DocumentEvidenceView<'a>),
    #[cfg(test)]
    TestEvidence(&'a [EvidenceRecord]),
}

impl CompleteReportSourceAuthority<'_> {
    fn matches(self, parts: &EvaluationReportParts) -> bool {
        match self {
            Self::Engine(view) => {
                parts.evidence.iter().copied().eq(view.records())
                    && parts
                        .checkpoints
                        .iter()
                        .map(CheckpointVerificationResult::descriptor_event)
                        .eq(view
                            .checkpoint_descriptor_event_ids()
                            .into_iter()
                            .flatten()
                            .copied())
                    && parts.checkpoints.iter().all(|checkpoint| {
                        checkpoint.chunk_events().iter().copied().eq(view
                            .checkpoint_chunk_event_ids(checkpoint.descriptor_event())
                            .into_iter()
                            .flatten()
                            .copied())
                    })
                    && resolved_manifest_event_id(&parts.manifest)
                        == view.selected_manifest().map(|selection| selection.event_id)
            }
            #[cfg(test)]
            Self::TestEvidence(evidence) => parts.evidence == evidence,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CompleteReportFieldAuthority {
    evidence: [u8; 32],
    checkpoints: [u8; 32],
    integrity_alerts: [u8; 32],
    manifest: [u8; 32],
    document: [u8; 32],
}

impl CompleteReportFieldAuthority {
    pub(crate) fn derive(
        evidence: &[EvidenceRecord],
        checkpoints: &[CheckpointVerificationResult],
        integrity_alerts: &[IntegrityAlert],
        manifest: &ResolvedManifestAvailability,
        document: Option<&MaterializedDocumentView>,
    ) -> Self {
        Self {
            evidence: evidence_authority(evidence),
            checkpoints: checkpoint_authority(checkpoints),
            integrity_alerts: alert_authority(integrity_alerts),
            manifest: manifest_authority(manifest),
            document: document_authority(document),
        }
    }

    fn matches(self, parts: &EvaluationReportParts) -> bool {
        self == Self::derive(
            &parts.evidence,
            &parts.checkpoints,
            &parts.integrity_alerts,
            &parts.manifest,
            parts.document.as_ref(),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReportConstructionPath {
    Complete,
    NoProgress,
}

impl ReportConstructionPath {
    pub(crate) const ALL: [Self; 2] = [Self::Complete, Self::NoProgress];

    pub(crate) const fn identifier(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::NoProgress => "no_progress",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReevaluationComparisonStage {
    PreviousSummary,
    CurrentSummary,
    Relationship,
    CurrentAlertPrefix,
    FinalConstruction,
}

impl ReevaluationComparisonStage {
    pub(crate) const ALL: [Self; 5] = [
        Self::PreviousSummary,
        Self::CurrentSummary,
        Self::Relationship,
        Self::CurrentAlertPrefix,
        Self::FinalConstruction,
    ];

    pub(crate) const fn index(self) -> usize {
        match self {
            Self::PreviousSummary => 0,
            Self::CurrentSummary => 1,
            Self::Relationship => 2,
            Self::CurrentAlertPrefix => 3,
            Self::FinalConstruction => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReevaluationConstructionError {
    Stopped(Completion),
    Invariant,
}

struct ReevaluationControlSummary<'a> {
    controls: &'a [EventId],
    accepted_at_tip: &'a [ChangeHash],
}

impl EvaluationReport {
    pub(crate) fn from_complete_parts(
        parts: EvaluationReportParts,
        witness: CompleteReportWitness<'_>,
    ) -> Result<Self, EvaluationError> {
        Self::from_parts(ReportConstructionPath::Complete, parts, Some(witness))
    }

    pub(crate) fn from_no_progress_parts(
        parts: EvaluationReportParts,
    ) -> Result<Self, EvaluationError> {
        Self::from_parts(ReportConstructionPath::NoProgress, parts, None)
    }

    pub(crate) fn from_reevaluation(
        mut current: Self,
        previous: &Self,
        mut charge: impl FnMut(ReevaluationComparisonStage) -> Result<(), Completion>,
    ) -> Result<Self, ReevaluationConstructionError> {
        if current.completion != Completion::Complete
            || previous.completion != Completion::Complete
            || current.revision != previous.revision
            || current.coordinate != previous.coordinate
        {
            return Err(ReevaluationConstructionError::Invariant);
        }
        let previous_summary = charged_control_chain_summary(
            previous,
            ReevaluationComparisonStage::PreviousSummary,
            &mut charge,
        )?;
        let current_summary = charged_control_chain_summary(
            &current,
            ReevaluationComparisonStage::CurrentSummary,
            &mut charge,
        )?;
        let reorganization =
            charged_detect_reorganization(&previous_summary, &current_summary, &mut charge)?;
        let current_alerts = core::mem::take(&mut current.integrity_alerts);
        let integrity_alerts =
            charged_reevaluation_alerts(current_alerts, reorganization, &mut charge)?;
        charge(ReevaluationComparisonStage::FinalConstruction)
            .map_err(ReevaluationConstructionError::Stopped)?;
        Ok(Self {
            integrity_alerts,
            ..current
        })
    }

    #[cfg(test)]
    pub(crate) fn control_chain_summary(&self) -> ControlChainSummary {
        let mut changes_by_control = BTreeMap::new();
        if let Some(tip) = self.canonical_controls.last().copied() {
            changes_by_control.insert(tip, self.accepted_changes.iter().copied().collect());
        }
        ControlChainSummary {
            controls: self.canonical_controls.clone(),
            changes_by_control,
        }
    }

    fn from_parts(
        construction: ReportConstructionPath,
        parts: EvaluationReportParts,
        complete_witness: Option<CompleteReportWitness<'_>>,
    ) -> Result<Self, EvaluationError> {
        let completion_matches_construction = matches!(
            (construction, parts.completion),
            (ReportConstructionPath::Complete, Completion::Complete)
                | (
                    ReportConstructionPath::NoProgress,
                    Completion::BudgetExhausted | Completion::Cancelled,
                )
        );
        let completion_matches_failure = matches!(
            (parts.completion, parts.failure),
            (Completion::Complete, None)
                | (
                    Completion::BudgetExhausted,
                    Some(EvaluationFailure::BudgetExhausted)
                )
                | (Completion::Cancelled, Some(EvaluationFailure::Cancelled))
        );
        if !completion_matches_construction || !completion_matches_failure {
            return Err(EvaluationError::ReportInvariant);
        }
        match (parts.completion, complete_witness) {
            (Completion::Complete, Some(witness)) => {
                if !complete_parts_are_canonical(&parts, witness) {
                    return Err(EvaluationError::ReportInvariant);
                }
            }
            (Completion::BudgetExhausted | Completion::Cancelled, None) => {
                if !no_progress_parts_are_canonical(&parts) {
                    return Err(EvaluationError::ReportInvariant);
                }
            }
            (Completion::Complete, None)
            | (Completion::BudgetExhausted | Completion::Cancelled, Some(_)) => {
                return Err(EvaluationError::ReportInvariant);
            }
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
                hints.coordinate() == parts.coordinate
                    && event_outcome(hints.event_id())
                        == Some((ProtocolDisposition::Accepted, None))
            }
            ResolvedManifestAvailability::Pending { hints, .. } => {
                hints.coordinate() == parts.coordinate
                    && event_outcome(hints.event_id()) == Some((ProtocolDisposition::Pending, None))
            }
            ResolvedManifestAvailability::Unavailable {
                event_id,
                diagnostic,
                ..
            } => {
                event_outcome(*event_id)
                    == Some((
                        if diagnostic.as_str() == "carrier.revision" {
                            ProtocolDisposition::UnsupportedRevision
                        } else {
                            ProtocolDisposition::Invalid
                        },
                        Some(*diagnostic),
                    ))
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

fn charge_reevaluation<F>(
    charge: &mut F,
    stage: ReevaluationComparisonStage,
) -> Result<(), ReevaluationConstructionError>
where
    F: FnMut(ReevaluationComparisonStage) -> Result<(), Completion> + ?Sized,
{
    charge(stage).map_err(ReevaluationConstructionError::Stopped)
}

fn charged_control_chain_summary<'a, F>(
    report: &'a EvaluationReport,
    stage: ReevaluationComparisonStage,
    charge: &mut F,
) -> Result<ReevaluationControlSummary<'a>, ReevaluationConstructionError>
where
    F: FnMut(ReevaluationComparisonStage) -> Result<(), Completion> + ?Sized,
{
    charge_reevaluation(charge, stage)?;
    let control_count = report.canonical_controls.len();
    for index in 0..control_count {
        charge_reevaluation(charge, stage)?;
        report
            .canonical_controls
            .get(index)
            .ok_or(ReevaluationConstructionError::Invariant)?;
    }

    charge_reevaluation(charge, stage)?;
    let accepted_count = report.accepted_changes.len();
    if report.canonical_controls.is_empty() && accepted_count != 0 {
        return Err(ReevaluationConstructionError::Invariant);
    }
    for index in 0..accepted_count {
        charge_reevaluation(charge, stage)?;
        report
            .accepted_changes
            .get(index)
            .ok_or(ReevaluationConstructionError::Invariant)?;
    }
    Ok(ReevaluationControlSummary {
        controls: &report.canonical_controls,
        accepted_at_tip: &report.accepted_changes,
    })
}

fn charged_detect_reorganization<F>(
    previous: &ReevaluationControlSummary<'_>,
    current: &ReevaluationControlSummary<'_>,
    charge: &mut F,
) -> Result<Option<IntegrityAlert>, ReevaluationConstructionError>
where
    F: FnMut(ReevaluationComparisonStage) -> Result<(), Completion> + ?Sized,
{
    let stage = ReevaluationComparisonStage::Relationship;
    charge_reevaluation(charge, stage)?;
    let previous_count = previous.controls.len();
    let current_count = current.controls.len();
    if previous_count == 0 || current_count == 0 {
        return Ok(None);
    }

    let shared_count = previous_count.min(current_count);
    let mut common = 0;
    while common < shared_count {
        charge_reevaluation(charge, stage)?;
        let previous_control = previous
            .controls
            .get(common)
            .ok_or(ReevaluationConstructionError::Invariant)?;
        let current_control = current
            .controls
            .get(common)
            .ok_or(ReevaluationConstructionError::Invariant)?;
        if previous_control != current_control {
            break;
        }
        common += 1;
    }
    if common == previous_count && previous_count <= current_count {
        return Ok(None);
    }

    let mut affected = Vec::new();
    if common < previous_count {
        affected = charged_merge_changes(&affected, previous.accepted_at_tip, charge)?;
    }
    if common < current_count {
        affected = charged_merge_changes(&affected, current.accepted_at_tip, charge)?;
    }

    charge_reevaluation(charge, stage)?;
    let previous_tip = previous
        .controls
        .last()
        .copied()
        .ok_or(ReevaluationConstructionError::Invariant)?;
    let current_tip = current
        .controls
        .last()
        .copied()
        .ok_or(ReevaluationConstructionError::Invariant)?;
    let alert =
        charged_canonical_reorganization_alert(previous_tip, current_tip, affected, charge)?;
    Ok(Some(IntegrityAlert::CanonicalControlReorganization(alert)))
}

fn charged_canonical_reorganization_alert<F>(
    previous_tip: EventId,
    current_tip: EventId,
    affected: Vec<ChangeHash>,
    charge: &mut F,
) -> Result<CanonicalControlReorganizationAlert, ReevaluationConstructionError>
where
    F: FnMut(ReevaluationComparisonStage) -> Result<(), Completion> + ?Sized,
{
    charged_canonical_reorganization_alert_with_observer(
        previous_tip,
        current_tip,
        affected,
        charge,
        &mut |_, _| {},
    )
}

fn charged_canonical_reorganization_alert_with_observer<F, O>(
    previous_tip: EventId,
    current_tip: EventId,
    affected: Vec<ChangeHash>,
    charge: &mut F,
    comparison_observer: &mut O,
) -> Result<CanonicalControlReorganizationAlert, ReevaluationConstructionError>
where
    F: FnMut(ReevaluationComparisonStage) -> Result<(), Completion> + ?Sized,
    O: FnMut(usize, std::cmp::Ordering) + ?Sized,
{
    let stage = ReevaluationComparisonStage::Relationship;
    let affected_count = affected.len();
    for index in 1..affected_count {
        charge_reevaluation(charge, stage)?;
        let left = affected
            .get(index.saturating_sub(1))
            .ok_or(ReevaluationConstructionError::Invariant)?;
        let right = affected
            .get(index)
            .ok_or(ReevaluationConstructionError::Invariant)?;
        let ordering = left.cmp(right);
        comparison_observer(index.saturating_sub(1), ordering);
        if ordering != std::cmp::Ordering::Less {
            return Err(ReevaluationConstructionError::Invariant);
        }
    }
    charge_reevaluation(charge, stage)?;
    let tip_ordering = previous_tip.cmp(&current_tip);
    comparison_observer(affected_count, tip_ordering);
    if tip_ordering == std::cmp::Ordering::Equal {
        return Err(ReevaluationConstructionError::Invariant);
    }
    Ok(CanonicalControlReorganizationAlert::from_validated_parts(
        previous_tip,
        current_tip,
        affected,
    ))
}

fn charged_merge_changes<F>(
    left: &[ChangeHash],
    right: &[ChangeHash],
    charge: &mut F,
) -> Result<Vec<ChangeHash>, ReevaluationConstructionError>
where
    F: FnMut(ReevaluationComparisonStage) -> Result<(), Completion> + ?Sized,
{
    let stage = ReevaluationComparisonStage::Relationship;
    charge_reevaluation(charge, stage)?;
    let left_count = left.len();
    let right_count = right.len();
    let mut left_index = 0;
    let mut right_index = 0;
    let mut left_value = None;
    let mut right_value = None;
    let mut merged = Vec::new();

    while left_index < left_count
        || right_index < right_count
        || left_value.is_some()
        || right_value.is_some()
    {
        if left_value.is_none() && left_index < left_count {
            charge_reevaluation(charge, stage)?;
            left_value = left.get(left_index).copied();
            if left_value.is_none() {
                return Err(ReevaluationConstructionError::Invariant);
            }
            left_index += 1;
        }
        if right_value.is_none() && right_index < right_count {
            charge_reevaluation(charge, stage)?;
            right_value = right.get(right_index).copied();
            if right_value.is_none() {
                return Err(ReevaluationConstructionError::Invariant);
            }
            right_index += 1;
        }

        charge_reevaluation(charge, stage)?;
        let value = match (left_value, right_value) {
            (Some(left), Some(right)) => match left.cmp(&right) {
                std::cmp::Ordering::Less => {
                    left_value = None;
                    left
                }
                std::cmp::Ordering::Greater => {
                    right_value = None;
                    right
                }
                std::cmp::Ordering::Equal => {
                    left_value = None;
                    right_value = None;
                    left
                }
            },
            (Some(left), None) => {
                left_value = None;
                left
            }
            (None, Some(right)) => {
                right_value = None;
                right
            }
            (None, None) => return Err(ReevaluationConstructionError::Invariant),
        };
        charge_reevaluation(charge, stage)?;
        merged.push(value);
    }
    Ok(merged)
}

fn charged_reevaluation_alerts<F>(
    mut current: Vec<IntegrityAlert>,
    reorganization: Option<IntegrityAlert>,
    charge: &mut F,
) -> Result<Vec<IntegrityAlert>, ReevaluationConstructionError>
where
    F: FnMut(ReevaluationComparisonStage) -> Result<(), Completion> + ?Sized,
{
    let stage = ReevaluationComparisonStage::CurrentAlertPrefix;
    charge_reevaluation(charge, stage)?;
    let alert_count = current.len();
    for index in 0..alert_count {
        charge_reevaluation(charge, stage)?;
        current
            .get(index)
            .ok_or(ReevaluationConstructionError::Invariant)?;
    }
    if let Some(reorganization) = reorganization {
        charge_reevaluation(charge, stage)?;
        current.push(reorganization);
    }
    Ok(current)
}

fn no_progress_parts_are_canonical(parts: &EvaluationReportParts) -> bool {
    let Ok(history_digest) =
        crate::canonical_history_digest(parts.revision, parts.coordinate, &[], &[], &[])
    else {
        return false;
    };
    let Ok(dispositions_digest) =
        crate::canonical_dispositions_digest(parts.revision, parts.coordinate, &[])
    else {
        return false;
    };
    parts.canonical_controls.is_empty()
        && parts.disposition_records.is_empty()
        && parts.control_dispositions.is_empty()
        && parts.dispositions.is_empty()
        && parts.change_carrier_dispositions.is_empty()
        && parts.accepted_changes.is_empty()
        && parts.pending_changes.is_empty()
        && parts.excluded_changes.is_empty()
        && parts.invalid_changes.is_empty()
        && parts.heads.is_empty()
        && parts.evidence.is_empty()
        && parts.checkpoints.is_empty()
        && parts.integrity_alerts.is_empty()
        && matches!(parts.manifest, ResolvedManifestAvailability::Missing)
        && parts.document.is_none()
        && parts.history_digest == history_digest
        && parts.dispositions_digest == dispositions_digest
}

fn authority_hasher(domain: &[u8]) -> Sha256 {
    let mut hasher = Sha256::new();
    hasher.update(b"nostr-automerge-report-authority-v1\0");
    hasher.update((domain.len() as u128).to_be_bytes());
    hasher.update(domain);
    hasher
}

fn authority_bytes(hasher: Sha256) -> [u8; 32] {
    hasher.finalize().into()
}

fn hash_len(hasher: &mut Sha256, len: usize) {
    hasher.update((len as u128).to_be_bytes());
}

fn hash_slice(hasher: &mut Sha256, bytes: &[u8]) {
    hash_len(hasher, bytes.len());
    hasher.update(bytes);
}

fn hash_diagnostic(hasher: &mut Sha256, diagnostic: Option<crate::DiagnosticCode>) {
    match diagnostic {
        Some(diagnostic) => {
            hasher.update([1]);
            hash_slice(hasher, diagnostic.as_str().as_bytes());
        }
        None => hasher.update([0]),
    }
}

fn evidence_authority(records: &[EvidenceRecord]) -> [u8; 32] {
    let mut hasher = authority_hasher(b"evidence");
    hash_len(&mut hasher, records.len());
    for record in records {
        match record.identifier() {
            crate::EvidenceIdentifier::Event(event_id) => {
                hasher.update([0]);
                hasher.update(event_id.as_bytes());
            }
            crate::EvidenceIdentifier::InvalidRawSha256(checksum) => {
                hasher.update([1]);
                hasher.update(checksum);
            }
        }
        hasher.update([evidence_status_code(record.status())]);
        hash_diagnostic(&mut hasher, record.diagnostic());
    }
    authority_bytes(hasher)
}

fn checkpoint_authority(checkpoints: &[CheckpointVerificationResult]) -> [u8; 32] {
    let mut hasher = authority_hasher(b"checkpoints");
    hash_len(&mut hasher, checkpoints.len());
    for checkpoint in checkpoints {
        hasher.update(checkpoint.descriptor_event().as_bytes());
        hash_len(&mut hasher, checkpoint.chunk_events().len());
        for event_id in checkpoint.chunk_events() {
            hasher.update(event_id.as_bytes());
        }
        hasher.update(checkpoint.snapshot_hash().as_bytes());
        hash_len(&mut hasher, checkpoint.heads().len());
        for hash in checkpoint.heads() {
            hasher.update(hash.as_bytes());
        }
        hasher.update(checkpoint.change_count().to_be_bytes());
        hasher.update(checkpoint.change_set_hash());
        hash_len(&mut hasher, checkpoint.historical_carriers().len());
        for hash in checkpoint.historical_carriers() {
            hasher.update(hash.as_bytes());
        }
        hash_len(&mut hasher, checkpoint.accepted_at_control().len());
        for hash in checkpoint.accepted_at_control() {
            hasher.update(hash.as_bytes());
        }
        hasher.update([checkpoint_status_code(checkpoint.status())]);
    }
    authority_bytes(hasher)
}

fn alert_authority(alerts: &[IntegrityAlert]) -> [u8; 32] {
    let mut hasher = authority_hasher(b"integrity-alerts");
    hash_len(&mut hasher, alerts.len());
    for alert in alerts {
        match alert {
            IntegrityAlert::ControllerEquivocation(alert) => {
                hasher.update([0]);
                match alert.parent_control() {
                    Some(parent) => {
                        hasher.update([1]);
                        hasher.update(parent.as_bytes());
                    }
                    None => hasher.update([0]),
                }
                hash_len(&mut hasher, alert.candidate_controls().len());
                for event_id in alert.candidate_controls() {
                    hasher.update(event_id.as_bytes());
                }
                hasher.update(alert.selected_control().as_bytes());
            }
            IntegrityAlert::CanonicalControlReorganization(alert) => {
                hasher.update([1]);
                hasher.update(alert.previous_tip().as_bytes());
                hasher.update(alert.new_tip().as_bytes());
                hash_len(&mut hasher, alert.affected_changes().len());
                for hash in alert.affected_changes() {
                    hasher.update(hash.as_bytes());
                }
            }
            IntegrityAlert::DeviceEquivocation(alert) => {
                hasher.update([2]);
                hasher.update(alert.actor_id().as_bytes());
                hasher.update(alert.first_sequence().to_be_bytes());
                hash_len(&mut hasher, alert.conflicting_changes().len());
                for hash in alert.conflicting_changes() {
                    hasher.update(hash.as_bytes());
                }
                hash_len(&mut hasher, alert.affected_descendants().len());
                for hash in alert.affected_descendants() {
                    hasher.update(hash.as_bytes());
                }
            }
            IntegrityAlert::PotentialClonedDeviceKey(alert) => {
                hasher.update([3]);
                hasher.update(alert.actor_id().as_bytes());
                hasher.update(alert.first_sequence().to_be_bytes());
                hash_len(&mut hasher, alert.carrier_event_ids().len());
                for event_id in alert.carrier_event_ids() {
                    hasher.update(event_id.as_bytes());
                }
            }
            IntegrityAlert::CheckpointMismatch(alert) => {
                hasher.update([4]);
                hasher.update(alert.descriptor_event_id().as_bytes());
                hash_slice(&mut hasher, alert.code().as_str().as_bytes());
            }
        }
    }
    authority_bytes(hasher)
}

fn hash_manifest_hints(hasher: &mut Sha256, hints: &crate::ManifestHints) {
    hasher.update(hints.event_id().as_bytes());
    hasher.update(hints.coordinate().controller().as_bytes());
    hasher.update(hints.coordinate().document_id().as_bytes());
    hasher.update(hints.control().as_bytes());
    match hints.checkpoint() {
        Some(checkpoint) => {
            hasher.update([1]);
            hasher.update(checkpoint.as_bytes());
        }
        None => hasher.update([0]),
    }
    hash_len(hasher, hints.relays().len());
    for relay in hints.relays() {
        hash_slice(hasher, relay.as_bytes());
    }
}

fn manifest_authority(manifest: &ResolvedManifestAvailability) -> [u8; 32] {
    let mut hasher = authority_hasher(b"manifest");
    match manifest {
        ResolvedManifestAvailability::Missing => hasher.update([0]),
        ResolvedManifestAvailability::Available {
            hints,
            control_status,
        } => {
            hasher.update([1]);
            hash_manifest_hints(&mut hasher, hints);
            hasher.update([manifest_control_status_code(*control_status)]);
        }
        ResolvedManifestAvailability::Pending { hints, reason } => {
            hasher.update([2]);
            hash_manifest_hints(&mut hasher, hints);
            hasher.update([manifest_pending_reason_code(*reason)]);
        }
        ResolvedManifestAvailability::Unavailable {
            event_id,
            control,
            diagnostic,
        } => {
            hasher.update([3]);
            hasher.update(event_id.as_bytes());
            match control {
                Some(control) => {
                    hasher.update([1]);
                    hasher.update(control.as_bytes());
                }
                None => hasher.update([0]),
            }
            hash_slice(&mut hasher, diagnostic.as_str().as_bytes());
        }
    }
    authority_bytes(hasher)
}

fn document_authority(document: Option<&MaterializedDocumentView>) -> [u8; 32] {
    let mut hasher = authority_hasher(b"materialized-document");
    match document {
        Some(document) => {
            hasher.update([1]);
            hash_slice(&mut hasher, document.canonical_bytes());
        }
        None => hasher.update([0]),
    }
    authority_bytes(hasher)
}

const fn evidence_status_code(status: crate::EvidenceStatus) -> u8 {
    match status {
        crate::EvidenceStatus::Valid => 0,
        crate::EvidenceStatus::Pending => 1,
        crate::EvidenceStatus::Invalid => 2,
        crate::EvidenceStatus::Unsupported => 3,
        crate::EvidenceStatus::Irrelevant => 4,
        crate::EvidenceStatus::Duplicate => 5,
    }
}

const fn checkpoint_status_code(status: crate::CheckpointVerificationStatus) -> u8 {
    use crate::CheckpointVerificationStatus as Status;
    match status {
        Status::Verified => 0,
        Status::PendingControl => 1,
        Status::Unauthorized => 2,
        Status::ChunkAuthorMismatch => 3,
        Status::ChunkCoordinateMismatch => 4,
        Status::ChunkDescriptorMismatch => 5,
        Status::ChunkCountMismatch => 6,
        Status::DuplicateChunk => 7,
        Status::MissingChunk => 8,
        Status::ChunkSizeMismatch => 9,
        Status::ChunkAssemblyMismatch => 10,
        Status::MerkleMismatch => 11,
        Status::SnapshotSizeMismatch => 12,
        Status::SnapshotHashMismatch => 13,
        Status::SnapshotLoad => 14,
        Status::HeadMismatch => 15,
        Status::CommitmentMismatch => 16,
        Status::ClosureMismatch => 17,
        Status::MissingHistoricalCarrier => 18,
        Status::NotAcceptedAtControl => 19,
        Status::BudgetExhausted => 20,
        Status::Cancelled => 21,
    }
}

const fn manifest_control_status_code(status: crate::ManifestControlStatus) -> u8 {
    match status {
        crate::ManifestControlStatus::Canonical => 0,
        crate::ManifestControlStatus::Noncanonical => 1,
    }
}

const fn manifest_pending_reason_code(reason: crate::ManifestPendingReason) -> u8 {
    match reason {
        crate::ManifestPendingReason::MissingControl => 0,
        crate::ManifestPendingReason::ControlPending => 1,
    }
}

const fn resolved_manifest_event_id(manifest: &ResolvedManifestAvailability) -> Option<EventId> {
    match manifest {
        ResolvedManifestAvailability::Missing => None,
        ResolvedManifestAvailability::Available { hints, .. }
        | ResolvedManifestAvailability::Pending { hints, .. } => Some(hints.event_id()),
        ResolvedManifestAvailability::Unavailable { event_id, .. } => Some(*event_id),
    }
}

fn complete_parts_are_canonical(
    parts: &EvaluationReportParts,
    witness: CompleteReportWitness<'_>,
) -> bool {
    let Ok(history_digest) = crate::canonical_history_digest(
        parts.revision,
        parts.coordinate,
        &parts.canonical_controls,
        &parts.accepted_changes,
        &parts.heads,
    ) else {
        return false;
    };
    let Ok(dispositions_digest) = crate::canonical_dispositions_digest(
        parts.revision,
        parts.coordinate,
        &parts.disposition_records,
    ) else {
        return false;
    };
    if parts.failure.is_some()
        || parts.document.is_none()
        || parts.history_digest != history_digest
        || parts.dispositions_digest != dispositions_digest
        || !witness.source_authority.matches(parts)
        || !witness.field_authority.matches(parts)
        || !checkpoint_records_are_canonical(&parts.checkpoints)
        || !integrity_alerts_are_causal(parts, witness.control_children, witness.carrier_outcomes)
    {
        return false;
    }
    let canonical_controls = parts
        .canonical_controls
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let accepted_controls = parts
        .control_dispositions
        .iter()
        .filter_map(|(event_id, disposition)| {
            (*disposition == ProtocolDisposition::Accepted).then_some(*event_id)
        })
        .collect::<BTreeSet<_>>();
    if canonical_controls.len() != parts.canonical_controls.len()
        || canonical_controls != accepted_controls
        || !canonical_control_chain_matches(&parts.canonical_controls, witness.control_children)
        || !parts.dispositions.iter().copied().eq(witness
            .semantic_dispositions
            .iter()
            .map(|(hash, disposition)| (*hash, *disposition)))
        || !semantic_partitions_match(parts)
        || !carrier_outcomes_match(parts, witness.carrier_outcomes)
    {
        return false;
    }
    match (
        parts.canonical_controls.is_empty(),
        witness.accepted_changes,
        witness.heads,
    ) {
        (true, None, None) => parts.accepted_changes.is_empty() && parts.heads.is_empty(),
        (false, Some(accepted), Some(heads)) => {
            parts
                .accepted_changes
                .iter()
                .copied()
                .eq(accepted.iter().copied())
                && parts.heads.iter().copied().eq(heads.iter().copied())
        }
        (true, Some(accepted), Some(heads)) => {
            accepted.is_empty()
                && heads.is_empty()
                && parts.accepted_changes.is_empty()
                && parts.heads.is_empty()
        }
        (false, None, None) | (_, Some(_), None) | (_, None, Some(_)) => false,
    }
}

fn checkpoint_records_are_canonical(checkpoints: &[CheckpointVerificationResult]) -> bool {
    checkpoints.iter().all(|checkpoint| {
        strictly_sorted(checkpoint.chunk_events())
            && strictly_sorted(checkpoint.heads())
            && strictly_sorted(checkpoint.historical_carriers())
            && strictly_sorted(checkpoint.accepted_at_control())
    })
}

fn integrity_alerts_are_causal(
    parts: &EvaluationReportParts,
    control_children: Option<&BTreeMap<Option<EventId>, BTreeSet<EventId>>>,
    carrier_outcomes: &BTreeMap<EventId, AttributableCarrierOutcome>,
) -> bool {
    parts.integrity_alerts.iter().all(|alert| match alert {
        IntegrityAlert::ControllerEquivocation(alert) => {
            let Some(children) =
                control_children.and_then(|children| children.get(&alert.parent_control()))
            else {
                return false;
            };
            alert
                .candidate_controls()
                .iter()
                .all(|event_id| children.contains(event_id))
                && alert.candidate_controls().iter().all(|event_id| {
                    parts
                        .control_dispositions
                        .binary_search_by_key(event_id, |(candidate, _)| *candidate)
                        .is_ok()
                })
        }
        IntegrityAlert::CanonicalControlReorganization(_) => false,
        IntegrityAlert::DeviceEquivocation(alert) => alert
            .conflicting_changes()
            .iter()
            .chain(alert.affected_descendants())
            .all(|hash| {
                parts
                    .dispositions
                    .binary_search_by_key(hash, |(candidate, _)| *candidate)
                    .is_ok()
            }),
        IntegrityAlert::PotentialClonedDeviceKey(alert) => alert
            .carrier_event_ids()
            .iter()
            .all(|event_id| carrier_outcomes.contains_key(event_id)),
        IntegrityAlert::CheckpointMismatch(alert) => parts
            .checkpoints
            .binary_search_by_key(&alert.descriptor_event_id(), |checkpoint| {
                checkpoint.descriptor_event()
            })
            .is_ok_and(|index| {
                parts.checkpoints[index].status().event_outcome().1 == Some(alert.code())
            }),
    })
}

fn canonical_control_chain_matches(
    canonical_controls: &[EventId],
    control_children: Option<&BTreeMap<Option<EventId>, BTreeSet<EventId>>>,
) -> bool {
    let Some((first, rest)) = canonical_controls.split_first() else {
        return true;
    };
    let Some(control_children) = control_children else {
        return false;
    };
    control_children
        .get(&None)
        .is_some_and(|children| children.contains(first))
        && rest.iter().zip(canonical_controls).all(|(child, parent)| {
            control_children
                .get(&Some(*parent))
                .is_some_and(|children| children.contains(child))
        })
}

fn semantic_partitions_match(parts: &EvaluationReportParts) -> bool {
    if parts.accepted_changes.len()
        + parts.pending_changes.len()
        + parts.excluded_changes.len()
        + parts.invalid_changes.len()
        != parts.dispositions.len()
    {
        return false;
    }
    let semantic_record_count = parts
        .disposition_records
        .iter()
        .filter(|record| matches!(record.identifier(), ProtocolItemIdentifier::ChangeHash(_)))
        .count();
    if semantic_record_count != parts.dispositions.len() {
        return false;
    }
    parts.dispositions.iter().all(|(hash, disposition)| {
        let partition = match disposition {
            ProtocolDisposition::Accepted => &parts.accepted_changes,
            ProtocolDisposition::Pending => &parts.pending_changes,
            ProtocolDisposition::Excluded => &parts.excluded_changes,
            ProtocolDisposition::Invalid => &parts.invalid_changes,
            ProtocolDisposition::UnsupportedRevision => return false,
        };
        let identifier = ProtocolItemIdentifier::from(*hash);
        partition.binary_search(hash).is_ok()
            && parts
                .disposition_records
                .binary_search_by_key(&identifier, DispositionRecord::identifier)
                .is_ok_and(|index| {
                    let record = parts.disposition_records[index];
                    record.disposition() == *disposition && record.diagnostic().is_none()
                })
    })
}

fn carrier_outcomes_match(
    parts: &EvaluationReportParts,
    expected: &BTreeMap<EventId, AttributableCarrierOutcome>,
) -> bool {
    if expected
        .iter()
        .any(|(event_id, outcome)| *event_id != outcome.event_id())
        || !parts
            .change_carrier_dispositions
            .iter()
            .copied()
            .eq(expected.values().filter_map(|outcome| {
                outcome
                    .change_hash()
                    .map(|hash| (outcome.event_id(), hash, outcome.disposition()))
            }))
        || expected.values().any(|outcome| {
            let identifier = ProtocolItemIdentifier::event(outcome.event_id());
            !parts
                .disposition_records
                .binary_search_by_key(&identifier, DispositionRecord::identifier)
                .is_ok_and(|index| {
                    let record = parts.disposition_records[index];
                    record.disposition() == outcome.disposition()
                        && record.diagnostic() == outcome.diagnostic()
                })
        })
        || expected
            .values()
            .filter_map(|outcome| outcome.change_hash())
            .any(|hash| {
                parts
                    .dispositions
                    .binary_search_by_key(&hash, |(candidate, _)| *candidate)
                    .is_err()
            })
    {
        return false;
    }

    parts.dispositions.iter().all(|(hash, disposition)| {
        let mut matching = expected
            .values()
            .filter(|outcome| outcome.change_hash() == Some(*hash));
        let Some(first) = matching.next() else {
            return false;
        };
        let has_accepted = first.disposition() == ProtocolDisposition::Accepted
            || matching.any(|outcome| outcome.disposition() == ProtocolDisposition::Accepted);
        (*disposition == ProtocolDisposition::Accepted) == has_accepted
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
        AttributableCarrierOutcome, CanonicalControlReorganizationAlert,
        CompleteReportFieldAuthority, CompleteReportSourceAuthority, CompleteReportWitness,
        DispositionRecord, EvaluationReport, EvaluationReportParts, ProtocolItemIdentifier,
        ReevaluationComparisonStage, ReevaluationConstructionError, ReevaluationControlSummary,
        ReportConstructionPath, charged_canonical_reorganization_alert_with_observer,
        charged_detect_reorganization, disposition_records_are_canonical,
    };
    use crate::control::reorganization::detect_reorganization;
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn report_construction_inventory_is_closed_and_ordered() {
        let identifiers = ReportConstructionPath::ALL.map(ReportConstructionPath::identifier);
        assert_eq!(identifiers, ["complete", "no_progress"]);

        for invalid in [
            &["complete"][..],
            &["complete", "no_progress", "alternate"][..],
            &["no_progress", "complete"][..],
            &["complete", "stale"][..],
        ] {
            assert_ne!(invalid, identifiers);
        }
    }
    use crate::{
        ChangeHash, CheckpointVerificationResult, CheckpointVerificationStatus, Completion,
        DispositionsDigest, DocumentCoordinate, EventId, HistoryDigest, IntegrityAlert,
        SnapshotHash,
    };

    struct CompleteTestAuthority {
        has_controls: bool,
        control_children: BTreeMap<Option<EventId>, BTreeSet<EventId>>,
        semantic_dispositions: BTreeMap<ChangeHash, crate::ProtocolDisposition>,
        carrier_outcomes: BTreeMap<EventId, AttributableCarrierOutcome>,
        accepted_changes: BTreeSet<ChangeHash>,
        heads: BTreeSet<ChangeHash>,
        evidence: Vec<crate::EvidenceRecord>,
        fields: CompleteReportFieldAuthority,
    }

    impl CompleteTestAuthority {
        fn for_parts(parts: &EvaluationReportParts) -> Self {
            let mut control_children = BTreeMap::new();
            if let Some(first) = parts.canonical_controls.first().copied() {
                control_children
                    .entry(None)
                    .or_insert_with(BTreeSet::new)
                    .insert(first);
                for pair in parts.canonical_controls.windows(2) {
                    control_children
                        .entry(Some(pair[0]))
                        .or_insert_with(BTreeSet::new)
                        .insert(pair[1]);
                }
            }
            let carrier_outcomes = parts
                .change_carrier_dispositions
                .iter()
                .map(|(event_id, hash, disposition)| {
                    let identifier = ProtocolItemIdentifier::event(*event_id);
                    let diagnostic = parts
                        .disposition_records
                        .binary_search_by_key(&identifier, DispositionRecord::identifier)
                        .ok()
                        .and_then(|index| parts.disposition_records[index].diagnostic());
                    (
                        *event_id,
                        AttributableCarrierOutcome::verified_change(
                            *event_id,
                            *hash,
                            *disposition,
                            diagnostic,
                        ),
                    )
                })
                .collect();
            Self {
                has_controls: !parts.canonical_controls.is_empty(),
                control_children,
                semantic_dispositions: parts.dispositions.iter().copied().collect(),
                carrier_outcomes,
                accepted_changes: parts.accepted_changes.iter().copied().collect(),
                heads: parts.heads.iter().copied().collect(),
                evidence: parts.evidence.clone(),
                fields: CompleteReportFieldAuthority::derive(
                    &parts.evidence,
                    &parts.checkpoints,
                    &parts.integrity_alerts,
                    &parts.manifest,
                    parts.document.as_ref(),
                ),
            }
        }

        fn witness(&self) -> CompleteReportWitness<'_> {
            CompleteReportWitness::new(
                Some(&self.control_children),
                &self.semantic_dispositions,
                &self.carrier_outcomes,
                self.has_controls.then_some(&self.accepted_changes),
                self.has_controls.then_some(&self.heads),
                CompleteReportSourceAuthority::TestEvidence(&self.evidence),
                self.fields,
            )
        }
    }

    fn complete_report(
        parts: EvaluationReportParts,
        authority: &CompleteTestAuthority,
    ) -> Result<EvaluationReport, super::EvaluationError> {
        EvaluationReport::from_complete_parts(parts, authority.witness())
    }

    fn semantic_partition_mut(
        parts: &mut EvaluationReportParts,
        index: u8,
    ) -> &mut Vec<ChangeHash> {
        match index {
            0 => &mut parts.accepted_changes,
            1 => &mut parts.pending_changes,
            2 => &mut parts.excluded_changes,
            _ => &mut parts.invalid_changes,
        }
    }

    fn no_progress_parts(completion: Completion) -> Option<EvaluationReportParts> {
        let coordinate = format!("31624:{}:{}", "41".repeat(32), "42".repeat(32))
            .parse::<DocumentCoordinate>()
            .ok()?;
        let revision = crate::ProtocolRevision::draft_v1();
        let history_digest =
            crate::canonical_history_digest(revision, coordinate, &[], &[], &[]).ok()?;
        let dispositions_digest =
            crate::canonical_dispositions_digest(revision, coordinate, &[]).ok()?;
        let failure = match completion {
            Completion::BudgetExhausted => super::EvaluationFailure::BudgetExhausted,
            Completion::Cancelled => super::EvaluationFailure::Cancelled,
            Completion::Complete => return None,
        };
        Some(EvaluationReportParts {
            coordinate,
            revision,
            canonical_controls: Vec::new(),
            disposition_records: Vec::new(),
            control_dispositions: Vec::new(),
            dispositions: Vec::new(),
            change_carrier_dispositions: Vec::new(),
            accepted_changes: Vec::new(),
            pending_changes: Vec::new(),
            excluded_changes: Vec::new(),
            invalid_changes: Vec::new(),
            heads: Vec::new(),
            evidence: Vec::new(),
            checkpoints: Vec::new(),
            history_digest,
            dispositions_digest,
            integrity_alerts: Vec::new(),
            manifest: crate::ResolvedManifestAvailability::Missing,
            completion,
            failure: Some(failure),
            document: None,
        })
    }

    fn complete_matrix_parts() -> Option<EvaluationReportParts> {
        let coordinate = format!("31624:{}:{}", "61".repeat(32), "62".repeat(32))
            .parse::<DocumentCoordinate>()
            .ok()?;
        let root = EventId::from_bytes([9; 32]);
        let child = EventId::from_bytes([1; 32]);
        let rejected = EventId::from_bytes([12; 32]);
        let hashes = (1_u8..=9)
            .map(|value| ChangeHash::from_bytes([value; 32]))
            .collect::<Vec<_>>();
        let carriers = (20_u8..=28)
            .map(|value| EventId::from_bytes([value; 32]))
            .collect::<Vec<_>>();
        let dispositions = vec![
            (hashes[0], crate::ProtocolDisposition::Accepted),
            (hashes[1], crate::ProtocolDisposition::Accepted),
            (hashes[2], crate::ProtocolDisposition::Accepted),
            (hashes[3], crate::ProtocolDisposition::Pending),
            (hashes[4], crate::ProtocolDisposition::Pending),
            (hashes[5], crate::ProtocolDisposition::Excluded),
            (hashes[6], crate::ProtocolDisposition::Excluded),
            (hashes[7], crate::ProtocolDisposition::Invalid),
            (hashes[8], crate::ProtocolDisposition::Invalid),
        ];
        let disposition_records = [
            DispositionRecord::new(
                ProtocolItemIdentifier::control_event(child),
                crate::ProtocolDisposition::Accepted,
                None,
            ),
            DispositionRecord::new(
                ProtocolItemIdentifier::control_event(root),
                crate::ProtocolDisposition::Accepted,
                None,
            ),
            DispositionRecord::new(
                ProtocolItemIdentifier::control_event(rejected),
                crate::ProtocolDisposition::Excluded,
                None,
            ),
        ]
        .into_iter()
        .chain(dispositions.iter().map(|(hash, disposition)| {
            DispositionRecord::new(ProtocolItemIdentifier::from(*hash), *disposition, None)
        }))
        .chain(
            carriers
                .iter()
                .zip(dispositions.iter())
                .map(|(event_id, (_, disposition))| {
                    DispositionRecord::new(
                        ProtocolItemIdentifier::event(*event_id),
                        *disposition,
                        None,
                    )
                }),
        )
        .collect();
        let change_carrier_dispositions = carriers
            .iter()
            .zip(dispositions.iter())
            .map(|(event_id, (hash, disposition))| (*event_id, *hash, *disposition))
            .collect();
        let mut parts = EvaluationReportParts {
            coordinate,
            revision: crate::ProtocolRevision::draft_v1(),
            canonical_controls: vec![root, child],
            disposition_records,
            control_dispositions: vec![
                (child, crate::ProtocolDisposition::Accepted),
                (root, crate::ProtocolDisposition::Accepted),
                (rejected, crate::ProtocolDisposition::Excluded),
            ],
            dispositions,
            change_carrier_dispositions,
            accepted_changes: hashes[0..3].to_vec(),
            pending_changes: hashes[3..5].to_vec(),
            excluded_changes: hashes[5..7].to_vec(),
            invalid_changes: hashes[7..9].to_vec(),
            heads: hashes[1..3].to_vec(),
            evidence: Vec::new(),
            checkpoints: Vec::new(),
            history_digest: HistoryDigest::from_bytes([0; 32]),
            dispositions_digest: DispositionsDigest::from_bytes([0; 32]),
            integrity_alerts: Vec::new(),
            manifest: crate::ResolvedManifestAvailability::Missing,
            completion: Completion::Complete,
            failure: None,
            document:
                crate::automerge_adapter::materialized_view::MaterializedDocumentView::empty_for_test()
                    .ok(),
        };
        parts.history_digest = crate::canonical_history_digest(
            parts.revision,
            parts.coordinate,
            &parts.canonical_controls,
            &parts.accepted_changes,
            &parts.heads,
        )
        .ok()?;
        parts.dispositions_digest = crate::canonical_dispositions_digest(
            parts.revision,
            parts.coordinate,
            &parts.disposition_records,
        )
        .ok()?;
        Some(parts)
    }

    fn recompute_complete_digests(parts: &mut EvaluationReportParts) -> Option<()> {
        parts.history_digest = crate::canonical_history_digest(
            parts.revision,
            parts.coordinate,
            &parts.canonical_controls,
            &parts.accepted_changes,
            &parts.heads,
        )
        .ok()?;
        parts.dispositions_digest = crate::canonical_dispositions_digest(
            parts.revision,
            parts.coordinate,
            &parts.disposition_records,
        )
        .ok()?;
        Some(())
    }

    fn complete_exact_field_parts() -> Option<EvaluationReportParts> {
        let mut parts = complete_matrix_parts()?;
        let manifest_event = EventId::from_bytes([40; 32]);
        let descriptor_event = EventId::from_bytes([41; 32]);
        let chunk_event = EventId::from_bytes([42; 32]);
        let hints = crate::ManifestHints::new(
            manifest_event,
            parts.coordinate,
            parts.canonical_controls[1],
            Some(descriptor_event),
            vec!["wss://relay.example".to_owned()],
        );
        parts.manifest = crate::ResolvedManifestAvailability::Available {
            hints,
            control_status: crate::ManifestControlStatus::Canonical,
        };
        parts
            .checkpoints
            .push(CheckpointVerificationResult::from_trusted_ordered(
                descriptor_event,
                vec![chunk_event],
                SnapshotHash::from_bytes([43; 32]),
                parts.heads.clone(),
                u64::try_from(parts.accepted_changes.len()).ok()?,
                [44; 32],
                parts.accepted_changes.clone(),
                parts.accepted_changes.clone(),
                CheckpointVerificationStatus::Verified,
            ));
        parts.disposition_records.extend([
            DispositionRecord::new(
                ProtocolItemIdentifier::event(manifest_event),
                crate::ProtocolDisposition::Accepted,
                None,
            ),
            DispositionRecord::new(
                ProtocolItemIdentifier::event(descriptor_event),
                crate::ProtocolDisposition::Accepted,
                None,
            ),
            DispositionRecord::new(
                ProtocolItemIdentifier::event(chunk_event),
                crate::ProtocolDisposition::Accepted,
                None,
            ),
        ]);
        parts
            .disposition_records
            .sort_by_key(DispositionRecord::identifier);
        let mut corpus = crate::CorpusBuilder::new();
        let _ = corpus.ingest_bytes(b"{}");
        parts.evidence = corpus.finish().records().collect();
        let alert = crate::DeviceEquivocationAlert::new(
            crate::ActorId::from_bytes([45; 32]),
            1,
            vec![parts.invalid_changes[0], parts.invalid_changes[1]],
            Vec::new(),
        )
        .ok()?;
        parts
            .integrity_alerts
            .push(IntegrityAlert::DeviceEquivocation(alert));
        recompute_complete_digests(&mut parts)?;
        Some(parts)
    }

    fn nonempty_document() -> Option<crate::MaterializedDocumentView> {
        crate::automerge_adapter::materialized_view::MaterializedDocumentView::nonempty_for_test()
            .ok()
    }

    fn replace_canonical_tip(parts: &mut EvaluationReportParts, tip: EventId) -> Option<()> {
        let prior = *parts.canonical_controls.get(1)?;
        parts.canonical_controls[1] = tip;
        let control_index = parts
            .control_dispositions
            .binary_search_by_key(&prior, |(event_id, _)| *event_id)
            .ok()?;
        parts.control_dispositions[control_index].0 = tip;
        parts
            .control_dispositions
            .sort_by_key(|(event_id, _)| *event_id);
        let identifier = ProtocolItemIdentifier::control_event(prior);
        let record_index = parts
            .disposition_records
            .binary_search_by_key(&identifier, DispositionRecord::identifier)
            .ok()?;
        parts.disposition_records[record_index] = DispositionRecord::new(
            ProtocolItemIdentifier::control_event(tip),
            crate::ProtocolDisposition::Accepted,
            None,
        );
        parts
            .disposition_records
            .sort_by_key(DispositionRecord::identifier);
        recompute_complete_digests(parts)
    }

    fn reevaluation_report_pair() -> Option<(EvaluationReport, EvaluationReport, IntegrityAlert)> {
        let mut current_parts = complete_matrix_parts()?;
        let base_alert = crate::DeviceEquivocationAlert::new(
            crate::ActorId::from_bytes([80; 32]),
            1,
            current_parts.invalid_changes.clone(),
            Vec::new(),
        )
        .ok()?;
        current_parts
            .integrity_alerts
            .push(IntegrityAlert::DeviceEquivocation(base_alert));
        let mut previous_parts = current_parts.clone();
        replace_canonical_tip(&mut previous_parts, EventId::from_bytes([2; 32]))?;
        let current_authority = CompleteTestAuthority::for_parts(&current_parts);
        let previous_authority = CompleteTestAuthority::for_parts(&previous_parts);
        let current = complete_report(current_parts, &current_authority).ok()?;
        let previous = complete_report(previous_parts, &previous_authority).ok()?;
        let alert = detect_reorganization(
            &previous.control_chain_summary(),
            &current.control_chain_summary(),
        )?;
        Some((previous, current, alert))
    }

    #[test]
    fn reevaluation_alert_construction_is_exact_and_cannot_bypass_validation() {
        let Some((previous, current, reorganization)) = reevaluation_report_pair() else {
            return;
        };
        let base_alert = current.integrity_alerts()[0].clone();
        let canonical = vec![base_alert.clone(), reorganization.clone()];
        let report = EvaluationReport::from_reevaluation(current.clone(), &previous, |_| Ok(()));
        assert!(report.is_ok());
        let Ok(report) = report else { return };
        assert_eq!(report.integrity_alerts(), canonical);

        let mut injected_parts = complete_matrix_parts();
        assert!(injected_parts.is_some());
        let Some(ref mut injected_parts) = injected_parts else {
            return;
        };
        injected_parts.integrity_alerts.push(reorganization.clone());
        let injected_authority = CompleteTestAuthority::for_parts(injected_parts);
        assert_eq!(
            complete_report(injected_parts.clone(), &injected_authority),
            Err(super::EvaluationError::ReportInvariant)
        );
    }

    #[test]
    fn reevaluation_comparison_is_charged_per_item_and_preserves_typed_stops() {
        let Some((previous, current, _)) = reevaluation_report_pair() else {
            return;
        };

        let mut full_trace = Vec::new();
        let complete = EvaluationReport::from_reevaluation(current.clone(), &previous, |stage| {
            full_trace.push(stage);
            Ok(())
        });
        assert!(complete.is_ok());
        assert!(!full_trace.is_empty());
        assert!(
            full_trace
                .windows(2)
                .all(|pair| pair[0].index() <= pair[1].index())
        );
        for stage in ReevaluationComparisonStage::ALL {
            assert!(
                full_trace.contains(&stage),
                "missing charged stage {stage:?}"
            );
        }

        for stage in ReevaluationComparisonStage::ALL {
            let stage_end = full_trace
                .iter()
                .rposition(|candidate| *candidate == stage)
                .map(|index| index + 1);
            assert!(stage_end.is_some());
            let Some(stage_end) = stage_end else { continue };
            let n = u64::try_from(stage_end).unwrap_or(u64::MAX);

            let mut budget = crate::WorkBudget::new(0, n.saturating_sub(1));
            let mut observed = Vec::new();
            let stopped =
                EvaluationReport::from_reevaluation(current.clone(), &previous, |candidate| {
                    budget
                        .charge(crate::WorkCounter::Assertion, 1)
                        .map_err(|_| Completion::BudgetExhausted)?;
                    observed.push(candidate);
                    Ok(())
                });
            assert_eq!(
                stopped,
                Err(ReevaluationConstructionError::Stopped(
                    Completion::BudgetExhausted
                )),
                "{stage:?} N-1"
            );
            assert_eq!(observed, full_trace[..stage_end.saturating_sub(1)]);

            let mut budget = crate::WorkBudget::new(0, n);
            let mut observed = Vec::new();
            let at_boundary =
                EvaluationReport::from_reevaluation(current.clone(), &previous, |candidate| {
                    budget
                        .charge(crate::WorkCounter::Assertion, 1)
                        .map_err(|_| Completion::BudgetExhausted)?;
                    observed.push(candidate);
                    Ok(())
                });
            assert_eq!(observed, full_trace[..stage_end]);
            if stage == ReevaluationComparisonStage::FinalConstruction {
                assert!(at_boundary.is_ok(), "{stage:?} N");
            } else {
                assert_eq!(
                    at_boundary,
                    Err(ReevaluationConstructionError::Stopped(
                        Completion::BudgetExhausted
                    )),
                    "{stage:?} N"
                );
            }

            let stop_index = stage_end.saturating_sub(1);
            let mut callback_calls = 0_usize;
            let mut observed = Vec::new();
            let cancelled =
                EvaluationReport::from_reevaluation(current.clone(), &previous, |candidate| {
                    let call = callback_calls;
                    callback_calls = callback_calls.saturating_add(1);
                    if call == stop_index {
                        return Err(Completion::Cancelled);
                    }
                    observed.push(candidate);
                    Ok(())
                });
            assert_eq!(
                cancelled,
                Err(ReevaluationConstructionError::Stopped(
                    Completion::Cancelled
                )),
                "{stage:?} cancellation"
            );
            assert_eq!(callback_calls, stage_end);
            assert_eq!(observed, full_trace[..stop_index]);
        }
    }

    #[test]
    fn reevaluation_comparison_does_not_mask_an_unexpected_callback_panic() {
        #[derive(Debug, PartialEq, Eq)]
        struct UnexpectedComparisonFailure(u8);

        let Some((previous, current, _)) = reevaluation_report_pair() else {
            return;
        };
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = EvaluationReport::from_reevaluation(current, &previous, |stage| {
                if stage == ReevaluationComparisonStage::Relationship {
                    std::panic::resume_unwind(Box::new(UnexpectedComparisonFailure(82)));
                }
                Ok(())
            });
        }));
        assert!(panic.is_err());
        let Err(panic) = panic else { return };
        assert_eq!(
            panic.downcast_ref::<UnexpectedComparisonFailure>(),
            Some(&UnexpectedComparisonFailure(82))
        );
    }

    #[test]
    fn canonical_alert_comparisons_are_interleaved_with_successful_charges() {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum Observation {
            Charge,
            Comparison(usize, std::cmp::Ordering),
        }

        let previous_tip = EventId::from_bytes([100; 32]);
        let current_tip = EventId::from_bytes([101; 32]);
        let affected = vec![
            ChangeHash::from_bytes([102; 32]),
            ChangeHash::from_bytes([103; 32]),
            ChangeHash::from_bytes([104; 32]),
        ];
        let observations = std::cell::RefCell::new(Vec::new());
        let result = charged_canonical_reorganization_alert_with_observer(
            previous_tip,
            current_tip,
            affected.clone(),
            &mut |_| {
                observations.borrow_mut().push(Observation::Charge);
                Ok(())
            },
            &mut |index, ordering| {
                observations
                    .borrow_mut()
                    .push(Observation::Comparison(index, ordering));
            },
        );
        assert!(result.is_ok());
        assert_eq!(
            observations.into_inner(),
            vec![
                Observation::Charge,
                Observation::Comparison(0, std::cmp::Ordering::Less),
                Observation::Charge,
                Observation::Comparison(1, std::cmp::Ordering::Less),
                Observation::Charge,
                Observation::Comparison(3, std::cmp::Ordering::Less),
            ]
        );

        let comparison_count = affected.len();
        let mut budget = crate::WorkBudget::new(
            0,
            u64::try_from(comparison_count.saturating_sub(1)).unwrap_or(u64::MAX),
        );
        let mut comparisons = 0_usize;
        let stopped = charged_canonical_reorganization_alert_with_observer(
            previous_tip,
            current_tip,
            affected.clone(),
            &mut |_| {
                budget
                    .charge(crate::WorkCounter::Assertion, 1)
                    .map_err(|_| Completion::BudgetExhausted)
            },
            &mut |_, _| comparisons = comparisons.saturating_add(1),
        );
        assert_eq!(
            stopped,
            Err(ReevaluationConstructionError::Stopped(
                Completion::BudgetExhausted
            ))
        );
        assert_eq!(comparisons, comparison_count.saturating_sub(1));

        for cancelled_comparison in 0..comparison_count {
            let mut callback_calls = 0_usize;
            let mut comparisons = 0_usize;
            let stopped = charged_canonical_reorganization_alert_with_observer(
                previous_tip,
                current_tip,
                affected.clone(),
                &mut |_| {
                    let call = callback_calls;
                    callback_calls = callback_calls.saturating_add(1);
                    if call == cancelled_comparison {
                        Err(Completion::Cancelled)
                    } else {
                        Ok(())
                    }
                },
                &mut |_, _| comparisons = comparisons.saturating_add(1),
            );
            assert_eq!(
                stopped,
                Err(ReevaluationConstructionError::Stopped(
                    Completion::Cancelled
                ))
            );
            assert_eq!(callback_calls, cancelled_comparison.saturating_add(1));
            assert_eq!(comparisons, cancelled_comparison);
        }

        for noncanonical in [
            vec![affected[0], affected[0]],
            vec![affected[1], affected[0]],
        ] {
            let result = charged_canonical_reorganization_alert_with_observer(
                previous_tip,
                current_tip,
                noncanonical,
                &mut |_| Ok(()),
                &mut |_, _| {},
            );
            assert_eq!(result, Err(ReevaluationConstructionError::Invariant));
        }
        let equal_tip = charged_canonical_reorganization_alert_with_observer(
            previous_tip,
            previous_tip,
            Vec::new(),
            &mut |_| Ok(()),
            &mut |_, _| {},
        );
        assert_eq!(equal_tip, Err(ReevaluationConstructionError::Invariant));
        assert!(
            CanonicalControlReorganizationAlert::new(
                previous_tip,
                current_tip,
                vec![affected[0], affected[0]],
            )
            .is_err()
        );
    }

    #[test]
    fn charged_reevaluation_relationship_matches_the_canonical_state_table() {
        let root = EventId::from_bytes([90; 32]);
        let old = EventId::from_bytes([91; 32]);
        let new = EventId::from_bytes([92; 32]);
        let extension = EventId::from_bytes([93; 32]);
        let old_change = ChangeHash::from_bytes([94; 32]);
        let new_change = ChangeHash::from_bytes([95; 32]);

        let cases = [
            ("empty", Vec::new(), Vec::new(), Vec::new(), Vec::new()),
            (
                "identical",
                vec![root, old],
                vec![old_change],
                vec![root, old],
                vec![old_change],
            ),
            (
                "extension",
                vec![root, old],
                vec![old_change],
                vec![root, old, extension],
                vec![old_change, new_change],
            ),
            (
                "rollback",
                vec![root, old],
                vec![old_change],
                vec![root],
                vec![new_change],
            ),
            (
                "fork",
                vec![root, old],
                vec![old_change],
                vec![root, new],
                vec![new_change],
            ),
        ];

        for (name, previous_controls, previous_changes, current_controls, current_changes) in cases
        {
            let legacy = |controls: Vec<EventId>, changes: Vec<ChangeHash>| {
                let mut changes_by_control = BTreeMap::new();
                if let Some(tip) = controls.last().copied() {
                    changes_by_control.insert(tip, changes.iter().copied().collect());
                }
                crate::control::reorganization::ControlChainSummary {
                    controls,
                    changes_by_control,
                }
            };
            let expected = detect_reorganization(
                &legacy(previous_controls.clone(), previous_changes.clone()),
                &legacy(current_controls.clone(), current_changes.clone()),
            );
            let previous = ReevaluationControlSummary {
                controls: &previous_controls,
                accepted_at_tip: &previous_changes,
            };
            let current = ReevaluationControlSummary {
                controls: &current_controls,
                accepted_at_tip: &current_changes,
            };
            let mut observations = 0_u64;
            let actual = charged_detect_reorganization(&previous, &current, &mut |_| {
                observations = observations.saturating_add(1);
                Ok(())
            });
            assert_eq!(actual, Ok(expected), "{name}");
            assert!(observations > 0, "{name}");
        }
    }

    #[test]
    fn complete_report_rejects_exact_field_and_coordinated_rewrite_mutations() {
        let Some(parts) = complete_exact_field_parts() else {
            return;
        };
        let authority = CompleteTestAuthority::for_parts(&parts);
        assert!(complete_report(parts.clone(), &authority).is_ok());
        let assert_rejected = |name: &str, mutation: EvaluationReportParts| {
            assert_eq!(
                complete_report(mutation, &authority),
                Err(super::EvaluationError::ReportInvariant),
                "{name}"
            );
        };

        let mut mutation = parts.clone();
        mutation.history_digest = HistoryDigest::from_bytes([70; 32]);
        assert_rejected("history_digest", mutation);
        let mut mutation = parts.clone();
        mutation.dispositions_digest = DispositionsDigest::from_bytes([71; 32]);
        assert_rejected("dispositions_digest", mutation);
        let mut mutation = parts.clone();
        mutation.evidence.clear();
        assert_rejected("evidence_missing", mutation);
        let mut mutation = parts.clone();
        mutation.evidence.push(parts.evidence[0]);
        assert_rejected("evidence_duplicate", mutation);
        let mut mutation = parts.clone();
        mutation.integrity_alerts.clear();
        assert_rejected("integrity_alert_missing", mutation);
        let mut mutation = parts.clone();
        mutation.manifest = crate::ResolvedManifestAvailability::Missing;
        assert_rejected("manifest_rewritten", mutation);
        let mut mutation = parts.clone();
        mutation.document = nonempty_document();
        assert_rejected("materialized_document_content", mutation);

        let checkpoint = &parts.checkpoints[0];
        let changed_checkpoint = CheckpointVerificationResult::from_trusted_ordered(
            checkpoint.descriptor_event(),
            checkpoint.chunk_events().to_vec(),
            SnapshotHash::from_bytes([72; 32]),
            checkpoint.heads().to_vec(),
            checkpoint.change_count(),
            checkpoint.change_set_hash(),
            checkpoint.historical_carriers().to_vec(),
            checkpoint.accepted_at_control().to_vec(),
            checkpoint.status(),
        );
        let mut mutation = parts.clone();
        mutation.checkpoints[0] = changed_checkpoint;
        assert_rejected("checkpoint_commitment", mutation);

        let unsorted_checkpoint = CheckpointVerificationResult::from_trusted_ordered(
            checkpoint.descriptor_event(),
            vec![EventId::from_bytes([43; 32]), EventId::from_bytes([42; 32])],
            checkpoint.snapshot_hash(),
            checkpoint.heads().to_vec(),
            checkpoint.change_count(),
            checkpoint.change_set_hash(),
            checkpoint.historical_carriers().to_vec(),
            checkpoint.accepted_at_control().to_vec(),
            checkpoint.status(),
        );
        let mut mutation = parts.clone();
        mutation.checkpoints[0] = unsorted_checkpoint;
        assert_rejected("checkpoint_unsorted_chunks", mutation);

        let mut mutation = parts.clone();
        mutation.accepted_changes.pop();
        mutation.heads.pop();
        assert!(recompute_complete_digests(&mut mutation).is_some());
        assert_rejected("coordinated_history_rehash", mutation);

        let descriptor = checkpoint.descriptor_event();
        let changed_checkpoint = CheckpointVerificationResult::from_trusted_ordered(
            descriptor,
            checkpoint.chunk_events().to_vec(),
            checkpoint.snapshot_hash(),
            checkpoint.heads().to_vec(),
            checkpoint.change_count(),
            checkpoint.change_set_hash(),
            checkpoint.historical_carriers().to_vec(),
            checkpoint.accepted_at_control().to_vec(),
            CheckpointVerificationStatus::PendingControl,
        );
        let mut mutation = parts.clone();
        mutation.checkpoints[0] = changed_checkpoint;
        for event_id in
            core::iter::once(descriptor).chain(checkpoint.chunk_events().iter().copied())
        {
            let identifier = ProtocolItemIdentifier::event(event_id);
            let index = mutation
                .disposition_records
                .binary_search_by_key(&identifier, DispositionRecord::identifier);
            assert!(index.is_ok());
            if let Ok(index) = index {
                mutation.disposition_records[index] =
                    DispositionRecord::new(identifier, crate::ProtocolDisposition::Pending, None);
            }
        }
        assert!(recompute_complete_digests(&mut mutation).is_some());
        assert_rejected("coordinated_checkpoint_and_disposition_rehash", mutation);

        let manifest_event = match &parts.manifest {
            crate::ResolvedManifestAvailability::Available { hints, .. } => hints.event_id(),
            _ => return,
        };
        let mut mutation = parts.clone();
        mutation.manifest = crate::ResolvedManifestAvailability::Missing;
        mutation
            .disposition_records
            .retain(|record| record.identifier() != ProtocolItemIdentifier::event(manifest_event));
        assert!(recompute_complete_digests(&mut mutation).is_some());
        assert_rejected("coordinated_manifest_and_record_rewrite", mutation);
    }

    #[test]
    fn complete_report_rejects_every_partition_control_and_head_mutation() {
        let Some(parts) = complete_matrix_parts() else {
            return;
        };
        let authority = CompleteTestAuthority::for_parts(&parts);
        assert!(super::complete_parts_are_canonical(
            &parts,
            authority.witness()
        ));
        assert!(super::carrier_outcomes_match(
            &parts,
            &authority.carrier_outcomes
        ));
        assert!(super::disposition_records_are_canonical(
            &parts.disposition_records
        ));
        assert!(complete_report(parts.clone(), &authority).is_ok());
        let assert_rejected = |name: &str, mutation: EvaluationReportParts| {
            assert_eq!(
                complete_report(mutation, &authority),
                Err(super::EvaluationError::ReportInvariant),
                "{name}"
            );
        };

        let mut mutation = parts.clone();
        mutation.canonical_controls.pop();
        assert_rejected("canonical_controls_missing", mutation);
        let mut mutation = parts.clone();
        mutation
            .canonical_controls
            .push(EventId::from_bytes([12; 32]));
        assert_rejected("canonical_controls_extra", mutation);
        let mut mutation = parts.clone();
        mutation
            .canonical_controls
            .push(parts.canonical_controls[1]);
        assert_rejected("canonical_controls_duplicate", mutation);
        let mut mutation = parts.clone();
        mutation.canonical_controls.swap(0, 1);
        assert_rejected("canonical_controls_reordered", mutation);
        let mut mutation = parts.clone();
        mutation
            .canonical_controls
            .push(EventId::from_bytes([12; 32]));
        mutation.control_dispositions[2].1 = crate::ProtocolDisposition::Accepted;
        assert_rejected("canonical_controls_coordinated_nonchain", mutation);

        let mut mutation = parts.clone();
        mutation.control_dispositions.remove(0);
        assert_rejected("control_dispositions_missing", mutation);
        let mut mutation = parts.clone();
        mutation.control_dispositions[2].1 = crate::ProtocolDisposition::Accepted;
        assert_rejected("control_dispositions_extra_accepted", mutation);
        let mut mutation = parts.clone();
        mutation
            .control_dispositions
            .push(mutation.control_dispositions[2]);
        assert_rejected("control_dispositions_duplicate", mutation);
        let mut mutation = parts.clone();
        mutation.control_dispositions.swap(0, 1);
        assert_rejected("control_dispositions_unsorted", mutation);

        let foreign = ChangeHash::from_bytes([10; 32]);
        for (name, index) in [
            ("accepted", 0_u8),
            ("pending", 1),
            ("excluded", 2),
            ("invalid", 3),
        ] {
            let mut mutation = parts.clone();
            semantic_partition_mut(&mut mutation, index).pop();
            assert_rejected(&format!("{name}_missing"), mutation);
            let mut mutation = parts.clone();
            semantic_partition_mut(&mut mutation, index).push(foreign);
            assert_rejected(&format!("{name}_extra"), mutation);
            let mut mutation = parts.clone();
            let duplicate = *semantic_partition_mut(&mut mutation, index)
                .last()
                .unwrap_or(&foreign);
            semantic_partition_mut(&mut mutation, index).push(duplicate);
            assert_rejected(&format!("{name}_duplicate"), mutation);
            let mut mutation = parts.clone();
            semantic_partition_mut(&mut mutation, index).swap(0, 1);
            assert_rejected(&format!("{name}_unsorted"), mutation);
            let mut mutation = parts.clone();
            let overlap = match index {
                0 => parts.pending_changes[0],
                1 => parts.accepted_changes[1],
                2 => parts.pending_changes[1],
                _ => parts.excluded_changes[1],
            };
            semantic_partition_mut(&mut mutation, index).push(overlap);
            semantic_partition_mut(&mut mutation, index).sort_unstable();
            assert_rejected(&format!("{name}_overlap"), mutation);
        }

        let mut mutation = parts.clone();
        mutation.dispositions.pop();
        assert_rejected("semantic_dispositions_missing", mutation);
        let mut mutation = parts.clone();
        mutation
            .dispositions
            .push((foreign, crate::ProtocolDisposition::Invalid));
        mutation.invalid_changes.push(foreign);
        mutation.disposition_records.push(DispositionRecord::new(
            ProtocolItemIdentifier::from(foreign),
            crate::ProtocolDisposition::Invalid,
            None,
        ));
        mutation
            .disposition_records
            .sort_by_key(DispositionRecord::identifier);
        assert_rejected("semantic_dispositions_extra", mutation);
        let mut mutation = parts.clone();
        mutation.dispositions[3].1 = crate::ProtocolDisposition::Invalid;
        mutation.pending_changes.remove(0);
        mutation.invalid_changes.insert(0, parts.pending_changes[0]);
        let identifier = ProtocolItemIdentifier::from(parts.pending_changes[0]);
        let record = mutation
            .disposition_records
            .binary_search_by_key(&identifier, DispositionRecord::identifier);
        assert!(record.is_ok());
        if let Ok(index) = record {
            mutation.disposition_records[index] =
                DispositionRecord::new(identifier, crate::ProtocolDisposition::Invalid, None);
        }
        assert_rejected("semantic_dispositions_wrong_outcome", mutation);
        let mut mutation = parts.clone();
        mutation.dispositions.push(mutation.dispositions[7]);
        assert_rejected("semantic_dispositions_duplicate", mutation);
        let mut mutation = parts.clone();
        mutation.dispositions.swap(0, 1);
        assert_rejected("semantic_dispositions_unsorted", mutation);

        let semantic_identifier = ProtocolItemIdentifier::from(parts.accepted_changes[0]);
        let mut mutation = parts.clone();
        mutation
            .disposition_records
            .retain(|record| record.identifier() != semantic_identifier);
        assert_rejected("semantic_record_missing", mutation);
        let mut mutation = parts.clone();
        mutation.disposition_records.push(DispositionRecord::new(
            ProtocolItemIdentifier::from(foreign),
            crate::ProtocolDisposition::Invalid,
            None,
        ));
        mutation
            .disposition_records
            .sort_by_key(DispositionRecord::identifier);
        assert_rejected("semantic_record_extra", mutation);
        let mut mutation = parts.clone();
        let record = mutation
            .disposition_records
            .binary_search_by_key(&semantic_identifier, DispositionRecord::identifier);
        assert!(record.is_ok());
        if let Ok(index) = record {
            mutation.disposition_records[index] = DispositionRecord::new(
                semantic_identifier,
                crate::ProtocolDisposition::Pending,
                None,
            );
        }
        assert_rejected("semantic_record_wrong_outcome", mutation);

        let mut mutation = parts.clone();
        mutation.heads.pop();
        assert_rejected("heads_missing", mutation);
        let mut mutation = parts.clone();
        mutation.heads.push(parts.pending_changes[0]);
        assert_rejected("heads_extra_nonaccepted", mutation);
        let mut mutation = parts.clone();
        mutation.heads.push(parts.accepted_changes[0]);
        mutation.heads.sort_unstable();
        assert_rejected("heads_extra_accepted_nonfrontier", mutation);
        let mut mutation = parts.clone();
        mutation.heads.push(mutation.heads[1]);
        assert_rejected("heads_duplicate", mutation);
        let mut mutation = parts.clone();
        mutation.heads.swap(0, 1);
        assert_rejected("heads_unsorted", mutation);

        let mut mutation = parts.clone();
        mutation.document = None;
        assert_rejected("document_missing", mutation);
        let mut mutation = parts;
        mutation.failure = Some(super::EvaluationFailure::Cancelled);
        assert_rejected("complete_failure_present", mutation);

        assert!("AA".repeat(32).parse::<ChangeHash>().is_err());
        assert!("gg".repeat(32).parse::<ChangeHash>().is_err());
        assert!("AA".repeat(32).parse::<EventId>().is_err());
        assert!("gg".repeat(32).parse::<EventId>().is_err());
    }

    fn complete_carrier_matrix_parts() -> Option<(EvaluationReportParts, CompleteTestAuthority)> {
        let mut parts = complete_matrix_parts()?;
        let reused_bytes_event = EventId::from_bytes([1; 32]);
        let mixed_invalid_event = EventId::from_bytes([29; 32]);
        let unsupported_event = EventId::from_bytes([30; 32]);
        let accepted_hash = parts.accepted_changes[0];
        parts.change_carrier_dispositions.insert(
            0,
            (
                reused_bytes_event,
                accepted_hash,
                crate::ProtocolDisposition::Accepted,
            ),
        );
        parts.change_carrier_dispositions.push((
            mixed_invalid_event,
            accepted_hash,
            crate::ProtocolDisposition::Invalid,
        ));
        parts.disposition_records.extend([
            DispositionRecord::new(
                ProtocolItemIdentifier::event(reused_bytes_event),
                crate::ProtocolDisposition::Accepted,
                None,
            ),
            DispositionRecord::new(
                ProtocolItemIdentifier::event(mixed_invalid_event),
                crate::ProtocolDisposition::Invalid,
                Some(crate::DiagnosticCode::registered("change.actor")),
            ),
            DispositionRecord::new(
                ProtocolItemIdentifier::event(unsupported_event),
                crate::ProtocolDisposition::UnsupportedRevision,
                Some(crate::DiagnosticCode::registered("carrier.revision")),
            ),
        ]);
        parts
            .disposition_records
            .sort_by_key(DispositionRecord::identifier);
        recompute_complete_digests(&mut parts)?;
        let mut authority = CompleteTestAuthority::for_parts(&parts);
        authority.carrier_outcomes.insert(
            unsupported_event,
            AttributableCarrierOutcome::event_only(
                unsupported_event,
                crate::ProtocolDisposition::UnsupportedRevision,
                Some(crate::DiagnosticCode::registered("carrier.revision")),
            ),
        );
        Some((parts, authority))
    }

    #[test]
    fn complete_report_carrier_coverage_and_namespaces_are_exact() {
        let Some((parts, authority)) = complete_carrier_matrix_parts() else {
            return;
        };
        let accepted_hash = parts.accepted_changes[0];
        let accepted_event = EventId::from_bytes([1; 32]);
        let mixed_invalid_event = EventId::from_bytes([29; 32]);
        let unsupported_event = EventId::from_bytes([30; 32]);
        let semantic_identifier = ProtocolItemIdentifier::from(accepted_hash);
        assert_eq!(accepted_event.as_bytes(), accepted_hash.as_bytes());
        assert!(parts.disposition_records.iter().any(|record| {
            record.identifier() == ProtocolItemIdentifier::control_event(accepted_event)
        }));
        assert!(
            parts.disposition_records.iter().any(|record| {
                record.identifier() == ProtocolItemIdentifier::from(accepted_hash)
            })
        );
        assert!(parts.disposition_records.iter().any(|record| {
            record.identifier() == ProtocolItemIdentifier::event(accepted_event)
        }));
        assert_eq!(
            parts
                .dispositions
                .binary_search_by_key(&accepted_hash, |(hash, _)| *hash)
                .ok()
                .map(|index| parts.dispositions[index].1),
            Some(crate::ProtocolDisposition::Accepted)
        );
        assert_eq!(
            authority
                .carrier_outcomes
                .get(&accepted_event)
                .map(|outcome| outcome.disposition()),
            Some(crate::ProtocolDisposition::Accepted)
        );
        assert_eq!(
            authority
                .carrier_outcomes
                .get(&mixed_invalid_event)
                .map(|outcome| (outcome.change_hash(), outcome.disposition())),
            Some((Some(accepted_hash), crate::ProtocolDisposition::Invalid)),
            "an accepted aggregate does not rewrite its invalid duplicate carrier"
        );
        assert!(complete_report(parts.clone(), &authority).is_ok());
        assert_eq!(
            authority
                .carrier_outcomes
                .get(&unsupported_event)
                .and_then(|outcome| outcome.change_hash()),
            None,
            "an unsupported unverified x tag remains Event-only"
        );

        let assert_rejected = |name: &str, mutation: EvaluationReportParts| {
            assert_eq!(
                complete_report(mutation, &authority),
                Err(super::EvaluationError::ReportInvariant),
                "{name}"
            );
        };

        let mut mutation = parts.clone();
        mutation.change_carrier_dispositions.remove(0);
        assert_rejected("carrier_missing", mutation);
        let mut mutation = parts.clone();
        mutation.change_carrier_dispositions.push((
            EventId::from_bytes([31; 32]),
            accepted_hash,
            crate::ProtocolDisposition::Accepted,
        ));
        mutation.disposition_records.push(DispositionRecord::new(
            ProtocolItemIdentifier::event(EventId::from_bytes([31; 32])),
            crate::ProtocolDisposition::Accepted,
            None,
        ));
        assert_rejected("carrier_and_event_coordinated_extra", mutation);
        let mut mutation = parts.clone();
        mutation
            .change_carrier_dispositions
            .push(mutation.change_carrier_dispositions[0]);
        assert_rejected("carrier_duplicate", mutation);
        let mut mutation = parts.clone();
        mutation.change_carrier_dispositions.swap(0, 1);
        assert_rejected("carrier_unsorted", mutation);
        let mut mutation = parts.clone();
        mutation.change_carrier_dispositions[0].1 = parts.pending_changes[0];
        assert_rejected("carrier_wrong_hash", mutation);
        let mut mutation = parts.clone();
        mutation.change_carrier_dispositions[0].2 = crate::ProtocolDisposition::Invalid;
        assert_rejected("carrier_wrong_outcome", mutation);
        let mut mutation = parts.clone();
        let unrepresented_hash = ChangeHash::from_bytes([31; 32]);
        let unrepresented_event = EventId::from_bytes([31; 32]);
        mutation.change_carrier_dispositions.push((
            unrepresented_event,
            unrepresented_hash,
            crate::ProtocolDisposition::Invalid,
        ));
        mutation.disposition_records.push(DispositionRecord::new(
            ProtocolItemIdentifier::event(unrepresented_event),
            crate::ProtocolDisposition::Invalid,
            Some(crate::DiagnosticCode::registered("change.actor")),
        ));
        let forged_authority = CompleteTestAuthority::for_parts(&mutation);
        assert_eq!(
            complete_report(mutation, &forged_authority),
            Err(super::EvaluationError::ReportInvariant),
            "a verified carrier cannot omit its semantic ChangeHash record"
        );

        let accepted_event_identifier = ProtocolItemIdentifier::event(accepted_event);
        let mixed_invalid_identifier = ProtocolItemIdentifier::event(mixed_invalid_event);
        let unsupported_identifier = ProtocolItemIdentifier::event(unsupported_event);
        let mut mutation = parts.clone();
        mutation
            .disposition_records
            .retain(|record| record.identifier() != accepted_event_identifier);
        assert_rejected("carrier_with_no_event_record", mutation);
        let mut mutation = parts.clone();
        let index = mutation
            .disposition_records
            .binary_search_by_key(&mixed_invalid_identifier, DispositionRecord::identifier);
        assert!(index.is_ok());
        if let Ok(index) = index {
            mutation.disposition_records[index] = DispositionRecord::new(
                mixed_invalid_identifier,
                crate::ProtocolDisposition::Accepted,
                Some(crate::DiagnosticCode::registered("change.actor")),
            );
        }
        assert_rejected("carrier_event_outcome_mismatch", mutation);
        let mut mutation = parts.clone();
        let index = mutation
            .disposition_records
            .binary_search_by_key(&mixed_invalid_identifier, DispositionRecord::identifier);
        assert!(index.is_ok());
        if let Ok(index) = index {
            mutation.disposition_records[index] = DispositionRecord::new(
                mixed_invalid_identifier,
                crate::ProtocolDisposition::Invalid,
                Some(crate::DiagnosticCode::registered("control.parent")),
            );
        }
        assert_rejected("carrier_event_diagnostic_mismatch", mutation);
        let mut mutation = parts.clone();
        let index = mutation
            .disposition_records
            .binary_search_by_key(&semantic_identifier, DispositionRecord::identifier);
        assert!(index.is_ok());
        if let Ok(index) = index {
            mutation.disposition_records[index] = DispositionRecord::new(
                semantic_identifier,
                crate::ProtocolDisposition::Accepted,
                Some(crate::DiagnosticCode::registered("change.actor")),
            );
        }
        assert_rejected("change_hash_record_diagnostic_mismatch", mutation);
        let mut mutation = parts.clone();
        let index = mutation
            .disposition_records
            .binary_search_by_key(&unsupported_identifier, DispositionRecord::identifier);
        assert!(index.is_ok());
        if let Ok(index) = index {
            mutation.disposition_records[index] = DispositionRecord::new(
                unsupported_identifier,
                crate::ProtocolDisposition::UnsupportedRevision,
                None,
            );
        }
        assert_rejected("unsupported_event_diagnostic_missing", mutation);
        let mut mutation = parts.clone();
        let duplicate = mutation
            .disposition_records
            .binary_search_by_key(&accepted_event_identifier, DispositionRecord::identifier)
            .ok()
            .map(|index| mutation.disposition_records[index]);
        assert!(duplicate.is_some());
        if let Some(duplicate) = duplicate {
            mutation.disposition_records.push(duplicate);
            mutation
                .disposition_records
                .sort_by_key(DispositionRecord::identifier);
        }
        assert_rejected("carrier_event_record_duplicate", mutation);
        let mut mutation = parts.clone();
        let event_start = mutation.disposition_records.partition_point(|record| {
            !matches!(record.identifier(), ProtocolItemIdentifier::Event(_))
        });
        mutation
            .disposition_records
            .swap(event_start, event_start + 1);
        assert_rejected("carrier_event_record_unsorted", mutation);

        let mut mutation = parts.clone();
        let index = mutation
            .disposition_records
            .binary_search_by_key(&accepted_event_identifier, DispositionRecord::identifier);
        assert!(index.is_ok());
        if let Ok(index) = index {
            mutation.disposition_records[index] = DispositionRecord::new(
                ProtocolItemIdentifier::from(ChangeHash::from_bytes([1; 32])),
                crate::ProtocolDisposition::Accepted,
                None,
            );
        }
        mutation
            .disposition_records
            .sort_by_key(DispositionRecord::identifier);
        assert_rejected("carrier_event_retyped_as_change_hash", mutation);
        let mut mutation = parts.clone();
        let index = mutation
            .disposition_records
            .binary_search_by_key(&semantic_identifier, DispositionRecord::identifier);
        assert!(index.is_ok());
        if let Ok(index) = index {
            mutation.disposition_records[index] = DispositionRecord::new(
                ProtocolItemIdentifier::event(accepted_event),
                crate::ProtocolDisposition::Accepted,
                None,
            );
        }
        mutation
            .disposition_records
            .sort_by_key(DispositionRecord::identifier);
        assert_rejected("change_hash_retyped_as_carrier_event", mutation);

        let coordinated_hash = ChangeHash::from_bytes([31; 32]);
        let coordinated_event = EventId::from_bytes([31; 32]);
        let mut mutation = parts.clone();
        mutation
            .dispositions
            .push((coordinated_hash, crate::ProtocolDisposition::Invalid));
        mutation.invalid_changes.push(coordinated_hash);
        mutation.change_carrier_dispositions.push((
            coordinated_event,
            coordinated_hash,
            crate::ProtocolDisposition::Invalid,
        ));
        mutation.disposition_records.extend([
            DispositionRecord::new(
                ProtocolItemIdentifier::from(coordinated_hash),
                crate::ProtocolDisposition::Invalid,
                None,
            ),
            DispositionRecord::new(
                ProtocolItemIdentifier::event(coordinated_event),
                crate::ProtocolDisposition::Invalid,
                Some(crate::DiagnosticCode::registered("change.actor")),
            ),
        ]);
        mutation
            .disposition_records
            .sort_by_key(DispositionRecord::identifier);
        assert_rejected("carrier_hash_partition_records_coordinated_extra", mutation);

        let removed_hash = parts.invalid_changes[1];
        let removed_event = parts
            .change_carrier_dispositions
            .iter()
            .find_map(|(event_id, hash, _)| (*hash == removed_hash).then_some(*event_id));
        assert!(removed_event.is_some());
        let mut mutation = parts.clone();
        mutation
            .dispositions
            .retain(|(hash, _)| *hash != removed_hash);
        mutation
            .invalid_changes
            .retain(|hash| *hash != removed_hash);
        mutation
            .change_carrier_dispositions
            .retain(|(_, hash, _)| *hash != removed_hash);
        mutation.disposition_records.retain(|record| {
            record.identifier() != ProtocolItemIdentifier::from(removed_hash)
                && removed_event.is_none_or(|event_id| {
                    record.identifier() != ProtocolItemIdentifier::event(event_id)
                })
        });
        assert_rejected(
            "carrier_hash_partition_records_coordinated_missing",
            mutation,
        );

        let pending_hash = parts.pending_changes[0];
        let mut mutation = parts.clone();
        let pending_events = mutation
            .change_carrier_dispositions
            .iter()
            .filter_map(|(event_id, hash, _)| (*hash == pending_hash).then_some(*event_id))
            .collect::<BTreeSet<_>>();
        mutation
            .change_carrier_dispositions
            .retain(|(_, hash, _)| *hash != pending_hash);
        mutation.disposition_records.retain(|record| {
            !matches!(record.identifier(), ProtocolItemIdentifier::Event(event_id) if pending_events.contains(&event_id))
        });
        assert_rejected("verified_hash_without_carrier", mutation);

        let mut forged_authority = CompleteTestAuthority::for_parts(&parts);
        forged_authority
            .semantic_dispositions
            .insert(accepted_hash, crate::ProtocolDisposition::Invalid);
        forged_authority.accepted_changes.remove(&accepted_hash);
        let mut mutation = parts.clone();
        let disposition_index = mutation
            .dispositions
            .binary_search_by_key(&accepted_hash, |(hash, _)| *hash);
        assert!(disposition_index.is_ok());
        if let Ok(index) = disposition_index {
            mutation.dispositions[index].1 = crate::ProtocolDisposition::Invalid;
        }
        mutation.accepted_changes.remove(0);
        mutation.invalid_changes.insert(0, accepted_hash);
        let semantic_index = mutation
            .disposition_records
            .binary_search_by_key(&semantic_identifier, DispositionRecord::identifier);
        assert!(semantic_index.is_ok());
        if let Ok(index) = semantic_index {
            mutation.disposition_records[index] = DispositionRecord::new(
                semantic_identifier,
                crate::ProtocolDisposition::Invalid,
                None,
            );
        }
        assert_eq!(
            complete_report(mutation, &forged_authority),
            Err(super::EvaluationError::ReportInvariant),
            "an accepted carrier dominates a forged nonaccepted aggregate"
        );
    }

    #[test]
    fn incomplete_report_shape_rejects_every_nonempty_or_mismatched_field() {
        let Some(parts) = no_progress_parts(Completion::Cancelled) else {
            return;
        };
        assert!(EvaluationReport::from_no_progress_parts(parts.clone()).is_ok());

        let event = EventId::from_bytes([1; 32]);
        let second_event = EventId::from_bytes([2; 32]);
        let hash = ChangeHash::from_bytes([3; 32]);
        let disposition_record = DispositionRecord::new(
            ProtocolItemIdentifier::event(event),
            crate::ProtocolDisposition::Invalid,
            crate::DiagnosticCode::lookup("carrier.kind"),
        );
        let checkpoint = CheckpointVerificationResult::new(
            event,
            vec![second_event],
            SnapshotHash::from_bytes([4; 32]),
            Vec::new(),
            0,
            [5; 32],
            Vec::new(),
            Vec::new(),
            CheckpointVerificationStatus::PendingControl,
        );
        let mut corpus = crate::CorpusBuilder::new();
        let _ = corpus.ingest_bytes(b"{}");
        let Some(evidence) = corpus.finish().records().next() else {
            return;
        };
        let alert = crate::ControllerEquivocationAlert::new(None, vec![event, second_event], event)
            .ok()
            .map(IntegrityAlert::ControllerEquivocation);
        let Some(alert) = alert else { return };
        let Some(manifest_diagnostic) = crate::DiagnosticCode::lookup("carrier.kind") else {
            return;
        };
        let document =
            crate::automerge_adapter::materialized_view::MaterializedDocumentView::empty_for_test()
                .ok();
        let Some(document) = document else { return };

        let assert_rejected = |name: &str, mutation: EvaluationReportParts| {
            assert_eq!(
                EvaluationReport::from_no_progress_parts(mutation),
                Err(super::EvaluationError::ReportInvariant),
                "{name}"
            );
        };
        let mut mutations = Vec::new();

        let mut mutation = parts.clone();
        mutation.canonical_controls.push(event);
        mutations.push(("canonical_controls", mutation));
        let mut mutation = parts.clone();
        mutation.disposition_records.push(disposition_record);
        mutations.push(("disposition_records", mutation));
        let mut mutation = parts.clone();
        mutation
            .control_dispositions
            .push((event, crate::ProtocolDisposition::Invalid));
        mutations.push(("control_dispositions", mutation));
        let mut mutation = parts.clone();
        mutation
            .dispositions
            .push((hash, crate::ProtocolDisposition::Invalid));
        mutations.push(("dispositions", mutation));
        let mut mutation = parts.clone();
        mutation.change_carrier_dispositions.push((
            event,
            hash,
            crate::ProtocolDisposition::Invalid,
        ));
        mutations.push(("change_carrier_dispositions", mutation));
        for (name, select) in [
            ("accepted_changes", 0_u8),
            ("pending_changes", 1),
            ("excluded_changes", 2),
            ("invalid_changes", 3),
            ("heads", 4),
        ] {
            let mut mutation = parts.clone();
            match select {
                0 => mutation.accepted_changes.push(hash),
                1 => mutation.pending_changes.push(hash),
                2 => mutation.excluded_changes.push(hash),
                3 => mutation.invalid_changes.push(hash),
                _ => mutation.heads.push(hash),
            }
            mutations.push((name, mutation));
        }
        let mut mutation = parts.clone();
        mutation.evidence.push(evidence);
        mutations.push(("evidence", mutation));
        let mut mutation = parts.clone();
        mutation.checkpoints.push(checkpoint);
        mutations.push(("checkpoints", mutation));
        let mut mutation = parts.clone();
        mutation.history_digest = HistoryDigest::from_bytes([6; 32]);
        mutations.push(("history_digest", mutation));
        let mut mutation = parts.clone();
        mutation.dispositions_digest = DispositionsDigest::from_bytes([7; 32]);
        mutations.push(("dispositions_digest", mutation));
        let mut mutation = parts.clone();
        mutation.integrity_alerts.push(alert);
        mutations.push(("integrity_alerts", mutation));
        let mut mutation = parts.clone();
        mutation.manifest = crate::ResolvedManifestAvailability::Unavailable {
            event_id: event,
            control: None,
            diagnostic: manifest_diagnostic,
        };
        mutations.push(("manifest", mutation));
        let mut mutation = parts.clone();
        mutation.document = Some(document);
        mutations.push(("document", mutation));
        let mut mutation = parts.clone();
        let Some(other_coordinate) = format!("31624:{}:{}", "51".repeat(32), "52".repeat(32))
            .parse::<DocumentCoordinate>()
            .ok()
        else {
            return;
        };
        mutation.coordinate = other_coordinate;
        mutations.push(("coordinate_digest_binding", mutation));
        let mut mutation = parts.clone();
        mutation.failure = None;
        mutations.push(("missing_failure", mutation));
        let mut mutation = parts.clone();
        mutation.failure = Some(super::EvaluationFailure::BudgetExhausted);
        mutations.push(("wrong_failure", mutation));
        let mut mutation = parts.clone();
        mutation.completion = Completion::Complete;
        mutation.failure = None;
        mutations.push(("wrong_completion", mutation));

        for (name, mutation) in mutations {
            assert_rejected(name, mutation);
        }
        assert_eq!(crate::ProtocolRevision::lookup("draft_2026_09"), None);
        let authority = CompleteTestAuthority::for_parts(&parts);
        assert_eq!(
            complete_report(parts.clone(), &authority),
            Err(super::EvaluationError::ReportInvariant)
        );
        let mut complete = parts;
        complete.completion = Completion::Complete;
        complete.failure = None;
        assert_eq!(
            EvaluationReport::from_no_progress_parts(complete),
            Err(super::EvaluationError::ReportInvariant)
        );
    }

    #[test]
    fn budget_and_cancel_no_progress_reports_differ_only_by_typed_stop() {
        let Some(budget_parts) = no_progress_parts(Completion::BudgetExhausted) else {
            return;
        };
        let Some(cancel_parts) = no_progress_parts(Completion::Cancelled) else {
            return;
        };
        let budget = EvaluationReport::from_no_progress_parts(budget_parts);
        let cancelled = EvaluationReport::from_no_progress_parts(cancel_parts);
        assert!(budget.is_ok() && cancelled.is_ok());
        let (Ok(budget), Ok(cancelled)) = (budget, cancelled) else {
            return;
        };
        assert_eq!(budget.coordinate(), cancelled.coordinate());
        assert_eq!(budget.revision(), cancelled.revision());
        assert_eq!(budget.canonical_controls(), cancelled.canonical_controls());
        assert_eq!(
            budget.disposition_records(),
            cancelled.disposition_records()
        );
        assert_eq!(
            budget.control_dispositions(),
            cancelled.control_dispositions()
        );
        assert_eq!(budget.dispositions(), cancelled.dispositions());
        assert_eq!(budget.accepted_changes(), cancelled.accepted_changes());
        assert_eq!(budget.pending_changes(), cancelled.pending_changes());
        assert_eq!(budget.excluded_changes(), cancelled.excluded_changes());
        assert_eq!(budget.invalid_changes(), cancelled.invalid_changes());
        assert_eq!(budget.heads(), cancelled.heads());
        assert_eq!(budget.evidence(), cancelled.evidence());
        assert_eq!(budget.checkpoints(), cancelled.checkpoints());
        assert_eq!(budget.history_digest(), cancelled.history_digest());
        assert_eq!(
            budget.dispositions_digest(),
            cancelled.dispositions_digest()
        );
        assert_eq!(budget.integrity_alerts(), cancelled.integrity_alerts());
        assert_eq!(budget.manifest(), cancelled.manifest());
        assert_eq!(budget.document(), cancelled.document());
        assert_eq!(budget.completion(), Completion::BudgetExhausted);
        assert_eq!(cancelled.completion(), Completion::Cancelled);
        assert_eq!(
            budget.failure(),
            Some(super::EvaluationFailure::BudgetExhausted)
        );
        assert_eq!(
            cancelled.failure(),
            Some(super::EvaluationFailure::Cancelled)
        );
    }

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
        let authority = CompleteTestAuthority::for_parts(&parts());
        let report = complete_report(parts(), &authority);
        assert!(report.is_err());

        let mut invalid = parts();
        invalid.accepted_changes = vec![
            ChangeHash::from_bytes([2; 32]),
            ChangeHash::from_bytes([1; 32]),
        ];
        assert!(complete_report(invalid, &authority).is_err());

        let mut incomplete_with_document = parts();
        incomplete_with_document.completion = Completion::Cancelled;
        incomplete_with_document.failure = Some(super::EvaluationFailure::Cancelled);
        assert!(EvaluationReport::from_no_progress_parts(incomplete_with_document).is_err());
    }

    #[test]
    fn report_invariant_mutations_return_typed_errors() {
        let coordinate =
            format!("31624:{}:{}", "31".repeat(32), "32".repeat(32)).parse::<DocumentCoordinate>();
        assert!(coordinate.is_ok());
        let Ok(coordinate) = coordinate else { return };
        let hash = ChangeHash::from_bytes([2; 32]);
        let carrier_id = EventId::from_bytes([9; 32]);
        let canonical_controls = vec![EventId::from_bytes([1; 32])];
        let disposition_records = vec![
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
        let history_digest = crate::canonical_history_digest(
            crate::ProtocolRevision::draft_v1(),
            coordinate,
            &canonical_controls,
            &[hash],
            &[hash],
        );
        let dispositions_digest = crate::canonical_dispositions_digest(
            crate::ProtocolRevision::draft_v1(),
            coordinate,
            &disposition_records,
        );
        assert!(history_digest.is_ok() && dispositions_digest.is_ok());
        let (Ok(history_digest), Ok(dispositions_digest)) = (history_digest, dispositions_digest)
        else {
            return;
        };
        let parts = || {
            EvaluationReportParts {
            coordinate,
            revision: crate::ProtocolRevision::draft_v1(),
            canonical_controls: canonical_controls.clone(),
            disposition_records: disposition_records.clone(),
            control_dispositions: vec![(
                EventId::from_bytes([1; 32]),
                crate::ProtocolDisposition::Accepted,
            )],
            dispositions: vec![(hash, crate::ProtocolDisposition::Accepted)],
            change_carrier_dispositions: vec![(
                carrier_id,
                hash,
                crate::ProtocolDisposition::Accepted,
            )],
            accepted_changes: vec![hash],
            pending_changes: vec![],
            excluded_changes: vec![],
            invalid_changes: vec![],
            heads: vec![hash],
            evidence: vec![],
            checkpoints: vec![],
            history_digest,
            dispositions_digest,
            integrity_alerts: vec![],
            manifest: crate::ResolvedManifestAvailability::Missing,
            completion: Completion::Complete,
            failure: None,
            document: crate::automerge_adapter::materialized_view::MaterializedDocumentView::empty_for_test().ok(),
        }
        };
        let authority = CompleteTestAuthority::for_parts(&parts());
        assert!(complete_report(parts(), &authority).is_ok());

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
        assert!(complete_report(carrier_consistent.clone(), &authority).is_ok());

        let mut missing_carrier_record = carrier_consistent.clone();
        missing_carrier_record.disposition_records.pop();
        assert!(complete_report(missing_carrier_record, &authority).is_err());

        let mut duplicate_carrier = carrier_consistent.clone();
        duplicate_carrier.change_carrier_dispositions.push((
            carrier_id,
            hash,
            crate::ProtocolDisposition::Accepted,
        ));
        assert!(complete_report(duplicate_carrier, &authority).is_err());

        let mut wrong_carrier_hash = carrier_consistent;
        wrong_carrier_hash.change_carrier_dispositions[0].1 = ChangeHash::from_bytes([8; 32]);
        assert!(complete_report(wrong_carrier_hash, &authority).is_err());

        let assert_invariant = |parts| {
            assert_eq!(
                complete_report(parts, &authority),
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
        completion_mismatch.failure = Some(super::EvaluationFailure::Cancelled);
        assert_invariant(completion_mismatch);

        let mut document_mismatch = parts();
        document_mismatch.document = None;
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
        inconsistent_checkpoint.disposition_records.extend([
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
        ]);
        assert_invariant(inconsistent_checkpoint);

        let mut consistent_checkpoint = parts();
        consistent_checkpoint.checkpoints.push(checkpoint);
        consistent_checkpoint.disposition_records.extend([
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
        ]);
        assert!(recompute_complete_digests(&mut consistent_checkpoint).is_some());
        let consistent_checkpoint_authority =
            CompleteTestAuthority::for_parts(&consistent_checkpoint);
        assert!(complete_report(consistent_checkpoint, &consistent_checkpoint_authority).is_ok());

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
        consistent_refusal.disposition_records.extend([
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
        ]);
        assert!(recompute_complete_digests(&mut consistent_refusal).is_some());
        let consistent_refusal_authority = CompleteTestAuthority::for_parts(&consistent_refusal);
        assert!(complete_report(consistent_refusal.clone(), &consistent_refusal_authority).is_ok());

        let mut missing_descriptor_diagnostic = consistent_refusal.clone();
        missing_descriptor_diagnostic.disposition_records[2] = DispositionRecord::new(
            ProtocolItemIdentifier::event(descriptor),
            crate::ProtocolDisposition::Invalid,
            None,
        );
        assert_invariant(missing_descriptor_diagnostic);

        let mut wrong_chunk_diagnostic = consistent_refusal;
        wrong_chunk_diagnostic.disposition_records[3] = DispositionRecord::new(
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
    #[allow(clippy::bool_assert_comparison)]
    fn finding_081_incomplete_report_rejects_canonical_cross_view_state() {
        let coordinate =
            format!("31624:{}:{}", "41".repeat(32), "42".repeat(32)).parse::<DocumentCoordinate>();
        assert!(coordinate.is_ok());
        let Ok(coordinate) = coordinate else { return };
        let control = EventId::from_bytes([1; 32]);
        let hash = ChangeHash::from_bytes([2; 32]);
        let report = EvaluationReport::from_no_progress_parts(EvaluationReportParts {
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
        });
        assert_eq!(
            report.is_err(),
            true,
            "FINDING_081 regression: incomplete report parts must reject canonical state and arbitrary digests"
        );
    }
}
