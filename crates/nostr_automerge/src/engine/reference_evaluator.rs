use std::sync::Arc;

use crate::carrier::VerifiedCarrier;
use crate::checkpoint::authorize::{DescriptorControlOutcome, authorize_descriptor_with};
use crate::checkpoint::join::{JoinError, join_chunks};
use crate::checkpoint::reference_state::resolve_referenced_descriptor;
use crate::checkpoint::{
    HistoricalCarrierCoverage, HistoryVerificationError, historical_carrier_coverage,
};
use crate::conformance::dispositions_digest::{disposition_items, dispositions_digest};
use crate::conformance::history_digest::history_digest;
use crate::control::candidate::{
    CandidateResult, evaluate_account_continuity_metered, evaluate_device_ancestry_metered,
    evaluate_parent_continuity, evaluate_role_continuity_metered, evaluate_terminal_continuity,
};
use crate::control::genesis::classify_genesis;
use crate::control::reference_state::{
    ControlParentState, ReferencedControlState, resolve_referenced_control,
};
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
    AttributableCarrierOutcome, CompleteReportFieldAuthority, CompleteReportWitness,
    DispositionRecord, EvaluationError, EvaluationFailure, EvaluationReport, EvaluationReportParts,
    ProtocolItemIdentifier, REPORT_INVARIANT_ITEMS, ReevaluationComparisonStage,
    ReevaluationConstructionError,
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

