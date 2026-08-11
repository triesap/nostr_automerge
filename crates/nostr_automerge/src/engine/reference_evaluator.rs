use crate::carrier::VerifiedCarrier;
use crate::checkpoint::authorize::{DescriptorAuthorization, authorize_descriptor};
use crate::checkpoint::join::{JoinError, join_chunks};
use crate::checkpoint::{HistoryVerificationError, historical_carrier_coverage};
use crate::conformance::dispositions_digest::{disposition_items, dispositions_digest};
use crate::conformance::history_digest::history_digest;
use crate::control::candidate::{
    CandidateResult, evaluate_account_continuity, evaluate_device_ancestry,
    evaluate_parent_continuity, evaluate_role_continuity, evaluate_terminal_continuity,
};
use crate::control::genesis::classify_genesis;
use crate::control::reorganization::{ControlChainSummary, detect_reorganization};
use crate::control::validate::ControlEnvelope;
use crate::evidence::corpus_builder::ManifestSelectionState;
use crate::evidence::event::EventEvidence;
use crate::graph::change_candidate::{CandidateCarrier, ChangeCandidate};
use crate::reference::epoch_engine::AcceptedAtControl;
use crate::reference::evaluate::{BatchChange, BatchControl, evaluate_batch};
use crate::types::role::Role;
use crate::{
    CancellationCheck, ChangeHash, CheckpointVerificationResult, CheckpointVerificationStatus,
    Completion, DocumentCoordinate, EvidenceCorpus, EvidenceIdentifier, EvidenceStatus,
    ManifestControlStatus, ManifestPendingReason, ProtocolDisposition, ProtocolRevision,
    ResolvedManifestAvailability, WorkBudget, WorkCounter,
};

use super::evaluation_report::{
    DispositionRecord, EvaluationError, EvaluationFailure, EvaluationReport, EvaluationReportParts,
    ProtocolItemIdentifier,
};
use crate::automerge_adapter::materialized_view::MaterializedDocumentView;

/// Stateless deterministic batch evaluator for immutable signed evidence.
///
/// Evaluation performs no networking, storage, clock access, signing, or key
/// custody. Callers supply an immutable evidence corpus plus explicit local
/// work and cancellation policy to the evaluation operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferenceEvaluator {
    revision: ProtocolRevision,
}

impl ReferenceEvaluator {
    /// Creates an evaluator for the sealed protocol revision.
    #[must_use]
    pub const fn new(revision: ProtocolRevision) -> Self {
        Self { revision }
    }

    /// Returns the sealed revision interpreted by this evaluator.
    #[must_use]
    pub const fn revision(&self) -> ProtocolRevision {
        self.revision
    }

    /// Evaluates one document coordinate from fully retained signed evidence.
    ///
    /// The operation performs no I/O and derives every control and change input
    /// from validated carriers in `corpus`. Local exhaustion or cancellation is
    /// represented by [`crate::Completion`] without changing dispositions.
    #[must_use = "evaluation reports and typed errors must be handled"]
    pub fn evaluate(
        &self,
        corpus: &EvidenceCorpus,
        coordinate: DocumentCoordinate,
        budget: &mut WorkBudget,
        cancellation: &impl CancellationCheck,
    ) -> Result<EvaluationReport, EvaluationError> {
        let ingress_complete = charge_ingress(corpus, budget);
        let (controls, preliminary_control_dispositions) = if ingress_complete {
            prepare_controls(corpus, coordinate)
        } else {
            (Vec::new(), std::collections::BTreeMap::new())
        };
        let mut batch = evaluate_batch(controls, budget, cancellation);
        if !ingress_complete {
            batch.completion = Completion::BudgetExhausted;
            batch.failure = Some(EvaluationFailure::BudgetExhausted);
            batch.materialized_document = None;
        }
        if !matches!(
            batch.failure,
            None | Some(EvaluationFailure::BudgetExhausted | EvaluationFailure::Cancelled)
        ) {
            return Err(match batch.failure {
                Some(EvaluationFailure::Graph) => EvaluationError::Graph,
                Some(EvaluationFailure::Decode) => EvaluationError::Decode,
                Some(EvaluationFailure::Apply) => EvaluationError::Apply,
                Some(
                    EvaluationFailure::InvalidEvidence
                    | EvaluationFailure::InvariantViolation
                    | EvaluationFailure::BudgetExhausted
                    | EvaluationFailure::Cancelled,
                )
                | None => EvaluationError::ReportInvariant,
            });
        }
        let canonical_controls = batch.canonical_controls;
        let mut control_disposition_map = preliminary_control_dispositions;
        control_disposition_map.extend(batch.control_dispositions);
        let manifest = resolve_selected_manifest(
            corpus,
            coordinate,
            &control_disposition_map,
            &batch.statefully_valid_controls,
        );
        let control_dispositions = control_disposition_map.into_iter().collect::<Vec<_>>();
        let mut disposition_records = control_dispositions
            .iter()
            .map(|(event_id, disposition)| {
                DispositionRecord::new(
                    ProtocolItemIdentifier::control_event(*event_id),
                    *disposition,
                    None,
                )
            })
            .collect::<Vec<_>>();
        let checkpoints = match batch.completion {
            Completion::Complete => verify_checkpoints(
                corpus,
                coordinate,
                &canonical_controls,
                &batch.accepted_at_control,
                budget,
                cancellation,
            ),
            Completion::BudgetExhausted => checkpoint_refusals(
                corpus,
                coordinate,
                CheckpointVerificationStatus::BudgetExhausted,
            ),
            Completion::Cancelled => {
                checkpoint_refusals(corpus, coordinate, CheckpointVerificationStatus::Cancelled)
            }
        };
        if checkpoints
            .iter()
            .any(|checkpoint| checkpoint.status() == CheckpointVerificationStatus::Cancelled)
        {
            batch.completion = Completion::Cancelled;
            batch.failure = Some(EvaluationFailure::Cancelled);
        } else if checkpoints
            .iter()
            .any(|checkpoint| checkpoint.status() == CheckpointVerificationStatus::BudgetExhausted)
        {
            batch.completion = Completion::BudgetExhausted;
            batch.failure = Some(EvaluationFailure::BudgetExhausted);
        }
        let dispositions = batch.dispositions.into_iter().collect::<Vec<_>>();
        disposition_records.extend(dispositions.iter().map(|(hash, disposition)| {
            DispositionRecord::new(ProtocolItemIdentifier::from(*hash), *disposition, None)
        }));
        disposition_records.extend(event_disposition_records(corpus, &manifest, &checkpoints));
        let accepted_changes = disposition_hashes(&dispositions, ProtocolDisposition::Accepted);
        let heads = batch.heads.into_iter().collect::<Vec<_>>();
        let pending_changes = disposition_hashes(&dispositions, ProtocolDisposition::Pending);
        let excluded_changes = disposition_hashes(&dispositions, ProtocolDisposition::Excluded);
        let invalid_changes = disposition_hashes(&dispositions, ProtocolDisposition::Invalid);
        let history_digest = history_digest(
            self.revision,
            coordinate,
            &canonical_controls,
            &accepted_changes,
            &heads,
        )
        .map_err(|_| EvaluationError::ReportInvariant)?;
        let disposition_items = disposition_items(&disposition_records)
            .map_err(|_| EvaluationError::ReportInvariant)?;
        let dispositions_digest =
            dispositions_digest(self.revision, coordinate, &disposition_items)
                .map_err(|_| EvaluationError::ReportInvariant)?;
        let projection = match batch.completion {
            Completion::Complete => {
                project_document(batch.materialized_document, budget, cancellation)
            }
            Completion::BudgetExhausted => {
                Err(crate::automerge_adapter::materialized_view::ProjectionError::Budget)
            }
            Completion::Cancelled => {
                Err(crate::automerge_adapter::materialized_view::ProjectionError::Cancelled)
            }
        };
        let document = match projection {
            Ok(document) => document,
            Err(crate::automerge_adapter::materialized_view::ProjectionError::Budget) => {
                batch.completion = Completion::BudgetExhausted;
                batch.failure = Some(EvaluationFailure::BudgetExhausted);
                None
            }
            Err(crate::automerge_adapter::materialized_view::ProjectionError::Cancelled) => {
                batch.completion = Completion::Cancelled;
                batch.failure = Some(EvaluationFailure::Cancelled);
                None
            }
            Err(crate::automerge_adapter::materialized_view::ProjectionError::Invalid) => {
                return Err(EvaluationError::Projection);
            }
        };
        EvaluationReport::from_parts(EvaluationReportParts {
            coordinate,
            canonical_controls,
            disposition_records,
            control_dispositions,
            dispositions,
            accepted_changes,
            pending_changes,
            excluded_changes,
            invalid_changes,
            heads,
            evidence: corpus.records().collect(),
            checkpoints,
            history_digest,
            dispositions_digest,
            integrity_alerts: batch.integrity_alerts,
            manifest,
            completion: batch.completion,
            failure: batch.failure,
            document,
        })
        .map_err(|_| EvaluationError::ReportInvariant)
    }

