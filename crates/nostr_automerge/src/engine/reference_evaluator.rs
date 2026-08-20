use crate::carrier::VerifiedCarrier;
use crate::checkpoint::authorize::{DescriptorAuthorization, authorize_descriptor};
use crate::checkpoint::join::{JoinError, join_chunks};
use crate::checkpoint::reference_state::resolve_referenced_descriptor;
use crate::checkpoint::{HistoryVerificationError, historical_carrier_coverage};
use crate::conformance::dispositions_digest::{disposition_items, dispositions_digest};
use crate::conformance::history_digest::history_digest;
use crate::control::candidate::{
    CandidateResult, evaluate_account_continuity, evaluate_device_ancestry,
    evaluate_parent_continuity, evaluate_role_continuity, evaluate_terminal_continuity,
};
use crate::control::genesis::classify_genesis;
use crate::control::reference_state::{
    ControlParentState, ReferencedControlState, resolve_referenced_control,
};
use crate::control::reorganization::{ControlChainSummary, detect_reorganization};
use crate::control::validate::ControlEnvelope;
use crate::evidence::corpus_builder::ManifestSelectionState;
use crate::evidence::document_view::DocumentEvidenceView;
use crate::evidence::event::EventEvidence;
use crate::graph::change_candidate::{CandidateCarrier, ChangeCandidate};
use crate::reference::epoch_engine::AcceptedAtControl;
use crate::reference::epoch_engine::PriorChangeKnowledge;
use crate::reference::evaluate::{
    BatchChange, BatchControl, BatchEvaluationReport, evaluate_batch_with_prior,
    propagate_control_parent_dispositions,
};
use crate::types::role::Role;
use crate::{
    CancellationCheck, ChangeHash, CheckpointVerificationResult, CheckpointVerificationStatus,
    Completion, DocumentCoordinate, EvidenceCorpus, EvidenceIdentifier, EvidenceStatus,
    ManifestControlStatus, ManifestPendingReason, ProtocolDisposition, ProtocolRevision,
    ResolvedManifestAvailability, WorkBudget, WorkCounter,
};