#[cfg(test)]
std::thread_local! {
    static REEVALUATION_STAGE_OBSERVATIONS: std::cell::Cell<[u64; 5]> = const {
        std::cell::Cell::new([0; 5])
    };
    static FINALIZATION_PASS_OBSERVATIONS: std::cell::RefCell<Vec<FinalizationPassObservation>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

fn observe_reevaluation_stage(stage: ReevaluationComparisonStage) {
    #[cfg(test)]
    REEVALUATION_STAGE_OBSERVATIONS.with(|observations| {
        let mut counts = observations.get();
        counts[stage.index()] = counts[stage.index()].saturating_add(1);
        observations.set(counts);
    });
    #[cfg(not(test))]
    let _ = stage;
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
            return fixed_fallback_report(self.revision, coordinate, Completion::Cancelled);
        }
        let view = DocumentEvidenceView::derive(corpus, coordinate);
        let plan = match ReportFinalizationPlan::from_view(&view) {
            Ok(plan) => plan,
            Err(_) => {
                return fixed_fallback_report(
                    self.revision,
                    coordinate,
                    Completion::BudgetExhausted,
                );
            }
        };
        let mut finalization = match ReportFinalizationPermit::reserve(plan, budget) {
            Ok(permit) => permit,
            Err(_) => {
                return fixed_fallback_report(
                    self.revision,
                    coordinate,
                    Completion::BudgetExhausted,
                );
            }
        };
        macro_rules! complete_pass {
            ($result:expr) => {
                match $result {
                    Ok(value) => value,
                    Err(FinalizationBoundaryError::Stopped(completion)) => {
                        return reserved_interrupted_report(
                            self.revision,
                            coordinate,
                            completion,
                            &mut finalization,
                        );
                    }
                    Err(FinalizationBoundaryError::Permit) => {
                        return Err(settle_reserved_error(
                            &mut finalization,
                            EvaluationError::ReportInvariant,
                        ));
                    }
                }
            };
        }
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
        match (batch.completion, batch.failure) {
            (Completion::Complete, None) => {}
            (Completion::BudgetExhausted, Some(EvaluationFailure::BudgetExhausted)) => {
                return reserved_interrupted_report(
                    self.revision,
                    coordinate,
                    Completion::BudgetExhausted,
                    &mut finalization,
                );
            }
            (Completion::Cancelled, Some(EvaluationFailure::Cancelled)) => {
                return reserved_interrupted_report(
                    self.revision,
                    coordinate,
                    Completion::Cancelled,
                    &mut finalization,
                );
            }
            (Completion::Complete, Some(EvaluationFailure::Graph)) => {
                return Err(settle_reserved_error(
                    &mut finalization,
                    EvaluationError::Graph,
                ));
            }
            (Completion::Complete, Some(EvaluationFailure::Decode)) => {
                return Err(settle_reserved_error(
                    &mut finalization,
                    EvaluationError::Decode,
                ));
            }
            (Completion::Complete, Some(EvaluationFailure::Apply)) => {
                return Err(settle_reserved_error(
                    &mut finalization,
                    EvaluationError::Apply,
                ));
            }
            _ => {
                return Err(settle_reserved_error(
                    &mut finalization,
                    EvaluationError::ReportInvariant,
                ));
            }
        }
        let mut control_disposition_map = preliminary_control_dispositions;
        control_disposition_map.extend(core::mem::take(&mut batch.control_dispositions));
        batch.control_dispositions = control_disposition_map;
        let change_carrier_dispositions =
            match reduce_change_dispositions(&view, &mut batch, budget, cancellation) {
                Ok(dispositions) => dispositions,
                Err(completion) => {
                    return reserved_interrupted_report(
                        self.revision,
                        coordinate,
                        completion,
                        &mut finalization,
                    );
                }
            };
        if let Err(completion) =
            charge_evaluation_work(budget, cancellation, WorkCounter::Carrier, 1)
        {
            return reserved_interrupted_report(
                self.revision,
                coordinate,
                completion,
                &mut finalization,
            );
        }
        let manifest = resolve_selected_manifest(
            &view,
            &batch.control_dispositions,
            &batch.statefully_valid_controls,
        );
        let control_dispositions = complete_pass!(finalization.consume_before(
            [FinalizationReservationUnit::new(
                CompleteReportPass::ControlRecords,
                plan.control_records,
            )],
            budget,
            cancellation,
            || {
                batch
                    .control_dispositions
                    .iter()
                    .map(|(id, disposition)| (*id, *disposition))
                    .collect::<Vec<_>>()
            },
        ));
        if report_record_copy_units(control_dispositions.len()) != Some(plan.control_records) {
            return Err(settle_reserved_error(
                &mut finalization,
                EvaluationError::ReportInvariant,
            ));
        }
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
            CheckpointHistoryInputs {
                accepted_at_control: &batch.accepted_at_control,
                branch_change_dispositions: &batch.branch_change_dispositions,
                change_carrier_dispositions: &change_carrier_dispositions,
            },
            budget,
            cancellation,
        );
        let checkpoint_record_count = checkpoint_evaluation.record_count;
        let checkpoints = checkpoint_evaluation.results;
        if let Some(stop) = checkpoint_evaluation.stop {
            return reserved_interrupted_report(
                self.revision,
                coordinate,
                stop.completion(),
                &mut finalization,
            );
        }
        let Some(checkpoint_record_count) = checkpoint_record_count else {
            return Err(settle_reserved_error(
                &mut finalization,
                EvaluationError::ReportInvariant,
            ));
        };
        let dispositions = complete_pass!(finalization.consume_before(
            [FinalizationReservationUnit::new(
                CompleteReportPass::SemanticChangeRecords,
                plan.semantic_change_records,
            )],
            budget,
            cancellation,
            || {
                batch
                    .dispositions
                    .iter()
                    .map(|(hash, disposition)| (*hash, *disposition))
                    .collect::<Vec<_>>()
            },
        ));
        if report_record_copy_units(dispositions.len()) != Some(plan.semantic_change_records) {
            return Err(settle_reserved_error(
                &mut finalization,
                EvaluationError::ReportInvariant,
            ));
        }
        disposition_records.extend(dispositions.iter().map(|(hash, disposition)| {
            DispositionRecord::new(ProtocolItemIdentifier::from(*hash), *disposition, None)
        }));
        let event_records = match complete_pass!(finalization.consume_before(
            [
                FinalizationReservationUnit::new(
                    CompleteReportPass::ChangeCarrierEvents,
                    plan.change_carrier_events,
                ),
                FinalizationReservationUnit::new(
                    CompleteReportPass::OtherEvents,
                    plan.other_events,
                ),
            ],
            budget,
            cancellation,
            || {
                event_disposition_records(
                    &view,
                    &change_carrier_dispositions,
                    &manifest,
                    &checkpoints,
                )
            },
        )) {
            Ok(records) => records,
            Err(error) => {
                return Err(settle_reserved_error(&mut finalization, error));
            }
        };
        let finalized_change_carriers = event_records.carrier_outcomes.len();
        let finalized_events = event_records.records.len();
        let Some(finalized_other_events) = finalized_events.checked_sub(finalized_change_carriers)
        else {
            return Err(settle_reserved_error(
                &mut finalization,
                EvaluationError::ReportInvariant,
            ));
        };
        let finalized_event_reservations = event_report_reservations(
            control_dispositions.len(),
            finalized_change_carriers,
            finalized_other_events,
            view.evidence_record_count(),
            checkpoint_record_count,
        );
        if !finalized_event_reservations.is_some_and(|(carrier_units, other_units)| {
            carrier_units <= plan.change_carrier_events && other_units <= plan.other_events
        }) {
            return Err(settle_reserved_error(
                &mut finalization,
                EvaluationError::ReportInvariant,
            ));
        }
        disposition_records.extend(event_records.records);
        let finalized_checkpoints = complete_pass!(finalization.consume_before(
            [FinalizationReservationUnit::new(
                CompleteReportPass::CheckpointRecords,
                checkpoint_record_count,
            )],
            budget,
            cancellation,
            || {
                checkpoints.iter().try_fold(0_u64, |total, checkpoint| {
                    total.checked_add(1).and_then(|value| {
                        u64::try_from(checkpoint.chunk_events().len())
                            .ok()
                            .and_then(|chunks| value.checked_add(chunks))
                    })
                })
            },
        ));
        if finalized_checkpoints != Some(checkpoint_record_count) {
            return Err(settle_reserved_error(
                &mut finalization,
                EvaluationError::ReportInvariant,
            ));
        }
        let finalized_change_classifications = u64::try_from(dispositions.len())
            .ok()
            .and_then(|count| count.checked_mul(5));
        if finalized_change_classifications != Some(plan.change_classifications) {
            return Err(settle_reserved_error(
                &mut finalization,
                EvaluationError::ReportInvariant,
            ));
        }
        let (accepted_changes, pending_changes, excluded_changes, invalid_changes) =
            complete_pass!(finalization.consume_before(
                [FinalizationReservationUnit::new(
                    CompleteReportPass::ChangeClassifications,
                    plan.change_classifications,
                )],
                budget,
                cancellation,
                || {
                    (
                        disposition_hashes(&dispositions, ProtocolDisposition::Accepted),
                        disposition_hashes(&dispositions, ProtocolDisposition::Pending),
                        disposition_hashes(&dispositions, ProtocolDisposition::Excluded),
                        disposition_hashes(&dispositions, ProtocolDisposition::Invalid),
                    )
                },
            ));
        let finalized_history_digest = batch
            .canonical_controls
            .len()
            .checked_add(accepted_changes.len())
            .and_then(|value| value.checked_add(batch.heads.len()))
            .and_then(|value| {
                value.checked_add(ReportFinalizationPlan::HISTORY_DIGEST_FIXED_UNITS as usize)
            })
            .and_then(|value| u64::try_from(value).ok());
        let Some(finalized_history_digest) = finalized_history_digest else {
            return Err(settle_reserved_error(
                &mut finalization,
                EvaluationError::ReportInvariant,
            ));
        };
        let finalized_dispositions_digest = u64::try_from(disposition_records.len())
            .ok()
            .and_then(|value| value.checked_mul(2))
            .and_then(|value| {
                value.checked_add(ReportFinalizationPlan::DISPOSITIONS_DIGEST_FIXED_UNITS)
            });
        let Some(finalized_dispositions_digest) = finalized_dispositions_digest else {
            return Err(settle_reserved_error(
                &mut finalization,
                EvaluationError::ReportInvariant,
            ));
        };
        let (heads, history_digest) = complete_pass!(finalization.consume_before(
            [FinalizationReservationUnit::new(
                CompleteReportPass::HistoryDigest,
                finalized_history_digest,
            )],
            budget,
            cancellation,
            || {
                let heads = batch.heads.iter().copied().collect::<Vec<_>>();
                let digest = history_digest(
                    self.revision,
                    coordinate,
                    &batch.canonical_controls,
                    &accepted_changes,
                    &heads,
                );
                (heads, digest)
            },
        ));
        let history_digest = history_digest.map_err(|_| {
            settle_reserved_error(&mut finalization, EvaluationError::ReportInvariant)
        })?;
        let dispositions_digest = complete_pass!(finalization.consume_before(
            [FinalizationReservationUnit::new(
                CompleteReportPass::DispositionsDigest,
                finalized_dispositions_digest,
            )],
            budget,
            cancellation,
            || {
                disposition_items(&disposition_records)
                    .and_then(|items| dispositions_digest(self.revision, coordinate, &items))
            },
        ))
        .map_err(|_| settle_reserved_error(&mut finalization, EvaluationError::ReportInvariant))?;
        let projection = project_document(
            core::mem::take(&mut batch.materialized_document),
            budget,
            cancellation,
        );
        let document = match projection {
            Ok(document) => document,
            Err(crate::automerge_adapter::materialized_view::ProjectionError::Budget) => {
                return reserved_interrupted_report(
                    self.revision,
                    coordinate,
                    Completion::BudgetExhausted,
                    &mut finalization,
                );
            }
            Err(crate::automerge_adapter::materialized_view::ProjectionError::Cancelled) => {
                return reserved_interrupted_report(
                    self.revision,
                    coordinate,
                    Completion::Cancelled,
                    &mut finalization,
                );
            }
            Err(crate::automerge_adapter::materialized_view::ProjectionError::Invalid) => {
                return Err(settle_reserved_error(
                    &mut finalization,
                    EvaluationError::Projection,
                ));
            }
        };
        let evidence = complete_pass!(finalization.consume_before(
            [FinalizationReservationUnit::new(
                CompleteReportPass::EvidenceRecords,
                plan.evidence_records,
            )],
            budget,
            cancellation,
            || view.records().collect::<Vec<_>>(),
        ));
        if u64::try_from(evidence.len()).ok() != Some(plan.evidence_records) {
            return Err(settle_reserved_error(
                &mut finalization,
                EvaluationError::ReportInvariant,
            ));
        }
        let report = complete_pass!(finalization.consume_before(
            [
                FinalizationReservationUnit::new(
                    CompleteReportPass::ReportInvariants,
                    plan.report_invariants,
                ),
                FinalizationReservationUnit::new(
                    CompleteReportPass::FixedOverhead,
                    plan.fixed_overhead,
                ),
            ],
            budget,
            cancellation,
            || {
                let accepted_state = batch
                    .canonical_controls
                    .last()
                    .and_then(|control| batch.accepted_at_control.get(control));
                let field_authority = CompleteReportFieldAuthority::derive(
                    &evidence,
                    &checkpoints,
                    &batch.integrity_alerts,
                    &manifest,
                    document.as_ref(),
                );
                let witness = CompleteReportWitness::new(
                    view.parent_relationships(),
                    &batch.dispositions,
                    &event_records.carrier_outcomes,
                    accepted_state.map(AcceptedAtControl::accepted_closure),
                    accepted_state.map(AcceptedAtControl::frontier_heads),
                    crate::engine::evaluation_report::CompleteReportSourceAuthority::Engine(&view),
                    field_authority,
                );
                EvaluationReport::from_complete_parts(
                    EvaluationReportParts {
                        coordinate,
                        revision: self.revision,
                        canonical_controls: batch.canonical_controls,
                        disposition_records,
                        control_dispositions,
                        dispositions,
                        change_carrier_dispositions: change_carrier_dispositions
                            .values()
                            .map(|outcome| {
                                (outcome.event_id, outcome.change_hash, outcome.disposition)
                            })
                            .collect(),
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
                    },
                    witness,
                )
            },
        ));
        let report = match report {
            Ok(report) => Ok(report),
            Err(_) => Err(EvaluationError::ReportInvariant),
        };
        finalization.finish_complete_report(budget, report)
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
        if previous.revision() != self.revision {
            return Err(EvaluationError::ReportInvariant);
        }
        let current = self.evaluate(corpus, coordinate, budget, cancellation)?;
        if previous.completion() != Completion::Complete
            || current.completion() != Completion::Complete
        {
            return Ok(current);
        }
        if previous.coordinate() != coordinate {
            return Ok(current);
        }
        match EvaluationReport::from_reevaluation(current, previous, |stage| {
            charge_evaluation_work(budget, cancellation, WorkCounter::Assertion, 1)?;
            observe_reevaluation_stage(stage);
            Ok(())
        }) {
            Ok(report) => Ok(report),
            Err(ReevaluationConstructionError::Stopped(completion)) => {
                fixed_fallback_report(self.revision, coordinate, completion)
            }
            Err(ReevaluationConstructionError::Invariant) => Err(EvaluationError::ReportInvariant),
        }
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AggregateChangeContribution {
    AuthorizedCanonical,
    Unresolved,
    AuthorizedExcluded,
    ConclusiveInvalid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ChangeCarrierOutcome {
    event_id: crate::EventId,
    change_hash: ChangeHash,
    control_id: crate::EventId,
    disposition: ProtocolDisposition,
    reason: ChangeClaimReason,
}

impl ChangeCarrierOutcome {
    const fn new(
        event_id: crate::EventId,
        change_hash: ChangeHash,
        control_id: crate::EventId,
        reason: ChangeClaimReason,
    ) -> Self {
        Self {
            event_id,
            change_hash,
            control_id,
            disposition: change_carrier_disposition(reason),
            reason,
        }
    }
}

const fn change_carrier_disposition(reason: ChangeClaimReason) -> ProtocolDisposition {
    match reason {
        ChangeClaimReason::AuthorizedCanonical => ProtocolDisposition::Accepted,
        ChangeClaimReason::UnresolvedControl => ProtocolDisposition::Pending,
        ChangeClaimReason::AuthorizedNoncanonical
        | ChangeClaimReason::AuthorizedCurrentExcluded => ProtocolDisposition::Excluded,
        ChangeClaimReason::InvalidReferencedControl | ChangeClaimReason::Unauthorized => {
            ProtocolDisposition::Invalid
        }
    }
}

const fn aggregate_change_contribution(reason: ChangeClaimReason) -> AggregateChangeContribution {
    match reason {
        ChangeClaimReason::AuthorizedCanonical => AggregateChangeContribution::AuthorizedCanonical,
        ChangeClaimReason::UnresolvedControl => AggregateChangeContribution::Unresolved,
        ChangeClaimReason::AuthorizedNoncanonical
        | ChangeClaimReason::AuthorizedCurrentExcluded => {
            AggregateChangeContribution::AuthorizedExcluded
        }
        ChangeClaimReason::InvalidReferencedControl | ChangeClaimReason::Unauthorized => {
            AggregateChangeContribution::ConclusiveInvalid
        }
    }
}

impl ChangeClaimReason {
    const fn diagnostic(self) -> Option<crate::DiagnosticCode> {
        let code = match self {
            Self::InvalidReferencedControl => "control.parent",
            Self::Unauthorized => "change.actor",
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

fn noncanonical_branch_claim_reason(outcome: Option<ProtocolDisposition>) -> ChangeClaimReason {
    match outcome {
        Some(ProtocolDisposition::Accepted) => ChangeClaimReason::AuthorizedNoncanonical,
        Some(ProtocolDisposition::Pending) => ChangeClaimReason::UnresolvedControl,
        Some(ProtocolDisposition::Excluded) => ChangeClaimReason::AuthorizedCurrentExcluded,
        Some(ProtocolDisposition::Invalid | ProtocolDisposition::UnsupportedRevision) | None => {
            ChangeClaimReason::InvalidReferencedControl
        }
    }
}

fn reduce_change_dispositions(
    view: &DocumentEvidenceView<'_>,
    batch: &mut BatchEvaluationReport,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<std::collections::BTreeMap<crate::EventId, ChangeCarrierOutcome>, Completion> {
    let corpus = view.corpus();
    let final_accepted = batch.accepted_changes.clone();
    let mut change_carrier_dispositions = std::collections::BTreeMap::new();
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
        let mut outcomes = Vec::new();
        let mut aggregate_contributions = Vec::new();
        for event_id in view.change_claim_event_ids(hash) {
            charge_evaluation_work(budget, cancellation, WorkCounter::Carrier, 1)?;
            let Some(claim) = corpus.indexes.changes.claims_by_event.get(&event_id) else {
                continue;
            };
            let Some(semantic) = corpus.indexes.changes.semantic_by_hash.get(&hash) else {
                continue;
            };
            let state = resolve_referenced_control(
                corpus,
                claim.control_id,
                view.coordinate(),
                &batch.control_dispositions,
                &batch.statefully_valid_controls,
            );
            let reason = match state {
                ReferencedControlState::Canonical(control) => {
                    charge_evaluation_work(
                        budget,
                        cancellation,
                        WorkCounter::Control,
                        u64::try_from(control.members().len()).unwrap_or(u64::MAX),
                    )?;
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
                    charge_evaluation_work(
                        budget,
                        cancellation,
                        WorkCounter::Control,
                        u64::try_from(control.members().len()).unwrap_or(u64::MAX),
                    )?;
                    let authorized = !control.terminal()
                        && control.members().iter().any(|member| {
                            member.actor == semantic.actor
                                && member.device == claim.author
                                && member.roles.contains(&Role::Write)
                        });
                    if !authorized {
                        ChangeClaimReason::Unauthorized
                    } else {
                        noncanonical_branch_claim_reason(
                            batch.referenced_branch_change_disposition(claim.control_id, hash),
                        )
                    }
                }
                ReferencedControlState::Pending(_) | ReferencedControlState::Missing => {
                    ChangeClaimReason::UnresolvedControl
                }
                ReferencedControlState::UnsupportedRevision => {
                    ChangeClaimReason::InvalidReferencedControl
                }
                ReferencedControlState::WrongKind
                | ReferencedControlState::WrongCoordinate
                | ReferencedControlState::StaticInvalid
                | ReferencedControlState::DynamicInvalid(_) => {
                    ChangeClaimReason::InvalidReferencedControl
                }
            };
            aggregate_contributions.push(aggregate_change_contribution(reason));
            outcomes.push(ChangeCarrierOutcome::new(
                claim.event_id,
                claim.change_hash,
                claim.control_id,
                reason,
            ));
        }
        let disposition = reduce_aggregate_change_outcome(lineage, &aggregate_contributions);
        batch.dispositions.insert(hash, disposition);
        change_carrier_dispositions.extend(
            outcomes
                .into_iter()
                .map(|outcome| (outcome.event_id, outcome)),
        );
    }
    Ok(change_carrier_dispositions)
}

fn reduce_aggregate_change_outcome(
    lineage: FinalLineageChangeState,
    contributions: &[AggregateChangeContribution],
) -> ProtocolDisposition {
    if lineage == FinalLineageChangeState::Accepted {
        ProtocolDisposition::Accepted
    } else if lineage == FinalLineageChangeState::CanonicalPruned {
        ProtocolDisposition::Excluded
    } else if contributions.contains(&AggregateChangeContribution::Unresolved) {
        ProtocolDisposition::Pending
    } else if contributions.contains(&AggregateChangeContribution::AuthorizedExcluded) {
        ProtocolDisposition::Excluded
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

struct EventDispositionRecords {
    records: Vec<DispositionRecord>,
    carrier_outcomes: std::collections::BTreeMap<crate::EventId, AttributableCarrierOutcome>,
}

fn event_disposition_records(
    view: &DocumentEvidenceView<'_>,
    change_carrier_dispositions: &std::collections::BTreeMap<crate::EventId, ChangeCarrierOutcome>,
    manifest: &ResolvedManifestAvailability,
    checkpoints: &[CheckpointVerificationResult],
) -> Result<EventDispositionRecords, EvaluationError> {
    let corpus = view.corpus();
    let mut verified_change_count = 0_usize;
    let mut carrier_outcomes = std::collections::BTreeMap::new();
    for event_id in view.reportable_event_ids() {
        let Some(evidence) = corpus.events.get(event_id) else {
            return Err(EvaluationError::ReportInvariant);
        };
        let outcome = match evidence {
            EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::Change(change),
                ..
            } => {
                let Some(next_count) = verified_change_count.checked_add(1) else {
                    return Err(EvaluationError::ReportInvariant);
                };
                verified_change_count = next_count;
                let Some(outcome) = change_carrier_dispositions.get(event_id) else {
                    return Err(EvaluationError::ReportInvariant);
                };
                if *event_id != outcome.event_id
                    || change.event_id() != *event_id
                    || change.change_hash() != outcome.change_hash
                    || change.control_id() != outcome.control_id
                {
                    return Err(EvaluationError::ReportInvariant);
                }
                Some(AttributableCarrierOutcome::verified_change(
                    *event_id,
                    outcome.change_hash,
                    outcome.disposition,
                    outcome.reason.diagnostic(),
                ))
            }
            EventEvidence::InvalidCarrier {
                event, diagnostic, ..
            } if event.kind() == 1_624 => {
                if event.event_id() != *event_id {
                    return Err(EvaluationError::ReportInvariant);
                }
                Some(AttributableCarrierOutcome::event_only(
                    *event_id,
                    ProtocolDisposition::Invalid,
                    Some(*diagnostic),
                ))
            }
            EventEvidence::UnsupportedRevision {
                carrier: VerifiedCarrier::UnsupportedRevision { event, .. },
                diagnostic,
                ..
            } if event.kind() == 1_624 => {
                if event.event_id() != *event_id {
                    return Err(EvaluationError::ReportInvariant);
                }
                Some(AttributableCarrierOutcome::event_only(
                    *event_id,
                    ProtocolDisposition::UnsupportedRevision,
                    Some(*diagnostic),
                ))
            }
            EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::UnsupportedRevision { event, .. },
                ..
            } if event.kind() == 1_624 => {
                if event.event_id() != *event_id {
                    return Err(EvaluationError::ReportInvariant);
                }
                Some(AttributableCarrierOutcome::event_only(
                    *event_id,
                    ProtocolDisposition::UnsupportedRevision,
                    Some(crate::DiagnosticCode::registered("carrier.revision")),
                ))
            }
            _ => None,
        };
        if let Some(outcome) = outcome {
            carrier_outcomes.insert(*event_id, outcome);
        }
    }
    if verified_change_count != change_carrier_dispositions.len() {
        return Err(EvaluationError::ReportInvariant);
    }
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

    for outcome in change_carrier_dispositions.values() {
        records.insert(
            outcome.event_id,
            (outcome.disposition, outcome.reason.diagnostic()),
        );
    }

    for record in scoped_dynamic_event_disposition_records(view, manifest, checkpoints)? {
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

    if records
        .keys()
        .any(|event_id| !view.contains_reportable(event_id))
    {
        return Err(EvaluationError::ReportInvariant);
    }
    Ok(EventDispositionRecords {
        records: records
            .into_iter()
            .map(|(event_id, (disposition, diagnostic))| {
                DispositionRecord::new(
                    ProtocolItemIdentifier::event(event_id),
                    disposition,
                    diagnostic,
                )
            })
            .collect(),
        carrier_outcomes,
    })
}

fn scoped_dynamic_event_disposition_records(
    view: &DocumentEvidenceView<'_>,
    manifest: &ResolvedManifestAvailability,
    checkpoints: &[CheckpointVerificationResult],
) -> Result<Vec<DispositionRecord>, EvaluationError> {
    let records = dynamic_event_disposition_records(manifest, checkpoints);
    if records.iter().any(|record| match record.identifier() {
        ProtocolItemIdentifier::Event(event_id) => !view.contains_reportable(&event_id),
        ProtocolItemIdentifier::ControlEvent(_) | ProtocolItemIdentifier::ChangeHash(_) => true,
    }) {
        return Err(EvaluationError::ReportInvariant);
    }
    Ok(records)
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
        let (disposition, diagnostic) = checkpoint.status().event_outcome();
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

struct CheckpointEvaluation {
    results: Vec<CheckpointVerificationResult>,
    record_count: Option<u64>,
    stop: Option<CheckpointWorkStop>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CheckpointDownstreamStage {
    ChunkSetCollection,
    ChunkEventCollection,
    CarrierHistoryCoverage,
    AcceptedAtControlLookup,
    SnapshotLoad,
    HistoryVerification,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CheckpointReportAttributionStage {
    IndexedChunkEvent,
    DescriptorHead,
    AcceptedAtControlHash,
    BranchDispositionHash,
}

trait CheckpointWorkObserver {
    fn authorized_precharge(&mut self) {}

    fn enter_downstream(
        &mut self,
        _descriptor_id: crate::EventId,
        _stage: CheckpointDownstreamStage,
    ) {
    }

    fn enter_report_attribution(
        &mut self,
        _descriptor_id: crate::EventId,
        _stage: CheckpointReportAttributionStage,
    ) {
    }

    fn report_attribution_item(
        &mut self,
        _descriptor_id: crate::EventId,
        _stage: CheckpointReportAttributionStage,
    ) {
    }
}

struct NoopCheckpointWorkObserver;

impl CheckpointWorkObserver for NoopCheckpointWorkObserver {}

struct PreparedCheckpointInputs<'view, 'corpus> {
    view: &'view DocumentEvidenceView<'corpus>,
    canonical_controls: &'view [crate::EventId],
    history: CheckpointHistoryInputs<'view>,
    authorizations: &'view std::collections::BTreeMap<crate::EventId, DescriptorControlOutcome>,
}

#[derive(Clone, Copy)]
struct CheckpointHistoryInputs<'view> {
    accepted_at_control: &'view std::collections::BTreeMap<crate::EventId, AcceptedAtControl>,
    branch_change_dispositions: &'view std::collections::BTreeMap<
        crate::EventId,
        crate::reference::branch_state::PersistentDeltaMap<ChangeHash, ProtocolDisposition>,
    >,
    change_carrier_dispositions:
        &'view std::collections::BTreeMap<crate::EventId, ChangeCarrierOutcome>,
}

struct CheckpointReportAttribution {
    chunk_events: Vec<crate::EventId>,
    heads: Vec<ChangeHash>,
    historical_carriers: Vec<crate::EventId>,
    accepted_at_control: Vec<ChangeHash>,
}

fn verify_checkpoints(
    view: &DocumentEvidenceView<'_>,
    canonical_controls: &[crate::EventId],
    control_dispositions: &std::collections::BTreeMap<crate::EventId, ProtocolDisposition>,
    statefully_valid_controls: &std::collections::BTreeSet<crate::EventId>,
    history: CheckpointHistoryInputs<'_>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> CheckpointEvaluation {
    let authorizations = match checkpoint_authorizations(
        view,
        control_dispositions,
        statefully_valid_controls,
        budget,
        cancellation,
    ) {
        Ok(authorizations) => authorizations,
        Err(stop) => {
            return CheckpointEvaluation {
                results: Vec::new(),
                record_count: Some(0),
                stop: Some(stop),
            };
        }
    };
    let mut observer = NoopCheckpointWorkObserver;
    verify_prepared_checkpoints(
        PreparedCheckpointInputs {
            view,
            canonical_controls,
            history,
            authorizations: &authorizations,
        },
        budget,
        cancellation,
        &mut observer,
    )
}

fn verify_prepared_checkpoints(
    inputs: PreparedCheckpointInputs<'_, '_>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
    observer: &mut impl CheckpointWorkObserver,
) -> CheckpointEvaluation {
    let corpus = inputs.view.corpus();
    let mut results = Vec::new();
    let mut record_count = 0_u64;
    let mut coverage_controls_accounted = false;
    for descriptor_id in inputs
        .view
        .checkpoint_descriptor_event_ids()
        .into_iter()
        .flatten()
        .copied()
    {
        if let Err(stop) = charge_checkpoint_work(budget, cancellation, 1) {
            return CheckpointEvaluation {
                results,
                record_count: Some(record_count),
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
        let commitments = descriptor.descriptor();
        let control_outcome = inputs.authorizations.get(&descriptor_id).copied();
        let downstream = checkpoint_after_authorization(control_outcome, || {
            if !coverage_controls_accounted {
                charge_checkpoint_work(budget, cancellation, 1)?;
                coverage_controls_accounted = true;
                observer.authorized_precharge();
            }
            let chunk_set =
                checkpoint_chunk_set(inputs.view, descriptor, budget, cancellation, observer)?;
            let chunk_events = checkpoint_chunk_event_ids(
                inputs.view,
                descriptor_id,
                budget,
                cancellation,
                observer,
            )?;
            let coverage_result = checkpoint_carrier_coverage(
                inputs.view,
                inputs.canonical_controls,
                descriptor,
                inputs.history.change_carrier_dispositions,
                budget,
                cancellation,
                observer,
            )?;
            let accepted_result = checkpoint_accepted_history(
                descriptor,
                inputs.history.accepted_at_control,
                budget,
                cancellation,
                observer,
            )?;
            let status = match (coverage_result.as_ref(), accepted_result.as_ref()) {
                (Ok(coverage), Ok(accepted)) => verify_one_checkpoint(
                    descriptor,
                    Some(&chunk_set),
                    &coverage.change_hashes,
                    accepted,
                    budget,
                    cancellation,
                    observer,
                ),
                (Err(error), _) | (_, Err(error)) => history_refusal_status(error),
            };
            let coverage = coverage_result.ok().unwrap_or_default();
            let accepted = accepted_result.ok().unwrap_or_default();
            let mut heads = Vec::new();
            for head in &commitments.heads {
                charge_checkpoint_work(budget, cancellation, 1)?;
                heads.push(*head);
            }
            let mut historical_carriers = Vec::new();
            for event_id in coverage.carrier_event_ids {
                charge_checkpoint_work(budget, cancellation, 1)?;
                historical_carriers.push(event_id);
            }
            let mut accepted_at_control = Vec::new();
            for change_hash in accepted {
                charge_checkpoint_work(budget, cancellation, 1)?;
                accepted_at_control.push(change_hash);
            }
            Ok::<_, CheckpointWorkStop>((
                chunk_events,
                heads,
                historical_carriers,
                accepted_at_control,
                status,
            ))
        });
        let (chunk_events, heads, coverage, accepted, status) = match downstream {
            Err(status) => {
                let attribution = checkpoint_refusal_report_attribution(
                    inputs.view,
                    descriptor,
                    control_outcome,
                    inputs.history,
                    budget,
                    cancellation,
                    observer,
                );
                let attribution = match attribution {
                    Ok(attribution) => attribution,
                    Err(stop) => {
                        return CheckpointEvaluation {
                            results,
                            record_count: Some(record_count),
                            stop: Some(stop),
                        };
                    }
                };
                (
                    attribution.chunk_events,
                    attribution.heads,
                    attribution.historical_carriers,
                    attribution.accepted_at_control,
                    status,
                )
            }
            Ok(Ok(prepared)) => prepared,
            Ok(Err(stop)) => {
                return CheckpointEvaluation {
                    results,
                    record_count: Some(record_count),
                    stop: Some(stop),
                };
            }
        };
        let stop = match status {
            CheckpointVerificationStatus::BudgetExhausted => Some(CheckpointWorkStop::Budget),
            CheckpointVerificationStatus::Cancelled => Some(CheckpointWorkStop::Cancelled),
            _ => None,
        };
        if let Some(stop) = stop {
            return CheckpointEvaluation {
                results,
                record_count: Some(record_count),
                stop: Some(stop),
            };
        }
        let Some(next_record_count) = u64::try_from(chunk_events.len())
            .ok()
            .and_then(|chunks| record_count.checked_add(1)?.checked_add(chunks))
        else {
            return CheckpointEvaluation {
                results,
                record_count: None,
                stop: None,
            };
        };
        results.push(CheckpointVerificationResult::from_trusted_ordered(
            descriptor_id,
            chunk_events,
            descriptor.snapshot_hash(),
            heads,
            commitments.change_count,
            commitments.change_set_hash,
            coverage,
            accepted,
            status,
        ));
        record_count = next_record_count;
    }
    CheckpointEvaluation {
        results,
        record_count: Some(record_count),
        stop: None,
    }
}

fn checkpoint_refusal_report_attribution(
    view: &DocumentEvidenceView<'_>,
    descriptor: &crate::carrier::checkpoint_descriptor::ValidatedCheckpointDescriptorCarrier,
    outcome: Option<DescriptorControlOutcome>,
    history: CheckpointHistoryInputs<'_>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
    observer: &mut impl CheckpointWorkObserver,
) -> Result<CheckpointReportAttribution, CheckpointWorkStop> {
    let descriptor_id = descriptor.event_id();
    observer.enter_report_attribution(
        descriptor_id,
        CheckpointReportAttributionStage::IndexedChunkEvent,
    );
    let mut chunk_events = Vec::new();
    for event_id in view
        .checkpoint_chunk_event_ids(descriptor_id)
        .into_iter()
        .flatten()
    {
        charge_checkpoint_work(budget, cancellation, 1)?;
        observer.report_attribution_item(
            descriptor_id,
            CheckpointReportAttributionStage::IndexedChunkEvent,
        );
        chunk_events.push(*event_id);
    }
    observer.enter_report_attribution(
        descriptor_id,
        CheckpointReportAttributionStage::DescriptorHead,
    );
    let mut heads = Vec::new();
    for head in &descriptor.descriptor().heads {
        charge_checkpoint_work(budget, cancellation, 1)?;
        observer.report_attribution_item(
            descriptor_id,
            CheckpointReportAttributionStage::DescriptorHead,
        );
        heads.push(*head);
    }
    let mut accepted = Vec::new();
    if matches!(
        outcome,
        Some(DescriptorControlOutcome::Noncanonical | DescriptorControlOutcome::RoleDenied)
    ) {
        observer.enter_report_attribution(
            descriptor_id,
            CheckpointReportAttributionStage::AcceptedAtControlHash,
        );
        if let Some(state) = history.accepted_at_control.get(&descriptor.control_id()) {
            for hash in state.accepted_closure() {
                charge_checkpoint_work(budget, cancellation, 1)?;
                observer.report_attribution_item(
                    descriptor_id,
                    CheckpointReportAttributionStage::AcceptedAtControlHash,
                );
                accepted.push(*hash);
            }
        }
    }
    let mut coverage = Vec::new();
    if outcome == Some(DescriptorControlOutcome::RoleDenied) {
        observer.enter_report_attribution(
            descriptor_id,
            CheckpointReportAttributionStage::BranchDispositionHash,
        );
        if let Some(dispositions) = history
            .branch_change_dispositions
            .get(&descriptor.control_id())
        {
            for (event_id, carrier) in history.change_carrier_dispositions {
                charge_checkpoint_work(budget, cancellation, 1)?;
                observer.report_attribution_item(
                    descriptor_id,
                    CheckpointReportAttributionStage::BranchDispositionHash,
                );
                if dispositions.contains_key(&carrier.change_hash)
                    && carrier_control_is_historical(
                        view,
                        descriptor.control_id(),
                        carrier.control_id,
                    )
                    && matches!(
                        carrier.reason,
                        ChangeClaimReason::AuthorizedCanonical
                            | ChangeClaimReason::AuthorizedCurrentExcluded
                    )
                {
                    coverage.push(*event_id);
                }
            }
        }
    }
    Ok(CheckpointReportAttribution {
        chunk_events,
        heads,
        historical_carriers: coverage,
        accepted_at_control: accepted,
    })
}

fn carrier_control_is_historical(
    view: &DocumentEvidenceView<'_>,
    through: crate::EventId,
    candidate: crate::EventId,
) -> bool {
    if candidate == through {
        return true;
    }
    let control_sequence = |event_id| match view.corpus().events.get(&event_id) {
        Some(EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::Control(control),
            ..
        }) => Some(control.sequence()),
        _ => None,
    };
    matches!(
        (control_sequence(candidate), control_sequence(through)),
        (Some(candidate_sequence), Some(through_sequence))
            if candidate_sequence < through_sequence
    )
}

fn checkpoint_after_authorization<T>(
    outcome: Option<DescriptorControlOutcome>,
    downstream: impl FnOnce() -> T,
) -> Result<T, CheckpointVerificationStatus> {
    match checkpoint_control_refusal(outcome) {
        Some(status) => Err(status),
        None => Ok(downstream()),
    }
}

const fn checkpoint_control_refusal(
    outcome: Option<DescriptorControlOutcome>,
) -> Option<CheckpointVerificationStatus> {
    match outcome {
        Some(DescriptorControlOutcome::CanonicalAuthorized) => None,
        Some(DescriptorControlOutcome::Missing | DescriptorControlOutcome::Pending) => {
            Some(CheckpointVerificationStatus::PendingControl)
        }
        Some(
            DescriptorControlOutcome::Noncanonical
            | DescriptorControlOutcome::WrongKind
            | DescriptorControlOutcome::WrongCoordinate
            | DescriptorControlOutcome::StaticInvalid
            | DescriptorControlOutcome::DynamicInvalid
            | DescriptorControlOutcome::UnsupportedRevision
            | DescriptorControlOutcome::RoleDenied,
        )
        | None => Some(CheckpointVerificationStatus::Unauthorized),
    }
}

#[cfg(test)]
const fn checkpoint_preflight_refusal(
    control_refusal: Option<CheckpointVerificationStatus>,
    history_refusal: Option<HistoryVerificationError>,
) -> Option<CheckpointVerificationStatus> {
    match control_refusal {
        Some(status) => Some(status),
        None => match history_refusal {
            Some(error) => Some(history_refusal_status(&error)),
            None => None,
        },
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
    observer: &mut impl CheckpointWorkObserver,
) -> Result<Vec<crate::EventId>, CheckpointWorkStop> {
    observer.enter_downstream(
        descriptor_id,
        CheckpointDownstreamStage::ChunkEventCollection,
    );
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
    chunks: Option<&Result<Vec<crate::checkpoint::CheckpointChunk>, JoinError>>,
    coverage: &std::collections::BTreeSet<ChangeHash>,
    accepted: &std::collections::BTreeSet<ChangeHash>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
    observer: &mut impl CheckpointWorkObserver,
) -> CheckpointVerificationStatus {
    use crate::checkpoint::{HistoryVerificationError, VerifyError};
    if cancellation.is_cancelled() {
        return CheckpointVerificationStatus::Cancelled;
    }
    if budget.charge_checkpoint_items(1).is_err() {
        return CheckpointVerificationStatus::BudgetExhausted;
    }
    let chunks = match chunks {
        Some(Ok(chunks)) => chunks,
        Some(Err(error)) => return join_status(*error),
        None => return CheckpointVerificationStatus::MissingChunk,
    };
    let bytes = match crate::checkpoint::assemble_ordered_chunks(
        descriptor.descriptor(),
        chunks,
        budget,
        cancellation,
    ) {
        Ok(bytes) => bytes,
        Err(error) => return assembly_status(error),
    };
    observer.enter_downstream(
        descriptor.event_id(),
        CheckpointDownstreamStage::SnapshotLoad,
    );
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
    if let Err(error) =
        snapshot.verify_commitments_metered(descriptor.descriptor(), budget, cancellation)
    {
        return match error {
            VerifyError::Budget => CheckpointVerificationStatus::BudgetExhausted,
            VerifyError::Cancelled => CheckpointVerificationStatus::Cancelled,
            VerifyError::Load => CheckpointVerificationStatus::SnapshotLoad,
            VerifyError::Commitments => CheckpointVerificationStatus::CommitmentMismatch,
            VerifyError::Heads => CheckpointVerificationStatus::HeadMismatch,
            VerifyError::Closure => CheckpointVerificationStatus::ClosureMismatch,
        };
    }
    if let Err(error) = snapshot.verify_exact_closure_metered(budget, cancellation) {
        return match error {
            VerifyError::Budget => CheckpointVerificationStatus::BudgetExhausted,
            VerifyError::Cancelled => CheckpointVerificationStatus::Cancelled,
            _ => CheckpointVerificationStatus::ClosureMismatch,
        };
    }
    observer.enter_downstream(
        descriptor.event_id(),
        CheckpointDownstreamStage::HistoryVerification,
    );
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
        JoinError::Budget => CheckpointVerificationStatus::BudgetExhausted,
        JoinError::Cancelled => CheckpointVerificationStatus::Cancelled,
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
    descriptor: &crate::carrier::checkpoint_descriptor::ValidatedCheckpointDescriptorCarrier,
    accepted_at_control: &std::collections::BTreeMap<crate::EventId, AcceptedAtControl>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
    observer: &mut impl CheckpointWorkObserver,
) -> Result<
    Result<std::collections::BTreeSet<ChangeHash>, HistoryVerificationError>,
    CheckpointWorkStop,
> {
    observer.enter_downstream(
        descriptor.event_id(),
        CheckpointDownstreamStage::AcceptedAtControlLookup,
    );
    charge_checkpoint_work(budget, cancellation, 1)?;
    let Some(state) = accepted_at_control.get(&descriptor.control_id()) else {
        return Ok(Err(HistoryVerificationError::UnknownControl));
    };
    let mut accepted = std::collections::BTreeSet::new();
    for change_hash in state.accepted_closure() {
        charge_checkpoint_work(budget, cancellation, 1)?;
        accepted.insert(*change_hash);
    }
    Ok(Ok(accepted))
}

fn checkpoint_carrier_coverage(
    view: &DocumentEvidenceView<'_>,
    canonical_controls: &[crate::EventId],
    descriptor: &crate::carrier::checkpoint_descriptor::ValidatedCheckpointDescriptorCarrier,
    change_carrier_dispositions: &std::collections::BTreeMap<crate::EventId, ChangeCarrierOutcome>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
    observer: &mut impl CheckpointWorkObserver,
) -> Result<Result<HistoricalCarrierCoverage, HistoryVerificationError>, CheckpointWorkStop> {
    observer.enter_downstream(
        descriptor.event_id(),
        CheckpointDownstreamStage::CarrierHistoryCoverage,
    );
    charge_checkpoint_work(budget, cancellation, 1)?;
    match historical_carrier_coverage(
        view,
        canonical_controls,
        descriptor.control_id(),
        budget,
        cancellation,
        |event_id, change_hash, control_id| {
            change_carrier_dispositions
                .get(&event_id)
                .is_some_and(|carrier| {
                    carrier.change_hash == change_hash
                        && carrier.control_id == control_id
                        && matches!(
                            carrier.reason,
                            ChangeClaimReason::AuthorizedCanonical
                                | ChangeClaimReason::AuthorizedCurrentExcluded
                        )
                })
        },
    ) {
        Err(HistoryVerificationError::Budget) => Err(CheckpointWorkStop::Budget),
        Err(HistoryVerificationError::Cancelled) => Err(CheckpointWorkStop::Cancelled),
        result => Ok(result),
    }
}

fn checkpoint_chunk_set(
    view: &DocumentEvidenceView<'_>,
    descriptor: &crate::carrier::checkpoint_descriptor::ValidatedCheckpointDescriptorCarrier,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
    observer: &mut impl CheckpointWorkObserver,
) -> Result<Result<Vec<crate::checkpoint::CheckpointChunk>, JoinError>, CheckpointWorkStop> {
    observer.enter_downstream(
        descriptor.event_id(),
        CheckpointDownstreamStage::ChunkSetCollection,
    );
    let corpus = view.corpus();
    charge_checkpoint_work(budget, cancellation, 1)?;
    let mut chunks = Vec::new();
    for chunk_id in view
        .checkpoint_chunk_event_ids(descriptor.event_id())
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
    match join_chunks(descriptor, chunks, budget, cancellation) {
        Err(JoinError::Budget) => Err(CheckpointWorkStop::Budget),
        Err(JoinError::Cancelled) => Err(CheckpointWorkStop::Cancelled),
        result => Ok(result),
    }
}

fn checkpoint_authorizations(
    view: &DocumentEvidenceView<'_>,
    control_dispositions: &std::collections::BTreeMap<crate::EventId, ProtocolDisposition>,
    statefully_valid_controls: &std::collections::BTreeSet<crate::EventId>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<std::collections::BTreeMap<crate::EventId, DescriptorControlOutcome>, CheckpointWorkStop>
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
            let outcome = authorize_descriptor_with(descriptor, state, &mut || {
                charge_checkpoint_work(budget, cancellation, 1)
            })?;
            authorizations.insert(*descriptor_id, outcome);
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
    control_records: u64,
    semantic_change_records: u64,
    change_carrier_events: u64,
    other_events: u64,
    checkpoint_records: u64,
    change_classifications: u64,
    history_digest: u64,
    dispositions_digest: u64,
    evidence_records: u64,
    report_invariants: u64,
    fixed_overhead: u64,
}

impl ReportFinalizationPlan {
    const HISTORY_DIGEST_FIXED_UNITS: u64 = 4;
    const DISPOSITIONS_DIGEST_FIXED_UNITS: u64 = 4;
    const FIXED_OVERHEAD_UNITS: u64 = 8;

    fn from_view(view: &DocumentEvidenceView<'_>) -> Result<Self, ()> {
        let controls = u64::try_from(view.control_count()).map_err(|_| ())?;
        let hashes = u64::try_from(view.change_hash_count()).map_err(|_| ())?;
        let change_carrier_events =
            u64::try_from(view.change_carrier_event_count()).map_err(|_| ())?;
        let other_events = u64::try_from(view.other_event_count()).map_err(|_| ())?;
        let reportable_events = change_carrier_events.checked_add(other_events).ok_or(())?;
        let non_control_events = u64::try_from(view.reportable_event_count())
            .map_err(|_| ())?
            .checked_sub(controls)
            .ok_or(())?;
        if reportable_events != non_control_events {
            return Err(());
        }
        let checkpoint_records = u64::try_from(view.checkpoint_descriptor_count())
            .map_err(|_| ())?
            .checked_add(u64::try_from(view.checkpoint_chunk_count()).map_err(|_| ())?)
            .ok_or(())?;
        let control_records = controls.checked_mul(2).ok_or(())?;
        let semantic_change_records = hashes.checked_mul(2).ok_or(())?;
        let (change_carrier_events, other_events) = event_report_reservations(
            view.control_count(),
            view.change_carrier_event_count(),
            view.other_event_count(),
            view.evidence_record_count(),
            checkpoint_records,
        )
        .ok_or(())?;
        let change_classifications = hashes.checked_mul(5).ok_or(())?;
        let history_digest = controls
            .checked_add(hashes.checked_mul(2).ok_or(())?)
            .and_then(|value| value.checked_add(Self::HISTORY_DIGEST_FIXED_UNITS))
            .ok_or(())?;
        let dispositions_digest = controls
            .checked_add(hashes)
            .and_then(|value| value.checked_add(reportable_events))
            .and_then(|value| value.checked_mul(2))
            .and_then(|value| value.checked_add(Self::DISPOSITIONS_DIGEST_FIXED_UNITS))
            .ok_or(())?;
        let plan = Self {
            control_records,
            semantic_change_records,
            change_carrier_events,
            other_events,
            checkpoint_records,
            change_classifications,
            history_digest,
            dispositions_digest,
            evidence_records: u64::try_from(view.evidence_record_count()).map_err(|_| ())?,
            report_invariants: report_invariant_reservation(view).ok_or(())?,
            fixed_overhead: Self::FIXED_OVERHEAD_UNITS,
        };
        plan.total().ok_or(())?;
        Ok(plan)
    }

    fn total(self) -> Option<u64> {
        self.reservations()
            .into_iter()
            .try_fold(0_u64, |total, reservation| {
                total.checked_add(reservation.units)
            })
    }

    const fn reservations(self) -> [FinalizationReservationUnit; 11] {
        [
            FinalizationReservationUnit::new(
                CompleteReportPass::ControlRecords,
                self.control_records,
            ),
            FinalizationReservationUnit::new(
                CompleteReportPass::SemanticChangeRecords,
                self.semantic_change_records,
            ),
            FinalizationReservationUnit::new(
                CompleteReportPass::ChangeCarrierEvents,
                self.change_carrier_events,
            ),
            FinalizationReservationUnit::new(CompleteReportPass::OtherEvents, self.other_events),
            FinalizationReservationUnit::new(
                CompleteReportPass::CheckpointRecords,
                self.checkpoint_records,
            ),
            FinalizationReservationUnit::new(
                CompleteReportPass::ChangeClassifications,
                self.change_classifications,
            ),
            FinalizationReservationUnit::new(
                CompleteReportPass::HistoryDigest,
                self.history_digest,
            ),
            FinalizationReservationUnit::new(
                CompleteReportPass::DispositionsDigest,
                self.dispositions_digest,
            ),
            FinalizationReservationUnit::new(
                CompleteReportPass::EvidenceRecords,
                self.evidence_records,
            ),
            FinalizationReservationUnit::new(
                CompleteReportPass::ReportInvariants,
                self.report_invariants,
            ),
            FinalizationReservationUnit::new(
                CompleteReportPass::FixedOverhead,
                self.fixed_overhead,
            ),
        ]
    }
}

fn report_record_copy_units(count: usize) -> Option<u64> {
    u64::try_from(count).ok()?.checked_mul(2)
}

fn event_report_reservations(
    controls: usize,
    change_carrier_events: usize,
    other_events: usize,
    evidence_records: usize,
    checkpoint_records: u64,
) -> Option<(u64, u64)> {
    // These coefficients conservatively cover every ordered report projection
    // and its closed-shape/authority validation pass. The reservation is made
    // before any of those target-sized passes begin; checked overflow falls
    // back to the constant no-progress report.
    const EVENT_PASSES: u64 = 10;
    const CHECKPOINT_PASSES: u64 = 4;

    let controls = u64::try_from(controls).ok()?;
    let change_carrier_events = u64::try_from(change_carrier_events).ok()?;
    let other_events = u64::try_from(other_events).ok()?;
    let evidence_records = u64::try_from(evidence_records).ok()?;
    let carrier_units = change_carrier_events.checked_mul(EVENT_PASSES)?;
    let other_units = other_events
        .checked_add(controls)?
        .checked_mul(EVENT_PASSES)?
        .checked_add(evidence_records)?
        .checked_add(checkpoint_records.checked_mul(CHECKPOINT_PASSES)?)?;
    Some((carrier_units, other_units))
}

fn report_invariant_reservation(view: &DocumentEvidenceView<'_>) -> Option<u64> {
    // Complete-report validation deliberately repeats canonical-order,
    // cross-namespace, and authority checks. This checked upper bound includes
    // nested checkpoint vectors and the worst-case alert-source relation table
    // so no invariant traversal begins without an owning reservation.
    const CONTROL_UNITS: u64 = 16;
    const HASH_UNITS: u64 = 24;
    const EVENT_UNITS: u64 = 12;
    const CARRIER_UNITS: u64 = 4;
    const EVIDENCE_UNITS: u64 = 4;
    const RELATIONSHIP_UNITS: u64 = 4;
    const CHECKPOINT_UNITS: u64 = 8;
    const NESTED_CHECKPOINT_UNITS: u64 = 2;
    const ALERT_PAIR_UNITS: u64 = 2;

    let controls = u64::try_from(view.control_count()).ok()?;
    let hashes = u64::try_from(view.change_hash_count()).ok()?;
    let events = u64::try_from(view.reportable_event_count()).ok()?;
    let carriers = u64::try_from(view.change_carrier_event_count()).ok()?;
    let evidence = u64::try_from(view.evidence_record_count()).ok()?;
    let relationships = u64::try_from(view.control_relationship_count()).ok()?;
    let descriptors = u64::try_from(view.checkpoint_descriptor_count()).ok()?;
    let chunks = u64::try_from(view.checkpoint_chunk_count()).ok()?;
    let checkpoint_records = descriptors.checked_add(chunks)?;
    let nested_checkpoint_items = descriptors
        .checked_mul(hashes.checked_mul(3)?.checked_add(events)?)?
        .checked_add(chunks)?;
    let alert_sources = controls
        .checked_add(hashes)?
        .checked_add(carriers)?
        .checked_add(descriptors)?;
    let alert_pairs = alert_sources.checked_mul(alert_sources)?;

    [
        REPORT_INVARIANT_ITEMS,
        controls.checked_mul(CONTROL_UNITS)?,
        hashes.checked_mul(HASH_UNITS)?,
        events.checked_mul(EVENT_UNITS)?,
        carriers.checked_mul(CARRIER_UNITS)?,
        evidence.checked_mul(EVIDENCE_UNITS)?,
        relationships.checked_mul(RELATIONSHIP_UNITS)?,
        checkpoint_records.checked_mul(CHECKPOINT_UNITS)?,
        nested_checkpoint_items.checked_mul(NESTED_CHECKPOINT_UNITS)?,
        alert_pairs.checked_mul(ALERT_PAIR_UNITS)?,
    ]
    .into_iter()
    .try_fold(0_u64, u64::checked_add)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CompleteReportPass {
    ControlRecords,
    SemanticChangeRecords,
    ChangeCarrierEvents,
    OtherEvents,
    CheckpointRecords,
    ChangeClassifications,
    HistoryDigest,
    DispositionsDigest,
    EvidenceRecords,
    ReportInvariants,
    FixedOverhead,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FinalizationPassObservation {
    Consumed(CompleteReportPass),
    WorkStarted(CompleteReportPass),
}

#[cfg(test)]
fn observe_finalization_pass(observation: FinalizationPassObservation) {
    FINALIZATION_PASS_OBSERVATIONS.with(|observations| {
        observations.borrow_mut().push(observation);
    });
}

impl CompleteReportPass {
    const ALL: [Self; 11] = [
        Self::ControlRecords,
        Self::SemanticChangeRecords,
        Self::ChangeCarrierEvents,
        Self::OtherEvents,
        Self::CheckpointRecords,
        Self::ChangeClassifications,
        Self::HistoryDigest,
        Self::DispositionsDigest,
        Self::EvidenceRecords,
        Self::ReportInvariants,
        Self::FixedOverhead,
    ];

    const fn dimension(self) -> FinalizationDimension {
        match self {
            Self::ControlRecords => FinalizationDimension::ControlRecords,
            Self::SemanticChangeRecords => FinalizationDimension::SemanticChangeRecords,
            Self::ChangeCarrierEvents => FinalizationDimension::ChangeCarrierEvents,
            Self::OtherEvents => FinalizationDimension::OtherEvents,
            Self::CheckpointRecords => FinalizationDimension::CheckpointRecords,
            Self::ChangeClassifications => FinalizationDimension::ChangeClassifications,
            Self::HistoryDigest => FinalizationDimension::HistoryDigest,
            Self::DispositionsDigest => FinalizationDimension::DispositionsDigest,
            Self::EvidenceRecords => FinalizationDimension::EvidenceRecords,
            Self::ReportInvariants => FinalizationDimension::ReportInvariants,
            Self::FixedOverhead => FinalizationDimension::FixedOverhead,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FinalizationReservationUnit {
    pass: CompleteReportPass,
    units: u64,
}

impl FinalizationReservationUnit {
    const fn new(pass: CompleteReportPass, units: u64) -> Self {
        Self { pass, units }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FinalizationDimension {
    ControlRecords,
    SemanticChangeRecords,
    ChangeCarrierEvents,
    OtherEvents,
    CheckpointRecords,
    ChangeClassifications,
    HistoryDigest,
    DispositionsDigest,
    EvidenceRecords,
    ReportInvariants,
    FixedOverhead,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FinalizationPermitError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FinalizationBoundaryError {
    Permit,
    Stopped(Completion),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct FinalizationSettlement {
    reserved: u64,
    consumed: u64,
    refunded: u64,
    forfeited: u64,
    remainder_closed: bool,
}

impl FinalizationSettlement {
    const fn new(reserved: u64) -> Self {
        Self {
            reserved,
            consumed: 0,
            refunded: 0,
            forfeited: 0,
            remainder_closed: false,
        }
    }

    fn classified(self) -> Option<u64> {
        self.consumed
            .checked_add(self.refunded)?
            .checked_add(self.forfeited)
    }

    fn remaining(self) -> Option<u64> {
        self.reserved.checked_sub(self.classified()?)
    }

    fn consume(&mut self, amount: u64) -> Result<(), FinalizationPermitError> {
        if self.remainder_closed || amount > self.remaining().ok_or(FinalizationPermitError)? {
            return Err(FinalizationPermitError);
        }
        self.consumed = self
            .consumed
            .checked_add(amount)
            .ok_or(FinalizationPermitError)?;
        Ok(())
    }

    fn refund_remaining(&mut self) -> Result<u64, FinalizationPermitError> {
        if self.remainder_closed {
            return Err(FinalizationPermitError);
        }
        let amount = self.remaining().ok_or(FinalizationPermitError)?;
        self.refunded = self
            .refunded
            .checked_add(amount)
            .ok_or(FinalizationPermitError)?;
        self.remainder_closed = true;
        Ok(amount)
    }

    fn forfeit_remaining(&mut self) -> Result<(), FinalizationPermitError> {
        if self.remainder_closed {
            return Err(FinalizationPermitError);
        }
        let amount = self.remaining().ok_or(FinalizationPermitError)?;
        self.forfeited = self
            .forfeited
            .checked_add(amount)
            .ok_or(FinalizationPermitError)?;
        self.remainder_closed = true;
        Ok(())
    }

    fn is_settled(self) -> bool {
        self.remainder_closed && self.classified() == Some(self.reserved)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ReportFinalizationLedger {
    control_records: FinalizationSettlement,
    semantic_change_records: FinalizationSettlement,
    change_carrier_events: FinalizationSettlement,
    other_events: FinalizationSettlement,
    checkpoint_records: FinalizationSettlement,
    change_classifications: FinalizationSettlement,
    history_digest: FinalizationSettlement,
    dispositions_digest: FinalizationSettlement,
    evidence_records: FinalizationSettlement,
    report_invariants: FinalizationSettlement,
    fixed_overhead: FinalizationSettlement,
}

impl ReportFinalizationLedger {
    const fn from_plan(plan: ReportFinalizationPlan) -> Self {
        Self {
            control_records: FinalizationSettlement::new(plan.control_records),
            semantic_change_records: FinalizationSettlement::new(plan.semantic_change_records),
            change_carrier_events: FinalizationSettlement::new(plan.change_carrier_events),
            other_events: FinalizationSettlement::new(plan.other_events),
            checkpoint_records: FinalizationSettlement::new(plan.checkpoint_records),
            change_classifications: FinalizationSettlement::new(plan.change_classifications),
            history_digest: FinalizationSettlement::new(plan.history_digest),
            dispositions_digest: FinalizationSettlement::new(plan.dispositions_digest),
            evidence_records: FinalizationSettlement::new(plan.evidence_records),
            report_invariants: FinalizationSettlement::new(plan.report_invariants),
            fixed_overhead: FinalizationSettlement::new(plan.fixed_overhead),
        }
    }

    fn dimension_mut(&mut self, dimension: FinalizationDimension) -> &mut FinalizationSettlement {
        match dimension {
            FinalizationDimension::ControlRecords => &mut self.control_records,
            FinalizationDimension::SemanticChangeRecords => &mut self.semantic_change_records,
            FinalizationDimension::ChangeCarrierEvents => &mut self.change_carrier_events,
            FinalizationDimension::OtherEvents => &mut self.other_events,
            FinalizationDimension::CheckpointRecords => &mut self.checkpoint_records,
            FinalizationDimension::ChangeClassifications => &mut self.change_classifications,
            FinalizationDimension::HistoryDigest => &mut self.history_digest,
            FinalizationDimension::DispositionsDigest => &mut self.dispositions_digest,
            FinalizationDimension::EvidenceRecords => &mut self.evidence_records,
            FinalizationDimension::ReportInvariants => &mut self.report_invariants,
            FinalizationDimension::FixedOverhead => &mut self.fixed_overhead,
        }
    }

    fn settlements(&self) -> [&FinalizationSettlement; 11] {
        [
            &self.control_records,
            &self.semantic_change_records,
            &self.change_carrier_events,
            &self.other_events,
            &self.checkpoint_records,
            &self.change_classifications,
            &self.history_digest,
            &self.dispositions_digest,
            &self.evidence_records,
            &self.report_invariants,
            &self.fixed_overhead,
        ]
    }

    fn is_settled(&self) -> bool {
        self.settlements()
            .into_iter()
            .all(|settlement| settlement.is_settled())
    }

    fn is_interrupted_settlement(&self) -> bool {
        self.is_settled()
            && self
                .settlements()
                .into_iter()
                .all(|settlement| settlement.refunded == 0)
    }

    fn remaining_total(&self) -> Option<u64> {
        self.settlements()
            .into_iter()
            .try_fold(0_u64, |total, settlement| {
                total.checked_add(settlement.remaining()?)
            })
    }

    fn refund_all_remaining(&mut self) -> Result<(), FinalizationPermitError> {
        for pass in CompleteReportPass::ALL {
            self.dimension_mut(pass.dimension()).refund_remaining()?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FixedFallbackPass {
    Digests,
    FixedOverhead,
    Invariants,
}

impl FixedFallbackPass {
    const ALL: [Self; 3] = [Self::Digests, Self::FixedOverhead, Self::Invariants];
}

#[derive(Debug, PartialEq, Eq)]
struct FixedFallbackLedger {
    digests: FinalizationSettlement,
    fixed_overhead: FinalizationSettlement,
    invariants: FinalizationSettlement,
    next_pass: usize,
}

impl FixedFallbackLedger {
    const DIGEST_UNITS: u64 = 8;
    const FIXED_OVERHEAD_UNITS: u64 = 8;

    const fn new() -> Self {
        Self {
            digests: FinalizationSettlement::new(Self::DIGEST_UNITS),
            fixed_overhead: FinalizationSettlement::new(Self::FIXED_OVERHEAD_UNITS),
            invariants: FinalizationSettlement::new(REPORT_INVARIANT_ITEMS),
            next_pass: 0,
        }
    }

    fn settlement_mut(&mut self, pass: FixedFallbackPass) -> &mut FinalizationSettlement {
        match pass {
            FixedFallbackPass::Digests => &mut self.digests,
            FixedFallbackPass::FixedOverhead => &mut self.fixed_overhead,
            FixedFallbackPass::Invariants => &mut self.invariants,
        }
    }

    fn consume(
        &mut self,
        pass: FixedFallbackPass,
        amount: u64,
    ) -> Result<(), FinalizationPermitError> {
        if FixedFallbackPass::ALL.get(self.next_pass) != Some(&pass)
            || self
                .settlement_mut(pass)
                .remaining()
                .ok_or(FinalizationPermitError)?
                != amount
        {
            return Err(FinalizationPermitError);
        }
        self.settlement_mut(pass).consume(amount)?;
        self.next_pass = self
            .next_pass
            .checked_add(1)
            .ok_or(FinalizationPermitError)?;
        Ok(())
    }

    fn close_consumed(&mut self) -> Result<(), FinalizationPermitError> {
        if self.next_pass != FixedFallbackPass::ALL.len() {
            return Err(FinalizationPermitError);
        }
        for pass in FixedFallbackPass::ALL {
            if self
                .settlement_mut(pass)
                .remaining()
                .ok_or(FinalizationPermitError)?
                != 0
            {
                return Err(FinalizationPermitError);
            }
            self.settlement_mut(pass).forfeit_remaining()?;
        }
        Ok(())
    }

    fn forfeit_all(&mut self) -> Result<(), FinalizationPermitError> {
        for pass in FixedFallbackPass::ALL {
            self.settlement_mut(pass).forfeit_remaining()?;
        }
        Ok(())
    }

    fn is_consumed_settlement(&self) -> bool {
        self.next_pass == FixedFallbackPass::ALL.len()
            && [&self.digests, &self.fixed_overhead, &self.invariants]
                .into_iter()
                .all(|settlement| {
                    settlement.is_settled() && settlement.refunded == 0 && settlement.forfeited == 0
                })
    }

    fn is_forfeited_settlement(&self) -> bool {
        [&self.digests, &self.fixed_overhead, &self.invariants]
            .into_iter()
            .all(|settlement| {
                settlement.is_settled() && settlement.consumed == 0 && settlement.refunded == 0
            })
    }

    fn build_report(
        &mut self,
        revision: ProtocolRevision,
        coordinate: DocumentCoordinate,
        completion: Completion,
    ) -> Result<EvaluationReport, EvaluationError> {
        self.consume(FixedFallbackPass::Digests, Self::DIGEST_UNITS)
            .map_err(|_| EvaluationError::ReportInvariant)?;
        let history_digest = history_digest(revision, coordinate, &[], &[], &[])
            .map_err(|_| EvaluationError::ReportInvariant)?;
        let disposition_items =
            disposition_items(&[]).map_err(|_| EvaluationError::ReportInvariant)?;
        let dispositions_digest = dispositions_digest(revision, coordinate, &disposition_items)
            .map_err(|_| EvaluationError::ReportInvariant)?;
        let failure = match completion {
            Completion::BudgetExhausted => EvaluationFailure::BudgetExhausted,
            Completion::Cancelled => EvaluationFailure::Cancelled,
            Completion::Complete => return Err(EvaluationError::ReportInvariant),
        };
        self.consume(FixedFallbackPass::FixedOverhead, Self::FIXED_OVERHEAD_UNITS)
            .and_then(|()| self.consume(FixedFallbackPass::Invariants, REPORT_INVARIANT_ITEMS))
            .map_err(|_| EvaluationError::ReportInvariant)?;
        let report = build_no_progress_interrupted_report(
            revision,
            coordinate,
            completion,
            failure,
            history_digest,
            dispositions_digest,
        )?;
        self.close_consumed()
            .map_err(|_| EvaluationError::ReportInvariant)?;
        debug_assert!(self.is_consumed_settlement());
        Ok(report)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FinalizationPermitState {
    Active,
    Complete,
    Interrupted,
    Failed,
}

#[derive(Debug)]
struct ReportFinalizationPermit {
    ledger: ReportFinalizationLedger,
    fallback: FixedFallbackLedger,
    next_complete_pass: usize,
    state: FinalizationPermitState,
}

impl ReportFinalizationPermit {
    fn reserve(
        plan: ReportFinalizationPlan,
        budget: &mut WorkBudget,
    ) -> Result<Self, crate::BudgetExhausted> {
        budget.charge(WorkCounter::Assertion, plan.total().unwrap_or(u64::MAX))?;
        Ok(Self {
            ledger: ReportFinalizationLedger::from_plan(plan),
            fallback: FixedFallbackLedger::new(),
            next_complete_pass: 0,
            state: FinalizationPermitState::Active,
        })
    }

    fn consume_pass(
        &mut self,
        reservation: FinalizationReservationUnit,
    ) -> Result<(), FinalizationPermitError> {
        if self.state != FinalizationPermitState::Active
            || CompleteReportPass::ALL.get(self.next_complete_pass) != Some(&reservation.pass)
        {
            return Err(FinalizationPermitError);
        }
        self.ledger
            .dimension_mut(reservation.pass.dimension())
            .consume(reservation.units)?;
        self.next_complete_pass = self
            .next_complete_pass
            .checked_add(1)
            .ok_or(FinalizationPermitError)?;
        Ok(())
    }

    fn consume_before<T, const N: usize>(
        &mut self,
        reservations: [FinalizationReservationUnit; N],
        budget: &mut WorkBudget,
        cancellation: &impl CancellationCheck,
        work: impl FnOnce() -> T,
    ) -> Result<T, FinalizationBoundaryError> {
        for reservation in &reservations {
            charge_evaluation_work(budget, cancellation, WorkCounter::Assertion, 0)
                .map_err(FinalizationBoundaryError::Stopped)?;
            self.consume_pass(*reservation)
                .map_err(|_| FinalizationBoundaryError::Permit)?;
            #[cfg(test)]
            observe_finalization_pass(FinalizationPassObservation::Consumed(reservation.pass));
        }
        #[cfg(test)]
        for reservation in &reservations {
            observe_finalization_pass(FinalizationPassObservation::WorkStarted(reservation.pass));
        }
        Ok(work())
    }

    fn finish_interrupted(&mut self) -> Result<(), FinalizationPermitError> {
        if self.state != FinalizationPermitState::Active
            || !self.ledger.is_interrupted_settlement()
            || !self.fallback.is_consumed_settlement()
        {
            return Err(FinalizationPermitError);
        }
        self.state = FinalizationPermitState::Interrupted;
        Ok(())
    }

    fn build_interrupted_report(
        &mut self,
        revision: ProtocolRevision,
        coordinate: DocumentCoordinate,
        completion: Completion,
    ) -> Result<EvaluationReport, EvaluationError> {
        self.forfeit_all_remaining()
            .map_err(|_| EvaluationError::ReportInvariant)?;
        let report = self
            .fallback
            .build_report(revision, coordinate, completion)?;
        self.finish_interrupted()
            .map_err(|_| EvaluationError::ReportInvariant)?;
        Ok(report)
    }

    fn forfeit(&mut self, dimension: FinalizationDimension) -> Result<(), FinalizationPermitError> {
        if self.state != FinalizationPermitState::Active {
            return Err(FinalizationPermitError);
        }
        self.ledger.dimension_mut(dimension).forfeit_remaining()
    }

    fn finish_failed(&mut self) -> Result<(), FinalizationPermitError> {
        if self.state != FinalizationPermitState::Active {
            return Err(FinalizationPermitError);
        }
        self.forfeit_all_remaining()?;
        self.fallback.forfeit_all()?;
        if !self.ledger.is_settled() || !self.fallback.is_forfeited_settlement() {
            return Err(FinalizationPermitError);
        }
        self.state = FinalizationPermitState::Failed;
        Ok(())
    }

    fn finish_complete_report(
        &mut self,
        budget: &mut WorkBudget,
        report: Result<EvaluationReport, EvaluationError>,
    ) -> Result<EvaluationReport, EvaluationError> {
        let report = match report {
            Ok(report) if report.completion() == Completion::Complete => report,
            Ok(_) => {
                return Err(settle_reserved_error(
                    self,
                    EvaluationError::ReportInvariant,
                ));
            }
            Err(error) => return Err(settle_reserved_error(self, error)),
        };
        self.refund(budget)
            .map_err(|_| settle_reserved_error(self, EvaluationError::ReportInvariant))?;
        Ok(report)
    }

    fn refund(&mut self, budget: &mut WorkBudget) -> Result<(), FinalizationPermitError> {
        if self.state != FinalizationPermitState::Active
            || self.next_complete_pass != CompleteReportPass::ALL.len()
        {
            return Err(FinalizationPermitError);
        }
        let remaining = self
            .ledger
            .remaining_total()
            .ok_or(FinalizationPermitError)?;
        budget
            .refund(WorkCounter::Assertion, remaining)
            .map_err(|_| FinalizationPermitError)?;
        self.ledger.refund_all_remaining()?;
        self.fallback.forfeit_all()?;
        if !self.ledger.is_settled() || !self.fallback.is_forfeited_settlement() {
            return Err(FinalizationPermitError);
        }
        self.state = FinalizationPermitState::Complete;
        Ok(())
    }

    fn forfeit_all_remaining(&mut self) -> Result<(), FinalizationPermitError> {
        if self.state != FinalizationPermitState::Active {
            return Err(FinalizationPermitError);
        }
        for pass in CompleteReportPass::ALL {
            self.forfeit(pass.dimension())?;
        }
        Ok(())
    }
}

fn settle_reserved_error(
    permit: &mut ReportFinalizationPermit,
    error: EvaluationError,
) -> EvaluationError {
    if permit.finish_failed().is_ok() {
        error
    } else {
        EvaluationError::ReportInvariant
    }
}

fn reserved_interrupted_report(
    revision: ProtocolRevision,
    coordinate: DocumentCoordinate,
    completion: Completion,
    permit: &mut ReportFinalizationPermit,
) -> Result<EvaluationReport, EvaluationError> {
    permit.build_interrupted_report(revision, coordinate, completion)
}

fn fixed_fallback_report(
    revision: ProtocolRevision,
    coordinate: DocumentCoordinate,
    completion: Completion,
) -> Result<EvaluationReport, EvaluationError> {
    FixedFallbackLedger::new().build_report(revision, coordinate, completion)
}

fn build_no_progress_interrupted_report(
    revision: ProtocolRevision,
    coordinate: DocumentCoordinate,
    completion: Completion,
    failure: EvaluationFailure,
    history_digest: crate::HistoryDigest,
    dispositions_digest: crate::DispositionsDigest,
) -> Result<EvaluationReport, EvaluationError> {
    let parts = EvaluationReportParts {
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
        manifest: ResolvedManifestAvailability::Missing,
        completion,
        failure: Some(failure),
        document: None,
    };
    EvaluationReport::from_no_progress_parts(parts).map_err(|_| EvaluationError::ReportInvariant)
}

fn prepare_controls(
    view: &DocumentEvidenceView<'_>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<PreparedControls, Completion> {
    let corpus = view.corpus();
    let coordinate = view.coordinate();
    let ancestry_index = build_control_ancestry_index(view, budget, cancellation)?;
    let mut assumed_statefully_valid = std::collections::BTreeSet::new();
    let mut assumed_control_dispositions = std::collections::BTreeMap::new();
    for event_id in view.input_event_ids() {
        charge_evaluation_work(budget, cancellation, WorkCounter::Control, 1)?;
        if matches!(
            corpus.events.get(&event_id),
            Some(EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::Control(_),
                ..
            })
        ) {
            assumed_statefully_valid.insert(event_id);
            assumed_control_dispositions.insert(event_id, ProtocolDisposition::Accepted);
        }
    }
    let mut dispositions = std::collections::BTreeMap::new();
    let mut controls = Vec::new();
    for control_id in view.control_event_ids() {
        charge_evaluation_work(budget, cancellation, WorkCounter::Control, 1)?;
        let Some(EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::Control(control),
            ..
        }) = corpus.events.get(&control_id)
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
            || !device_ancestry_is_valid(&ancestry_index, control, budget, cancellation)?
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
    let parents = view
        .parent_relationships()
        .into_iter()
        .flat_map(std::collections::BTreeMap::iter)
        .flat_map(|(parent, children)| children.iter().map(move |child| (*child, *parent)))
        .collect::<std::collections::BTreeMap<_, _>>();
    propagate_control_parent_dispositions(&parents, &mut dispositions, budget, cancellation)?;
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
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<bool, Completion> {
    let Some(ancestry) = ancestry_index
        .get(&child.event_id())
        .and_then(Option::as_ref)
    else {
        return Ok(false);
    };
    let child = ControlEnvelope::from_validated(child.clone());
    let mut visit = || charge_evaluation_work(budget, cancellation, WorkCounter::Control, 1);
    Ok(evaluate_device_ancestry_metered(ancestry, &child, &mut visit)? == CandidateResult::Valid)
}

fn build_control_ancestry_index(
    view: &DocumentEvidenceView<'_>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<std::collections::BTreeMap<crate::EventId, Option<Vec<ControlEnvelope>>>, Completion> {
    let corpus = view.corpus();
    let mut parents = std::collections::BTreeMap::new();
    for (parent, children) in view
        .parent_relationships()
        .into_iter()
        .flat_map(std::collections::BTreeMap::iter)
    {
        for child in children {
            charge_evaluation_work(budget, cancellation, WorkCounter::Control, 1)?;
            parents.insert(*child, *parent);
        }
    }
    debug_assert_eq!(parents.len(), view.control_relationship_count());
    let mut index =
        std::collections::BTreeMap::<crate::EventId, Option<Vec<ControlEnvelope>>>::new();
    for root in view.control_event_ids() {
        charge_evaluation_work(budget, cancellation, WorkCounter::Control, 1)?;
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
                        base = Vec::with_capacity(ancestry.len().saturating_add(1));
                        for control in ancestry {
                            charge_evaluation_work(budget, cancellation, WorkCounter::Control, 1)?;
                            base.push(control.clone());
                        }
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
                carrier: VerifiedCarrier::Control(_),
                ..
            }) = corpus.events.get(&event_id)
            else {
                valid = false;
                break;
            };
            path.push(event_id);
            current = parents.get(&event_id).copied().flatten();
        }
        if !valid {
            for event_id in path {
                charge_evaluation_work(budget, cancellation, WorkCounter::Control, 1)?;
                index.insert(event_id, None);
            }
            continue;
        }
        for event_id in path.into_iter().rev() {
            let mut ancestry = Vec::with_capacity(base.len());
            for control in &base {
                charge_evaluation_work(budget, cancellation, WorkCounter::Control, 1)?;
                ancestry.push(control.clone());
            }
            index.insert(event_id, Some(ancestry));
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
    let parent = ControlEnvelope::from_validated(parent.as_ref().clone());
    let child = ControlEnvelope::from_validated(child.clone());
    let mut visit = || charge_evaluation_work(budget, cancellation, WorkCounter::Control, 1);
    Ok(evaluate_role_continuity_metered(&parent, &child, &mut visit)? == CandidateResult::Valid)
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
    let parent = ControlEnvelope::from_validated(parent.as_ref().clone());
    let child = ControlEnvelope::from_validated(child.clone());
    let mut visit = || charge_evaluation_work(budget, cancellation, WorkCounter::Control, 1);
    Ok(evaluate_account_continuity_metered(&parent, &child, &mut visit)? == CandidateResult::Valid)
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
    let raw = view.raw_change_arc(hash);
    charge_evaluation_work(
        budget,
        cancellation,
        WorkCounter::DecodeByte,
        raw.map_or(u64::MAX, |bytes| {
            u64::try_from(bytes.len()).unwrap_or(u64::MAX)
        }),
    )?;
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
        raw_change: raw.map(Arc::clone),
    }))
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
        AggregateChangeContribution, BatchEvaluationReport, ChangeCarrierOutcome,
        ChangeClaimReason, CheckpointDownstreamStage, CheckpointHistoryInputs,
        CheckpointReportAttributionStage, CheckpointWorkObserver, CheckpointWorkStop,
        CompleteReportPass, EvaluationError, FINALIZATION_PASS_OBSERVATIONS,
        FinalLineageChangeState, FinalizationBoundaryError, FinalizationDimension,
        FinalizationPassObservation, FinalizationPermitError, FinalizationPermitState,
        FinalizationReservationUnit, FixedFallbackLedger, FixedFallbackPass,
        PreparedCheckpointInputs, REEVALUATION_STAGE_OBSERVATIONS, REPORT_INVARIANT_ITEMS,
        ReferenceEvaluator, ReportFinalizationPermit, ReportFinalizationPlan,
        aggregate_change_contribution, assembly_status, carrier_control_is_historical,
        change_carrier_disposition, charge_checkpoint_work, checkpoint_control_refusal,
        checkpoint_preflight_refusal, join_status, noncanonical_branch_claim_reason,
        reduce_aggregate_change_outcome, reduce_change_dispositions,
        scoped_dynamic_event_disposition_records, verify_prepared_checkpoints,
    };
    use crate::CheckpointVerificationStatus as Status;
    use crate::authoring::{ActorState, AuthoringDocument};
    use crate::carrier::VerifiedCarrier;
    use crate::carrier::checkpoint_chunk::ValidatedCheckpointChunkCarrier;
    use crate::carrier::checkpoint_descriptor::ValidatedCheckpointDescriptorCarrier;
    use crate::carrier::control::{DeviceGrant, ValidatedControlCarrier, ValidatedControlContent};
    use crate::checkpoint::authorize::DescriptorControlOutcome;
    use crate::checkpoint::join::JoinError;
    use crate::checkpoint::{
        AssemblyError, CheckpointChunk, CheckpointDescriptor, HistoryVerificationError, leaf_hash,
    };
    use crate::evidence::corpus_builder::EvidenceCorpus;
    use crate::evidence::document_view::DocumentEvidenceView;
    use crate::evidence::event::{EventEvidence, RawChecksum};
    use crate::evidence::indexes::{
        ChangeCarrierClaim, CoordinateWorkMetadata, SemanticChangeRecord, TrustedIndexes,
        derive_trusted_indexes,
    };
    use crate::reference::epoch_engine::AcceptedAtControl;
    use crate::types::role::Role;
    use crate::{
        ActorId, ChangeHash, CheckpointVerificationResult, ChunkHash, Completion,
        ControllerPublicKey, DevicePublicKey, DocumentCoordinate, DocumentId, EventId,
        ProtocolDisposition, ResolvedManifestAvailability, SnapshotHash, WorkBudget, WorkCounter,
    };
    use sha2::{Digest, Sha256};

    fn consume_fixed_fallback(ledger: &mut FixedFallbackLedger) {
        assert!(
            ledger
                .consume(
                    FixedFallbackPass::Digests,
                    FixedFallbackLedger::DIGEST_UNITS,
                )
                .is_ok()
        );
        assert!(
            ledger
                .consume(
                    FixedFallbackPass::FixedOverhead,
                    FixedFallbackLedger::FIXED_OVERHEAD_UNITS,
                )
                .is_ok()
        );
        assert!(
            ledger
                .consume(FixedFallbackPass::Invariants, REPORT_INVARIANT_ITEMS)
                .is_ok()
        );
        assert!(ledger.close_consumed().is_ok());
        assert!(ledger.is_consumed_settlement());
    }

    #[test]
    fn finding_082_reevaluation_stops_before_post_incomplete_alert_work() {
        let coordinate = DocumentCoordinate::new(
            ControllerPublicKey::from_bytes([0xc1; 32]),
            DocumentId::from_bytes([0xc2; 32]),
        );
        let corpus = crate::CorpusBuilder::new().finish();
        let evaluator = ReferenceEvaluator::new(crate::ProtocolRevision::draft_v1());
        let previous = evaluator.evaluate(
            &corpus,
            coordinate,
            &mut WorkBudget::new(1_000_000, 1_000_000),
            &crate::NeverCancelled,
        );
        assert!(previous.is_ok());
        let Ok(previous) = previous else { return };
        assert_eq!(previous.completion(), crate::Completion::Complete);
        REEVALUATION_STAGE_OBSERVATIONS.with(|observations| observations.set([0; 5]));

        let current = evaluator.reevaluate(
            &corpus,
            coordinate,
            &previous,
            &mut WorkBudget::new(0, 0),
            &crate::NeverCancelled,
        );
        assert!(current.is_ok());
        let Ok(current) = current else { return };
        assert_eq!(current.completion(), crate::Completion::BudgetExhausted);
        let observations = REEVALUATION_STAGE_OBSERVATIONS.with(std::cell::Cell::get);
        assert_eq!(
            observations, [0; 5],
            "FINDING_082 reproduced: reevaluation performs summary work after incomplete finalization"
        );

        let previous_incomplete = evaluator.evaluate(
            &corpus,
            coordinate,
            &mut WorkBudget::new(0, 0),
            &crate::NeverCancelled,
        );
        assert!(previous_incomplete.is_ok());
        let Ok(previous_incomplete) = previous_incomplete else {
            return;
        };
        assert_eq!(
            previous_incomplete.completion(),
            crate::Completion::BudgetExhausted
        );
        REEVALUATION_STAGE_OBSERVATIONS.with(|observations| observations.set([0; 5]));
        let current = evaluator.reevaluate(
            &corpus,
            coordinate,
            &previous_incomplete,
            &mut WorkBudget::new(1_000_000, 1_000_000),
            &crate::NeverCancelled,
        );
        assert!(current.is_ok());
        let Ok(current) = current else { return };
        assert_eq!(current.completion(), crate::Completion::Complete);
        let observations = REEVALUATION_STAGE_OBSERVATIONS.with(std::cell::Cell::get);
        assert_eq!(
            observations, [0; 5],
            "an incomplete previous report must bypass all reevaluation stages"
        );
    }

    #[test]
    fn complete_reevaluation_has_exact_final_budget_and_cancellation_boundaries() {
        let coordinate = DocumentCoordinate::new(
            ControllerPublicKey::from_bytes([0xd1; 32]),
            DocumentId::from_bytes([0xd2; 32]),
        );
        let corpus = crate::CorpusBuilder::new().finish();
        let evaluator = ReferenceEvaluator::new(crate::ProtocolRevision::draft_v1());
        let previous = evaluator.evaluate(
            &corpus,
            coordinate,
            &mut WorkBudget::new(1_000_000, 1_000_000),
            &crate::NeverCancelled,
        );
        assert!(previous.is_ok());
        let Ok(previous) = previous else { return };

        let calls = std::cell::Cell::new(0_u64);
        let counting = || {
            calls.set(calls.get().saturating_add(1));
            false
        };
        let mut measured = WorkBudget::new(1_000_000, 1_000_000);
        REEVALUATION_STAGE_OBSERVATIONS.with(|observations| observations.set([0; 5]));
        let complete =
            evaluator.reevaluate(&corpus, coordinate, &previous, &mut measured, &counting);
        assert!(complete.is_ok());
        let Ok(complete) = complete else { return };
        assert_eq!(complete.completion(), crate::Completion::Complete);
        let full_observations = REEVALUATION_STAGE_OBSERVATIONS.with(std::cell::Cell::get);
        assert!(full_observations.into_iter().all(|count| count > 0));
        let exact_items = 1_000_000_u64.saturating_sub(measured.remaining().1);
        let exact_calls = calls.get();
        assert!(exact_items > 0 && exact_calls > 0);

        let mut short = WorkBudget::new(1_000_000, exact_items.saturating_sub(1));
        REEVALUATION_STAGE_OBSERVATIONS.with(|observations| observations.set([0; 5]));
        let stopped = evaluator.reevaluate(
            &corpus,
            coordinate,
            &previous,
            &mut short,
            &crate::NeverCancelled,
        );
        assert!(stopped.is_ok());
        let Ok(stopped) = stopped else { return };
        assert_eq!(stopped.completion(), crate::Completion::BudgetExhausted);
        assert!(stopped.integrity_alerts().is_empty());
        let short_observations = REEVALUATION_STAGE_OBSERVATIONS.with(std::cell::Cell::get);
        assert_eq!(
            short_observations[4],
            full_observations[4].saturating_sub(1),
            "N-1 must stop before final construction work"
        );

        let mut exact = WorkBudget::new(1_000_000, exact_items);
        REEVALUATION_STAGE_OBSERVATIONS.with(|observations| observations.set([0; 5]));
        let at_boundary = evaluator.reevaluate(
            &corpus,
            coordinate,
            &previous,
            &mut exact,
            &crate::NeverCancelled,
        );
        assert_eq!(at_boundary, Ok(complete.clone()));
        assert_eq!(exact.remaining().1, 0);
        assert_eq!(
            REEVALUATION_STAGE_OBSERVATIONS.with(std::cell::Cell::get),
            full_observations
        );

        let cancellation_calls = std::cell::Cell::new(0_u64);
        let cancel_at_final_boundary = || {
            let call = cancellation_calls.get().saturating_add(1);
            cancellation_calls.set(call);
            call == exact_calls
        };
        REEVALUATION_STAGE_OBSERVATIONS.with(|observations| observations.set([0; 5]));
        let cancelled = evaluator.reevaluate(
            &corpus,
            coordinate,
            &previous,
            &mut WorkBudget::new(1_000_000, 1_000_000),
            &cancel_at_final_boundary,
        );
        assert!(cancelled.is_ok());
        let Ok(cancelled) = cancelled else { return };
        assert_eq!(cancelled.completion(), crate::Completion::Cancelled);
        assert!(cancelled.integrity_alerts().is_empty());
        assert_eq!(
            cancellation_calls.get(),
            exact_calls,
            "a typed cancellation stop must not be re-queried"
        );
        let cancelled_observations = REEVALUATION_STAGE_OBSERVATIONS.with(std::cell::Cell::get);
        assert_eq!(cancelled_observations, short_observations);
    }

    #[test]
    fn carrier_claim_traversal_preserves_the_original_typed_stop() {
        let coordinate = DocumentCoordinate::new(
            ControllerPublicKey::from_bytes([0x11; 32]),
            DocumentId::from_bytes([0x12; 32]),
        );
        let hash = ChangeHash::from_bytes([0x13; 32]);
        let event_id = EventId::from_bytes([0x14; 32]);
        let control_id = EventId::from_bytes([0x15; 32]);
        let actor = ActorId::from_bytes([0x16; 32]);
        let mut indexes = TrustedIndexes::default();
        indexes
            .coordinates
            .change_hashes
            .insert(coordinate, std::collections::BTreeSet::from([hash]));
        indexes
            .changes
            .prior_claims_by_coordinate
            .entry(coordinate)
            .or_default()
            .insert(hash, std::collections::BTreeSet::from([event_id]));
        indexes.changes.claims_by_event.insert(
            event_id,
            ChangeCarrierClaim {
                event_id,
                coordinate,
                change_hash: hash,
                control_id,
                author: DevicePublicKey::from_bytes([0x17; 32]),
            },
        );
        indexes.changes.semantic_by_hash.insert(
            hash,
            SemanticChangeRecord {
                actor,
                sequence: 1,
                start_op: 1,
                operation_count: 1,
                dependencies: std::collections::BTreeSet::new(),
            },
        );
        let author = DevicePublicKey::from_bytes([0x17; 32]);
        let control = ValidatedControlCarrier::for_test(
            control_id,
            coordinate.controller(),
            coordinate,
            None,
            ValidatedControlContent {
                base_heads: Vec::new(),
                members: vec![DeviceGrant {
                    account: None,
                    actor,
                    device: author,
                    roles: vec![Role::Write],
                }],
                predecessor: None,
                sequence: 0,
                successor: None,
                terminal: false,
            },
        );
        let corpus = EvidenceCorpus {
            events: std::collections::BTreeMap::from([(
                control_id,
                EventEvidence::VerifiedCarrier {
                    carrier: VerifiedCarrier::Control(Box::new(control)),
                    raw_checksum: RawChecksum::test_only([0x18; 32]),
                },
            )]),
            invalid: std::collections::BTreeMap::new(),
            duplicates: Vec::new(),
            indexes,
        };
        let view = DocumentEvidenceView::derive(&corpus, coordinate);
        let batch = || BatchEvaluationReport {
            canonical_controls: Vec::new(),
            control_dispositions: std::collections::BTreeMap::new(),
            accepted_at_control: std::collections::BTreeMap::new(),
            statefully_valid_controls: std::collections::BTreeSet::new(),
            branch_states: std::collections::BTreeMap::new(),
            branch_change_dispositions: std::collections::BTreeMap::new(),
            dispositions: std::collections::BTreeMap::new(),
            accepted_changes: std::collections::BTreeSet::new(),
            heads: std::collections::BTreeSet::new(),
            materialized_document: None,
            integrity_alerts: Vec::new(),
            completion: crate::Completion::Complete,
            failure: None,
        };

        let budget_observations = std::cell::Cell::new(0_u64);
        let budget_cancellation = || {
            budget_observations.set(budget_observations.get().saturating_add(1));
            budget_observations.get() > 2
        };
        let mut exhausted = WorkBudget::new(0, 1);
        let mut budget_batch = batch();
        assert_eq!(
            reduce_change_dispositions(
                &view,
                &mut budget_batch,
                &mut exhausted,
                &budget_cancellation,
            ),
            Err(crate::Completion::BudgetExhausted)
        );
        assert_eq!(budget_observations.get(), 2);
        assert_eq!(exhausted.consumed().get(WorkCounter::GraphNode), 1);
        assert_eq!(exhausted.consumed().get(WorkCounter::Carrier), 0);

        let cancellation_observations = std::cell::Cell::new(0_u64);
        let cancellation = || {
            cancellation_observations.set(cancellation_observations.get().saturating_add(1));
            cancellation_observations.get() >= 2
        };
        let mut available = WorkBudget::new(0, 2);
        let mut cancelled_batch = batch();
        assert_eq!(
            reduce_change_dispositions(&view, &mut cancelled_batch, &mut available, &cancellation,),
            Err(crate::Completion::Cancelled)
        );
        assert_eq!(cancellation_observations.get(), 2);
        assert_eq!(available.consumed().get(WorkCounter::GraphNode), 1);
        assert_eq!(available.consumed().get(WorkCounter::Carrier), 0);

        let member_budget_observations = std::cell::Cell::new(0_u64);
        let member_budget_cancellation = || {
            member_budget_observations.set(member_budget_observations.get().saturating_add(1));
            member_budget_observations.get() > 3
        };
        let mut member_exhausted = WorkBudget::new(0, 2);
        let mut member_budget_batch = batch();
        member_budget_batch
            .control_dispositions
            .insert(control_id, ProtocolDisposition::Accepted);
        assert_eq!(
            reduce_change_dispositions(
                &view,
                &mut member_budget_batch,
                &mut member_exhausted,
                &member_budget_cancellation,
            ),
            Err(crate::Completion::BudgetExhausted)
        );
        assert_eq!(member_budget_observations.get(), 3);
        assert_eq!(member_exhausted.consumed().get(WorkCounter::GraphNode), 1);
        assert_eq!(member_exhausted.consumed().get(WorkCounter::Carrier), 1);
        assert_eq!(member_exhausted.consumed().get(WorkCounter::Control), 0);

        let member_cancellation_observations = std::cell::Cell::new(0_u64);
        let member_cancellation = || {
            member_cancellation_observations
                .set(member_cancellation_observations.get().saturating_add(1));
            member_cancellation_observations.get() >= 3
        };
        let mut member_available = WorkBudget::new(0, 3);
        let mut member_cancelled_batch = batch();
        member_cancelled_batch
            .control_dispositions
            .insert(control_id, ProtocolDisposition::Accepted);
        assert_eq!(
            reduce_change_dispositions(
                &view,
                &mut member_cancelled_batch,
                &mut member_available,
                &member_cancellation,
            ),
            Err(crate::Completion::Cancelled)
        );
        assert_eq!(member_cancellation_observations.get(), 3);
        assert_eq!(member_available.consumed().get(WorkCounter::GraphNode), 1);
        assert_eq!(member_available.consumed().get(WorkCounter::Carrier), 1);
        assert_eq!(member_available.consumed().get(WorkCounter::Control), 0);
    }

    #[test]
    fn dynamic_event_records_reject_foreign_membership() {
        let coordinate = DocumentCoordinate::new(
            ControllerPublicKey::from_bytes([1; 32]),
            DocumentId::from_bytes([2; 32]),
        );
        let corpus = EvidenceCorpus {
            events: std::collections::BTreeMap::new(),
            invalid: std::collections::BTreeMap::new(),
            duplicates: Vec::new(),
            indexes: TrustedIndexes::default(),
        };
        let view = DocumentEvidenceView::derive(&corpus, coordinate);
        let foreign = CheckpointVerificationResult::new(
            EventId::from_bytes([3; 32]),
            vec![EventId::from_bytes([4; 32])],
            SnapshotHash::from_bytes([5; 32]),
            Vec::new(),
            0,
            [0; 32],
            Vec::new(),
            Vec::new(),
            Status::Verified,
        );
        assert!(
            scoped_dynamic_event_disposition_records(
                &view,
                &ResolvedManifestAvailability::Missing,
                &[foreign]
            )
            .is_err()
        );
    }

    #[test]
    fn aggregate_change_outcome_uses_final_precedence() {
        use crate::ProtocolDisposition::{Accepted, Excluded, Invalid, Pending};
        use AggregateChangeContribution::{AuthorizedExcluded, ConclusiveInvalid, Unresolved};
        assert_eq!(
            reduce_aggregate_change_outcome(FinalLineageChangeState::Accepted, &[Unresolved]),
            Accepted
        );
        assert_eq!(
            reduce_aggregate_change_outcome(
                FinalLineageChangeState::CanonicalPruned,
                &[Unresolved]
            ),
            Excluded
        );
        assert_eq!(
            reduce_aggregate_change_outcome(
                FinalLineageChangeState::Current,
                &[AuthorizedExcluded, Unresolved]
            ),
            Pending
        );
        assert_eq!(
            reduce_aggregate_change_outcome(
                FinalLineageChangeState::Current,
                &[ConclusiveInvalid, Unresolved]
            ),
            Pending
        );
        assert_eq!(
            reduce_aggregate_change_outcome(
                FinalLineageChangeState::Current,
                &[AuthorizedExcluded, ConclusiveInvalid]
            ),
            Excluded
        );
        assert_eq!(
            reduce_aggregate_change_outcome(FinalLineageChangeState::Current, &[ConclusiveInvalid]),
            Invalid
        );
    }

    #[test]
    fn change_carrier_outcome_reason_mapping_is_exhaustive() {
        use crate::ProtocolDisposition::{Accepted, Excluded, Invalid, Pending};
        use ChangeClaimReason::{
            AuthorizedCanonical, AuthorizedCurrentExcluded, AuthorizedNoncanonical,
            InvalidReferencedControl, Unauthorized, UnresolvedControl,
        };

        use AggregateChangeContribution::{
            AuthorizedCanonical as AggregateAuthorizedCanonical, AuthorizedExcluded,
            ConclusiveInvalid, Unresolved,
        };

        for (reason, carrier_disposition, aggregate_contribution) in [
            (AuthorizedCanonical, Accepted, AggregateAuthorizedCanonical),
            (AuthorizedNoncanonical, Excluded, AuthorizedExcluded),
            (AuthorizedCurrentExcluded, Excluded, AuthorizedExcluded),
            (UnresolvedControl, Pending, Unresolved),
            (InvalidReferencedControl, Invalid, ConclusiveInvalid),
            (Unauthorized, Invalid, ConclusiveInvalid),
        ] {
            assert_eq!(change_carrier_disposition(reason), carrier_disposition);
            assert_eq!(
                aggregate_change_contribution(reason),
                aggregate_contribution
            );
            let outcome = ChangeCarrierOutcome::new(
                crate::EventId::from_bytes([1; 32]),
                crate::ChangeHash::from_bytes([2; 32]),
                crate::EventId::from_bytes([3; 32]),
                reason,
            );
            assert_eq!(outcome.disposition, carrier_disposition);
            assert_eq!(outcome.reason, reason);
        }
    }

    #[test]
    fn carrier_resolution_distinguishes_branch_and_reference_states() {
        use crate::ProtocolDisposition::{Accepted, Excluded, Invalid, Pending};

        for (branch, expected_reason, expected_disposition) in [
            (
                Some(Accepted),
                ChangeClaimReason::AuthorizedNoncanonical,
                Excluded,
            ),
            (Some(Pending), ChangeClaimReason::UnresolvedControl, Pending),
            (
                Some(Excluded),
                ChangeClaimReason::AuthorizedCurrentExcluded,
                Excluded,
            ),
            (
                Some(Invalid),
                ChangeClaimReason::InvalidReferencedControl,
                Invalid,
            ),
            (None, ChangeClaimReason::InvalidReferencedControl, Invalid),
        ] {
            let reason = noncanonical_branch_claim_reason(branch);
            assert_eq!(reason, expected_reason);
            assert_eq!(change_carrier_disposition(reason), expected_disposition);
        }
        assert_eq!(
            change_carrier_disposition(ChangeClaimReason::Unauthorized),
            Invalid
        );
        assert_eq!(
            change_carrier_disposition(ChangeClaimReason::InvalidReferencedControl),
            Invalid,
            "an unsupported referenced control invalidates a draft-v1 carrier"
        );
    }

    #[test]
    fn aggregate_reduction_cannot_rewrite_an_invalid_carrier() {
        use crate::ProtocolDisposition::{Accepted, Excluded, Invalid, Pending};
        use AggregateChangeContribution::{AuthorizedExcluded, ConclusiveInvalid, Unresolved};
        let valid = noncanonical_branch_claim_reason(Some(Accepted));
        let invalid = noncanonical_branch_claim_reason(Some(Invalid));
        let invalid_carrier = ChangeCarrierOutcome::new(
            crate::EventId::from_bytes([4; 32]),
            crate::ChangeHash::from_bytes([2; 32]),
            crate::EventId::from_bytes([5; 32]),
            ChangeClaimReason::Unauthorized,
        );
        assert_eq!(valid, ChangeClaimReason::AuthorizedNoncanonical);
        assert_eq!(invalid, ChangeClaimReason::InvalidReferencedControl);
        assert_eq!(invalid_carrier.disposition, Invalid);
        for (lineage, contributions, aggregate) in [
            (
                FinalLineageChangeState::Accepted,
                vec![ConclusiveInvalid],
                Accepted,
            ),
            (
                FinalLineageChangeState::CanonicalPruned,
                vec![ConclusiveInvalid],
                Excluded,
            ),
            (
                FinalLineageChangeState::Current,
                vec![Unresolved, ConclusiveInvalid],
                Pending,
            ),
            (
                FinalLineageChangeState::Current,
                vec![AuthorizedExcluded, ConclusiveInvalid],
                Excluded,
            ),
            (
                FinalLineageChangeState::Current,
                vec![ConclusiveInvalid],
                Invalid,
            ),
        ] {
            assert_eq!(
                reduce_aggregate_change_outcome(lineage, &contributions),
                aggregate
            );
            assert_eq!(invalid_carrier.disposition, Invalid);
            assert_eq!(invalid_carrier.reason, ChangeClaimReason::Unauthorized);
        }
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
        assert!(
            ChangeClaimReason::AuthorizedCanonical
                .diagnostic()
                .is_none()
        );
        assert!(ChangeClaimReason::UnresolvedControl.diagnostic().is_none());
    }

    #[test]
    fn carrier_and_aggregate_decision_table_is_exhaustive() {
        use crate::ProtocolDisposition::{Accepted, Excluded, Invalid, Pending};
        use AggregateChangeContribution::{AuthorizedExcluded, Unresolved};
        use ChangeClaimReason::{
            AuthorizedCanonical, AuthorizedCurrentExcluded, AuthorizedNoncanonical,
            InvalidReferencedControl, Unauthorized, UnresolvedControl,
        };

        let reasons = [
            AuthorizedCanonical,
            UnresolvedControl,
            AuthorizedNoncanonical,
            AuthorizedCurrentExcluded,
            InvalidReferencedControl,
            Unauthorized,
        ];
        let expected_carrier = |reason| match reason {
            AuthorizedCanonical => Accepted,
            UnresolvedControl => Pending,
            AuthorizedNoncanonical | AuthorizedCurrentExcluded => Excluded,
            InvalidReferencedControl | Unauthorized => Invalid,
        };
        let mut reason_sequences = vec![Vec::new()];
        let mut exact_length = vec![Vec::new()];
        for _ in 0..4 {
            exact_length = exact_length
                .iter()
                .flat_map(|prefix| {
                    reasons.iter().map(|reason| {
                        let mut sequence = prefix.clone();
                        sequence.push(*reason);
                        sequence
                    })
                })
                .collect();
            reason_sequences.extend(exact_length.clone());
        }
        assert_eq!(reason_sequences.len(), 1_555);

        for reason_sequence in reason_sequences {
            let contributions = reason_sequence
                .iter()
                .copied()
                .map(aggregate_change_contribution)
                .collect::<Vec<_>>();
            let carrier_outcomes = reason_sequence
                .iter()
                .copied()
                .enumerate()
                .map(|(index, reason)| {
                    ChangeCarrierOutcome::new(
                        crate::EventId::from_bytes(
                            [u8::try_from(index + 1).unwrap_or(u8::MAX); 32],
                        ),
                        crate::ChangeHash::from_bytes([7; 32]),
                        crate::EventId::from_bytes([8; 32]),
                        reason,
                    )
                })
                .collect::<Vec<_>>();
            assert!(
                carrier_outcomes
                    .iter()
                    .zip(&reason_sequence)
                    .all(|(outcome, reason)| outcome.disposition == expected_carrier(*reason))
            );

            for lineage in [
                FinalLineageChangeState::Accepted,
                FinalLineageChangeState::CanonicalPruned,
                FinalLineageChangeState::Current,
            ] {
                let expected_aggregate = match lineage {
                    FinalLineageChangeState::Accepted => Accepted,
                    FinalLineageChangeState::CanonicalPruned => Excluded,
                    FinalLineageChangeState::Current if contributions.contains(&Unresolved) => {
                        Pending
                    }
                    FinalLineageChangeState::Current
                        if contributions.contains(&AuthorizedExcluded) =>
                    {
                        Excluded
                    }
                    FinalLineageChangeState::Current => Invalid,
                };
                assert_eq!(
                    reduce_aggregate_change_outcome(lineage, &contributions),
                    expected_aggregate
                );
                let mut reversed = contributions.clone();
                reversed.reverse();
                assert_eq!(
                    reduce_aggregate_change_outcome(lineage, &reversed),
                    expected_aggregate
                );
                assert!(
                    carrier_outcomes
                        .iter()
                        .zip(&reason_sequence)
                        .all(|(outcome, reason)| outcome.disposition == expected_carrier(*reason))
                );
            }
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
    fn descriptor_control_and_role_state_precede_every_history_refusal() {
        use DescriptorControlOutcome::{
            CanonicalAuthorized, DynamicInvalid, Missing, Noncanonical, Pending, RoleDenied,
            StaticInvalid, UnsupportedRevision, WrongCoordinate, WrongKind,
        };

        let control_states = [
            (Some(CanonicalAuthorized), None, None),
            (
                Some(Missing),
                Some(Status::PendingControl),
                Some((ProtocolDisposition::Pending, None)),
            ),
            (
                Some(Pending),
                Some(Status::PendingControl),
                Some((ProtocolDisposition::Pending, None)),
            ),
            (Some(Noncanonical), Some(Status::Unauthorized), None),
            (Some(WrongKind), Some(Status::Unauthorized), None),
            (Some(WrongCoordinate), Some(Status::Unauthorized), None),
            (Some(StaticInvalid), Some(Status::Unauthorized), None),
            (Some(DynamicInvalid), Some(Status::Unauthorized), None),
            (Some(UnsupportedRevision), Some(Status::Unauthorized), None),
            (Some(RoleDenied), Some(Status::Unauthorized), None),
            (None, Some(Status::Unauthorized), None),
        ];
        let history_states = [
            (
                HistoryVerificationError::UnknownControl,
                Status::PendingControl,
            ),
            (
                HistoryVerificationError::MissingCarrier,
                Status::MissingHistoricalCarrier,
            ),
            (
                HistoryVerificationError::NotAccepted,
                Status::NotAcceptedAtControl,
            ),
            (HistoryVerificationError::Snapshot, Status::SnapshotLoad),
            (HistoryVerificationError::Budget, Status::BudgetExhausted),
            (HistoryVerificationError::Cancelled, Status::Cancelled),
        ];

        for (control, expected_control_refusal, pending_outcome) in control_states {
            let control_refusal = checkpoint_control_refusal(control);
            assert_eq!(control_refusal, expected_control_refusal);
            if let Some(status) = control_refusal {
                let (disposition, diagnostic) = status.event_outcome();
                let outcome = (disposition, diagnostic.map(crate::DiagnosticCode::as_str));
                assert_eq!(
                    outcome,
                    pending_outcome
                        .unwrap_or((ProtocolDisposition::Invalid, Some("checkpoint.history")))
                );
            }
            assert_eq!(
                checkpoint_preflight_refusal(control_refusal, None),
                expected_control_refusal
            );
            for (history, expected_history_refusal) in history_states {
                assert_eq!(
                    checkpoint_preflight_refusal(control_refusal, Some(history)),
                    expected_control_refusal.or(Some(expected_history_refusal))
                );
            }
        }
    }

    #[derive(Default)]
    struct RecordingCheckpointWorkObserver {
        authorized_precharges: u64,
        downstream: std::collections::BTreeMap<(EventId, CheckpointDownstreamStage), u64>,
        report_attribution_calls:
            std::collections::BTreeMap<(EventId, CheckpointReportAttributionStage), u64>,
        report_attribution_items:
            std::collections::BTreeMap<(EventId, CheckpointReportAttributionStage), u64>,
    }

    impl CheckpointWorkObserver for RecordingCheckpointWorkObserver {
        fn authorized_precharge(&mut self) {
            self.authorized_precharges += 1;
        }

        fn enter_downstream(&mut self, descriptor_id: EventId, stage: CheckpointDownstreamStage) {
            *self.downstream.entry((descriptor_id, stage)).or_default() += 1;
        }

        fn enter_report_attribution(
            &mut self,
            descriptor_id: EventId,
            stage: CheckpointReportAttributionStage,
        ) {
            *self
                .report_attribution_calls
                .entry((descriptor_id, stage))
                .or_default() += 1;
        }

        fn report_attribution_item(
            &mut self,
            descriptor_id: EventId,
            stage: CheckpointReportAttributionStage,
        ) {
            *self
                .report_attribution_items
                .entry((descriptor_id, stage))
                .or_default() += 1;
        }
    }

    impl RecordingCheckpointWorkObserver {
        fn calls(&self, descriptor_id: EventId, stage: CheckpointDownstreamStage) -> u64 {
            self.downstream
                .get(&(descriptor_id, stage))
                .copied()
                .unwrap_or_default()
        }

        fn descriptor_calls(&self, descriptor_id: EventId) -> u64 {
            self.downstream
                .iter()
                .filter(|((observed_id, _), _)| *observed_id == descriptor_id)
                .map(|(_, calls)| *calls)
                .sum()
        }

        fn report_calls(
            &self,
            descriptor_id: EventId,
            stage: CheckpointReportAttributionStage,
        ) -> u64 {
            self.report_attribution_calls
                .get(&(descriptor_id, stage))
                .copied()
                .unwrap_or_default()
        }

        fn report_items(
            &self,
            descriptor_id: EventId,
            stage: CheckpointReportAttributionStage,
        ) -> u64 {
            self.report_attribution_items
                .get(&(descriptor_id, stage))
                .copied()
                .unwrap_or_default()
        }

        fn descriptor_report_items(&self, descriptor_id: EventId) -> u64 {
            self.report_attribution_items
                .iter()
                .filter(|((observed_id, _), _)| *observed_id == descriptor_id)
                .map(|(_, items)| *items)
                .sum()
        }
    }

    struct PreparedCheckpointHarness {
        corpus: EvidenceCorpus,
        coordinate: DocumentCoordinate,
        control_id: EventId,
        attribution_hash: ChangeHash,
        attribution_event_id: EventId,
        descriptor_ids: Vec<EventId>,
    }

    fn prepared_checkpoint_harness(descriptor_bytes: &[u8]) -> PreparedCheckpointHarness {
        prepared_checkpoint_harness_with_report_shape(
            descriptor_bytes,
            std::collections::BTreeSet::new(),
            1,
        )
    }

    fn prepared_checkpoint_harness_with_report_shape(
        descriptor_bytes: &[u8],
        descriptor_heads: std::collections::BTreeSet<ChangeHash>,
        chunks_per_descriptor: u8,
    ) -> PreparedCheckpointHarness {
        assert!(chunks_per_descriptor > 0);
        let coordinate = DocumentCoordinate::new(
            ControllerPublicKey::from_bytes([0x70; 32]),
            DocumentId::from_bytes([0x71; 32]),
        );
        let author = DevicePublicKey::from_bytes([0x72; 32]);
        let control_id = EventId::from_bytes([0x73; 32]);
        let attribution_hash = ChangeHash::from_bytes([0x75; 32]);
        let attribution_event_id = EventId::from_bytes([0x76; 32]);
        let actor = ActorId::from_bytes([0x74; 32]);
        let document = AuthoringDocument::empty(ActorState::initial(
            actor,
            std::collections::BTreeSet::new(),
        ));
        assert!(document.is_ok());
        let snapshot = document.map_or_else(|_| Vec::new(), |value| value.accepted_state_bytes());
        assert!(!snapshot.is_empty());
        let snapshot_hash_bytes: [u8; 32] = Sha256::digest(&snapshot).into();
        let snapshot_hash = SnapshotHash::from_bytes(snapshot_hash_bytes);
        let chunk_hash = ChunkHash::from_bytes(snapshot_hash_bytes);
        let chunk_root = leaf_hash(0, 1, snapshot_hash_bytes);
        let empty_change_set_hash: [u8; 32] = Sha256::digest(
            [
                b"nostr-crdt/automerge/change-set/v1".as_slice(),
                &[0],
                &0_u64.to_be_bytes(),
            ]
            .concat(),
        )
        .into();
        let chunk_size = u32::try_from(snapshot.len()).unwrap_or(u32::MAX);
        let mut events = std::collections::BTreeMap::new();
        let mut descriptor_ids = Vec::new();
        for descriptor_byte in descriptor_bytes {
            let descriptor_id = EventId::from_bytes([*descriptor_byte; 32]);
            let descriptor = CheckpointDescriptor {
                snapshot_hash,
                heads: descriptor_heads.clone(),
                raw_size: u64::try_from(snapshot.len()).unwrap_or(u64::MAX),
                chunk_size,
                chunk_count: u32::from(chunks_per_descriptor),
                chunk_root,
                change_count: u64::try_from(descriptor_heads.len()).unwrap_or(u64::MAX),
                change_set_hash: empty_change_set_hash,
                dependency_edges: 0,
                total_ops: 0,
            };
            let descriptor = ValidatedCheckpointDescriptorCarrier::for_test(
                descriptor_id,
                author,
                coordinate,
                control_id,
                descriptor,
            );
            events.insert(
                descriptor_id,
                EventEvidence::VerifiedCarrier {
                    carrier: VerifiedCarrier::CheckpointDescriptor(Box::new(descriptor)),
                    raw_checksum: RawChecksum::test_only([*descriptor_byte; 32]),
                },
            );
            for chunk_index in 0..chunks_per_descriptor {
                let chunk_byte = descriptor_byte
                    .saturating_add(0x80)
                    .saturating_add(chunk_index);
                let chunk_id = EventId::from_bytes([chunk_byte; 32]);
                let chunk = ValidatedCheckpointChunkCarrier::for_test(
                    chunk_id,
                    author,
                    coordinate,
                    descriptor_id,
                    chunk_hash,
                    CheckpointChunk {
                        index: u32::from(chunk_index),
                        count: u32::from(chunks_per_descriptor),
                        data: snapshot.clone(),
                        proof: Vec::new(),
                    },
                );
                events.insert(
                    chunk_id,
                    EventEvidence::VerifiedCarrier {
                        carrier: VerifiedCarrier::CheckpointChunk(Box::new(chunk)),
                        raw_checksum: RawChecksum::test_only([chunk_byte.saturating_add(1); 32]),
                    },
                );
            }
            descriptor_ids.push(descriptor_id);
        }
        let indexes = derive_trusted_indexes(&events, &[]);
        PreparedCheckpointHarness {
            corpus: EvidenceCorpus {
                events,
                invalid: std::collections::BTreeMap::new(),
                duplicates: Vec::new(),
                indexes,
            },
            coordinate,
            control_id,
            attribution_hash,
            attribution_event_id,
            descriptor_ids,
        }
    }

    fn evaluate_prepared_checkpoints(
        harness: &PreparedCheckpointHarness,
        authorizations: &std::collections::BTreeMap<EventId, DescriptorControlOutcome>,
        budget: &mut WorkBudget,
        cancellation: &impl crate::CancellationCheck,
        observer: &mut RecordingCheckpointWorkObserver,
    ) -> super::CheckpointEvaluation {
        let view = DocumentEvidenceView::derive(&harness.corpus, harness.coordinate);
        let accepted_at_control = std::collections::BTreeMap::from([(
            harness.control_id,
            AcceptedAtControl::for_test(std::collections::BTreeSet::from([
                harness.attribution_hash
            ])),
        )]);
        let branch_change_dispositions = std::collections::BTreeMap::from([(
            harness.control_id,
            crate::reference::branch_state::PersistentDeltaMap::from_local(
                std::collections::BTreeMap::from([(
                    harness.attribution_hash,
                    ProtocolDisposition::Accepted,
                )]),
            ),
        )]);
        let change_carrier_dispositions = std::collections::BTreeMap::from([(
            harness.attribution_event_id,
            ChangeCarrierOutcome::new(
                harness.attribution_event_id,
                harness.attribution_hash,
                harness.control_id,
                ChangeClaimReason::AuthorizedCanonical,
            ),
        )]);
        verify_prepared_checkpoints(
            PreparedCheckpointInputs {
                view: &view,
                canonical_controls: &[harness.control_id],
                history: CheckpointHistoryInputs {
                    accepted_at_control: &accepted_at_control,
                    branch_change_dispositions: &branch_change_dispositions,
                    change_carrier_dispositions: &change_carrier_dispositions,
                },
                authorizations,
            },
            budget,
            cancellation,
            observer,
        )
    }

    #[test]
    fn refused_checkpoint_attribution_rejects_unknown_future_carrier_control() {
        let harness = prepared_checkpoint_harness(&[0x10]);
        let view = DocumentEvidenceView::derive(&harness.corpus, harness.coordinate);
        assert!(carrier_control_is_historical(
            &view,
            harness.control_id,
            harness.control_id,
        ));
        assert!(!carrier_control_is_historical(
            &view,
            harness.control_id,
            EventId::from_bytes([0x77; 32]),
        ));
    }

    #[test]
    #[ignore = "open FINDING_095 checkpoint-ancestry reproduction"]
    fn finding_095_lower_sequence_sibling_is_not_historical() {
        let coordinate = DocumentCoordinate::new(
            ControllerPublicKey::from_bytes([0x61; 32]),
            DocumentId::from_bytes([0x62; 32]),
        );
        let parent_id = EventId::from_bytes([0x63; 32]);
        let sibling_id = EventId::from_bytes([0x64; 32]);
        let through_id = EventId::from_bytes([0x65; 32]);
        let control = |event_id, parent, sequence| {
            ValidatedControlCarrier::for_test(
                event_id,
                coordinate.controller(),
                coordinate,
                parent,
                ValidatedControlContent {
                    base_heads: Vec::new(),
                    members: Vec::new(),
                    predecessor: None,
                    sequence,
                    successor: None,
                    terminal: true,
                },
            )
        };
        let checksum = RawChecksum::test_only([0x66; 32]);
        let events = std::collections::BTreeMap::from([
            (
                parent_id,
                EventEvidence::VerifiedCarrier {
                    carrier: VerifiedCarrier::Control(Box::new(control(parent_id, None, 0))),
                    raw_checksum: checksum,
                },
            ),
            (
                sibling_id,
                EventEvidence::VerifiedCarrier {
                    carrier: VerifiedCarrier::Control(Box::new(control(
                        sibling_id,
                        Some(parent_id),
                        1,
                    ))),
                    raw_checksum: checksum,
                },
            ),
            (
                through_id,
                EventEvidence::VerifiedCarrier {
                    carrier: VerifiedCarrier::Control(Box::new(control(
                        through_id,
                        Some(parent_id),
                        2,
                    ))),
                    raw_checksum: checksum,
                },
            ),
        ]);
        let indexes = derive_trusted_indexes(&events, &[]);
        let corpus = EvidenceCorpus {
            events,
            invalid: std::collections::BTreeMap::new(),
            duplicates: Vec::new(),
            indexes,
        };
        let view = DocumentEvidenceView::derive(&corpus, coordinate);
        assert!(
            !carrier_control_is_historical(&view, through_id, sibling_id),
            "FINDING_095 reproduced: lower sequence is accepted as checkpoint ancestry"
        );
    }

    fn assert_all_downstream_stages_once(
        observer: &RecordingCheckpointWorkObserver,
        descriptor_id: EventId,
    ) {
        for stage in [
            CheckpointDownstreamStage::ChunkSetCollection,
            CheckpointDownstreamStage::ChunkEventCollection,
            CheckpointDownstreamStage::CarrierHistoryCoverage,
            CheckpointDownstreamStage::AcceptedAtControlLookup,
            CheckpointDownstreamStage::SnapshotLoad,
            CheckpointDownstreamStage::HistoryVerification,
        ] {
            assert_eq!(observer.calls(descriptor_id, stage), 1);
        }
    }

    fn assert_no_downstream_stages(
        observer: &RecordingCheckpointWorkObserver,
        descriptor_id: EventId,
    ) {
        for stage in [
            CheckpointDownstreamStage::ChunkSetCollection,
            CheckpointDownstreamStage::ChunkEventCollection,
            CheckpointDownstreamStage::CarrierHistoryCoverage,
            CheckpointDownstreamStage::AcceptedAtControlLookup,
            CheckpointDownstreamStage::SnapshotLoad,
            CheckpointDownstreamStage::HistoryVerification,
        ] {
            assert_eq!(observer.calls(descriptor_id, stage), 0);
        }
    }

    fn assert_report_attribution(
        observer: &RecordingCheckpointWorkObserver,
        descriptor_id: EventId,
        accepted_hash: bool,
        branch_hash: bool,
    ) {
        assert_eq!(
            observer.report_calls(
                descriptor_id,
                CheckpointReportAttributionStage::IndexedChunkEvent
            ),
            1
        );
        assert_eq!(
            observer.report_items(
                descriptor_id,
                CheckpointReportAttributionStage::IndexedChunkEvent
            ),
            1
        );
        assert_eq!(
            observer.report_calls(
                descriptor_id,
                CheckpointReportAttributionStage::DescriptorHead
            ),
            1
        );
        assert_eq!(
            observer.report_items(
                descriptor_id,
                CheckpointReportAttributionStage::DescriptorHead
            ),
            0
        );
        for (stage, expected) in [
            (
                CheckpointReportAttributionStage::AcceptedAtControlHash,
                u64::from(accepted_hash),
            ),
            (
                CheckpointReportAttributionStage::BranchDispositionHash,
                u64::from(branch_hash),
            ),
        ] {
            assert_eq!(observer.report_calls(descriptor_id, stage), expected);
            assert_eq!(observer.report_items(descriptor_id, stage), expected);
        }
    }

    #[test]
    fn every_refused_descriptor_family_skips_each_checkpoint_verification_stage() {
        use DescriptorControlOutcome::{
            DynamicInvalid, Missing, Noncanonical, Pending, RoleDenied, StaticInvalid,
            UnsupportedRevision, WrongCoordinate, WrongKind,
        };

        let harness = prepared_checkpoint_harness(&[0x10]);
        let descriptor_id = harness.descriptor_ids[0];
        for (outcome, expected, accepted_hash, branch_hash) in [
            (Some(Missing), Status::PendingControl, false, false),
            (Some(Pending), Status::PendingControl, false, false),
            (Some(Noncanonical), Status::Unauthorized, true, false),
            (Some(WrongKind), Status::Unauthorized, false, false),
            (Some(WrongCoordinate), Status::Unauthorized, false, false),
            (Some(StaticInvalid), Status::Unauthorized, false, false),
            (Some(DynamicInvalid), Status::Unauthorized, false, false),
            (
                Some(UnsupportedRevision),
                Status::Unauthorized,
                false,
                false,
            ),
            (Some(RoleDenied), Status::Unauthorized, true, true),
            (None, Status::Unauthorized, false, false),
        ] {
            let authorizations = outcome
                .map(|value| std::collections::BTreeMap::from([(descriptor_id, value)]))
                .unwrap_or_default();
            let mut budget = WorkBudget::new(u64::MAX, u64::MAX);
            let mut observer = RecordingCheckpointWorkObserver::default();
            let evaluation = evaluate_prepared_checkpoints(
                &harness,
                &authorizations,
                &mut budget,
                &crate::NeverCancelled,
                &mut observer,
            );
            assert_eq!(evaluation.stop, None);
            assert_eq!(evaluation.results.len(), 1);
            assert_eq!(evaluation.results[0].status(), expected);
            let expected_event_outcome = if expected == Status::PendingControl {
                (ProtocolDisposition::Pending, None)
            } else {
                (
                    ProtocolDisposition::Invalid,
                    Some(crate::DiagnosticCode::registered("checkpoint.history")),
                )
            };
            assert_eq!(expected.event_outcome(), expected_event_outcome);
            assert_eq!(evaluation.results[0].chunk_events().len(), 1);
            assert_eq!(
                evaluation.results[0].accepted_at_control().len(),
                usize::from(accepted_hash)
            );
            assert_eq!(
                evaluation.results[0].historical_carriers().len(),
                usize::from(branch_hash)
            );
            assert_eq!(observer.authorized_precharges, 0);
            assert_no_downstream_stages(&observer, descriptor_id);
            assert_report_attribution(&observer, descriptor_id, accepted_hash, branch_hash);
            assert_eq!(
                budget.consumed().get(WorkCounter::CheckpointItem),
                2 + u64::from(accepted_hash) + u64::from(branch_hash)
            );
        }
    }

    #[test]
    fn authorized_precharge_and_checkpoint_verification_are_order_independent() {
        use DescriptorControlOutcome::{CanonicalAuthorized, Missing, RoleDenied};

        for outcomes in [
            vec![Missing, CanonicalAuthorized],
            vec![CanonicalAuthorized, RoleDenied],
            vec![
                CanonicalAuthorized,
                CanonicalAuthorized,
                CanonicalAuthorized,
            ],
        ] {
            let descriptor_bytes = (0..outcomes.len())
                .map(|offset| 0x20_u8.saturating_add(u8::try_from(offset).unwrap_or(u8::MAX)))
                .collect::<Vec<_>>();
            let harness = prepared_checkpoint_harness(&descriptor_bytes);
            let authorizations = harness
                .descriptor_ids
                .iter()
                .copied()
                .zip(outcomes.iter().copied())
                .collect::<std::collections::BTreeMap<_, _>>();
            let mut budget = WorkBudget::new(u64::MAX, u64::MAX);
            let mut observer = RecordingCheckpointWorkObserver::default();
            let evaluation = evaluate_prepared_checkpoints(
                &harness,
                &authorizations,
                &mut budget,
                &crate::NeverCancelled,
                &mut observer,
            );
            assert_eq!(evaluation.stop, None);
            assert_eq!(evaluation.results.len(), outcomes.len());
            assert_eq!(observer.authorized_precharges, 1);
            for (descriptor_id, outcome) in harness
                .descriptor_ids
                .iter()
                .copied()
                .zip(outcomes.iter().copied())
            {
                if outcome == CanonicalAuthorized {
                    assert_eq!(
                        evaluation
                            .results
                            .iter()
                            .find(|result| result.descriptor_event() == descriptor_id)
                            .map(CheckpointVerificationResult::status),
                        Some(Status::Verified)
                    );
                    assert_all_downstream_stages_once(&observer, descriptor_id);
                    assert_eq!(observer.descriptor_report_items(descriptor_id), 0);
                } else {
                    assert_eq!(observer.descriptor_calls(descriptor_id), 0);
                    assert_report_attribution(
                        &observer,
                        descriptor_id,
                        outcome == RoleDenied,
                        outcome == RoleDenied,
                    );
                }
            }
        }
    }

    #[test]
    fn checkpoint_authorization_gate_preserves_budget_and_cancellation_boundaries() {
        use DescriptorControlOutcome::{CanonicalAuthorized, Missing, RoleDenied};

        let harness = prepared_checkpoint_harness(&[0x30]);
        let descriptor_id = harness.descriptor_ids[0];
        let authorized = std::collections::BTreeMap::from([(descriptor_id, CanonicalAuthorized)]);

        let mut zero_budget = WorkBudget::new(0, 0);
        let mut zero_observer = RecordingCheckpointWorkObserver::default();
        let zero = evaluate_prepared_checkpoints(
            &harness,
            &authorized,
            &mut zero_budget,
            &crate::NeverCancelled,
            &mut zero_observer,
        );
        assert_eq!(zero.stop, Some(CheckpointWorkStop::Budget));
        assert_eq!(zero_budget.consumed().get(WorkCounter::CheckpointItem), 0);
        assert_eq!(zero_observer.authorized_precharges, 0);
        assert_eq!(zero_observer.descriptor_calls(descriptor_id), 0);

        let mut precharge_budget = WorkBudget::new(0, 1);
        let mut precharge_observer = RecordingCheckpointWorkObserver::default();
        let precharge_stop = evaluate_prepared_checkpoints(
            &harness,
            &authorized,
            &mut precharge_budget,
            &crate::NeverCancelled,
            &mut precharge_observer,
        );
        assert_eq!(precharge_stop.stop, Some(CheckpointWorkStop::Budget));
        assert_eq!(
            precharge_budget.consumed().get(WorkCounter::CheckpointItem),
            1
        );
        assert_eq!(precharge_observer.authorized_precharges, 0);
        assert_eq!(precharge_observer.descriptor_calls(descriptor_id), 0);

        let mut downstream_budget = WorkBudget::new(0, 2);
        let mut downstream_observer = RecordingCheckpointWorkObserver::default();
        let downstream_stop = evaluate_prepared_checkpoints(
            &harness,
            &authorized,
            &mut downstream_budget,
            &crate::NeverCancelled,
            &mut downstream_observer,
        );
        assert_eq!(downstream_stop.stop, Some(CheckpointWorkStop::Budget));
        assert_eq!(downstream_observer.authorized_precharges, 1);
        assert_eq!(
            downstream_observer.calls(descriptor_id, CheckpointDownstreamStage::ChunkSetCollection),
            1
        );
        assert_eq!(downstream_observer.descriptor_calls(descriptor_id), 1);

        let refused = std::collections::BTreeMap::from([(descriptor_id, Missing)]);
        let cancellation_calls = std::cell::Cell::new(0_u64);
        let cancellation = || {
            cancellation_calls.set(cancellation_calls.get() + 1);
            false
        };
        let mut refused_budget = WorkBudget::new(0, u64::MAX);
        let mut refused_observer = RecordingCheckpointWorkObserver::default();
        let refused_result = evaluate_prepared_checkpoints(
            &harness,
            &refused,
            &mut refused_budget,
            &cancellation,
            &mut refused_observer,
        );
        assert_eq!(refused_result.stop, None);
        assert_eq!(cancellation_calls.get(), 2);
        assert_eq!(refused_observer.authorized_precharges, 0);
        assert_eq!(refused_observer.descriptor_calls(descriptor_id), 0);
        assert_report_attribution(&refused_observer, descriptor_id, false, false);
        assert_eq!(
            refused_budget.consumed().get(WorkCounter::CheckpointItem),
            2
        );

        let role_denied = std::collections::BTreeMap::from([(descriptor_id, RoleDenied)]);
        let mut attribution_n_minus_one_budget = WorkBudget::new(0, 3);
        let mut attribution_n_minus_one_observer = RecordingCheckpointWorkObserver::default();
        let attribution_n_minus_one = evaluate_prepared_checkpoints(
            &harness,
            &role_denied,
            &mut attribution_n_minus_one_budget,
            &crate::NeverCancelled,
            &mut attribution_n_minus_one_observer,
        );
        assert_eq!(
            attribution_n_minus_one.stop,
            Some(CheckpointWorkStop::Budget)
        );
        assert!(attribution_n_minus_one.results.is_empty());
        assert_eq!(
            attribution_n_minus_one_budget
                .consumed()
                .get(WorkCounter::CheckpointItem),
            3
        );
        assert_eq!(
            attribution_n_minus_one_observer.descriptor_calls(descriptor_id),
            0
        );
        assert_eq!(
            attribution_n_minus_one_observer.descriptor_report_items(descriptor_id),
            2
        );
        assert_eq!(
            attribution_n_minus_one_observer.report_calls(
                descriptor_id,
                CheckpointReportAttributionStage::BranchDispositionHash
            ),
            1
        );
        assert_eq!(
            attribution_n_minus_one_observer.report_items(
                descriptor_id,
                CheckpointReportAttributionStage::BranchDispositionHash
            ),
            0
        );

        let mut attribution_exact_budget = WorkBudget::new(0, 4);
        let mut attribution_exact_observer = RecordingCheckpointWorkObserver::default();
        let attribution_exact = evaluate_prepared_checkpoints(
            &harness,
            &role_denied,
            &mut attribution_exact_budget,
            &crate::NeverCancelled,
            &mut attribution_exact_observer,
        );
        assert_eq!(attribution_exact.stop, None);
        assert_eq!(attribution_exact.results.len(), 1);
        assert_eq!(attribution_exact.results[0].status(), Status::Unauthorized);
        assert_eq!(attribution_exact.results[0].chunk_events().len(), 1);
        assert_eq!(
            attribution_exact.results[0].accepted_at_control(),
            &[harness.attribution_hash]
        );
        assert_eq!(
            attribution_exact.results[0].historical_carriers(),
            &[harness.attribution_event_id]
        );
        assert_eq!(
            attribution_exact_budget
                .consumed()
                .get(WorkCounter::CheckpointItem),
            4
        );
        assert_eq!(
            attribution_exact_observer.descriptor_calls(descriptor_id),
            0
        );
        assert_report_attribution(&attribution_exact_observer, descriptor_id, true, true);

        let attribution_cancel_calls = std::cell::Cell::new(0_u64);
        let attribution_cancellation = || {
            attribution_cancel_calls.set(attribution_cancel_calls.get() + 1);
            attribution_cancel_calls.get() >= 4
        };
        let mut attribution_cancel_budget = WorkBudget::new(0, u64::MAX);
        let mut attribution_cancel_observer = RecordingCheckpointWorkObserver::default();
        let attribution_cancelled = evaluate_prepared_checkpoints(
            &harness,
            &role_denied,
            &mut attribution_cancel_budget,
            &attribution_cancellation,
            &mut attribution_cancel_observer,
        );
        assert_eq!(
            attribution_cancelled.stop,
            Some(CheckpointWorkStop::Cancelled)
        );
        assert!(attribution_cancelled.results.is_empty());
        assert_eq!(attribution_cancel_calls.get(), 4);
        assert_eq!(
            attribution_cancel_budget
                .consumed()
                .get(WorkCounter::CheckpointItem),
            3
        );
        assert_eq!(
            attribution_cancel_observer.descriptor_calls(descriptor_id),
            0
        );
        assert_eq!(
            attribution_cancel_observer.descriptor_report_items(descriptor_id),
            2
        );

        let mut cancelled_budget = WorkBudget::new(0, u64::MAX);
        let mut cancelled_observer = RecordingCheckpointWorkObserver::default();
        let cancelled = evaluate_prepared_checkpoints(
            &harness,
            &authorized,
            &mut cancelled_budget,
            &|| true,
            &mut cancelled_observer,
        );
        assert_eq!(cancelled.stop, Some(CheckpointWorkStop::Cancelled));
        assert_eq!(cancelled_observer.authorized_precharges, 0);
        assert_eq!(cancelled_observer.descriptor_calls(descriptor_id), 0);
        assert_eq!(cancelled_observer.descriptor_report_items(descriptor_id), 0);

        let staged_calls = std::cell::Cell::new(0_u64);
        let staged_cancellation = || {
            staged_calls.set(staged_calls.get() + 1);
            staged_calls.get() >= 3
        };
        let mut staged_budget = WorkBudget::new(0, u64::MAX);
        let mut staged_observer = RecordingCheckpointWorkObserver::default();
        let staged = evaluate_prepared_checkpoints(
            &harness,
            &authorized,
            &mut staged_budget,
            &staged_cancellation,
            &mut staged_observer,
        );
        assert_eq!(staged.stop, Some(CheckpointWorkStop::Cancelled));
        assert_eq!(staged_observer.authorized_precharges, 1);
        assert_eq!(staged_observer.descriptor_calls(descriptor_id), 1);
    }

    #[test]
    fn refusal_report_attribution_has_exact_partial_result_boundaries() {
        use DescriptorControlOutcome::{Missing, RoleDenied};

        let harness = prepared_checkpoint_harness(&[0x40, 0x41]);
        let first = harness.descriptor_ids[0];
        let second = harness.descriptor_ids[1];
        let authorizations =
            std::collections::BTreeMap::from([(first, Missing), (second, RoleDenied)]);

        let mut n_minus_one_budget = WorkBudget::new(0, 5);
        let mut n_minus_one_observer = RecordingCheckpointWorkObserver::default();
        let n_minus_one = evaluate_prepared_checkpoints(
            &harness,
            &authorizations,
            &mut n_minus_one_budget,
            &crate::NeverCancelled,
            &mut n_minus_one_observer,
        );
        assert_eq!(n_minus_one.stop, Some(CheckpointWorkStop::Budget));
        assert_eq!(n_minus_one.results.len(), 1);
        assert_eq!(n_minus_one.results[0].descriptor_event(), first);
        assert_eq!(n_minus_one.results[0].status(), Status::PendingControl);
        assert_eq!(
            n_minus_one_budget
                .consumed()
                .get(WorkCounter::CheckpointItem),
            5
        );
        assert_eq!(n_minus_one_observer.descriptor_calls(first), 0);
        assert_eq!(n_minus_one_observer.descriptor_calls(second), 0);
        assert_report_attribution(&n_minus_one_observer, first, false, false);
        assert_eq!(n_minus_one_observer.descriptor_report_items(second), 2);
        assert_eq!(
            n_minus_one_observer.report_items(
                second,
                CheckpointReportAttributionStage::BranchDispositionHash
            ),
            0
        );

        let mut exact_budget = WorkBudget::new(0, 6);
        let mut exact_observer = RecordingCheckpointWorkObserver::default();
        let exact = evaluate_prepared_checkpoints(
            &harness,
            &authorizations,
            &mut exact_budget,
            &crate::NeverCancelled,
            &mut exact_observer,
        );
        assert_eq!(exact.stop, None);
        assert_eq!(exact.results.len(), 2);
        assert_eq!(exact.results[0].descriptor_event(), first);
        assert_eq!(exact.results[1].descriptor_event(), second);
        assert_eq!(exact.results[1].status(), Status::Unauthorized);
        assert_eq!(exact_budget.consumed().get(WorkCounter::CheckpointItem), 6);
        assert_report_attribution(&exact_observer, first, false, false);
        assert_report_attribution(&exact_observer, second, true, true);

        let cancellation_calls = std::cell::Cell::new(0_u64);
        let cancellation = || {
            cancellation_calls.set(cancellation_calls.get() + 1);
            cancellation_calls.get() >= 6
        };
        let mut cancelled_budget = WorkBudget::new(0, u64::MAX);
        let mut cancelled_observer = RecordingCheckpointWorkObserver::default();
        let cancelled = evaluate_prepared_checkpoints(
            &harness,
            &authorizations,
            &mut cancelled_budget,
            &cancellation,
            &mut cancelled_observer,
        );
        assert_eq!(cancelled.stop, Some(CheckpointWorkStop::Cancelled));
        assert_eq!(cancelled.results.len(), 1);
        assert_eq!(cancelled.results[0].descriptor_event(), first);
        assert_eq!(cancellation_calls.get(), 6);
        assert_eq!(
            cancelled_budget.consumed().get(WorkCounter::CheckpointItem),
            5
        );
        assert_eq!(cancelled_observer.descriptor_calls(first), 0);
        assert_eq!(cancelled_observer.descriptor_calls(second), 0);
        assert_eq!(cancelled_observer.descriptor_report_items(second), 2);
    }

    #[test]
    fn refusal_report_attribution_meters_ordered_heads_and_chunks_exactly() {
        let first_head = ChangeHash::from_bytes([0x11; 32]);
        let second_head = ChangeHash::from_bytes([0x22; 32]);
        let third_head = ChangeHash::from_bytes([0x33; 32]);
        let harness = prepared_checkpoint_harness_with_report_shape(
            &[0x42],
            std::collections::BTreeSet::from([third_head, first_head, second_head, second_head]),
            3,
        );
        let descriptor_id = harness.descriptor_ids[0];
        let authorizations =
            std::collections::BTreeMap::from([(descriptor_id, DescriptorControlOutcome::Missing)]);

        let mut n_minus_one_budget = WorkBudget::new(0, 6);
        let mut n_minus_one_observer = RecordingCheckpointWorkObserver::default();
        let n_minus_one = evaluate_prepared_checkpoints(
            &harness,
            &authorizations,
            &mut n_minus_one_budget,
            &crate::NeverCancelled,
            &mut n_minus_one_observer,
        );
        assert_eq!(n_minus_one.stop, Some(CheckpointWorkStop::Budget));
        assert!(n_minus_one.results.is_empty());
        assert_eq!(
            n_minus_one_budget
                .consumed()
                .get(WorkCounter::CheckpointItem),
            6
        );
        assert_eq!(n_minus_one_observer.descriptor_calls(descriptor_id), 0);
        assert_eq!(
            n_minus_one_observer.report_items(
                descriptor_id,
                CheckpointReportAttributionStage::IndexedChunkEvent
            ),
            3
        );
        assert_eq!(
            n_minus_one_observer.report_items(
                descriptor_id,
                CheckpointReportAttributionStage::DescriptorHead
            ),
            2
        );

        let mut exact_budget = WorkBudget::new(0, 7);
        let mut exact_observer = RecordingCheckpointWorkObserver::default();
        let exact = evaluate_prepared_checkpoints(
            &harness,
            &authorizations,
            &mut exact_budget,
            &crate::NeverCancelled,
            &mut exact_observer,
        );
        assert_eq!(exact.stop, None);
        assert_eq!(exact.results.len(), 1);
        assert_eq!(
            exact.results[0].chunk_events(),
            &[
                EventId::from_bytes([0xc2; 32]),
                EventId::from_bytes([0xc3; 32]),
                EventId::from_bytes([0xc4; 32]),
            ]
        );
        assert_eq!(
            exact.results[0].heads(),
            &[first_head, second_head, third_head]
        );
        assert_eq!(exact_budget.consumed().get(WorkCounter::CheckpointItem), 7);
        assert_eq!(exact_observer.descriptor_calls(descriptor_id), 0);
        assert_eq!(
            exact_observer.report_items(
                descriptor_id,
                CheckpointReportAttributionStage::IndexedChunkEvent
            ),
            3
        );
        assert_eq!(
            exact_observer.report_items(
                descriptor_id,
                CheckpointReportAttributionStage::DescriptorHead
            ),
            3
        );
        assert_eq!(exact_observer.descriptor_report_items(descriptor_id), 6);

        let cancellation_calls = std::cell::Cell::new(0_u64);
        let cancellation = || {
            cancellation_calls.set(cancellation_calls.get() + 1);
            cancellation_calls.get() >= 7
        };
        let mut cancelled_budget = WorkBudget::new(0, u64::MAX);
        let mut cancelled_observer = RecordingCheckpointWorkObserver::default();
        let cancelled = evaluate_prepared_checkpoints(
            &harness,
            &authorizations,
            &mut cancelled_budget,
            &cancellation,
            &mut cancelled_observer,
        );
        assert_eq!(cancelled.stop, Some(CheckpointWorkStop::Cancelled));
        assert!(cancelled.results.is_empty());
        assert_eq!(cancellation_calls.get(), 7);
        assert_eq!(
            cancelled_budget.consumed().get(WorkCounter::CheckpointItem),
            6
        );
        assert_eq!(cancelled_observer.descriptor_calls(descriptor_id), 0);
        assert_eq!(
            cancelled_observer.report_items(
                descriptor_id,
                CheckpointReportAttributionStage::DescriptorHead
            ),
            2
        );
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
            control_records: 8,
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
        for reservation in plan.reservations() {
            assert!(
                permit
                    .consume_pass(FinalizationReservationUnit::new(reservation.pass, 0))
                    .is_ok()
            );
        }
        assert!(permit.refund(&mut exact).is_ok());
        assert_eq!(exact.remaining(), (0, 8));
        assert_eq!(exact.consumed().get(WorkCounter::Assertion), 0);
    }

    #[test]
    fn complete_report_plan_is_exact_named_and_overflow_checked() {
        let coordinate = DocumentCoordinate::new(
            ControllerPublicKey::from_bytes([0xd1; 32]),
            DocumentId::from_bytes([0xd2; 32]),
        );
        let mut indexes = TrustedIndexes::default();
        indexes.coordinates.work.insert(
            coordinate,
            CoordinateWorkMetadata {
                control_count: 2,
                change_hash_count: 3,
                reportable_event_count: 8,
                change_carrier_event_count: 4,
                other_event_count: 2,
                evidence_record_count: 10,
                checkpoint_descriptor_count: 2,
                checkpoint_chunk_count: 3,
                ..CoordinateWorkMetadata::default()
            },
        );
        let corpus = EvidenceCorpus {
            events: std::collections::BTreeMap::new(),
            invalid: std::collections::BTreeMap::new(),
            duplicates: Vec::new(),
            indexes,
        };
        let view = DocumentEvidenceView::derive(&corpus, coordinate);
        let plan = ReportFinalizationPlan::from_view(&view);
        assert!(plan.is_ok());
        let Ok(plan) = plan else { return };
        assert_eq!(
            plan.reservations(),
            [
                FinalizationReservationUnit::new(CompleteReportPass::ControlRecords, 4),
                FinalizationReservationUnit::new(CompleteReportPass::SemanticChangeRecords, 6),
                FinalizationReservationUnit::new(CompleteReportPass::ChangeCarrierEvents, 40),
                FinalizationReservationUnit::new(CompleteReportPass::OtherEvents, 70),
                FinalizationReservationUnit::new(CompleteReportPass::CheckpointRecords, 5),
                FinalizationReservationUnit::new(CompleteReportPass::ChangeClassifications, 15),
                FinalizationReservationUnit::new(CompleteReportPass::HistoryDigest, 12),
                FinalizationReservationUnit::new(CompleteReportPass::DispositionsDigest, 26),
                FinalizationReservationUnit::new(CompleteReportPass::EvidenceRecords, 10),
                FinalizationReservationUnit::new(CompleteReportPass::ReportInvariants, 620),
                FinalizationReservationUnit::new(CompleteReportPass::FixedOverhead, 8),
            ]
        );
        assert_eq!(plan.total(), Some(816));

        let mut overflow_indexes = TrustedIndexes::default();
        overflow_indexes.coordinates.work.insert(
            coordinate,
            CoordinateWorkMetadata {
                change_hash_count: usize::MAX,
                ..CoordinateWorkMetadata::default()
            },
        );
        let overflow_corpus = EvidenceCorpus {
            events: std::collections::BTreeMap::new(),
            invalid: std::collections::BTreeMap::new(),
            duplicates: Vec::new(),
            indexes: overflow_indexes,
        };
        assert!(
            ReportFinalizationPlan::from_view(&DocumentEvidenceView::derive(
                &overflow_corpus,
                coordinate,
            ))
            .is_err()
        );
    }

    #[test]
    fn complete_finalization_passes_start_only_after_exact_consumption() {
        FINALIZATION_PASS_OBSERVATIONS.with(|observations| observations.borrow_mut().clear());
        let coordinate = DocumentCoordinate::new(
            ControllerPublicKey::from_bytes([0xd3; 32]),
            DocumentId::from_bytes([0xd4; 32]),
        );
        let report = ReferenceEvaluator::new(crate::ProtocolRevision::draft_v1()).evaluate(
            &crate::CorpusBuilder::new().finish(),
            coordinate,
            &mut WorkBudget::new(1_000, 1_000),
            &crate::NeverCancelled,
        );
        assert!(report.is_ok());
        assert_eq!(
            FINALIZATION_PASS_OBSERVATIONS.with(|observations| observations.borrow().clone()),
            [
                FinalizationPassObservation::Consumed(CompleteReportPass::ControlRecords),
                FinalizationPassObservation::WorkStarted(CompleteReportPass::ControlRecords),
                FinalizationPassObservation::Consumed(CompleteReportPass::SemanticChangeRecords,),
                FinalizationPassObservation::WorkStarted(CompleteReportPass::SemanticChangeRecords,),
                FinalizationPassObservation::Consumed(CompleteReportPass::ChangeCarrierEvents),
                FinalizationPassObservation::Consumed(CompleteReportPass::OtherEvents),
                FinalizationPassObservation::WorkStarted(CompleteReportPass::ChangeCarrierEvents,),
                FinalizationPassObservation::WorkStarted(CompleteReportPass::OtherEvents),
                FinalizationPassObservation::Consumed(CompleteReportPass::CheckpointRecords),
                FinalizationPassObservation::WorkStarted(CompleteReportPass::CheckpointRecords),
                FinalizationPassObservation::Consumed(CompleteReportPass::ChangeClassifications,),
                FinalizationPassObservation::WorkStarted(CompleteReportPass::ChangeClassifications,),
                FinalizationPassObservation::Consumed(CompleteReportPass::HistoryDigest),
                FinalizationPassObservation::WorkStarted(CompleteReportPass::HistoryDigest),
                FinalizationPassObservation::Consumed(CompleteReportPass::DispositionsDigest),
                FinalizationPassObservation::WorkStarted(CompleteReportPass::DispositionsDigest),
                FinalizationPassObservation::Consumed(CompleteReportPass::EvidenceRecords),
                FinalizationPassObservation::WorkStarted(CompleteReportPass::EvidenceRecords),
                FinalizationPassObservation::Consumed(CompleteReportPass::ReportInvariants),
                FinalizationPassObservation::Consumed(CompleteReportPass::FixedOverhead),
                FinalizationPassObservation::WorkStarted(CompleteReportPass::ReportInvariants),
                FinalizationPassObservation::WorkStarted(CompleteReportPass::FixedOverhead),
            ]
        );

        FINALIZATION_PASS_OBSERVATIONS.with(|observations| observations.borrow_mut().clear());
        let mut budget = WorkBudget::new(0, 1);
        let permit = ReportFinalizationPermit::reserve(
            ReportFinalizationPlan {
                control_records: 1,
                ..ReportFinalizationPlan::default()
            },
            &mut budget,
        );
        assert!(permit.is_ok());
        let Ok(mut permit) = permit else { return };
        let work_ran = std::cell::Cell::new(false);
        assert!(
            permit
                .consume_before(
                    [FinalizationReservationUnit::new(
                        CompleteReportPass::ControlRecords,
                        2,
                    )],
                    &mut budget,
                    &crate::NeverCancelled,
                    || work_ran.set(true),
                )
                .is_err()
        );
        assert!(!work_ran.get());
        assert!(
            FINALIZATION_PASS_OBSERVATIONS.with(|observations| observations.borrow().is_empty())
        );
    }

    #[test]
    fn interrupted_finalization_has_exact_zero_n_minus_one_and_n_boundaries() {
        let mut zero_budget = WorkBudget::new(0, 0);
        let zero =
            ReportFinalizationPermit::reserve(ReportFinalizationPlan::default(), &mut zero_budget);
        assert!(zero.is_ok());
        let Ok(mut zero) = zero else { return };
        assert!(zero.forfeit_all_remaining().is_ok());
        consume_fixed_fallback(&mut zero.fallback);
        assert!(zero.finish_interrupted().is_ok());

        let plan = ReportFinalizationPlan {
            control_records: 2,
            report_invariants: 1,
            ..ReportFinalizationPlan::default()
        };
        let mut n_minus_one = WorkBudget::new(0, 2);
        assert!(ReportFinalizationPermit::reserve(plan, &mut n_minus_one).is_err());
        let mut exact_n = WorkBudget::new(0, 3);
        let exact = ReportFinalizationPermit::reserve(plan, &mut exact_n);
        assert!(exact.is_ok());
        let Ok(mut exact) = exact else { return };
        assert!(
            exact
                .consume_pass(FinalizationReservationUnit::new(
                    CompleteReportPass::ControlRecords,
                    2,
                ))
                .is_ok()
        );
        assert!(exact.forfeit_all_remaining().is_ok());
        consume_fixed_fallback(&mut exact.fallback);
        assert!(exact.finish_interrupted().is_ok());
        assert_eq!(exact.ledger.control_records.consumed, 2);
        assert_eq!(exact.ledger.report_invariants.forfeited, 1);
    }

    #[test]
    fn complete_and_fallback_boundaries_preserve_exact_typed_stops() {
        let plan = ReportFinalizationPlan {
            control_records: 1,
            semantic_change_records: 1,
            change_carrier_events: 1,
            other_events: 1,
            checkpoint_records: 1,
            change_classifications: 1,
            history_digest: 1,
            dispositions_digest: 1,
            evidence_records: 1,
            report_invariants: 1,
            fixed_overhead: 1,
        };
        let reservations = plan.reservations();
        let total = plan.total();
        assert_eq!(total, Some(11));

        let mut n_minus_one = WorkBudget::new(0, 10);
        assert!(ReportFinalizationPermit::reserve(plan, &mut n_minus_one).is_err());
        assert_eq!(n_minus_one.remaining(), (0, 10));
        let mut exact_n = WorkBudget::new(0, 11);
        assert!(ReportFinalizationPermit::reserve(plan, &mut exact_n).is_ok());
        let mut n_plus_one = WorkBudget::new(0, 12);
        let permit = ReportFinalizationPermit::reserve(plan, &mut n_plus_one);
        assert!(permit.is_ok());
        assert_eq!(n_plus_one.remaining(), (0, 1));

        let coordinate = DocumentCoordinate::new(
            ControllerPublicKey::from_bytes([0xa1; 32]),
            DocumentId::from_bytes([0xa2; 32]),
        );
        for stop_index in 0..=reservations.len() {
            let mut budget = WorkBudget::new(0, 11);
            let permit = ReportFinalizationPermit::reserve(plan, &mut budget);
            assert!(permit.is_ok());
            let Ok(mut permit) = permit else { return };
            let cancellation_calls = std::cell::Cell::new(0_usize);
            let work_runs = std::cell::Cell::new(0_usize);
            let cancellation = || {
                let call = cancellation_calls.get();
                cancellation_calls.set(call + 1);
                call == stop_index
            };
            for (index, reservation) in reservations.iter().enumerate() {
                let result =
                    permit.consume_before([*reservation], &mut budget, &cancellation, || {
                        work_runs.set(work_runs.get() + 1)
                    });
                if index == stop_index {
                    assert_eq!(
                        result,
                        Err(FinalizationBoundaryError::Stopped(Completion::Cancelled))
                    );
                    break;
                }
                assert!(result.is_ok());
            }
            assert_eq!(work_runs.get(), stop_index);
            if stop_index < reservations.len() {
                let report = permit.build_interrupted_report(
                    crate::ProtocolRevision::draft_v1(),
                    coordinate,
                    Completion::Cancelled,
                );
                assert!(report.is_ok());
                let Ok(report) = report else { return };
                assert_eq!(report.completion(), Completion::Cancelled);
                assert_eq!(permit.state, FinalizationPermitState::Interrupted);
                assert_eq!(budget.remaining(), (0, 0));
            } else {
                assert!(permit.refund(&mut budget).is_ok());
                assert_eq!(permit.state, FinalizationPermitState::Complete);
            }
        }

        for completion in [Completion::BudgetExhausted, Completion::Cancelled] {
            let mut fallback = FixedFallbackLedger::new();
            let report =
                fallback.build_report(crate::ProtocolRevision::draft_v1(), coordinate, completion);
            assert!(report.is_ok());
            let Ok(report) = report else { return };
            assert_eq!(report.completion(), completion);
            assert!(fallback.is_consumed_settlement());
            assert_eq!(fallback.digests.consumed, FixedFallbackLedger::DIGEST_UNITS);
            assert_eq!(
                fallback.fixed_overhead.consumed,
                FixedFallbackLedger::FIXED_OVERHEAD_UNITS
            );
            assert_eq!(fallback.invariants.consumed, REPORT_INVARIANT_ITEMS);
        }
    }

    #[test]
    fn every_interrupted_prefix_uses_only_fallback_and_never_refunds() {
        let plan = ReportFinalizationPlan {
            control_records: 1,
            semantic_change_records: 1,
            change_carrier_events: 1,
            other_events: 1,
            checkpoint_records: 1,
            change_classifications: 1,
            history_digest: 1,
            dispositions_digest: 1,
            evidence_records: 1,
            report_invariants: 1,
            fixed_overhead: 1,
        };
        let reservations = plan.reservations();
        let coordinate = DocumentCoordinate::new(
            ControllerPublicKey::from_bytes([0x81; 32]),
            DocumentId::from_bytes([0x82; 32]),
        );
        for completion in [Completion::BudgetExhausted, Completion::Cancelled] {
            for consumed_prefix in 0..=reservations.len() {
                let mut budget = WorkBudget::new(0, 11);
                let permit = ReportFinalizationPermit::reserve(plan, &mut budget);
                assert!(permit.is_ok());
                let Ok(mut permit) = permit else { return };
                for reservation in &reservations[..consumed_prefix] {
                    assert!(permit.consume_pass(*reservation).is_ok());
                }
                FINALIZATION_PASS_OBSERVATIONS
                    .with(|observations| observations.borrow_mut().clear());

                let report = permit.build_interrupted_report(
                    crate::ProtocolRevision::draft_v1(),
                    coordinate,
                    completion,
                );
                assert!(report.is_ok());
                let Ok(report) = report else { return };
                assert_eq!(report.completion(), completion);
                assert!(report.canonical_controls().is_empty());
                assert!(report.disposition_records().is_empty());
                assert!(report.evidence().is_empty());
                assert!(report.checkpoints().is_empty());
                assert!(report.integrity_alerts().is_empty());
                assert!(report.document().is_none());
                assert_eq!(budget.remaining(), (0, 0));
                assert_eq!(budget.consumed().get(WorkCounter::Assertion), 11);
                assert!(
                    permit
                        .ledger
                        .settlements()
                        .into_iter()
                        .all(|settlement| settlement.refunded == 0)
                );
                for (index, settlement) in permit.ledger.settlements().into_iter().enumerate() {
                    assert_eq!(settlement.consumed, u64::from(index < consumed_prefix));
                    assert_eq!(settlement.forfeited, u64::from(index >= consumed_prefix));
                }
                assert!(permit.fallback.is_consumed_settlement());
                assert_eq!(permit.state, FinalizationPermitState::Interrupted);
                assert!(
                    FINALIZATION_PASS_OBSERVATIONS
                        .with(|observations| observations.borrow().is_empty())
                );
            }
        }
    }

    #[test]
    fn finalization_dimensions_reject_underflow_and_double_finish() {
        let plan = ReportFinalizationPlan {
            control_records: 2,
            report_invariants: 1,
            ..ReportFinalizationPlan::default()
        };
        let mut budget = WorkBudget::new(0, 3);
        let permit = ReportFinalizationPermit::reserve(plan, &mut budget);
        assert!(permit.is_ok());
        let Ok(mut permit) = permit else { return };
        assert!(
            permit
                .consume_pass(FinalizationReservationUnit::new(
                    CompleteReportPass::ControlRecords,
                    2,
                ))
                .is_ok()
        );
        assert!(
            permit
                .consume_pass(FinalizationReservationUnit::new(
                    CompleteReportPass::ControlRecords,
                    1,
                ))
                .is_err()
        );
        assert!(permit.finish_interrupted().is_err());
        for dimension in [
            FinalizationDimension::ControlRecords,
            FinalizationDimension::SemanticChangeRecords,
            FinalizationDimension::ChangeCarrierEvents,
            FinalizationDimension::OtherEvents,
            FinalizationDimension::CheckpointRecords,
            FinalizationDimension::ChangeClassifications,
            FinalizationDimension::HistoryDigest,
            FinalizationDimension::DispositionsDigest,
            FinalizationDimension::EvidenceRecords,
            FinalizationDimension::ReportInvariants,
            FinalizationDimension::FixedOverhead,
        ] {
            assert!(permit.forfeit(dimension).is_ok());
        }
        assert!(
            permit
                .forfeit(FinalizationDimension::ControlRecords)
                .is_err()
        );
        assert!(
            permit
                .consume_pass(FinalizationReservationUnit::new(
                    CompleteReportPass::SemanticChangeRecords,
                    0,
                ))
                .is_err()
        );
        assert_eq!(permit.ledger.control_records.consumed, 2);
        assert_eq!(permit.ledger.control_records.forfeited, 0);
        assert_eq!(permit.ledger.report_invariants.consumed, 0);
        assert_eq!(permit.ledger.report_invariants.forfeited, 1);
        assert!(permit.ledger.is_interrupted_settlement());
        consume_fixed_fallback(&mut permit.fallback);
        assert!(permit.finish_interrupted().is_ok());
        assert!(permit.finish_interrupted().is_err());
        assert!(
            permit
                .consume_pass(FinalizationReservationUnit::new(
                    CompleteReportPass::ReportInvariants,
                    1,
                ))
                .is_err()
        );
    }

    #[test]
    fn fixed_fallback_is_independent_of_caller_target_capacity() {
        let plan = ReportFinalizationPlan {
            control_records: 1,
            ..ReportFinalizationPlan::default()
        };
        let mut zero_budget = WorkBudget::new(0, 0);
        assert!(ReportFinalizationPermit::reserve(plan, &mut zero_budget).is_err());
        assert_eq!(zero_budget.remaining(), (0, 0));
        assert_eq!(zero_budget.consumed().get(WorkCounter::Assertion), 0);

        let report = super::fixed_fallback_report(
            crate::ProtocolRevision::draft_v1(),
            DocumentCoordinate::new(
                ControllerPublicKey::from_bytes([0x71; 32]),
                DocumentId::from_bytes([0x72; 32]),
            ),
            crate::Completion::BudgetExhausted,
        );
        assert!(report.is_ok());
        let Ok(report) = report else { return };
        assert_eq!(report.completion(), crate::Completion::BudgetExhausted);
        assert_eq!(zero_budget.remaining(), (0, 0));

        let corpus = crate::CorpusBuilder::new().finish();
        let mut entry_budget = WorkBudget::new(0, 0);
        let entry_report = ReferenceEvaluator::new(crate::ProtocolRevision::draft_v1()).evaluate(
            &corpus,
            report.coordinate(),
            &mut entry_budget,
            &crate::NeverCancelled,
        );
        assert!(entry_report.is_ok());
        let Ok(entry_report) = entry_report else {
            return;
        };
        assert_eq!(
            entry_report.completion(),
            crate::Completion::BudgetExhausted
        );
        assert_eq!(entry_budget.remaining(), (0, 0));
        assert_eq!(entry_budget.consumed().get(WorkCounter::Assertion), 0);

        let mut complete_budget = WorkBudget::new(0, 1);
        let permit = ReportFinalizationPermit::reserve(plan, &mut complete_budget);
        assert!(permit.is_ok());
        let Ok(mut permit) = permit else { return };
        assert_eq!(permit.ledger.control_records.remaining(), Some(1));
        assert_eq!(
            permit.fallback.digests.remaining(),
            Some(FixedFallbackLedger::DIGEST_UNITS)
        );
        assert!(
            permit
                .consume_pass(FinalizationReservationUnit::new(
                    CompleteReportPass::ControlRecords,
                    1,
                ))
                .is_ok()
        );
        assert_eq!(permit.ledger.control_records.remaining(), Some(0));
        assert_eq!(
            permit.fallback.digests.remaining(),
            Some(FixedFallbackLedger::DIGEST_UNITS),
            "complete-tier consumption must not borrow fixed fallback capacity"
        );
    }

    #[test]
    fn finding_076_finalization_rejects_reordered_named_passes() {
        let plan = ReportFinalizationPlan {
            control_records: 1,
            fixed_overhead: 1,
            ..ReportFinalizationPlan::default()
        };
        let mut budget = WorkBudget::new(0, 2);
        let permit = ReportFinalizationPermit::reserve(plan, &mut budget);
        assert!(permit.is_ok());
        let Ok(mut permit) = permit else { return };
        assert_eq!(
            permit.consume_pass(FinalizationReservationUnit::new(
                CompleteReportPass::FixedOverhead,
                1,
            )),
            Err(FinalizationPermitError),
            "FINDING_076 reproduced: finalization accepts a named pass out of order"
        );
    }

    #[test]
    fn finalization_order_and_single_settlement_reject_every_mutation() {
        let plan = ReportFinalizationPlan {
            control_records: 1,
            semantic_change_records: 1,
            change_carrier_events: 1,
            other_events: 1,
            checkpoint_records: 1,
            change_classifications: 1,
            history_digest: 1,
            dispositions_digest: 1,
            evidence_records: 1,
            report_invariants: 1,
            fixed_overhead: 1,
        };
        let mut budget = WorkBudget::new(0, 11);
        let permit = ReportFinalizationPermit::reserve(plan, &mut budget);
        assert!(permit.is_ok());
        let Ok(mut permit) = permit else { return };

        assert_eq!(
            permit.consume_pass(FinalizationReservationUnit::new(
                CompleteReportPass::SemanticChangeRecords,
                1,
            )),
            Err(FinalizationPermitError),
            "out-of-order passes must not advance the ledger"
        );
        assert_eq!(permit.next_complete_pass, 0);
        assert_eq!(permit.refund(&mut budget), Err(FinalizationPermitError));
        assert_eq!(budget.remaining(), (0, 0));

        assert!(permit.consume_pass(plan.reservations()[0]).is_ok());
        assert_eq!(
            permit.consume_pass(plan.reservations()[0]),
            Err(FinalizationPermitError),
            "a named pass cannot settle twice"
        );
        assert_eq!(permit.next_complete_pass, 1);
        assert_eq!(
            permit.consume_pass(FinalizationReservationUnit::new(
                CompleteReportPass::SemanticChangeRecords,
                2,
            )),
            Err(FinalizationPermitError),
            "a pass cannot borrow capacity from another reservation"
        );
        assert_eq!(permit.next_complete_pass, 1);
        for reservation in &plan.reservations()[1..] {
            assert!(permit.consume_pass(*reservation).is_ok());
        }
        assert!(permit.refund(&mut budget).is_ok());
        assert_eq!(permit.refund(&mut budget), Err(FinalizationPermitError));
        assert_eq!(permit.finish_failed(), Err(FinalizationPermitError));

        let mut interrupted_budget = WorkBudget::new(0, 11);
        let interrupted = ReportFinalizationPermit::reserve(plan, &mut interrupted_budget);
        assert!(interrupted.is_ok());
        let Ok(mut interrupted) = interrupted else {
            return;
        };
        assert!(interrupted.consume_pass(plan.reservations()[0]).is_ok());
        assert!(
            interrupted
                .forfeit(FinalizationDimension::SemanticChangeRecords)
                .is_ok()
        );
        assert_eq!(
            interrupted.consume_pass(plan.reservations()[1]),
            Err(FinalizationPermitError),
            "forfeited capacity cannot be consumed"
        );

        let mut fallback = FixedFallbackLedger::new();
        assert_eq!(
            fallback.consume(
                FixedFallbackPass::FixedOverhead,
                FixedFallbackLedger::FIXED_OVERHEAD_UNITS,
            ),
            Err(FinalizationPermitError)
        );
        assert_eq!(fallback.next_pass, 0);
        assert_eq!(
            fallback.consume(
                FixedFallbackPass::Digests,
                FixedFallbackLedger::DIGEST_UNITS - 1,
            ),
            Err(FinalizationPermitError)
        );
        assert_eq!(
            fallback.consume(
                FixedFallbackPass::Digests,
                FixedFallbackLedger::DIGEST_UNITS + 1,
            ),
            Err(FinalizationPermitError)
        );
        assert!(
            fallback
                .consume(
                    FixedFallbackPass::Digests,
                    FixedFallbackLedger::DIGEST_UNITS,
                )
                .is_ok()
        );
        assert_eq!(
            fallback.consume(
                FixedFallbackPass::Digests,
                FixedFallbackLedger::DIGEST_UNITS,
            ),
            Err(FinalizationPermitError)
        );
        assert_eq!(fallback.close_consumed(), Err(FinalizationPermitError));
        assert!(
            fallback
                .consume(
                    FixedFallbackPass::FixedOverhead,
                    FixedFallbackLedger::FIXED_OVERHEAD_UNITS,
                )
                .is_ok()
        );
        assert!(
            fallback
                .consume(FixedFallbackPass::Invariants, REPORT_INVARIANT_ITEMS)
                .is_ok()
        );
        assert!(fallback.close_consumed().is_ok());
        assert_eq!(fallback.close_consumed(), Err(FinalizationPermitError));
        assert_eq!(fallback.forfeit_all(), Err(FinalizationPermitError));
    }

    #[test]
    fn reserved_no_progress_wrapper_consumes_without_optional_expansion() {
        let source = include_str!("reference_evaluator.rs");
        let wrapper = source
            .split_once("fn reserved_interrupted_report(")
            .and_then(|(_, rest)| rest.split_once("fn fixed_fallback_report("))
            .map(|(body, _)| body)
            .unwrap_or_default();
        assert!(wrapper.contains("permit.build_interrupted_report("));
        assert!(!wrapper.contains("view."));
        let permit_method = source
            .split_once("fn build_interrupted_report(")
            .and_then(|(_, rest)| rest.split_once("fn forfeit("))
            .map(|(body, _)| body)
            .unwrap_or_default();
        assert!(permit_method.contains("self.forfeit_all_remaining()"));
        assert!(permit_method.contains(".fallback") && permit_method.contains(".build_report("));
        assert!(permit_method.contains("self.finish_interrupted()"));
        assert!(!permit_method.contains("view."));
        let obsolete_reserved = ["reserved_", "batch_report"].concat();
        let obsolete_preparation = ["prepare_", "interrupted_batch_report"].concat();
        let obsolete_construction = ["NoProgress", "ConstructionPath"].concat();
        assert!(!source.contains(&obsolete_reserved));
        assert!(!source.contains(&obsolete_preparation));
        assert!(!source.contains(&obsolete_construction));
    }

    #[test]
    fn report_validation_precedes_finalization_refund() {
        let coordinate = DocumentCoordinate::new(
            ControllerPublicKey::from_bytes([0x91; 32]),
            DocumentId::from_bytes([0x92; 32]),
        );
        let incomplete = super::fixed_fallback_report(
            crate::ProtocolRevision::draft_v1(),
            coordinate,
            Completion::BudgetExhausted,
        );
        assert!(incomplete.is_ok());
        let Ok(incomplete) = incomplete else { return };
        for candidate in [Err(EvaluationError::Projection), Ok(incomplete)] {
            let plan = ReportFinalizationPlan {
                control_records: 1,
                ..ReportFinalizationPlan::default()
            };
            let mut budget = WorkBudget::new(0, 1);
            let permit = ReportFinalizationPermit::reserve(plan, &mut budget);
            assert!(permit.is_ok());
            let Ok(mut permit) = permit else { return };

            let result = permit.finish_complete_report(&mut budget, candidate);
            assert!(result.is_err());
            assert_eq!(budget.remaining(), (0, 0));
            assert_eq!(budget.consumed().get(WorkCounter::Assertion), 1);
            assert_eq!(permit.state, FinalizationPermitState::Failed);
            assert!(
                permit
                    .ledger
                    .settlements()
                    .into_iter()
                    .all(|settlement| settlement.refunded == 0)
            );
            assert!(permit.fallback.is_forfeited_settlement());
        }
    }

    #[test]
    fn every_post_reservation_error_settles_the_permit() {
        let source = include_str!("reference_evaluator.rs");
        let evaluation = source
            .split_once("let mut finalization =")
            .and_then(|(_, rest)| rest.split_once("/// Replays the complete retained corpus"))
            .map(|(body, _)| body)
            .unwrap_or_default();
        assert!(!evaluation.contains("return Err(EvaluationError::"));
        assert!(!evaluation.contains("map_err(|_| EvaluationError::"));
        assert!(evaluation.contains("settle_reserved_error(&mut finalization"));
        assert!(evaluation.contains("reserved_interrupted_report("));
        let obsolete_reserved = ["reserved_", "batch_report("].concat();
        assert!(!evaluation.contains(&obsolete_reserved));
    }
}