    /// Replays the complete retained corpus and reports a canonical branch
    /// change relative to a prior report for the same coordinate.
    #[must_use = "reevaluation reports and typed errors must be handled"]
    pub fn reevaluate(
        &self,
        corpus: &EvidenceCorpus,
        coordinate: DocumentCoordinate,
        previous: &EvaluationReport,
        budget: &mut WorkBudget,
        cancellation: &impl CancellationCheck,
    ) -> Result<EvaluationReport, EvaluationError> {
        let mut current = self.evaluate(corpus, coordinate, budget, cancellation)?;
        if previous.coordinate() != coordinate {
            return Ok(current);
        }
        let summarize = |report: &EvaluationReport| {
            let mut changes_by_control = std::collections::BTreeMap::new();
            if let Some(tip) = report.canonical_controls().last().copied() {
                changes_by_control.insert(tip, report.accepted_changes().iter().copied().collect());
            }
            ControlChainSummary {
                controls: report.canonical_controls().to_vec(),
                changes_by_control,
            }
        };
        if let Some(alert) = detect_reorganization(&summarize(previous), &summarize(&current)) {
            current.push_integrity_alert(alert);
        }
        Ok(current)
    }
}

fn resolve_selected_manifest(
    corpus: &EvidenceCorpus,
    coordinate: DocumentCoordinate,
    control_dispositions: &std::collections::BTreeMap<crate::EventId, ProtocolDisposition>,
    statefully_valid_controls: &std::collections::BTreeSet<crate::EventId>,
) -> ResolvedManifestAvailability {
    let Some(selection) = corpus.selected_manifest_selection(coordinate) else {
        return ResolvedManifestAvailability::Missing;
    };
    let hints = match selection.state {
        ManifestSelectionState::Available(hints) => hints,
        ManifestSelectionState::Unavailable(diagnostic) => {
            return ResolvedManifestAvailability::Unavailable {
                event_id: selection.event_id,
                control: None,
                diagnostic,
            };
        }
    };
    let control_id = hints.control();
    let control = match corpus.events.get(&control_id) {
        Some(EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::Control(control),
            ..
        }) => control,
        Some(EventEvidence::InvalidCarrier { diagnostic, .. })
        | Some(EventEvidence::UnsupportedRevision { diagnostic, .. }) => {
            return ResolvedManifestAvailability::Unavailable {
                event_id: selection.event_id,
                control: Some(control_id),
                diagnostic: *diagnostic,
            };
        }
        Some(_) => {
            return ResolvedManifestAvailability::Unavailable {
                event_id: selection.event_id,
                control: Some(control_id),
                diagnostic: crate::DiagnosticCode::registered("carrier.kind"),
            };
        }
        None => {
            return ResolvedManifestAvailability::Pending {
                hints,
                reason: ManifestPendingReason::MissingControl,
            };
        }
    };
    if control.coordinate() != coordinate {
        return ResolvedManifestAvailability::Unavailable {
            event_id: selection.event_id,
            control: Some(control_id),
            diagnostic: crate::DiagnosticCode::registered("carrier.coordinate"),
        };
    }
    match control_dispositions.get(&control_id).copied() {
        Some(ProtocolDisposition::Accepted) => ResolvedManifestAvailability::Available {
            hints,
            control_status: ManifestControlStatus::Canonical,
        },
        Some(ProtocolDisposition::Excluded) if statefully_valid_controls.contains(&control_id) => {
            ResolvedManifestAvailability::Available {
                hints,
                control_status: ManifestControlStatus::Noncanonical,
            }
        }
        Some(ProtocolDisposition::Invalid | ProtocolDisposition::UnsupportedRevision) => {
            ResolvedManifestAvailability::Unavailable {
                event_id: selection.event_id,
                control: Some(control_id),
                diagnostic: crate::DiagnosticCode::registered("control.structure"),
            }
        }
        Some(ProtocolDisposition::Pending | ProtocolDisposition::Excluded) | None => {
            ResolvedManifestAvailability::Pending {
                hints,
                reason: ManifestPendingReason::ControlPending,
            }
        }
    }
}