use super::evaluation_report::{
    DispositionRecord, EvaluationError, EvaluationFailure, EvaluationReport, EvaluationReportParts,
    ProtocolItemIdentifier, REPORT_INVARIANT_ITEMS,
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
        if cancellation.is_cancelled() {
            return compact_interrupted_report(self.revision, coordinate, Completion::Cancelled);
        }
        let view = DocumentEvidenceView::derive(corpus, coordinate);
        let plan = ReportFinalizationPlan::from_view(&view);
        let mut finalization = match plan
            .and_then(|plan| ReportFinalizationPermit::reserve(plan, budget).map_err(|_| ()))
        {
            Ok(permit) => permit,
            Err(()) => {
                let completion = if cancellation.is_cancelled() {
                    Completion::Cancelled
                } else {
                    Completion::BudgetExhausted
                };
                return compact_interrupted_report(self.revision, coordinate, completion);
            }
        };
        if let Err(completion) = charge_ingress(&view, budget, cancellation) {
            return reserved_interrupted_report(
                self.revision,
                coordinate,
                completion,
                &mut finalization,
            );
        }
        let prepared_controls = match prepare_controls(&view, budget, cancellation) {
            Ok(prepared) => prepared,
            Err(completion) => {
                return reserved_interrupted_report(
                    self.revision,
                    coordinate,
                    completion,
                    &mut finalization,
                );
            }
        };
        let (controls, preliminary_control_dispositions) = prepared_controls.into_parts();
        let additional_prior =
            match additional_prior_knowledge(&view, &controls, budget, cancellation) {
                Ok(knowledge) => knowledge,
                Err(completion) => {
                    return reserved_interrupted_report(
                        self.revision,
                        coordinate,
                        completion,
                        &mut finalization,
                    );
                }
            };
        let mut batch =
            evaluate_batch_with_prior(controls, &additional_prior, budget, cancellation);
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
        let mut control_disposition_map = preliminary_control_dispositions;
        control_disposition_map.extend(core::mem::take(&mut batch.control_dispositions));
        batch.control_dispositions = control_disposition_map;
        if batch.completion != Completion::Complete {
            return reserved_batch_report(
                self.revision,
                coordinate,
                batch,
                ResolvedManifestAvailability::Missing,
                Vec::new(),
                &mut finalization,
            );
        }
        if let Err(completion) = reduce_change_dispositions(&view, &mut batch, budget, cancellation)
        {
            interrupt_batch(&mut batch, completion);
            return reserved_batch_report(
                self.revision,
                coordinate,
                batch,
                ResolvedManifestAvailability::Missing,
                Vec::new(),
                &mut finalization,
            );
        }
        if let Err(completion) =
            charge_evaluation_work(budget, cancellation, WorkCounter::Carrier, 1)
        {
            interrupt_batch(&mut batch, completion);
            return reserved_batch_report(
                self.revision,
                coordinate,
                batch,
                ResolvedManifestAvailability::Missing,
                Vec::new(),
                &mut finalization,
            );
        }
        let manifest = resolve_selected_manifest(
            &view,
            &batch.control_dispositions,
            &batch.statefully_valid_controls,
        );
        let control_record_work =
            u64::try_from(batch.control_dispositions.len()).unwrap_or(u64::MAX);
        if let Err(completion) = charge_evaluation_work(
            budget,
            cancellation,
            WorkCounter::Control,
            control_record_work,
        ) {
            interrupt_batch(&mut batch, completion);
            return reserved_batch_report(
                self.revision,
                coordinate,
                batch,
                manifest,
                Vec::new(),
                &mut finalization,
            );
        }
        let control_dispositions = batch
            .control_dispositions
            .iter()
            .map(|(id, disposition)| (*id, *disposition))
            .collect::<Vec<_>>();
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
        let checkpoint_evaluation = verify_checkpoints(
            &view,
            &batch.canonical_controls,
            &batch.control_dispositions,
            &batch.statefully_valid_controls,
            &batch.accepted_at_control,
            budget,
            cancellation,
        );
        let checkpoints = checkpoint_evaluation.results;
        if let Some(stop) = checkpoint_evaluation.stop {
            interrupt_batch(&mut batch, stop.completion());
        }
        if batch.completion != Completion::Complete {
            return reserved_batch_report(
                self.revision,
                coordinate,
                batch,
                manifest,
                checkpoints,
                &mut finalization,
            );
        }
        let disposition_work = u64::try_from(batch.dispositions.len()).unwrap_or(u64::MAX);
        if let Err(completion) = charge_evaluation_work(
            budget,
            cancellation,
            WorkCounter::GraphNode,
            disposition_work.saturating_mul(5),
        ) {
            interrupt_batch(&mut batch, completion);
            return reserved_batch_report(
                self.revision,
                coordinate,
                batch,
                manifest,
                checkpoints,
                &mut finalization,
            );
        }
        let dispositions = batch
            .dispositions
            .iter()
            .map(|(hash, disposition)| (*hash, *disposition))
            .collect::<Vec<_>>();
        disposition_records.extend(dispositions.iter().map(|(hash, disposition)| {
            DispositionRecord::new(ProtocolItemIdentifier::from(*hash), *disposition, None)
        }));
        let event_record_work = u64::try_from(view.evaluation_event_count()).unwrap_or(u64::MAX);
        if let Err(completion) = charge_evaluation_work(
            budget,
            cancellation,
            WorkCounter::Carrier,
            event_record_work.saturating_mul(3),
        ) {
            interrupt_batch(&mut batch, completion);
            return reserved_batch_report(
                self.revision,
                coordinate,
                batch,
                manifest,
                checkpoints,
                &mut finalization,
            );
        }
        let descriptor_reference_work =
            u64::try_from(view.corpus().indexes.checkpoints.descriptors_by_id.len())
                .unwrap_or(u64::MAX)
                .saturating_add(
                    u64::try_from(view.corpus().indexes.checkpoints.chunks_by_id.len())
                        .unwrap_or(u64::MAX),
                );
        if let Err(stop) = charge_checkpoint_work(budget, cancellation, descriptor_reference_work) {
            interrupt_batch(&mut batch, stop.completion());
            return reserved_batch_report(
                self.revision,
                coordinate,
                batch,
                manifest,
                checkpoints,
                &mut finalization,
            );
        }
        disposition_records.extend(event_disposition_records(&view, &manifest, &checkpoints));
        let accepted_changes = disposition_hashes(&dispositions, ProtocolDisposition::Accepted);
        let heads = batch.heads.iter().copied().collect::<Vec<_>>();
        let pending_changes = disposition_hashes(&dispositions, ProtocolDisposition::Pending);
        let excluded_changes = disposition_hashes(&dispositions, ProtocolDisposition::Excluded);
        let invalid_changes = disposition_hashes(&dispositions, ProtocolDisposition::Invalid);
        let digest_work = u64::try_from(
            batch
                .canonical_controls
                .len()
                .saturating_add(accepted_changes.len())
                .saturating_add(heads.len())
                .saturating_add(disposition_records.len()),
        )
        .unwrap_or(u64::MAX);
        if let Err(completion) =
            charge_evaluation_work(budget, cancellation, WorkCounter::Assertion, digest_work)
        {
            interrupt_batch(&mut batch, completion);
            return reserved_batch_report(
                self.revision,
                coordinate,
                batch,
                manifest,
                checkpoints,
                &mut finalization,
            );
        }
        let history_digest = history_digest(
            self.revision,
            coordinate,
            &batch.canonical_controls,
            &accepted_changes,
            &heads,
        )
        .map_err(|_| EvaluationError::ReportInvariant)?;
        let disposition_items = disposition_items(&disposition_records)
            .map_err(|_| EvaluationError::ReportInvariant)?;
        let dispositions_digest =
            dispositions_digest(self.revision, coordinate, &disposition_items)
                .map_err(|_| EvaluationError::ReportInvariant)?;
        let projection = project_document(
            core::mem::take(&mut batch.materialized_document),
            budget,
            cancellation,
        );
        let document = match projection {
            Ok(document) => document,
            Err(crate::automerge_adapter::materialized_view::ProjectionError::Budget) => {
                interrupt_batch(&mut batch, Completion::BudgetExhausted);
                return reserved_batch_report(
                    self.revision,
                    coordinate,
                    batch,
                    manifest,
                    checkpoints,
                    &mut finalization,
                );
            }
            Err(crate::automerge_adapter::materialized_view::ProjectionError::Cancelled) => {
                interrupt_batch(&mut batch, Completion::Cancelled);
                return reserved_batch_report(
                    self.revision,
                    coordinate,
                    batch,
                    manifest,
                    checkpoints,
                    &mut finalization,
                );
            }
            Err(crate::automerge_adapter::materialized_view::ProjectionError::Invalid) => {
                return Err(EvaluationError::Projection);
            }
        };
        let evidence_work = u64::try_from(view.evaluation_event_count()).unwrap_or(u64::MAX);
        if let Err(completion) =
            charge_evaluation_work(budget, cancellation, WorkCounter::Event, evidence_work)
        {
            interrupt_batch(&mut batch, completion);
            return reserved_batch_report(
                self.revision,
                coordinate,
                batch,
                manifest,
                checkpoints,
                &mut finalization,
            );
        }
        let evidence = view.records().collect::<Vec<_>>();
        finalization
            .consume(
                FinalizationDimension::Controls,
                u64::try_from(control_dispositions.len()).unwrap_or(u64::MAX),
            )
            .map_err(|_| EvaluationError::ReportInvariant)?;
        finalization
            .consume(
                FinalizationDimension::Changes,
                u64::try_from(dispositions.len()).unwrap_or(u64::MAX),
            )
            .map_err(|_| EvaluationError::ReportInvariant)?;
        let finalized_events = disposition_records
            .iter()
            .filter(|record| matches!(record.identifier(), ProtocolItemIdentifier::Event(_)))
            .count();
        finalization
            .consume(
                FinalizationDimension::Events,
                u64::try_from(finalized_events).unwrap_or(u64::MAX),
            )
            .map_err(|_| EvaluationError::ReportInvariant)?;
        let finalized_checkpoints = checkpoints
            .iter()
            .map(|checkpoint| 1_usize.saturating_add(checkpoint.chunk_events().len()))
            .sum::<usize>();
        finalization
            .consume(
                FinalizationDimension::Checkpoints,
                u64::try_from(finalized_checkpoints).unwrap_or(u64::MAX),
            )
            .map_err(|_| EvaluationError::ReportInvariant)?;
        let finalized_digests = batch
            .canonical_controls
            .len()
            .saturating_add(accepted_changes.len())
            .saturating_add(heads.len())
            .saturating_add(disposition_records.len())
            .saturating_add(8);
        finalization
            .consume(
                FinalizationDimension::Digests,
                u64::try_from(finalized_digests).unwrap_or(u64::MAX),
            )
            .map_err(|_| EvaluationError::ReportInvariant)?;
        finalization
            .consume(
                FinalizationDimension::Evidence,
                u64::try_from(evidence.len()).unwrap_or(u64::MAX),
            )
            .map_err(|_| EvaluationError::ReportInvariant)?;
        finalization
            .consume(FinalizationDimension::Invariants, REPORT_INVARIANT_ITEMS)
            .map_err(|_| EvaluationError::ReportInvariant)?;
        finalization
            .consume(FinalizationDimension::FixedOverhead, 8)
            .map_err(|_| EvaluationError::ReportInvariant)?;
        let report = EvaluationReport::from_parts(EvaluationReportParts {
            coordinate,
            canonical_controls: batch.canonical_controls,
            disposition_records,
            control_dispositions,
            dispositions,
            accepted_changes,
            pending_changes,
            excluded_changes,
            invalid_changes,
            heads,
            evidence,
            checkpoints,
            history_digest,
            dispositions_digest,
            integrity_alerts: batch.integrity_alerts,
            manifest,
            completion: batch.completion,
            failure: batch.failure,
            document,
        })
        .map_err(|_| EvaluationError::ReportInvariant)?;
        finalization
            .refund(budget)
            .map_err(|_| EvaluationError::ReportInvariant)?;
        Ok(report)
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

fn additional_prior_knowledge(
    view: &DocumentEvidenceView<'_>,
    controls: &[BatchControl],
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<
    std::collections::BTreeMap<
        crate::EventId,
        std::collections::BTreeMap<ChangeHash, PriorChangeKnowledge>,
    >,
    Completion,
> {
    charge_evaluation_work(budget, cancellation, WorkCounter::Control, 0)?;
    let corpus = view.corpus();
    let mut reasoned_by_hash = std::collections::BTreeMap::new();
    for hash in view.change_hashes() {
        charge_evaluation_work(budget, cancellation, WorkCounter::GraphNode, 1)?;
        let mut saw_claim = false;
        let mut all_unsupported = true;
        let mut all_invalid = true;
        for event_id in view.change_claim_event_ids(hash) {
            charge_evaluation_work(budget, cancellation, WorkCounter::Carrier, 1)?;
            let Some(claim) = corpus.indexes.changes.claims_by_event.get(&event_id) else {
                continue;
            };
            saw_claim = true;
            charge_evaluation_work(budget, cancellation, WorkCounter::Control, 1)?;
            match corpus.events.get(&claim.control_id) {
                Some(EventEvidence::UnsupportedRevision { .. }) => {
                    all_invalid = false;
                }
                Some(EventEvidence::VerifiedCarrier {
                    carrier: VerifiedCarrier::UnsupportedRevision { .. },
                    ..
                }) => {
                    all_invalid = false;
                }
                Some(EventEvidence::VerifiedCarrier {
                    carrier: VerifiedCarrier::Control(control),
                    ..
                }) if control.coordinate() == view.coordinate() => {
                    charge_evaluation_work(
                        budget,
                        cancellation,
                        WorkCounter::Control,
                        u64::try_from(control.members().len()).unwrap_or(u64::MAX),
                    )?;
                    all_unsupported = false;
                    all_invalid = false;
                }
                None => {
                    all_unsupported = false;
                    all_invalid = false;
                }
                Some(_) => {
                    all_unsupported = false;
                }
            }
        }
        if saw_claim {
            reasoned_by_hash.insert(
                hash,
                if all_unsupported {
                    PriorChangeKnowledge::KnownUnsupported
                } else if all_invalid {
                    PriorChangeKnowledge::KnownInvalid
                } else {
                    PriorChangeKnowledge::KnownOtherControl
                },
            );
        }
    }
    controls
        .iter()
        .map(|selected| {
            charge_evaluation_work(budget, cancellation, WorkCounter::Control, 1)?;
            let selected_hashes = view.change_hashes_for_control(selected.event_id);
            let mut knowledge = std::collections::BTreeMap::new();
            for hash in view.change_hashes() {
                charge_evaluation_work(budget, cancellation, WorkCounter::GraphNode, 1)?;
                let state = if selected.accepted_base.contains(&hash) {
                    Some(PriorChangeKnowledge::AcceptedInBase)
                } else if selected_hashes.is_some_and(|hashes| hashes.contains(&hash)) {
                    Some(PriorChangeKnowledge::SameEpochCandidate)
                } else {
                    reasoned_by_hash.get(&hash).copied()
                };
                if let Some(state) = state {
                    knowledge.insert(hash, state);
                }
            }
            Ok((selected.event_id, knowledge))
        })
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChangeClaimReason {
    AuthorizedCanonical,
    UnresolvedControl,
    AuthorizedNoncanonical,
    AuthorizedCurrentExcluded,
    InvalidReferencedControl,
    Unauthorized,
    UnsupportedCarrier,
}

impl ChangeClaimReason {
    #[cfg(test)]
    const fn diagnostic(self) -> Option<crate::DiagnosticCode> {
        let code = match self {
            Self::InvalidReferencedControl => "control.parent",
            Self::Unauthorized => "change.actor",
            Self::UnsupportedCarrier => "carrier.revision",
            Self::AuthorizedCanonical
            | Self::UnresolvedControl
            | Self::AuthorizedNoncanonical
            | Self::AuthorizedCurrentExcluded => return None,
        };
        Some(crate::DiagnosticCode::registered(code))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FinalLineageChangeState {
    Accepted,
    CanonicalPruned,
    Current,
}

fn reduce_change_dispositions(
    view: &DocumentEvidenceView<'_>,
    batch: &mut BatchEvaluationReport,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<(), Completion> {
    let corpus = view.corpus();
    let final_accepted = batch.accepted_changes.clone();
    for hash in view.change_hashes() {
        charge_evaluation_work(budget, cancellation, WorkCounter::GraphNode, 1)?;
        let lineage = if final_accepted.contains(&hash) {
            FinalLineageChangeState::Accepted
        } else if batch.canonical_controls.iter().any(|control_id| {
            batch
                .accepted_at_control
                .get(control_id)
                .is_some_and(|accepted| accepted.accepted_closure().contains(&hash))
        }) {
            FinalLineageChangeState::CanonicalPruned
        } else {
            FinalLineageChangeState::Current
        };
        let states = view
            .change_claim_event_ids(hash)
            .filter_map(|event_id| {
                if charge_evaluation_work(budget, cancellation, WorkCounter::Carrier, 1).is_err() {
                    return Some(Err(()));
                }
                let claim = corpus.indexes.changes.claims_by_event.get(&event_id)?;
                let semantic = corpus.indexes.changes.semantic_by_hash.get(&hash)?;
                let state = resolve_referenced_control(
                    corpus,
                    claim.control_id,
                    view.coordinate(),
                    &batch.control_dispositions,
                    &batch.statefully_valid_controls,
                );
                Some(Ok(match state {
                    ReferencedControlState::Canonical(control) => {
                        if charge_evaluation_work(
                            budget,
                            cancellation,
                            WorkCounter::Control,
                            u64::try_from(control.members().len()).unwrap_or(u64::MAX),
                        )
                        .is_err()
                        {
                            return Some(Err(()));
                        }
                        let authorized = !control.terminal()
                            && control.members().iter().any(|member| {
                                member.actor == semantic.actor
                                    && member.device == claim.author
                                    && member.roles.contains(&Role::Write)
                            });
                        if !authorized {
                            ChangeClaimReason::Unauthorized
                        } else {
                            match batch.dispositions.get(&hash).copied() {
                                Some(ProtocolDisposition::Pending) => {
                                    ChangeClaimReason::UnresolvedControl
                                }
                                Some(ProtocolDisposition::Excluded) => {
                                    ChangeClaimReason::AuthorizedCurrentExcluded
                                }
                                Some(ProtocolDisposition::Accepted) => {
                                    ChangeClaimReason::AuthorizedCanonical
                                }
                                Some(
                                    ProtocolDisposition::Invalid
                                    | ProtocolDisposition::UnsupportedRevision,
                                )
                                | None => ChangeClaimReason::InvalidReferencedControl,
                            }
                        }
                    }
                    ReferencedControlState::NoncanonicalValid(control) => {
                        if charge_evaluation_work(
                            budget,
                            cancellation,
                            WorkCounter::Control,
                            u64::try_from(control.members().len()).unwrap_or(u64::MAX),
                        )
                        .is_err()
                        {
                            return Some(Err(()));
                        }
                        let authorized = !control.terminal()
                            && control.members().iter().any(|member| {
                                member.actor == semantic.actor
                                    && member.device == claim.author
                                    && member.roles.contains(&Role::Write)
                            });
                        if authorized {
                            ChangeClaimReason::AuthorizedNoncanonical
                        } else {
                            ChangeClaimReason::Unauthorized
                        }
                    }
                    ReferencedControlState::Pending(_) | ReferencedControlState::Missing => {
                        ChangeClaimReason::UnresolvedControl
                    }
                    ReferencedControlState::UnsupportedRevision => {
                        ChangeClaimReason::InvalidReferencedControl
                    }
                    ReferencedControlState::DynamicInvalid(_)
                        if matches!(
                            batch.dispositions.get(&hash),
                            Some(ProtocolDisposition::Excluded)
                        ) =>
                    {
                        ChangeClaimReason::AuthorizedCurrentExcluded
                    }
                    ReferencedControlState::WrongKind
                    | ReferencedControlState::WrongCoordinate
                    | ReferencedControlState::StaticInvalid
                    | ReferencedControlState::DynamicInvalid(_) => {
                        ChangeClaimReason::InvalidReferencedControl
                    }
                }))
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|()| {
                if cancellation.is_cancelled() {
                    Completion::Cancelled
                } else {
                    Completion::BudgetExhausted
                }
            })?;
        let disposition = reduce_reasoned_change_outcome(lineage, &states);
        batch.dispositions.insert(hash, disposition);
    }
    Ok(())
}

fn reduce_reasoned_change_outcome(
    lineage: FinalLineageChangeState,
    claims: &[ChangeClaimReason],
) -> ProtocolDisposition {
    if lineage == FinalLineageChangeState::Accepted {
        ProtocolDisposition::Accepted
    } else if lineage == FinalLineageChangeState::CanonicalPruned {
        ProtocolDisposition::Excluded
    } else if claims.contains(&ChangeClaimReason::UnresolvedControl) {
        ProtocolDisposition::Pending
    } else if claims.contains(&ChangeClaimReason::AuthorizedNoncanonical)
        || claims.contains(&ChangeClaimReason::AuthorizedCurrentExcluded)
    {
        ProtocolDisposition::Excluded
    } else if !claims.is_empty()
        && claims
            .iter()
            .all(|state| *state == ChangeClaimReason::UnsupportedCarrier)
    {
        ProtocolDisposition::UnsupportedRevision
    } else {
        ProtocolDisposition::Invalid
    }
}

fn resolve_selected_manifest(
    view: &DocumentEvidenceView<'_>,
    control_dispositions: &std::collections::BTreeMap<crate::EventId, ProtocolDisposition>,
    statefully_valid_controls: &std::collections::BTreeSet<crate::EventId>,
) -> ResolvedManifestAvailability {
    let corpus = view.corpus();
    let coordinate = view.coordinate();
    let Some(selection) = view.selected_manifest() else {
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
    let state = resolve_referenced_control(
        corpus,
        control_id,
        coordinate,
        control_dispositions,
        statefully_valid_controls,
    );
    match state {
        ReferencedControlState::Canonical(_) => ResolvedManifestAvailability::Available {
            hints,
            control_status: ManifestControlStatus::Canonical,
        },
        ReferencedControlState::NoncanonicalValid(_) => ResolvedManifestAvailability::Available {
            hints,
            control_status: ManifestControlStatus::Noncanonical,
        },
        ReferencedControlState::Missing => ResolvedManifestAvailability::Pending {
            hints,
            reason: ManifestPendingReason::MissingControl,
        },
        ReferencedControlState::Pending(_) => ResolvedManifestAvailability::Pending {
            hints,
            reason: ManifestPendingReason::ControlPending,
        },
        state => ResolvedManifestAvailability::Unavailable {
            event_id: selection.event_id,
            control: Some(control_id),
            diagnostic: state.diagnostic(),
        },
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
    view: &DocumentEvidenceView<'_>,
    manifest: &ResolvedManifestAvailability,
    checkpoints: &[CheckpointVerificationResult],
) -> Vec<DispositionRecord> {
    let corpus = view.corpus();
    let represented_events = view
        .reportable_event_ids()
        .filter(|event_id| {
            matches!(
                corpus.events.get(event_id),
                Some(EventEvidence::VerifiedCarrier {
                    carrier: VerifiedCarrier::Control(_) | VerifiedCarrier::Change(_),
                    ..
                })
            )
        })
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let mut records = view
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

    for event_id in view.reportable_event_ids() {
        let Some(evidence) = corpus.events.get(event_id) else {
            continue;
        };
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

    for record in dynamic_event_disposition_records(manifest, checkpoints) {
        records.insert(
            match record.identifier() {
                ProtocolItemIdentifier::Event(event_id) => event_id,
                ProtocolItemIdentifier::ControlEvent(_) | ProtocolItemIdentifier::ChangeHash(_) => {
                    continue;
                }
            },
            (record.disposition(), record.diagnostic()),
        );
    }

    let descriptor_dispositions = view
        .checkpoint_descriptor_event_ids()
        .into_iter()
        .flatten()
        .filter_map(|event_id| {
            records
                .get(event_id)
                .map(|(disposition, _)| (*event_id, *disposition))
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    for chunk in
        view.reportable_event_ids()
            .filter_map(|event_id| match corpus.events.get(event_id) {
                Some(EventEvidence::VerifiedCarrier {
                    carrier: VerifiedCarrier::CheckpointChunk(chunk),
                    ..
                }) => Some(chunk.as_ref()),
                _ => None,
            })
    {
        let state = resolve_referenced_descriptor(
            corpus,
            chunk.descriptor_id(),
            view.coordinate(),
            &descriptor_dispositions,
        );
        let prior = records.get(&chunk.event_id()).copied();
        let final_record = state
            .dependent_disposition()
            .map(|disposition| {
                let diagnostic = prior.and_then(|(prior_disposition, diagnostic)| {
                    (prior_disposition == disposition).then_some(diagnostic)
                });
                (disposition, diagnostic.flatten())
            })
            .or_else(|| {
                prior.filter(|(disposition, _)| *disposition != ProtocolDisposition::Excluded)
            })
            .unwrap_or((ProtocolDisposition::Pending, None));
        records.insert(chunk.event_id(), final_record);
    }

    debug_assert!(view.reportable_event_ids().all(|event_id| {
        !matches!(
            corpus.events.get(event_id),
            Some(EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::CheckpointChunk(_),
                ..
            })
        ) || records
            .get(event_id)
            .is_some_and(|(disposition, _)| *disposition != ProtocolDisposition::Excluded)
    }));

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

fn dynamic_event_disposition_records(
    manifest: &ResolvedManifestAvailability,
    checkpoints: &[CheckpointVerificationResult],
) -> Vec<DispositionRecord> {
    let mut records = std::collections::BTreeMap::new();
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

struct CheckpointEvaluation {
    results: Vec<CheckpointVerificationResult>,
    stop: Option<CheckpointWorkStop>,
}

fn verify_checkpoints(
    view: &DocumentEvidenceView<'_>,
    canonical_controls: &[crate::EventId],
    control_dispositions: &std::collections::BTreeMap<crate::EventId, ProtocolDisposition>,
    statefully_valid_controls: &std::collections::BTreeSet<crate::EventId>,
    accepted_at_control: &std::collections::BTreeMap<crate::EventId, AcceptedAtControl>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> CheckpointEvaluation {
    let corpus = view.corpus();
    let prepared = (|| {
        charge_checkpoint_work(
            budget,
            cancellation,
            u64::try_from(canonical_controls.len()).unwrap_or(u64::MAX),
        )?;
        let authorizations = checkpoint_authorizations(
            view,
            control_dispositions,
            statefully_valid_controls,
            budget,
            cancellation,
        )?;
        let chunk_sets = checkpoint_chunk_sets(view, budget, cancellation)?;
        let carrier_coverage =
            checkpoint_carrier_coverage(view, canonical_controls, budget, cancellation)?;
        let accepted_history =
            checkpoint_accepted_history(view, accepted_at_control, budget, cancellation)?;
        Ok::<_, CheckpointWorkStop>((
            authorizations,
            chunk_sets,
            carrier_coverage,
            accepted_history,
        ))
    })();
    let (authorizations, chunk_sets, carrier_coverage, accepted_history) = match prepared {
        Ok(prepared) => prepared,
        Err(stop) => {
            return CheckpointEvaluation {
                results: Vec::new(),
                stop: Some(stop),
            };
        }
    };
    let mut results = Vec::new();
    for descriptor_id in view
        .checkpoint_descriptor_event_ids()
        .into_iter()
        .flatten()
        .copied()
    {
        if let Err(stop) = charge_checkpoint_work(budget, cancellation, 1) {
            return CheckpointEvaluation {
                results,
                stop: Some(stop),
            };
        }
        let Some(EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::CheckpointDescriptor(descriptor),
            ..
        }) = corpus.events.get(&descriptor_id)
        else {
            continue;
        };
        let chunk_events =
            match checkpoint_chunk_event_ids(view, descriptor_id, budget, cancellation) {
                Ok(events) => events,
                Err(stop) => {
                    return CheckpointEvaluation {
                        results,
                        stop: Some(stop),
                    };
                }
            };
        let coverage_result = carrier_coverage.get(&descriptor_id);
        let accepted_result = accepted_history.get(&descriptor_id);
        let coverage_ref = coverage_result.and_then(|value| value.as_ref().ok());
        let accepted_ref = accepted_result.and_then(|value| value.as_ref().ok());
        let commitments = descriptor.descriptor();
        let result_work = coverage_ref
            .map_or(0, std::collections::BTreeSet::len)
            .saturating_add(accepted_ref.map_or(0, std::collections::BTreeSet::len))
            .saturating_add(commitments.heads.len());
        if let Err(stop) = charge_checkpoint_work(
            budget,
            cancellation,
            u64::try_from(result_work).unwrap_or(u64::MAX),
        ) {
            return CheckpointEvaluation {
                results,
                stop: Some(stop),
            };
        }
        let coverage = coverage_ref.cloned().unwrap_or_default();
        let accepted = accepted_ref.cloned().unwrap_or_default();
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
        let stop = match status {
            CheckpointVerificationStatus::BudgetExhausted => Some(CheckpointWorkStop::Budget),
            CheckpointVerificationStatus::Cancelled => Some(CheckpointWorkStop::Cancelled),
            _ => None,
        };
        if let Some(stop) = stop {
            return CheckpointEvaluation {
                results,
                stop: Some(stop),
            };
        }
        results.push(CheckpointVerificationResult::new(
            descriptor_id,
            chunk_events,
            descriptor.snapshot_hash(),
            commitments.heads.iter().copied().collect(),
            commitments.change_count,
            commitments.change_set_hash,
            coverage.into_iter().collect(),
            accepted.into_iter().collect(),
            status,
        ));
    }
    CheckpointEvaluation {
        results,
        stop: None,
    }
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
    view: &DocumentEvidenceView<'_>,
    descriptor_id: crate::EventId,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<Vec<crate::EventId>, CheckpointWorkStop> {
    let event_ids = view
        .checkpoint_chunk_event_ids(descriptor_id)
        .into_iter()
        .flatten()
        .copied();
    let mut result = Vec::new();
    for event_id in event_ids {
        charge_checkpoint_work(budget, cancellation, 1)?;
        result.push(event_id);
    }
    Ok(result)
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
    view: &DocumentEvidenceView<'_>,
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
    let corpus = view.corpus();
    for descriptor_id in view.checkpoint_descriptor_event_ids().into_iter().flatten() {
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
    view: &DocumentEvidenceView<'_>,
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
    let corpus = view.corpus();
    for descriptor_id in view.checkpoint_descriptor_event_ids().into_iter().flatten() {
        charge_checkpoint_work(budget, cancellation, 1)?;
        if let Some(EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::CheckpointDescriptor(descriptor),
            ..
        }) = corpus.events.get(descriptor_id)
        {
            let result = historical_carrier_coverage(
                view,
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
    view: &DocumentEvidenceView<'_>,
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
    let corpus = view.corpus();
    let descriptor_ids = view.checkpoint_descriptor_event_ids().into_iter().flatten();
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
        for chunk_id in view
            .checkpoint_chunk_event_ids(*descriptor_id)
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
    view: &DocumentEvidenceView<'_>,
    control_dispositions: &std::collections::BTreeMap<crate::EventId, ProtocolDisposition>,
    statefully_valid_controls: &std::collections::BTreeSet<crate::EventId>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<std::collections::BTreeMap<crate::EventId, DescriptorAuthorization>, CheckpointWorkStop>
{
    let corpus = view.corpus();
    let coordinate = view.coordinate();
    let mut authorizations = std::collections::BTreeMap::new();
    for descriptor_id in view.checkpoint_descriptor_event_ids().into_iter().flatten() {
        charge_checkpoint_work(budget, cancellation, 1)?;
        if let Some(EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::CheckpointDescriptor(descriptor),
            ..
        }) = corpus.events.get(descriptor_id)
        {
            let state = resolve_referenced_control(
                corpus,
                descriptor.control_id(),
                coordinate,
                control_dispositions,
                statefully_valid_controls,
            );
            let member_work = match state {
                ReferencedControlState::Canonical(control) => {
                    u64::try_from(control.members().len()).unwrap_or(u64::MAX)
                }
                _ => 0,
            };
            charge_checkpoint_work(budget, cancellation, member_work)?;
            authorizations.insert(*descriptor_id, authorize_descriptor(descriptor, state));
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
    const fn completion(self) -> Completion {
        match self {
            Self::Budget => Completion::BudgetExhausted,
            Self::Cancelled => Completion::Cancelled,
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

fn charge_ingress(
    view: &DocumentEvidenceView<'_>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<(), Completion> {
    let event_count = u64::try_from(view.evaluation_event_count()).unwrap_or(u64::MAX);
    let carrier_count = u64::try_from(view.carrier_evidence_count()).unwrap_or(u64::MAX);
    let decode_bytes = view.decode_work_bytes().unwrap_or(u64::MAX);
    for (counter, amount) in [
        (WorkCounter::Event, event_count),
        (WorkCounter::Carrier, carrier_count),
        (WorkCounter::DecodeByte, decode_bytes),
    ] {
        charge_evaluation_work(budget, cancellation, counter, amount)?;
    }
    Ok(())
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ReportFinalizationPlan {
    controls: u64,
    changes: u64,
    events: u64,
    checkpoints: u64,
    digests: u64,
    evidence: u64,
    invariants: u64,
    fixed_overhead: u64,
}

impl ReportFinalizationPlan {
    fn from_view(view: &DocumentEvidenceView<'_>) -> Result<Self, ()> {
        let events = u64::try_from(view.evaluation_event_count()).map_err(|_| ())?;
        let controls = u64::try_from(view.control_count()).map_err(|_| ())?;
        let hashes = u64::try_from(view.change_hash_count()).map_err(|_| ())?;
        let digests = events
            .checked_add(controls)
            .and_then(|value| value.checked_add(hashes.checked_mul(3)?))
            .and_then(|value| value.checked_add(8))
            .ok_or(())?;
        let plan = Self {
            controls,
            changes: hashes,
            events,
            checkpoints: events,
            digests,
            evidence: events,
            invariants: REPORT_INVARIANT_ITEMS,
            fixed_overhead: 8,
        };
        plan.total().ok_or(())?;
        Ok(plan)
    }

    fn total(self) -> Option<u64> {
        [
            self.controls,
            self.changes,
            self.events,
            self.checkpoints,
            self.digests,
            self.evidence,
            self.invariants,
            self.fixed_overhead,
        ]
        .into_iter()
        .try_fold(0_u64, u64::checked_add)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FinalizationDimension {
    Controls,
    Changes,
    Events,
    Checkpoints,
    Digests,
    Evidence,
    Invariants,
    FixedOverhead,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FinalizationPermitError;

#[derive(Debug)]
struct ReportFinalizationPermit {
    remaining: ReportFinalizationPlan,
    active: bool,
}

impl ReportFinalizationPermit {
    fn reserve(
        plan: ReportFinalizationPlan,
        budget: &mut WorkBudget,
    ) -> Result<Self, crate::BudgetExhausted> {
        budget.charge(WorkCounter::Assertion, plan.total().unwrap_or(u64::MAX))?;
        Ok(Self {
            remaining: plan,
            active: true,
        })
    }

    fn consume(
        &mut self,
        dimension: FinalizationDimension,
        amount: u64,
    ) -> Result<(), FinalizationPermitError> {
        if !self.active {
            return Err(FinalizationPermitError);
        }
        let remaining = match dimension {
            FinalizationDimension::Controls => &mut self.remaining.controls,
            FinalizationDimension::Changes => &mut self.remaining.changes,
            FinalizationDimension::Events => &mut self.remaining.events,
            FinalizationDimension::Checkpoints => &mut self.remaining.checkpoints,
            FinalizationDimension::Digests => &mut self.remaining.digests,
            FinalizationDimension::Evidence => &mut self.remaining.evidence,
            FinalizationDimension::Invariants => &mut self.remaining.invariants,
            FinalizationDimension::FixedOverhead => &mut self.remaining.fixed_overhead,
        };
        *remaining = remaining
            .checked_sub(amount)
            .ok_or(FinalizationPermitError)?;
        Ok(())
    }

    fn finish_interrupted(&mut self) -> Result<(), FinalizationPermitError> {
        if !self.active || self.remaining != ReportFinalizationPlan::default() {
            return Err(FinalizationPermitError);
        }
        self.active = false;
        Ok(())
    }

    fn consume_interrupted_omissions(&mut self) -> Result<(), FinalizationPermitError> {
        let remaining = self.remaining;
        for (dimension, amount) in [
            (FinalizationDimension::Controls, remaining.controls),
            (FinalizationDimension::Changes, remaining.changes),
            (FinalizationDimension::Events, remaining.events),
            (FinalizationDimension::Checkpoints, remaining.checkpoints),
            (FinalizationDimension::Digests, remaining.digests),
            (FinalizationDimension::Evidence, remaining.evidence),
            (FinalizationDimension::Invariants, remaining.invariants),
            (
                FinalizationDimension::FixedOverhead,
                remaining.fixed_overhead,
            ),
        ] {
            self.consume(dimension, amount)?;
        }
        Ok(())
    }

    fn refund(&mut self, budget: &mut WorkBudget) -> Result<(), FinalizationPermitError> {
        if !self.active {
            return Err(FinalizationPermitError);
        }
        let remaining = core::mem::take(&mut self.remaining)
            .total()
            .unwrap_or(u64::MAX);
        self.active = false;
        budget
            .refund(WorkCounter::Assertion, remaining)
            .map_err(|_| FinalizationPermitError)
    }
}

fn reserved_interrupted_report(
    revision: ProtocolRevision,
    coordinate: DocumentCoordinate,
    completion: Completion,
    permit: &mut ReportFinalizationPermit,
) -> Result<EvaluationReport, EvaluationError> {
    permit
        .consume(FinalizationDimension::Digests, 8)
        .and_then(|()| permit.consume(FinalizationDimension::Invariants, REPORT_INVARIANT_ITEMS))
        .and_then(|()| permit.consume(FinalizationDimension::FixedOverhead, 8))
        .and_then(|()| permit.consume_interrupted_omissions())
        .and_then(|()| permit.finish_interrupted())
        .map_err(|_| EvaluationError::ReportInvariant)?;
    compact_interrupted_report(revision, coordinate, completion)
}

fn reserved_batch_report(
    revision: ProtocolRevision,
    coordinate: DocumentCoordinate,
    batch: BatchEvaluationReport,
    manifest: ResolvedManifestAvailability,
    checkpoints: Vec<CheckpointVerificationResult>,
    permit: &mut ReportFinalizationPermit,
) -> Result<EvaluationReport, EvaluationError> {
    let control_count = u64::try_from(batch.control_dispositions.len()).unwrap_or(u64::MAX);
    let change_count = u64::try_from(batch.dispositions.len()).unwrap_or(u64::MAX);
    let checkpoint_count = u64::try_from(checkpoints.len()).unwrap_or(u64::MAX);
    let event_count = checkpoint_count
        .saturating_add(
            checkpoints
                .iter()
                .map(|checkpoint| checkpoint.chunk_events().len())
                .sum::<usize>()
                .try_into()
                .unwrap_or(u64::MAX),
        )
        .saturating_add(u64::from(!matches!(
            manifest,
            ResolvedManifestAvailability::Missing
        )));
    let digest_count = control_count
        .saturating_add(change_count)
        .saturating_add(event_count)
        .saturating_add(8);
    for (dimension, amount) in [
        (FinalizationDimension::Controls, control_count),
        (FinalizationDimension::Changes, change_count),
        (FinalizationDimension::Events, event_count),
        (FinalizationDimension::Checkpoints, checkpoint_count),
        (FinalizationDimension::Digests, digest_count),
        (FinalizationDimension::Evidence, 0),
        (FinalizationDimension::Invariants, REPORT_INVARIANT_ITEMS),
        (FinalizationDimension::FixedOverhead, 8),
    ] {
        permit
            .consume(dimension, amount)
            .map_err(|_| EvaluationError::ReportInvariant)?;
    }
    permit
        .consume_interrupted_omissions()
        .and_then(|()| permit.finish_interrupted())
        .map_err(|_| EvaluationError::ReportInvariant)?;
    compact_batch_report(revision, coordinate, batch, manifest, checkpoints)
}

fn compact_interrupted_report(
    revision: ProtocolRevision,
    coordinate: DocumentCoordinate,
    completion: Completion,
) -> Result<EvaluationReport, EvaluationError> {
    let history_digest = history_digest(revision, coordinate, &[], &[], &[])
        .map_err(|_| EvaluationError::ReportInvariant)?;
    let disposition_items = disposition_items(&[]).map_err(|_| EvaluationError::ReportInvariant)?;
    let dispositions_digest = dispositions_digest(revision, coordinate, &disposition_items)
        .map_err(|_| EvaluationError::ReportInvariant)?;
    let failure = match completion {
        Completion::BudgetExhausted => EvaluationFailure::BudgetExhausted,
        Completion::Cancelled => EvaluationFailure::Cancelled,
        Completion::Complete => return Err(EvaluationError::ReportInvariant),
    };
    EvaluationReport::from_parts(EvaluationReportParts {
        coordinate,
        canonical_controls: Vec::new(),
        disposition_records: Vec::new(),
        control_dispositions: Vec::new(),
        dispositions: Vec::new(),
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
        manifest: ResolvedManifestAvailability::Missing,
        completion,
        failure: Some(failure),
        document: None,
    })
    .map_err(|_| EvaluationError::ReportInvariant)
}

fn compact_batch_report(
    revision: ProtocolRevision,
    coordinate: DocumentCoordinate,
    batch: BatchEvaluationReport,
    manifest: ResolvedManifestAvailability,
    checkpoints: Vec<CheckpointVerificationResult>,
) -> Result<EvaluationReport, EvaluationError> {
    if batch.completion == Completion::Complete {
        return Err(EvaluationError::ReportInvariant);
    }
    let control_dispositions = batch.control_dispositions.into_iter().collect::<Vec<_>>();
    let dispositions = batch.dispositions.into_iter().collect::<Vec<_>>();
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
    disposition_records.extend(dispositions.iter().map(|(hash, disposition)| {
        DispositionRecord::new(ProtocolItemIdentifier::from(*hash), *disposition, None)
    }));
    disposition_records.extend(dynamic_event_disposition_records(&manifest, &checkpoints));
    let accepted_changes = disposition_hashes(&dispositions, ProtocolDisposition::Accepted);
    let pending_changes = disposition_hashes(&dispositions, ProtocolDisposition::Pending);
    let excluded_changes = disposition_hashes(&dispositions, ProtocolDisposition::Excluded);
    let invalid_changes = disposition_hashes(&dispositions, ProtocolDisposition::Invalid);
    let heads = batch.heads.into_iter().collect::<Vec<_>>();
    let history_digest = history_digest(
        revision,
        coordinate,
        &batch.canonical_controls,
        &accepted_changes,
        &heads,
    )
    .map_err(|_| EvaluationError::ReportInvariant)?;
    let disposition_items =
        disposition_items(&disposition_records).map_err(|_| EvaluationError::ReportInvariant)?;
    let dispositions_digest = dispositions_digest(revision, coordinate, &disposition_items)
        .map_err(|_| EvaluationError::ReportInvariant)?;
    EvaluationReport::from_parts(EvaluationReportParts {
        coordinate,
        canonical_controls: batch.canonical_controls,
        disposition_records,
        control_dispositions,
        dispositions,
        accepted_changes,
        pending_changes,
        excluded_changes,
        invalid_changes,
        heads,
        evidence: Vec::new(),
        checkpoints,
        history_digest,
        dispositions_digest,
        integrity_alerts: batch.integrity_alerts,
        manifest,
        completion: batch.completion,
        failure: batch.failure,
        document: None,
    })
    .map_err(|_| EvaluationError::ReportInvariant)
}

fn interrupt_batch(batch: &mut BatchEvaluationReport, completion: Completion) {
    batch.completion = completion;
    batch.failure = Some(match completion {
        Completion::BudgetExhausted => EvaluationFailure::BudgetExhausted,
        Completion::Cancelled => EvaluationFailure::Cancelled,
        Completion::Complete => EvaluationFailure::InvariantViolation,
    });
    batch.materialized_document = None;
}

fn prepare_controls(
    view: &DocumentEvidenceView<'_>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<PreparedControls, Completion> {
    let corpus = view.corpus();
    let coordinate = view.coordinate();
    let ancestry_index = build_control_ancestry_index(view, budget, cancellation)?;
    let assumed_control_dispositions = corpus
        .indexes
        .controls
        .controls_by_id
        .keys()
        .copied()
        .map(|event_id| (event_id, ProtocolDisposition::Accepted))
        .collect::<std::collections::BTreeMap<_, _>>();
    let assumed_statefully_valid = corpus
        .indexes
        .controls
        .controls_by_id
        .keys()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let mut dispositions = std::collections::BTreeMap::new();
    let mut controls = Vec::new();
    for control_id in corpus
        .indexes
        .controls
        .controls_by_id
        .keys()
        .filter(|event_id| view.contains_reportable(event_id))
    {
        charge_evaluation_work(budget, cancellation, WorkCounter::Control, 1)?;
        let Some(EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::Control(control),
            ..
        }) = corpus.events.get(control_id)
        else {
            continue;
        };
        if control.coordinate() != coordinate {
            continue;
        }
        let envelope = ControlEnvelope::from_validated(control.as_ref().clone());
        let parent_disposition = control.parent().and_then(|parent_id| {
            ControlParentState::from(resolve_referenced_control(
                corpus,
                parent_id,
                coordinate,
                &assumed_control_dispositions,
                &assumed_statefully_valid,
            ))
            .dependent_disposition()
        });
        let frontier_missing = control
            .base_heads()
            .any(|head| view.change_carrier_event_ids(head).is_none());
        let disposition = if let Some(disposition) = parent_disposition {
            disposition
        } else if frontier_missing {
            ProtocolDisposition::Pending
        } else if control.parent().is_none() {
            let predecessor = if let Some(link) = control.predecessor() {
                charge_evaluation_work(budget, cancellation, WorkCounter::Control, 1)?;
                match ControlParentState::from(resolve_referenced_control(
                    corpus,
                    link.terminal_control,
                    link.coordinate,
                    &assumed_control_dispositions,
                    &assumed_statefully_valid,
                )) {
                    ControlParentState::Canonical(terminal)
                    | ControlParentState::NoncanonicalValid(terminal) => {
                        Ok(Some(ControlEnvelope::from_validated(terminal.clone())))
                    }
                    ControlParentState::Pending(_) | ControlParentState::Missing => {
                        Err(ProtocolDisposition::Pending)
                    }
                    ControlParentState::WrongKind
                    | ControlParentState::WrongCoordinate
                    | ControlParentState::StaticInvalid
                    | ControlParentState::DynamicInvalid(_)
                    | ControlParentState::UnsupportedRevision => Err(ProtocolDisposition::Invalid),
                }
            } else {
                Ok(None)
            };
            let outcome = match predecessor {
                Ok(predecessor) => classify_genesis(&envelope, predecessor.as_ref()).disposition(),
                Err(disposition) => disposition,
            };
            if outcome == ProtocolDisposition::Accepted {
                ProtocolDisposition::Excluded
            } else {
                outcome
            }
        } else if !parent_continuity_is_valid(corpus, control, budget, cancellation)?
            || !account_continuity_is_valid(corpus, control, budget, cancellation)?
            || !role_continuity_is_valid(corpus, control, budget, cancellation)?
            || !device_ancestry_is_valid(&ancestry_index, control)
            || !terminal_continuity_is_valid(corpus, control, budget, cancellation)?
        {
            ProtocolDisposition::Invalid
        } else {
            ProtocolDisposition::Excluded
        };
        dispositions.insert(control.event_id(), disposition);
        if disposition != ProtocolDisposition::Excluded {
            continue;
        }
        controls.push(BatchControl {
            event_id: control.event_id(),
            parent: control.parent(),
            accepted_base: control.base_heads().collect(),
            frozen: control.terminal(),
            changes: changes_for_control(view, control, budget, cancellation)?,
            envelope: Some(envelope),
        });
    }
    let parents = corpus
        .indexes
        .controls
        .controls_by_id
        .values()
        .filter(|record| view.contains_reportable(&record.event_id))
        .map(|record| (record.event_id, record.parent))
        .collect::<std::collections::BTreeMap<_, _>>();
    propagate_control_parent_dispositions(&parents, &mut dispositions);
    controls.retain(|control| {
        dispositions.get(&control.event_id) == Some(&ProtocolDisposition::Excluded)
    });
    let states = dispositions
        .iter()
        .map(|(event_id, disposition)| {
            let state = match disposition {
                ProtocolDisposition::Accepted | ProtocolDisposition::Excluded => {
                    PreparedControlState::Ready
                }
                ProtocolDisposition::Pending => PreparedControlState::Pending,
                ProtocolDisposition::Invalid | ProtocolDisposition::UnsupportedRevision => {
                    PreparedControlState::Invalid
                }
            };
            (*event_id, state)
        })
        .collect();
    Ok(PreparedControls {
        controls,
        dispositions,
        states,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PreparedControlState {
    Ready,
    Pending,
    Invalid,
}

#[derive(Debug)]
struct PreparedControls {
    controls: Vec<BatchControl>,
    dispositions: std::collections::BTreeMap<crate::EventId, ProtocolDisposition>,
    states: std::collections::BTreeMap<crate::EventId, PreparedControlState>,
}

impl PreparedControls {
    fn into_parts(
        self,
    ) -> (
        Vec<BatchControl>,
        std::collections::BTreeMap<crate::EventId, ProtocolDisposition>,
    ) {
        debug_assert_eq!(self.states.len(), self.dispositions.len());
        (self.controls, self.dispositions)
    }
}

fn device_ancestry_is_valid(
    ancestry_index: &std::collections::BTreeMap<crate::EventId, Option<Vec<ControlEnvelope>>>,
    child: &crate::carrier::control::ValidatedControlCarrier,
) -> bool {
    let Some(ancestry) = ancestry_index
        .get(&child.event_id())
        .and_then(Option::as_ref)
    else {
        return false;
    };
    let child = ControlEnvelope::from_validated(child.clone());
    evaluate_device_ancestry(ancestry, &child) == CandidateResult::Valid
}

fn build_control_ancestry_index(
    view: &DocumentEvidenceView<'_>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<std::collections::BTreeMap<crate::EventId, Option<Vec<ControlEnvelope>>>, Completion> {
    let corpus = view.corpus();
    let mut index =
        std::collections::BTreeMap::<crate::EventId, Option<Vec<ControlEnvelope>>>::new();
    for root in corpus
        .indexes
        .controls
        .controls_by_id
        .keys()
        .filter(|event_id| view.contains_reportable(event_id))
        .copied()
    {
        if index.contains_key(&root) {
            continue;
        }
        let mut path = Vec::new();
        let mut visited = std::collections::BTreeSet::new();
        let mut current = Some(root);
        let mut base = Vec::new();
        let mut valid = true;
        while let Some(event_id) = current {
            if !view.contains_input(&event_id) {
                valid = false;
                break;
            }
            if let Some(cached) = index.get(&event_id) {
                match cached {
                    Some(ancestry) => {
                        base = ancestry.clone();
                        let Some(EventEvidence::VerifiedCarrier {
                            carrier: VerifiedCarrier::Control(control),
                            ..
                        }) = corpus.events.get(&event_id)
                        else {
                            valid = false;
                            break;
                        };
                        base.push(ControlEnvelope::from_validated(control.as_ref().clone()));
                    }
                    None => valid = false,
                }
                break;
            }
            charge_evaluation_work(budget, cancellation, WorkCounter::Control, 1)?;
            if !visited.insert(event_id) {
                valid = false;
                break;
            }
            let Some(EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::Control(control),
                ..
            }) = corpus.events.get(&event_id)
            else {
                valid = false;
                break;
            };
            path.push(event_id);
            current = control.parent();
        }
        if !valid {
            for event_id in path {
                index.insert(event_id, None);
            }
            continue;
        }
        for event_id in path.into_iter().rev() {
            charge_evaluation_work(
                budget,
                cancellation,
                WorkCounter::Control,
                u64::try_from(base.len()).unwrap_or(u64::MAX),
            )?;
            index.insert(event_id, Some(base.clone()));
            let Some(EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::Control(control),
                ..
            }) = corpus.events.get(&event_id)
            else {
                index.insert(event_id, None);
                break;
            };
            base.push(ControlEnvelope::from_validated(control.as_ref().clone()));
        }
    }
    Ok(index)
}

fn role_continuity_is_valid(
    corpus: &EvidenceCorpus,
    child: &crate::carrier::control::ValidatedControlCarrier,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<bool, Completion> {
    let Some(parent_id) = child.parent() else {
        return Ok(true);
    };
    charge_evaluation_work(budget, cancellation, WorkCounter::Control, 1)?;
    let Some(EventEvidence::VerifiedCarrier {
        carrier: VerifiedCarrier::Control(parent),
        ..
    }) = corpus.events.get(&parent_id)
    else {
        return Ok(false);
    };
    charge_member_comparisons(parent, child, budget, cancellation)?;
    let parent = ControlEnvelope::from_validated(parent.as_ref().clone());
    let child = ControlEnvelope::from_validated(child.clone());
    Ok(evaluate_role_continuity(&parent, &child) == CandidateResult::Valid)
}

fn account_continuity_is_valid(
    corpus: &EvidenceCorpus,
    child: &crate::carrier::control::ValidatedControlCarrier,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<bool, Completion> {
    let Some(parent_id) = child.parent() else {
        return Ok(true);
    };
    charge_evaluation_work(budget, cancellation, WorkCounter::Control, 1)?;
    let Some(EventEvidence::VerifiedCarrier {
        carrier: VerifiedCarrier::Control(parent),
        ..
    }) = corpus.events.get(&parent_id)
    else {
        return Ok(false);
    };
    charge_member_comparisons(parent, child, budget, cancellation)?;
    let parent = ControlEnvelope::from_validated(parent.as_ref().clone());
    let child = ControlEnvelope::from_validated(child.clone());
    Ok(evaluate_account_continuity(&parent, &child) == CandidateResult::Valid)
}

fn parent_continuity_is_valid(
    corpus: &EvidenceCorpus,
    child: &crate::carrier::control::ValidatedControlCarrier,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<bool, Completion> {
    let Some(parent_id) = child.parent() else {
        return Ok(true);
    };
    charge_evaluation_work(budget, cancellation, WorkCounter::Control, 1)?;
    let Some(EventEvidence::VerifiedCarrier {
        carrier: VerifiedCarrier::Control(parent),
        ..
    }) = corpus.events.get(&parent_id)
    else {
        return Ok(false);
    };
    let parent = ControlEnvelope::from_validated(parent.as_ref().clone());
    let child = ControlEnvelope::from_validated(child.clone());
    Ok(evaluate_parent_continuity(&parent, &child) == CandidateResult::Valid)
}

fn terminal_continuity_is_valid(
    corpus: &EvidenceCorpus,
    child: &crate::carrier::control::ValidatedControlCarrier,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<bool, Completion> {
    let Some(parent_id) = child.parent() else {
        return Ok(true);
    };
    charge_evaluation_work(budget, cancellation, WorkCounter::Control, 1)?;
    let Some(EventEvidence::VerifiedCarrier {
        carrier: VerifiedCarrier::Control(parent),
        ..
    }) = corpus.events.get(&parent_id)
    else {
        return Ok(false);
    };
    let parent = ControlEnvelope::from_validated(parent.as_ref().clone());
    let child = ControlEnvelope::from_validated(child.clone());
    Ok(evaluate_terminal_continuity(&parent, &child) == CandidateResult::Valid)
}

fn changes_for_control(
    view: &DocumentEvidenceView<'_>,
    control: &crate::carrier::control::ValidatedControlCarrier,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<Vec<BatchChange>, Completion> {
    let hashes = view.change_hashes_for_control(control.event_id());
    let mut changes = Vec::new();
    for hash in hashes.into_iter().flatten().copied() {
        charge_evaluation_work(budget, cancellation, WorkCounter::Carrier, 1)?;
        if let Some(change) = change_for_hash(view, control, hash, budget, cancellation)? {
            changes.push(change);
        }
    }
    Ok(changes)
}

fn change_for_hash(
    view: &DocumentEvidenceView<'_>,
    control: &crate::carrier::control::ValidatedControlCarrier,
    hash: ChangeHash,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<Option<BatchChange>, Completion> {
    let Some(event_ids) = view.change_carrier_event_ids(hash) else {
        return Ok(None);
    };
    let corpus = view.corpus();
    let mut raw = None;
    let mut carriers = Vec::new();
    for event_id in event_ids {
        charge_evaluation_work(budget, cancellation, WorkCounter::Carrier, 1)?;
        if let Some(EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::Change(change),
            ..
        }) = corpus.events.get(event_id)
            && change.control_id() == control.event_id()
            && change.coordinate() == view.coordinate()
        {
            let raw_bytes = change.canonical_raw_bytes();
            charge_evaluation_work(
                budget,
                cancellation,
                WorkCounter::DecodeByte,
                u64::try_from(raw_bytes.len()).unwrap_or(u64::MAX),
            )?;
            raw.get_or_insert_with(|| raw_bytes.to_vec());
            let dependency_count = u64::try_from(change.dependencies().count()).unwrap_or(u64::MAX);
            charge_evaluation_work(
                budget,
                cancellation,
                WorkCounter::GraphEdge,
                dependency_count,
            )?;
            carriers.push(CandidateCarrier {
                event_id: change.event_id(),
                change_hash: change.change_hash(),
                actor: change.actor(),
                sequence: change.sequence(),
                start_op: change.start_op(),
                operation_count: change.operation_count(),
                dependencies: change.dependencies().collect(),
                control_id: change.control_id(),
                author: change.author_device(),
            });
        }
    }
    let Ok(candidate) = ChangeCandidate::from_carriers(carriers) else {
        return Ok(None);
    };
    charge_evaluation_work(
        budget,
        cancellation,
        WorkCounter::Control,
        u64::try_from(control.members().len()).unwrap_or(u64::MAX),
    )?;
    let authorized = control.members().iter().any(|member| {
        member.actor == candidate.actor
            && member.device == candidate.author
            && member.roles.contains(&Role::Write)
    });
    Ok(Some(BatchChange {
        candidate,
        legacy_eligible: authorized && !control.terminal(),
        raw_change: raw,
    }))
}

fn charge_member_comparisons(
    parent: &crate::carrier::control::ValidatedControlCarrier,
    child: &crate::carrier::control::ValidatedControlCarrier,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<(), Completion> {
    let parent_members = u64::try_from(parent.members().len()).unwrap_or(u64::MAX);
    let child_members = u64::try_from(child.members().len()).unwrap_or(u64::MAX);
    let member_pairs = parent_members.saturating_mul(child_members);
    let role_pairs = child
        .members()
        .iter()
        .try_fold(0_u64, |total, grant| {
            total.checked_add(
                u64::try_from(grant.roles.len())
                    .unwrap_or(u64::MAX)
                    .saturating_mul(2),
            )
        })
        .unwrap_or(u64::MAX);
    charge_evaluation_work(
        budget,
        cancellation,
        WorkCounter::Control,
        member_pairs.saturating_add(role_pairs),
    )
}

fn charge_evaluation_work(
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
    counter: WorkCounter,
    amount: u64,
) -> Result<(), Completion> {
    if cancellation.is_cancelled() {
        return Err(Completion::Cancelled);
    }
    budget
        .charge(counter, amount)
        .map_err(|_| Completion::BudgetExhausted)
}

#[cfg(test)]
mod tests {
    use super::{
        ChangeClaimReason, CheckpointWorkStop, FinalLineageChangeState, FinalizationDimension,
        ReportFinalizationPermit, ReportFinalizationPlan, assembly_status, charge_checkpoint_work,
        join_status, reduce_reasoned_change_outcome,
    };
    use crate::CheckpointVerificationStatus as Status;
    use crate::checkpoint::AssemblyError;
    use crate::checkpoint::join::JoinError;
    use crate::{WorkBudget, WorkCounter};

    #[test]
    fn reasoned_change_outcome_uses_final_precedence() {
        use crate::ProtocolDisposition::{
            Accepted, Excluded, Invalid, Pending, UnsupportedRevision,
        };
        assert_eq!(
            reduce_reasoned_change_outcome(
                FinalLineageChangeState::Accepted,
                &[ChangeClaimReason::UnresolvedControl]
            ),
            Accepted
        );
        assert_eq!(
            reduce_reasoned_change_outcome(
                FinalLineageChangeState::CanonicalPruned,
                &[ChangeClaimReason::UnresolvedControl]
            ),
            Excluded
        );
        assert_eq!(
            reduce_reasoned_change_outcome(
                FinalLineageChangeState::Current,
                &[
                    ChangeClaimReason::AuthorizedNoncanonical,
                    ChangeClaimReason::UnresolvedControl
                ]
            ),
            Pending
        );
        assert_eq!(
            reduce_reasoned_change_outcome(
                FinalLineageChangeState::Current,
                &[
                    ChangeClaimReason::InvalidReferencedControl,
                    ChangeClaimReason::UnresolvedControl
                ]
            ),
            Pending
        );
        assert_eq!(
            reduce_reasoned_change_outcome(
                FinalLineageChangeState::Current,
                &[
                    ChangeClaimReason::AuthorizedNoncanonical,
                    ChangeClaimReason::InvalidReferencedControl
                ]
            ),
            Excluded
        );
        assert_eq!(
            reduce_reasoned_change_outcome(
                FinalLineageChangeState::Current,
                &[
                    ChangeClaimReason::UnsupportedCarrier,
                    ChangeClaimReason::UnsupportedCarrier
                ]
            ),
            UnsupportedRevision
        );
        assert_eq!(
            reduce_reasoned_change_outcome(
                FinalLineageChangeState::Current,
                &[
                    ChangeClaimReason::UnsupportedCarrier,
                    ChangeClaimReason::InvalidReferencedControl
                ]
            ),
            Invalid
        );
    }

    #[test]
    fn change_claim_failures_have_stable_diagnostics() {
        assert_eq!(
            ChangeClaimReason::InvalidReferencedControl
                .diagnostic()
                .map(crate::DiagnosticCode::as_str),
            Some("control.parent")
        );
        assert_eq!(
            ChangeClaimReason::Unauthorized
                .diagnostic()
                .map(crate::DiagnosticCode::as_str),
            Some("change.actor")
        );
        assert_eq!(
            ChangeClaimReason::UnsupportedCarrier
                .diagnostic()
                .map(crate::DiagnosticCode::as_str),
            Some("carrier.revision")
        );
        assert!(
            ChangeClaimReason::AuthorizedCanonical
                .diagnostic()
                .is_none()
        );
        assert!(ChangeClaimReason::UnresolvedControl.diagnostic().is_none());
    }

    #[test]
    fn reasoned_change_precedence_matrix_is_complete() {
        use crate::ProtocolDisposition::{
            Accepted, Excluded, Invalid, Pending, UnsupportedRevision,
        };
        use ChangeClaimReason::{
            AuthorizedCanonical, AuthorizedCurrentExcluded, AuthorizedNoncanonical,
            InvalidReferencedControl, Unauthorized, UnresolvedControl, UnsupportedCarrier,
        };
        let cases = [
            (
                FinalLineageChangeState::Accepted,
                vec![UnresolvedControl],
                Accepted,
            ),
            (
                FinalLineageChangeState::CanonicalPruned,
                vec![UnresolvedControl, InvalidReferencedControl],
                Excluded,
            ),
            (
                FinalLineageChangeState::Current,
                vec![UnresolvedControl, AuthorizedNoncanonical],
                Pending,
            ),
            (
                FinalLineageChangeState::Current,
                vec![UnresolvedControl, InvalidReferencedControl],
                Pending,
            ),
            (
                FinalLineageChangeState::Current,
                vec![AuthorizedNoncanonical, InvalidReferencedControl],
                Excluded,
            ),
            (
                FinalLineageChangeState::Current,
                vec![AuthorizedCurrentExcluded, Unauthorized],
                Excluded,
            ),
            (
                FinalLineageChangeState::Current,
                vec![UnsupportedCarrier, UnsupportedCarrier],
                UnsupportedRevision,
            ),
            (
                FinalLineageChangeState::Current,
                vec![UnsupportedCarrier, InvalidReferencedControl],
                Invalid,
            ),
            (
                FinalLineageChangeState::Current,
                vec![Unauthorized],
                Invalid,
            ),
            (
                FinalLineageChangeState::Current,
                vec![AuthorizedCanonical],
                Invalid,
            ),
        ];
        for (lineage, claims, expected) in cases {
            assert_eq!(reduce_reasoned_change_outcome(lineage, &claims), expected);
        }
    }

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

    #[test]
    fn finalization_reservation_is_atomic_and_refundable() {
        let plan = ReportFinalizationPlan {
            invariants: 8,
            ..ReportFinalizationPlan::default()
        };
        let mut insufficient = WorkBudget::new(0, 7);
        assert!(ReportFinalizationPermit::reserve(plan, &mut insufficient).is_err());
        assert_eq!(insufficient.remaining(), (0, 7));
        assert_eq!(insufficient.consumed().get(WorkCounter::Assertion), 0);

        let mut exact = WorkBudget::new(0, 8);
        let permit = ReportFinalizationPermit::reserve(plan, &mut exact);
        assert!(permit.is_ok());
        let Ok(mut permit) = permit else { return };
        assert_eq!(exact.remaining(), (0, 0));
        assert!(permit.refund(&mut exact).is_ok());
        assert_eq!(exact.remaining(), (0, 8));
        assert_eq!(exact.consumed().get(WorkCounter::Assertion), 0);
    }

    #[test]
    fn finalization_dimensions_reject_underflow_and_double_finish() {
        let plan = ReportFinalizationPlan {
            controls: 2,
            invariants: 1,
            ..ReportFinalizationPlan::default()
        };
        let mut budget = WorkBudget::new(0, 3);
        let permit = ReportFinalizationPermit::reserve(plan, &mut budget);
        assert!(permit.is_ok());
        let Ok(mut permit) = permit else { return };
        assert!(permit.consume(FinalizationDimension::Controls, 2).is_ok());
        assert!(permit.consume(FinalizationDimension::Controls, 1).is_err());
        assert!(permit.finish_interrupted().is_err());
        assert!(permit.consume_interrupted_omissions().is_ok());
        assert!(permit.finish_interrupted().is_ok());
        assert!(permit.finish_interrupted().is_err());
        assert!(
            permit
                .consume(FinalizationDimension::Invariants, 1)
                .is_err()
        );
    }

    #[test]
    fn reserved_report_wrappers_consume_without_optional_expansion() {
        let source = include_str!("reference_evaluator.rs");
        for (start, end) in [
            (
                "fn reserved_interrupted_report(",
                "fn reserved_batch_report(",
            ),
            (
                "fn reserved_batch_report(",
                "fn compact_interrupted_report(",
            ),
        ] {
            let wrapper = source
                .split_once(start)
                .and_then(|(_, rest)| rest.split_once(end))
                .map(|(body, _)| body)
                .unwrap_or_default();
            assert!(wrapper.contains(".consume("));
            assert!(!wrapper.contains("view."));
        }
    }

    #[test]
    fn report_validation_precedes_finalization_refund() {
        let source = include_str!("reference_evaluator.rs");
        let complete_path = source
            .split_once("let report = EvaluationReport::from_parts")
            .map(|(_, path)| path)
            .unwrap_or_default();
        let validation = complete_path.find("map_err(|_| EvaluationError::ReportInvariant)?;");
        let refund = complete_path.find(".refund(budget)");
        assert!(matches!((validation, refund), (Some(left), Some(right)) if left < right));
    }
}