fn project_document(
    canonical_bytes: Option<Vec<u8>>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<
    Option<MaterializedDocumentView>,
    crate::automerge_adapter::materialized_view::ProjectionError,
> {
    canonical_bytes
        .map(|bytes| {
            MaterializedDocumentView::from_canonical_bytes_metered(bytes, budget, cancellation)
        })
        .transpose()
}

fn event_disposition_records(
    corpus: &EvidenceCorpus,
    manifest: &ResolvedManifestAvailability,
    checkpoints: &[CheckpointVerificationResult],
) -> Vec<DispositionRecord> {
    let represented_events = corpus
        .control_ids()
        .chain(
            corpus
                .indexes
                .changes
                .carriers_by_hash
                .values()
                .flat_map(|event_ids| event_ids.iter().copied()),
        )
        .collect::<std::collections::BTreeSet<_>>();
    let mut records = corpus
        .records()
        .filter_map(|record| {
            let EvidenceIdentifier::Event(event_id) = record.identifier() else {
                return None;
            };
            if represented_events.contains(&event_id) {
                return None;
            }
            if matches!(
                corpus.events.get(&event_id),
                Some(EventEvidence::VerifiedCarrier {
                    carrier: VerifiedCarrier::Manifest(_)
                        | VerifiedCarrier::CheckpointDescriptor(_)
                        | VerifiedCarrier::CheckpointChunk(_),
                    ..
                })
            ) {
                return None;
            }
            let disposition = match record.status() {
                EvidenceStatus::Valid => ProtocolDisposition::Accepted,
                EvidenceStatus::Pending => ProtocolDisposition::Pending,
                EvidenceStatus::Invalid => ProtocolDisposition::Invalid,
                EvidenceStatus::Unsupported => ProtocolDisposition::UnsupportedRevision,
                EvidenceStatus::Irrelevant | EvidenceStatus::Duplicate => return None,
            };
            Some((event_id, (disposition, record.diagnostic())))
        })
        .collect::<std::collections::BTreeMap<_, _>>();

    for evidence in corpus.events.values() {
        let EventEvidence::VerifiedCarrier { carrier, .. } = evidence else {
            continue;
        };
        match carrier {
            VerifiedCarrier::Manifest(manifest) => {
                records.insert(manifest.event_id, (ProtocolDisposition::Excluded, None));
            }
            VerifiedCarrier::CheckpointDescriptor(descriptor) => {
                let disposition = if corpus
                    .indexes
                    .checkpoints
                    .pending_descriptors
                    .contains(&descriptor.event_id())
                {
                    ProtocolDisposition::Pending
                } else {
                    ProtocolDisposition::Excluded
                };
                records.insert(descriptor.event_id(), (disposition, None));
            }
            VerifiedCarrier::CheckpointChunk(chunk) => {
                let disposition = if corpus
                    .indexes
                    .checkpoints
                    .pending_chunks
                    .contains(&chunk.event_id())
                {
                    ProtocolDisposition::Pending
                } else {
                    ProtocolDisposition::Excluded
                };
                records.insert(chunk.event_id(), (disposition, None));
            }
            VerifiedCarrier::Control(_)
            | VerifiedCarrier::Change(_)
            | VerifiedCarrier::UnsupportedRevision { .. } => {}
        }
    }

    match manifest {
        ResolvedManifestAvailability::Missing => {}
        ResolvedManifestAvailability::Available { hints, .. } => {
            records.insert(hints.event_id(), (ProtocolDisposition::Accepted, None));
        }
        ResolvedManifestAvailability::Pending { hints, .. } => {
            records.insert(hints.event_id(), (ProtocolDisposition::Pending, None));
        }
        ResolvedManifestAvailability::Unavailable {
            event_id,
            diagnostic,
            ..
        } => {
            let disposition = if diagnostic.as_str() == "carrier.revision" {
                ProtocolDisposition::UnsupportedRevision
            } else {
                ProtocolDisposition::Invalid
            };
            records.insert(*event_id, (disposition, Some(*diagnostic)));
        }
    }

    for checkpoint in checkpoints {
        let disposition = checkpoint_event_disposition(checkpoint.status());
        let diagnostic = checkpoint_event_diagnostic(checkpoint.status());
        records.insert(checkpoint.descriptor_event(), (disposition, diagnostic));
        for event_id in checkpoint.chunk_events() {
            records.insert(*event_id, (disposition, diagnostic));
        }
    }

    records
        .into_iter()
        .map(|(event_id, (disposition, diagnostic))| {
            DispositionRecord::new(
                ProtocolItemIdentifier::event(event_id),
                disposition,
                diagnostic,
            )
        })
        .collect()
}

fn checkpoint_event_disposition(status: CheckpointVerificationStatus) -> ProtocolDisposition {
    match status {
        CheckpointVerificationStatus::Verified => ProtocolDisposition::Accepted,
        CheckpointVerificationStatus::PendingControl
        | CheckpointVerificationStatus::MissingChunk
        | CheckpointVerificationStatus::MissingHistoricalCarrier
        | CheckpointVerificationStatus::BudgetExhausted
        | CheckpointVerificationStatus::Cancelled => ProtocolDisposition::Pending,
        CheckpointVerificationStatus::Unauthorized
        | CheckpointVerificationStatus::ChunkAuthorMismatch
        | CheckpointVerificationStatus::ChunkCoordinateMismatch
        | CheckpointVerificationStatus::ChunkDescriptorMismatch
        | CheckpointVerificationStatus::ChunkCountMismatch
        | CheckpointVerificationStatus::DuplicateChunk
        | CheckpointVerificationStatus::ChunkSizeMismatch
        | CheckpointVerificationStatus::ChunkAssemblyMismatch
        | CheckpointVerificationStatus::MerkleMismatch
        | CheckpointVerificationStatus::SnapshotSizeMismatch
        | CheckpointVerificationStatus::SnapshotHashMismatch
        | CheckpointVerificationStatus::SnapshotLoad
        | CheckpointVerificationStatus::HeadMismatch
        | CheckpointVerificationStatus::CommitmentMismatch
        | CheckpointVerificationStatus::ClosureMismatch
        | CheckpointVerificationStatus::NotAcceptedAtControl => ProtocolDisposition::Invalid,
    }
}

fn checkpoint_event_diagnostic(
    status: CheckpointVerificationStatus,
) -> Option<crate::DiagnosticCode> {
    let code = match status {
        CheckpointVerificationStatus::Verified
        | CheckpointVerificationStatus::PendingControl
        | CheckpointVerificationStatus::MissingChunk
        | CheckpointVerificationStatus::BudgetExhausted
        | CheckpointVerificationStatus::Cancelled => return None,
        CheckpointVerificationStatus::ChunkAuthorMismatch
        | CheckpointVerificationStatus::ChunkCoordinateMismatch
        | CheckpointVerificationStatus::ChunkDescriptorMismatch
        | CheckpointVerificationStatus::ChunkCountMismatch
        | CheckpointVerificationStatus::DuplicateChunk
        | CheckpointVerificationStatus::ChunkSizeMismatch
        | CheckpointVerificationStatus::ChunkAssemblyMismatch => "checkpoint.chunk",
        CheckpointVerificationStatus::MerkleMismatch => "checkpoint.merkle",
        CheckpointVerificationStatus::SnapshotSizeMismatch
        | CheckpointVerificationStatus::SnapshotHashMismatch
        | CheckpointVerificationStatus::SnapshotLoad => "checkpoint.snapshot",
        CheckpointVerificationStatus::HeadMismatch => "checkpoint.heads",
        CheckpointVerificationStatus::Unauthorized
        | CheckpointVerificationStatus::CommitmentMismatch
        | CheckpointVerificationStatus::ClosureMismatch
        | CheckpointVerificationStatus::MissingHistoricalCarrier
        | CheckpointVerificationStatus::NotAcceptedAtControl => "checkpoint.history",
    };
    Some(crate::DiagnosticCode::registered(code))
}

fn verify_checkpoints(
    corpus: &EvidenceCorpus,
    coordinate: DocumentCoordinate,
    canonical_controls: &[crate::EventId],
    accepted_at_control: &std::collections::BTreeMap<crate::EventId, AcceptedAtControl>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Vec<CheckpointVerificationResult> {
    let prepared = (|| {
        charge_checkpoint_work(
            budget,
            cancellation,
            u64::try_from(canonical_controls.len()).unwrap_or(u64::MAX),
        )?;
        let canonical_set = canonical_controls.iter().copied().collect();
        let authorizations =
            checkpoint_authorizations(corpus, coordinate, &canonical_set, budget, cancellation)?;
        let chunk_sets = checkpoint_chunk_sets(corpus, coordinate, budget, cancellation)?;
        let carrier_coverage = checkpoint_carrier_coverage(
            corpus,
            coordinate,
            canonical_controls,
            budget,
            cancellation,
        )?;
        let accepted_history = checkpoint_accepted_history(
            corpus,
            coordinate,
            accepted_at_control,
            budget,
            cancellation,
        )?;
        Ok::<_, CheckpointWorkStop>((
            authorizations,
            chunk_sets,
            carrier_coverage,
            accepted_history,
        ))
    })();
    let (authorizations, chunk_sets, carrier_coverage, accepted_history) = match prepared {
        Ok(prepared) => prepared,
        Err(stop) => return checkpoint_refusals(corpus, coordinate, stop.status()),
    };
    let mut stopped: Option<CheckpointVerificationStatus> = None;
    corpus
        .events
        .values()
        .filter_map(|evidence| match evidence {
            EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::CheckpointDescriptor(descriptor),
                ..
            } if descriptor.coordinate() == coordinate => Some(descriptor.as_ref()),
            _ => None,
        })
        .map(|descriptor| {
            let descriptor_id = descriptor.event_id();
            if let Some(status) = stopped {
                let commitments = descriptor.descriptor();
                return CheckpointVerificationResult::new(
                    descriptor_id,
                    Vec::new(),
                    descriptor.snapshot_hash(),
                    commitments.heads.iter().copied().collect(),
                    commitments.change_count,
                    commitments.change_set_hash,
                    Vec::new(),
                    Vec::new(),
                    status,
                );
            }
            let chunk_events = checkpoint_chunk_event_ids(corpus, descriptor_id);
            let coverage_result = carrier_coverage.get(&descriptor_id);
            let accepted_result = accepted_history.get(&descriptor_id);
            let coverage = coverage_result
                .and_then(|value| value.as_ref().ok())
                .cloned()
                .unwrap_or_default();
            let accepted = accepted_result
                .and_then(|value| value.as_ref().ok())
                .cloned()
                .unwrap_or_default();
            let history_refusal = coverage_result
                .and_then(|result| result.as_ref().err())
                .or_else(|| accepted_result.and_then(|result| result.as_ref().err()))
                .map(history_refusal_status);
            let status = history_refusal.unwrap_or_else(|| {
                verify_one_checkpoint(
                    descriptor,
                    authorizations.get(&descriptor_id).copied(),
                    chunk_sets.get(&descriptor_id),
                    &coverage,
                    &accepted,
                    budget,
                    cancellation,
                )
            });
            if matches!(
                status,
                CheckpointVerificationStatus::BudgetExhausted
                    | CheckpointVerificationStatus::Cancelled
            ) {
                stopped = Some(status);
            }
            let commitments = descriptor.descriptor();
            CheckpointVerificationResult::new(
                descriptor_id,
                chunk_events,
                descriptor.snapshot_hash(),
                commitments.heads.iter().copied().collect(),
                commitments.change_count,
                commitments.change_set_hash,
                coverage.into_iter().collect(),
                accepted.into_iter().collect(),
                status,
            )
        })
        .collect()
}

const fn history_refusal_status(error: &HistoryVerificationError) -> CheckpointVerificationStatus {
    match error {
        HistoryVerificationError::UnknownControl => CheckpointVerificationStatus::PendingControl,
        HistoryVerificationError::Budget => CheckpointVerificationStatus::BudgetExhausted,
        HistoryVerificationError::Cancelled => CheckpointVerificationStatus::Cancelled,
        HistoryVerificationError::MissingCarrier => {
            CheckpointVerificationStatus::MissingHistoricalCarrier
        }
        HistoryVerificationError::NotAccepted => CheckpointVerificationStatus::NotAcceptedAtControl,
        HistoryVerificationError::Snapshot => CheckpointVerificationStatus::SnapshotLoad,
    }
}

fn checkpoint_chunk_event_ids(
    corpus: &EvidenceCorpus,
    descriptor_id: crate::EventId,
) -> Vec<crate::EventId> {
    corpus
        .indexes
        .checkpoints
        .chunks_by_descriptor
        .get(&descriptor_id)
        .into_iter()
        .flatten()
        .copied()
        .collect()
}

fn verify_one_checkpoint(
    descriptor: &crate::carrier::checkpoint_descriptor::ValidatedCheckpointDescriptorCarrier,
    authorization: Option<DescriptorAuthorization>,
    chunks: Option<&Result<Vec<crate::checkpoint::CheckpointChunk>, JoinError>>,
    coverage: &std::collections::BTreeSet<ChangeHash>,
    accepted: &std::collections::BTreeSet<ChangeHash>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> CheckpointVerificationStatus {
    use crate::checkpoint::{HistoryVerificationError, VerifyError};
    if cancellation.is_cancelled() {
        return CheckpointVerificationStatus::Cancelled;
    }
    match authorization {
        Some(DescriptorAuthorization::Authorized) => {}
        Some(DescriptorAuthorization::PendingControl) | None => {
            return CheckpointVerificationStatus::PendingControl;
        }
        Some(DescriptorAuthorization::Invalid) => {
            return CheckpointVerificationStatus::Unauthorized;
        }
    }
    if budget.charge_checkpoint_items(1).is_err() {
        return CheckpointVerificationStatus::BudgetExhausted;
    }
    let mut chunks = match chunks {
        Some(Ok(chunks)) => chunks.clone(),
        Some(Err(error)) => return join_status(*error),
        None => return CheckpointVerificationStatus::MissingChunk,
    };
    let bytes = match crate::checkpoint::assemble_chunks(
        descriptor.descriptor(),
        &mut chunks,
        budget,
        cancellation,
    ) {
        Ok(bytes) => bytes,
        Err(error) => return assembly_status(error),
    };
    let snapshot = match crate::checkpoint::verify_snapshot_heads(
        &bytes,
        descriptor.descriptor(),
        budget,
        cancellation,
    ) {
        Ok(snapshot) => snapshot,
        Err(VerifyError::Budget) => return CheckpointVerificationStatus::BudgetExhausted,
        Err(VerifyError::Cancelled) => return CheckpointVerificationStatus::Cancelled,
        Err(VerifyError::Load) => return CheckpointVerificationStatus::SnapshotLoad,
        Err(VerifyError::Heads) => return CheckpointVerificationStatus::HeadMismatch,
        Err(VerifyError::Commitments) => return CheckpointVerificationStatus::CommitmentMismatch,
        Err(VerifyError::Closure) => return CheckpointVerificationStatus::ClosureMismatch,
    };
    if let Err(error) = snapshot.verify_commitments(descriptor.descriptor(), budget) {
        return match error {
            VerifyError::Budget => CheckpointVerificationStatus::BudgetExhausted,
            VerifyError::Cancelled => CheckpointVerificationStatus::Cancelled,
            VerifyError::Load => CheckpointVerificationStatus::SnapshotLoad,
            VerifyError::Commitments => CheckpointVerificationStatus::CommitmentMismatch,
            VerifyError::Heads => CheckpointVerificationStatus::HeadMismatch,
            VerifyError::Closure => CheckpointVerificationStatus::ClosureMismatch,
        };
    }
    if let Err(error) = snapshot.verify_exact_closure_metered(budget) {
        return match error {
            VerifyError::Budget => CheckpointVerificationStatus::BudgetExhausted,
            VerifyError::Cancelled => CheckpointVerificationStatus::Cancelled,
            _ => CheckpointVerificationStatus::ClosureMismatch,
        };
    }
    match crate::checkpoint::verify_full_history_metered(
        &snapshot,
        coverage,
        accepted,
        budget,
        cancellation,
    ) {
        Ok(()) => CheckpointVerificationStatus::Verified,
        Err(HistoryVerificationError::MissingCarrier) => {
            CheckpointVerificationStatus::MissingHistoricalCarrier
        }
        Err(HistoryVerificationError::NotAccepted) => {
            CheckpointVerificationStatus::NotAcceptedAtControl
        }
        Err(HistoryVerificationError::Snapshot) => CheckpointVerificationStatus::SnapshotLoad,
        Err(HistoryVerificationError::UnknownControl) => {
            CheckpointVerificationStatus::PendingControl
        }
        Err(HistoryVerificationError::Budget) => CheckpointVerificationStatus::BudgetExhausted,
        Err(HistoryVerificationError::Cancelled) => CheckpointVerificationStatus::Cancelled,
    }
}

const fn join_status(error: JoinError) -> CheckpointVerificationStatus {
    match error {
        JoinError::Author => CheckpointVerificationStatus::ChunkAuthorMismatch,
        JoinError::Coordinate => CheckpointVerificationStatus::ChunkCoordinateMismatch,
        JoinError::Descriptor => CheckpointVerificationStatus::ChunkDescriptorMismatch,
        JoinError::Count => CheckpointVerificationStatus::ChunkCountMismatch,
        JoinError::DuplicateIndex => CheckpointVerificationStatus::DuplicateChunk,
        JoinError::MissingIndex => CheckpointVerificationStatus::MissingChunk,
        JoinError::Size => CheckpointVerificationStatus::ChunkSizeMismatch,
    }
}

const fn assembly_status(error: crate::checkpoint::AssemblyError) -> CheckpointVerificationStatus {
    match error {
        crate::checkpoint::AssemblyError::Chunks => {
            CheckpointVerificationStatus::ChunkAssemblyMismatch
        }
        crate::checkpoint::AssemblyError::Proof => CheckpointVerificationStatus::MerkleMismatch,
        crate::checkpoint::AssemblyError::Budget => CheckpointVerificationStatus::BudgetExhausted,
        crate::checkpoint::AssemblyError::Cancelled => CheckpointVerificationStatus::Cancelled,
        crate::checkpoint::AssemblyError::SnapshotSize => {
            CheckpointVerificationStatus::SnapshotSizeMismatch
        }
        crate::checkpoint::AssemblyError::SnapshotHash => {
            CheckpointVerificationStatus::SnapshotHashMismatch
        }
    }
}

fn checkpoint_accepted_history(
    corpus: &EvidenceCorpus,
    coordinate: DocumentCoordinate,
    accepted_at_control: &std::collections::BTreeMap<crate::EventId, AcceptedAtControl>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<
    std::collections::BTreeMap<
        crate::EventId,
        Result<std::collections::BTreeSet<ChangeHash>, HistoryVerificationError>,
    >,
    CheckpointWorkStop,
> {
    let mut history = std::collections::BTreeMap::new();
    for descriptor_id in corpus
        .indexes
        .checkpoints
        .descriptors_by_coordinate
        .get(&coordinate)
        .into_iter()
        .flatten()
    {
        charge_checkpoint_work(budget, cancellation, 1)?;
        if let Some(EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::CheckpointDescriptor(descriptor),
            ..
        }) = corpus.events.get(descriptor_id)
        {
            let accepted = if let Some(state) = accepted_at_control.get(&descriptor.control_id()) {
                charge_checkpoint_work(
                    budget,
                    cancellation,
                    u64::try_from(state.accepted_closure().len()).unwrap_or(u64::MAX),
                )?;
                Ok(state.accepted_closure().clone())
            } else {
                Err(HistoryVerificationError::UnknownControl)
            };
            history.insert(*descriptor_id, accepted);
        }
    }
    Ok(history)
}

fn checkpoint_carrier_coverage(
    corpus: &EvidenceCorpus,
    coordinate: DocumentCoordinate,
    canonical_controls: &[crate::EventId],
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<
    std::collections::BTreeMap<
        crate::EventId,
        Result<std::collections::BTreeSet<ChangeHash>, HistoryVerificationError>,
    >,
    CheckpointWorkStop,
> {
    let mut coverage = std::collections::BTreeMap::new();
    for descriptor_id in corpus
        .indexes
        .checkpoints
        .descriptors_by_coordinate
        .get(&coordinate)
        .into_iter()
        .flatten()
    {
        charge_checkpoint_work(budget, cancellation, 1)?;
        if let Some(EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::CheckpointDescriptor(descriptor),
            ..
        }) = corpus.events.get(descriptor_id)
        {
            let result = historical_carrier_coverage(
                corpus,
                canonical_controls,
                descriptor.control_id(),
                budget,
                cancellation,
            );
            match result {
                Err(HistoryVerificationError::Budget) => {
                    return Err(CheckpointWorkStop::Budget);
                }
                Err(HistoryVerificationError::Cancelled) => {
                    return Err(CheckpointWorkStop::Cancelled);
                }
                result => {
                    coverage.insert(*descriptor_id, result);
                }
            }
        }
    }
    Ok(coverage)
}

fn checkpoint_chunk_sets(
    corpus: &EvidenceCorpus,
    coordinate: DocumentCoordinate,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<
    std::collections::BTreeMap<
        crate::EventId,
        Result<Vec<crate::checkpoint::CheckpointChunk>, JoinError>,
    >,
    CheckpointWorkStop,
> {
    let mut sets = std::collections::BTreeMap::new();
    let descriptor_ids = corpus
        .indexes
        .checkpoints
        .descriptors_by_coordinate
        .get(&coordinate)
        .into_iter()
        .flatten();
    for descriptor_id in descriptor_ids {
        charge_checkpoint_work(budget, cancellation, 1)?;
        let Some(EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::CheckpointDescriptor(descriptor),
            ..
        }) = corpus.events.get(descriptor_id)
        else {
            continue;
        };
        let mut chunks = Vec::new();
        for chunk_id in corpus
            .indexes
            .checkpoints
            .chunks_by_descriptor
            .get(descriptor_id)
            .into_iter()
            .flatten()
        {
            charge_checkpoint_work(budget, cancellation, 1)?;
            if let Some(EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::CheckpointChunk(chunk),
                ..
            }) = corpus.events.get(chunk_id)
            {
                chunks.push(chunk.as_ref());
            }
        }
        sets.insert(*descriptor_id, join_chunks(descriptor, chunks));
    }
    Ok(sets)
}

fn checkpoint_authorizations(
    corpus: &EvidenceCorpus,
    coordinate: DocumentCoordinate,
    canonical_controls: &std::collections::BTreeSet<crate::EventId>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<std::collections::BTreeMap<crate::EventId, DescriptorAuthorization>, CheckpointWorkStop>
{
    let mut controls = std::collections::BTreeMap::new();
    for control_id in corpus.indexes.controls.controls_by_id.keys() {
        charge_checkpoint_work(budget, cancellation, 1)?;
        if let Some(EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::Control(control),
            ..
        }) = corpus.events.get(control_id)
        {
            controls.insert(*control_id, control.as_ref());
        }
    }
    let mut authorizations = std::collections::BTreeMap::new();
    for descriptor_id in corpus
        .indexes
        .checkpoints
        .descriptors_by_coordinate
        .get(&coordinate)
        .into_iter()
        .flatten()
    {
        charge_checkpoint_work(budget, cancellation, 1)?;
        if let Some(EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::CheckpointDescriptor(descriptor),
            ..
        }) = corpus.events.get(descriptor_id)
        {
            let member_work = controls.get(&descriptor.control_id()).map_or(0, |control| {
                u64::try_from(control.members().len()).unwrap_or(u64::MAX)
            });
            charge_checkpoint_work(budget, cancellation, member_work)?;
            authorizations.insert(
                *descriptor_id,
                authorize_descriptor(descriptor, canonical_controls, &controls),
            );
        }
    }
    Ok(authorizations)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CheckpointWorkStop {
    Budget,
    Cancelled,
}

impl CheckpointWorkStop {
    const fn status(self) -> CheckpointVerificationStatus {
        match self {
            Self::Budget => CheckpointVerificationStatus::BudgetExhausted,
            Self::Cancelled => CheckpointVerificationStatus::Cancelled,
        }
    }
}

fn charge_checkpoint_work(
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
    amount: u64,
) -> Result<(), CheckpointWorkStop> {
    if cancellation.is_cancelled() {
        return Err(CheckpointWorkStop::Cancelled);
    }
    budget
        .charge_checkpoint_items(amount)
        .map_err(|_| CheckpointWorkStop::Budget)
}

fn checkpoint_refusals(
    corpus: &EvidenceCorpus,
    coordinate: DocumentCoordinate,
    status: CheckpointVerificationStatus,
) -> Vec<CheckpointVerificationResult> {
    corpus
        .indexes
        .checkpoints
        .descriptors_by_coordinate
        .get(&coordinate)
        .into_iter()
        .flatten()
        .filter_map(|descriptor_id| match corpus.events.get(descriptor_id) {
            Some(EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::CheckpointDescriptor(descriptor),
                ..
            }) => {
                let commitments = descriptor.descriptor();
                Some(CheckpointVerificationResult::new(
                    *descriptor_id,
                    Vec::new(),
                    descriptor.snapshot_hash(),
                    commitments.heads.iter().copied().collect(),
                    commitments.change_count,
                    commitments.change_set_hash,
                    Vec::new(),
                    Vec::new(),
                    status,
                ))
            }
            _ => None,
        })
        .collect()
}

fn charge_ingress(corpus: &EvidenceCorpus, budget: &mut WorkBudget) -> bool {
    let event_count = u64::try_from(corpus.evaluation_event_count()).unwrap_or(u64::MAX);
    let carrier_count = u64::try_from(corpus.carrier_evidence_count()).unwrap_or(u64::MAX);
    let decode_bytes = corpus.decode_work_bytes().unwrap_or(u64::MAX);
    budget.charge(WorkCounter::Event, event_count).is_ok()
        && budget.charge(WorkCounter::Carrier, carrier_count).is_ok()
        && budget.charge(WorkCounter::DecodeByte, decode_bytes).is_ok()
}

fn disposition_hashes(
    dispositions: &[(ChangeHash, ProtocolDisposition)],
    expected: ProtocolDisposition,
) -> Vec<ChangeHash> {
    dispositions
        .iter()
        .filter_map(|(hash, disposition)| (*disposition == expected).then_some(*hash))
        .collect()
}

fn prepare_controls(
    corpus: &EvidenceCorpus,
    coordinate: DocumentCoordinate,
) -> (
    Vec<BatchControl>,
    std::collections::BTreeMap<crate::EventId, ProtocolDisposition>,
) {
    let mut dispositions = std::collections::BTreeMap::new();
    let controls = corpus
        .events
        .values()
        .filter_map(|evidence| match evidence {
            EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::Control(control),
                ..
            } if control.coordinate() == coordinate => {
                let envelope = ControlEnvelope::from_validated(control.as_ref().clone());
                let disposition = if corpus
                    .indexes
                    .controls
                    .pending
                    .contains(&control.event_id())
                {
                    ProtocolDisposition::Pending
                } else if control.parent().is_none() {
                    let predecessor = control.predecessor().and_then(|link| {
                        match corpus.events.get(&link.terminal_control) {
                            Some(EventEvidence::VerifiedCarrier {
                                carrier: VerifiedCarrier::Control(terminal),
                                ..
                            }) => Some(ControlEnvelope::from_validated(terminal.as_ref().clone())),
                            _ => None,
                        }
                    });
                    let outcome = classify_genesis(&envelope, predecessor.as_ref()).disposition();
                    if outcome == ProtocolDisposition::Accepted {
                        ProtocolDisposition::Excluded
                    } else {
                        outcome
                    }
                } else if !parent_continuity_is_valid(corpus, control)
                    || !account_continuity_is_valid(corpus, control)
                    || !role_continuity_is_valid(corpus, control)
                    || !device_ancestry_is_valid(corpus, control)
                    || !terminal_continuity_is_valid(corpus, control)
                {
                    ProtocolDisposition::Invalid
                } else {
                    ProtocolDisposition::Excluded
                };
                dispositions.insert(control.event_id(), disposition);
                if disposition != ProtocolDisposition::Excluded {
                    return None;
                }
                Some(BatchControl {
                    event_id: control.event_id(),
                    parent: control.parent(),
                    accepted_base: control.base_heads().collect(),
                    frozen: control.terminal(),
                    changes: changes_for_control(corpus, control),
                    envelope: Some(envelope),
                })
            }
            _ => None,
        })
        .collect();
    (controls, dispositions)
}

fn device_ancestry_is_valid(
    corpus: &EvidenceCorpus,
    child: &crate::carrier::control::ValidatedControlCarrier,
) -> bool {
    let mut ancestry = Vec::new();
    let mut visited = std::collections::BTreeSet::new();
    let mut current = child.parent();
    while let Some(event_id) = current {
        if !visited.insert(event_id) {
            return false;
        }
        let Some(EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::Control(control),
            ..
        }) = corpus.events.get(&event_id)
        else {
            return false;
        };
        current = control.parent();
        ancestry.push(ControlEnvelope::from_validated(control.as_ref().clone()));
    }
    ancestry.reverse();
    let child = ControlEnvelope::from_validated(child.clone());
    evaluate_device_ancestry(&ancestry, &child) == CandidateResult::Valid
}

fn role_continuity_is_valid(
    corpus: &EvidenceCorpus,
    child: &crate::carrier::control::ValidatedControlCarrier,
) -> bool {
    let Some(parent_id) = child.parent() else {
        return true;
    };
    let Some(EventEvidence::VerifiedCarrier {
        carrier: VerifiedCarrier::Control(parent),
        ..
    }) = corpus.events.get(&parent_id)
    else {
        return false;
    };
    let parent = ControlEnvelope::from_validated(parent.as_ref().clone());
    let child = ControlEnvelope::from_validated(child.clone());
    evaluate_role_continuity(&parent, &child) == CandidateResult::Valid
}

fn account_continuity_is_valid(
    corpus: &EvidenceCorpus,
    child: &crate::carrier::control::ValidatedControlCarrier,
) -> bool {
    let Some(parent_id) = child.parent() else {
        return true;
    };
    let Some(EventEvidence::VerifiedCarrier {
        carrier: VerifiedCarrier::Control(parent),
        ..
    }) = corpus.events.get(&parent_id)
    else {
        return false;
    };
    let parent = ControlEnvelope::from_validated(parent.as_ref().clone());
    let child = ControlEnvelope::from_validated(child.clone());
    evaluate_account_continuity(&parent, &child) == CandidateResult::Valid
}

fn parent_continuity_is_valid(
    corpus: &EvidenceCorpus,
    child: &crate::carrier::control::ValidatedControlCarrier,
) -> bool {
    let Some(parent_id) = child.parent() else {
        return true;
    };
    let Some(EventEvidence::VerifiedCarrier {
        carrier: VerifiedCarrier::Control(parent),
        ..
    }) = corpus.events.get(&parent_id)
    else {
        return false;
    };
    let parent = ControlEnvelope::from_validated(parent.as_ref().clone());
    let child = ControlEnvelope::from_validated(child.clone());
    evaluate_parent_continuity(&parent, &child) == CandidateResult::Valid
}

fn terminal_continuity_is_valid(
    corpus: &EvidenceCorpus,
    child: &crate::carrier::control::ValidatedControlCarrier,
) -> bool {
    let Some(parent_id) = child.parent() else {
        return true;
    };
    let Some(EventEvidence::VerifiedCarrier {
        carrier: VerifiedCarrier::Control(parent),
        ..
    }) = corpus.events.get(&parent_id)
    else {
        return false;
    };
    let parent = ControlEnvelope::from_validated(parent.as_ref().clone());
    let child = ControlEnvelope::from_validated(child.clone());
    evaluate_terminal_continuity(&parent, &child) == CandidateResult::Valid
}

fn changes_for_control(
    corpus: &EvidenceCorpus,
    control: &crate::carrier::control::ValidatedControlCarrier,
) -> Vec<BatchChange> {
    let hashes = corpus
        .indexes
        .changes
        .hashes_by_control
        .get(&control.event_id())
        .cloned()
        .unwrap_or_default();
    hashes
        .into_iter()
        .filter_map(|hash| change_for_hash(corpus, control, hash))
        .collect()
}

fn change_for_hash(
    corpus: &EvidenceCorpus,
    control: &crate::carrier::control::ValidatedControlCarrier,
    hash: ChangeHash,
) -> Option<BatchChange> {
    let event_ids = corpus.indexes.changes.carriers_by_hash.get(&hash)?;
    let mut raw = None;
    let carriers = event_ids
        .iter()
        .filter_map(|event_id| match corpus.events.get(event_id) {
            Some(EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::Change(change),
                ..
            }) if change.control_id() == control.event_id()
                && change.coordinate() == control.coordinate() =>
            {
                raw.get_or_insert_with(|| change.canonical_raw_bytes().to_vec());
                Some(CandidateCarrier {
                    event_id: change.event_id(),
                    change_hash: change.change_hash(),
                    actor: change.actor(),
                    sequence: change.sequence(),
                    start_op: change.start_op(),
                    operation_count: change.operation_count(),
                    dependencies: change.dependencies().collect(),
                    control_id: change.control_id(),
                    author: change.author_device(),
                })
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let candidate = ChangeCandidate::from_carriers(carriers).ok()?;
    let authorized = control.members().iter().any(|member| {
        member.actor == candidate.actor
            && member.device == candidate.author
            && member.roles.contains(&Role::Write)
    });
    Some(BatchChange {
        candidate,
        legacy_eligible: authorized && !control.terminal(),
        raw_change: raw,
    })
}

#[cfg(test)]
mod tests {
    use super::{CheckpointWorkStop, assembly_status, charge_checkpoint_work, join_status};
    use crate::CheckpointVerificationStatus as Status;
    use crate::checkpoint::AssemblyError;
    use crate::checkpoint::join::JoinError;
    use crate::{WorkBudget, WorkCounter};

    #[test]
    fn checkpoint_preparation_charge_stops_before_optional_work() {
        let mut exhausted = WorkBudget::new(0, 1);
        assert_eq!(
            charge_checkpoint_work(&mut exhausted, &crate::NeverCancelled, 2),
            Err(CheckpointWorkStop::Budget)
        );
        assert_eq!(exhausted.consumed().get(WorkCounter::CheckpointItem), 0);
        let mut cancelled = WorkBudget::new(0, 2);
        assert_eq!(
            charge_checkpoint_work(&mut cancelled, &|| true, 1),
            Err(CheckpointWorkStop::Cancelled)
        );
        assert_eq!(cancelled.consumed().get(WorkCounter::CheckpointItem), 0);
    }

    #[test]
    fn every_checkpoint_refusal_has_a_stable_public_status() {
        assert_eq!(join_status(JoinError::Author), Status::ChunkAuthorMismatch);
        assert_eq!(
            join_status(JoinError::Coordinate),
            Status::ChunkCoordinateMismatch
        );
        assert_eq!(
            join_status(JoinError::Descriptor),
            Status::ChunkDescriptorMismatch
        );
        assert_eq!(join_status(JoinError::Count), Status::ChunkCountMismatch);
        assert_eq!(
            join_status(JoinError::DuplicateIndex),
            Status::DuplicateChunk
        );
        assert_eq!(join_status(JoinError::MissingIndex), Status::MissingChunk);
        assert_eq!(join_status(JoinError::Size), Status::ChunkSizeMismatch);
        assert_eq!(
            assembly_status(AssemblyError::Chunks),
            Status::ChunkAssemblyMismatch
        );
        assert_eq!(
            assembly_status(AssemblyError::Proof),
            Status::MerkleMismatch
        );
        assert_eq!(
            assembly_status(AssemblyError::SnapshotSize),
            Status::SnapshotSizeMismatch
        );
        assert_eq!(
            assembly_status(AssemblyError::SnapshotHash),
            Status::SnapshotHashMismatch
        );
        assert_eq!(
            assembly_status(AssemblyError::Budget),
            Status::BudgetExhausted
        );
        assert_eq!(assembly_status(AssemblyError::Cancelled), Status::Cancelled);
    }

    #[test]
    fn projection_failure_is_typed_error_internal() {
        assert_eq!(
            super::project_document(
                Some(vec![0xff]),
                &mut crate::WorkBudget::new(10, 10),
                &crate::NeverCancelled,
            ),
            Err(crate::automerge_adapter::materialized_view::ProjectionError::Invalid)
        );
        assert_eq!(
            super::project_document(
                None,
                &mut crate::WorkBudget::new(0, 0),
                &crate::NeverCancelled,
            ),
            Ok(None)
        );
    }
}
