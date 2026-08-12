use std::collections::{BTreeMap, BTreeSet};

use crate::automerge_adapter::document::{AppliedDocument, materialize_history};
use crate::automerge_adapter::materialized_view::MaterializedDocumentView;
use crate::control::candidate::{CandidateResult, evaluate_child};
use crate::control::candidate_outcome::ControlCandidateOutcome;
use crate::control::epoch_state::AcceptedEpochState;
use crate::control::frontier::accepted_frontier_closure;
use crate::control::parent_view::ParentEpochView;
use crate::control::select::select_valid_outcomes_with_alert;
use crate::control::validate::ControlEnvelope;
use crate::graph::change_candidate::ChangeCandidate;
use crate::graph::dependency_graph::build_graph;
use crate::graph::equivocation::QuarantineError;
use crate::graph::equivocation::quarantine_equivocation_descendants;
use crate::graph::schedule::{ScheduleError, schedule_candidates};
use crate::reference::epoch::{EpochCandidate, resolve_epoch};
use crate::reference::epoch_engine::{
    AcceptedAtControl, EpochEvaluationError, EpochEvaluationInput, EpochEvaluationResult,
    PriorChangeKnowledge, evaluate_epoch,
};
use crate::{
    CancellationCheck, ChangeHash, Completion, EvaluationFailure, EventId, IntegrityAlert,
    ProtocolDisposition, WorkBudget, WorkCounter,
};

#[derive(Clone, Debug)]
pub(crate) struct BatchChange {
    pub(crate) candidate: ChangeCandidate,
    /// Eligibility hint used only by the envelope-free unit-test adapter.
    /// Stateful public evaluation derives every semantic outcome independently.
    pub(crate) legacy_eligible: bool,
    pub(crate) raw_change: Option<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub(crate) struct BatchControl {
    pub(crate) event_id: EventId,
    pub(crate) parent: Option<EventId>,
    pub(crate) accepted_base: BTreeSet<ChangeHash>,
    pub(crate) frozen: bool,
    pub(crate) changes: Vec<BatchChange>,
    pub(crate) envelope: Option<ControlEnvelope>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BatchEvaluationReport {
    pub(crate) canonical_controls: Vec<EventId>,
    pub(crate) control_dispositions: BTreeMap<EventId, ProtocolDisposition>,
    pub(crate) accepted_at_control: BTreeMap<EventId, AcceptedAtControl>,
    pub(crate) statefully_valid_controls: BTreeSet<EventId>,
    pub(crate) dispositions: BTreeMap<ChangeHash, ProtocolDisposition>,
    pub(crate) accepted_changes: BTreeSet<ChangeHash>,
    pub(crate) heads: BTreeSet<ChangeHash>,
    pub(crate) materialized_document: Option<Vec<u8>>,
    pub(crate) integrity_alerts: Vec<IntegrityAlert>,
    pub(crate) completion: Completion,
    pub(crate) failure: Option<EvaluationFailure>,
}

#[derive(Clone, Debug, Default)]
struct PreservedBatchProgress {
    canonical_controls: Vec<EventId>,
    control_dispositions: BTreeMap<EventId, ProtocolDisposition>,
    accepted_at_control: BTreeMap<EventId, AcceptedAtControl>,
    statefully_valid_controls: BTreeSet<EventId>,
    dispositions: BTreeMap<ChangeHash, ProtocolDisposition>,
    accepted_changes: BTreeSet<ChangeHash>,
    integrity_alerts: Vec<IntegrityAlert>,
}

pub(crate) fn evaluate_batch(
    controls: impl IntoIterator<Item = BatchControl>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> BatchEvaluationReport {
    if cancellation.is_cancelled() {
        return incomplete_report(PreservedBatchProgress::default(), Completion::Cancelled);
    }
    let mut collected = Vec::new();
    for control in controls {
        if cancellation.is_cancelled() {
            return incomplete_report(PreservedBatchProgress::default(), Completion::Cancelled);
        }
        if budget.charge(WorkCounter::Control, 1).is_err() {
            return incomplete_report(
                PreservedBatchProgress::default(),
                Completion::BudgetExhausted,
            );
        }
        collected.push(control);
    }
    let controls = collected
        .into_iter()
        .map(|control| (control.event_id, control))
        .collect::<BTreeMap<_, _>>();
    let mut by_parent = BTreeMap::<Option<EventId>, BTreeSet<EventId>>::new();
    for control in controls.values() {
        by_parent
            .entry(control.parent)
            .or_default()
            .insert(control.event_id);
    }
    let mut control_dispositions = controls
        .keys()
        .copied()
        .map(|event_id| (event_id, ProtocolDisposition::Excluded))
        .collect::<BTreeMap<_, _>>();
    let mut dispositions = controls
        .values()
        .flat_map(|control| control.changes.iter())
        .map(|change| (change.candidate.change_hash, ProtocolDisposition::Excluded))
        .collect::<BTreeMap<_, _>>();
    let mut canonical_controls = Vec::new();
    let mut integrity_alerts = Vec::new();
    let mut completion = Completion::Complete;
    let mut failure = None;
    let mut accepted_changes = BTreeSet::new();
    let mut accepted_at_control = BTreeMap::new();
    let mut statefully_valid_controls = BTreeSet::new();
    let mut parent_epoch_result: Option<EpochEvaluationResult> = None;
    let mut parent_id = None;
    let mut canonical_ancestry = Vec::<ControlEnvelope>::new();
    while let Some(children) = by_parent.get(&parent_id) {
        if cancellation.is_cancelled() {
            completion = Completion::Cancelled;
            break;
        }
        if charge_control_transitions(children.len(), budget).is_err() {
            completion = Completion::BudgetExhausted;
            break;
        }
        let parent_view = parent_epoch_result
            .as_ref()
            .map(|result| ParentEpochView::from_accepted_state(result.accepted_state()));
        if let Some(view) = parent_view.as_ref()
            && let Err(interruption) = charge_control_closures(
                children,
                &controls,
                &canonical_ancestry,
                view,
                budget,
                cancellation,
            )
        {
            completion = interruption;
            break;
        }
        let ancestry = canonical_ancestry
            .iter()
            .map(ControlEnvelope::content)
            .collect::<Vec<_>>();
        let parent_envelope = parent_id.and_then(|event_id| {
            controls
                .get(&event_id)
                .and_then(|control| control.envelope.as_ref())
        });
        let outcomes = children
            .iter()
            .filter_map(|event_id| {
                let control = controls.get(event_id)?;
                let sequence = control
                    .envelope
                    .as_ref()
                    .map_or(0, ControlEnvelope::sequence);
                let Some(child) = control.envelope.as_ref() else {
                    return Some(ControlCandidateOutcome::valid(
                        *event_id,
                        control.parent,
                        sequence,
                        control.accepted_base.clone(),
                    ));
                };
                let Some(parent) = parent_envelope else {
                    return Some(ControlCandidateOutcome::valid(
                        *event_id,
                        None,
                        sequence,
                        BTreeSet::new(),
                    ));
                };
                let Some(view) = parent_view.as_ref() else {
                    return Some(ControlCandidateOutcome::invalid(
                        *event_id,
                        control.parent,
                        sequence,
                        crate::DiagnosticCode::registered("control.state"),
                        None,
                    ));
                };
                Some(match evaluate_child(parent, child, &ancestry, view) {
                    CandidateResult::Valid => {
                        let closure = accepted_frontier_closure(
                            child.base_heads(),
                            view.accepted(),
                            view.dependency_index(),
                        );
                        ControlCandidateOutcome::valid(
                            *event_id,
                            control.parent,
                            sequence,
                            closure.accepted,
                        )
                    }
                    CandidateResult::Pending(diagnostic) => ControlCandidateOutcome::pending(
                        *event_id,
                        control.parent,
                        sequence,
                        diagnostic,
                        None,
                    ),
                    CandidateResult::Invalid(diagnostic) => ControlCandidateOutcome::invalid(
                        *event_id,
                        control.parent,
                        sequence,
                        diagnostic,
                        None,
                    ),
                })
            })
            .collect::<Vec<_>>();
        statefully_valid_controls.extend(outcomes.iter().filter_map(|outcome| {
            (outcome.disposition() == ProtocolDisposition::Accepted).then_some(outcome.event_id())
        }));
        for outcome in &outcomes {
            if outcome.disposition() != ProtocolDisposition::Accepted {
                control_dispositions.insert(outcome.event_id(), outcome.disposition());
            }
        }
        let (selection, alert) =
            select_valid_outcomes_with_alert(parent_id, outcomes.iter().cloned());
        let Some(selected) = selection.selected else {
            break;
        };
        let selected_base = outcomes
            .iter()
            .find(|outcome| outcome.event_id() == selected)
            .and_then(ControlCandidateOutcome::validated_base_closure)
            .cloned()
            .unwrap_or_default();
        let Some(control) = controls.get(&selected) else {
            failure = Some(EvaluationFailure::InvariantViolation);
            break;
        };
        let Some(selected_state) =
            accepted_state_for_closure(&selected_base, &controls, parent_epoch_result.as_ref())
        else {
            failure = Some(EvaluationFailure::InvariantViolation);
            break;
        };
        canonical_controls.push(selected);
        control_dispositions.insert(selected, ProtocolDisposition::Accepted);
        if let Some(alert) = alert {
            integrity_alerts.push(alert);
        }
        for change in &control.changes {
            if !selected_base.contains(&change.candidate.change_hash) {
                dispositions.remove(&change.candidate.change_hash);
            }
        }
        if budget.charge(WorkCounter::Control, 1).is_err() {
            completion = Completion::BudgetExhausted;
            break;
        }
        for hash in accepted_changes.difference(&selected_base) {
            dispositions.insert(*hash, ProtocolDisposition::Excluded);
        }
        let prior_change_knowledge =
            prior_change_knowledge(parent_epoch_result.as_ref(), &selected_base, &dispositions);
        accepted_changes = selected_base;
        let epoch = resolve_authoritative_epoch(
            control,
            selected_state,
            prior_change_knowledge,
            &canonical_ancestry,
            &controls,
            budget,
            cancellation,
        );
        let resolved = match epoch {
            Ok(resolved) => resolved,
            Err(EpochResolutionError::Schedule(ScheduleError::BudgetExhausted)) => {
                completion = Completion::BudgetExhausted;
                break;
            }
            Err(EpochResolutionError::Schedule(ScheduleError::Cancelled)) => {
                completion = Completion::Cancelled;
                break;
            }
            Err(EpochResolutionError::InvalidState) => {
                failure = Some(EvaluationFailure::Graph);
                break;
            }
        };
        dispositions.extend(resolved.dispositions().clone());
        integrity_alerts.extend_from_slice(resolved.integrity_alerts());
        accepted_changes = resolved.accepted_state().accepted_closure().clone();
        accepted_at_control.insert(selected, AcceptedAtControl::from_result(&resolved));
        parent_epoch_result = Some(resolved);
        if let Some(envelope) = control.envelope.as_ref() {
            canonical_ancestry.push(envelope.clone());
        }
        if control.frozen {
            break;
        }
        parent_id = Some(selected);
    }

    if completion != Completion::Complete || failure.is_some() {
        let heads = derive_heads(&accepted_changes, &controls);
        let report = incomplete_report(
            PreservedBatchProgress {
                canonical_controls,
                control_dispositions,
                accepted_at_control,
                statefully_valid_controls,
                dispositions,
                accepted_changes,
                integrity_alerts,
            },
            completion,
        )
        .with_heads(heads);
        return match failure {
            Some(failure) => report.with_failure(failure),
            None => report,
        };
    }

    let candidates = controls
        .values()
        .flat_map(|control| control.changes.iter())
        .filter(|change| accepted_changes.contains(&change.candidate.change_hash))
        .map(|change| change.candidate.clone())
        .collect::<Vec<_>>();
    let ordered = match schedule_candidates(candidates, BTreeSet::new(), budget, cancellation) {
        Ok(schedule) => schedule.ordered,
        Err(error) => {
            let completion = match error {
                ScheduleError::BudgetExhausted => Completion::BudgetExhausted,
                ScheduleError::Cancelled => Completion::Cancelled,
            };
            let heads = derive_heads(&accepted_changes, &controls);
            return incomplete_report(
                PreservedBatchProgress {
                    canonical_controls,
                    control_dispositions,
                    accepted_at_control,
                    statefully_valid_controls,
                    dispositions,
                    accepted_changes,
                    integrity_alerts,
                },
                completion,
            )
            .with_heads(heads);
        }
    };
    let raw_changes = controls
        .values()
        .flat_map(|control| control.changes.iter())
        .filter_map(|change| {
            accepted_changes
                .contains(&change.candidate.change_hash)
                .then(|| {
                    change
                        .raw_change
                        .clone()
                        .map(|raw| (change.candidate.change_hash, raw))
                })
                .flatten()
        })
        .collect::<BTreeMap<_, _>>();
    let can_materialize = raw_changes.len() == accepted_changes.len();
    if can_materialize
        && let Err(completion) = charge_application_work(&ordered, budget, cancellation)
    {
        let heads = derive_heads(&accepted_changes, &controls);
        return incomplete_report(
            PreservedBatchProgress {
                canonical_controls,
                control_dispositions,
                accepted_at_control,
                statefully_valid_controls,
                dispositions,
                accepted_changes,
                integrity_alerts,
            },
            completion,
        )
        .with_heads(heads);
    }
    let materialized = if can_materialize {
        match materialize_history(&raw_changes, &ordered) {
            Ok(document) => Some(document),
            Err(_) => {
                let heads = derive_heads(&accepted_changes, &controls);
                return incomplete_report(
                    PreservedBatchProgress {
                        canonical_controls,
                        control_dispositions,
                        accepted_at_control,
                        statefully_valid_controls,
                        dispositions,
                        accepted_changes,
                        integrity_alerts,
                    },
                    Completion::Complete,
                )
                .with_failure(EvaluationFailure::Apply)
                .with_heads(heads);
            }
        }
    } else {
        None
    };
    let derived_heads = derive_heads(&accepted_changes, &controls);
    let (heads, materialized_document) = match materialized {
        Some(AppliedDocument {
            heads,
            canonical_bytes,
        }) if applied_heads_agree(&derived_heads, &heads) => (heads, Some(canonical_bytes)),
        Some(_) => {
            return incomplete_report(
                PreservedBatchProgress {
                    canonical_controls,
                    control_dispositions,
                    accepted_at_control,
                    statefully_valid_controls,
                    dispositions,
                    accepted_changes,
                    integrity_alerts,
                },
                Completion::Complete,
            )
            .with_failure(EvaluationFailure::InvariantViolation)
            .with_heads(derived_heads);
        }
        None => (derived_heads, None),
    };
    BatchEvaluationReport {
        canonical_controls,
        control_dispositions,
        accepted_at_control,
        statefully_valid_controls,
        dispositions,
        accepted_changes,
        heads,
        materialized_document,
        integrity_alerts,
        completion,
        failure: None,
    }
}

fn charge_control_transitions(
    candidate_count: usize,
    budget: &mut WorkBudget,
) -> Result<(), crate::BudgetExhausted> {
    budget.charge(
        WorkCounter::Control,
        u64::try_from(candidate_count).unwrap_or(u64::MAX),
    )
}

fn charge_control_closures(
    children: &BTreeSet<EventId>,
    controls: &BTreeMap<EventId, BatchControl>,
    ancestry: &[ControlEnvelope],
    view: &ParentEpochView,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<(), Completion> {
    if cancellation.is_cancelled() {
        return Err(Completion::Cancelled);
    }
    let comparison_work = children
        .iter()
        .filter_map(|event_id| controls.get(event_id)?.envelope.as_ref())
        .try_fold(0_u64, |total, child| {
            let child_members = u64::try_from(child.content().members.len()).unwrap_or(u64::MAX);
            let parent_members = child
                .parent()
                .and_then(|parent| controls.get(&parent))
                .and_then(|parent| parent.envelope.as_ref())
                .map_or(0, |parent| {
                    u64::try_from(parent.content().members.len()).unwrap_or(u64::MAX)
                });
            let ancestry_members = ancestry
                .iter()
                .try_fold(0_u64, |subtotal, control| {
                    subtotal.checked_add(
                        u64::try_from(control.content().members.len()).unwrap_or(u64::MAX),
                    )
                })
                .unwrap_or(u64::MAX);
            total.checked_add(
                child_members
                    .saturating_mul(parent_members.saturating_mul(2))
                    .saturating_add(ancestry_members)
                    .saturating_add(u64::try_from(ancestry.len()).unwrap_or(u64::MAX)),
            )
        })
        .unwrap_or(u64::MAX);
    budget
        .charge(WorkCounter::Control, comparison_work)
        .map_err(|_| Completion::BudgetExhausted)?;
    if cancellation.is_cancelled() {
        return Err(Completion::Cancelled);
    }
    let passes = children
        .iter()
        .filter_map(|event_id| controls.get(event_id)?.envelope.as_ref())
        .try_fold(0_u64, |total, child| {
            total.checked_add(
                u64::try_from(child.content().base_heads.len())
                    .unwrap_or(u64::MAX)
                    .saturating_add(2),
            )
        })
        .unwrap_or(u64::MAX);
    let nodes = u64::try_from(view.accepted().len())
        .unwrap_or(u64::MAX)
        .saturating_add(1)
        .saturating_mul(passes);
    let edges = view
        .dependency_index()
        .values()
        .try_fold(0_u64, |total, dependencies| {
            total.checked_add(u64::try_from(dependencies.len()).unwrap_or(u64::MAX))
        })
        .unwrap_or(u64::MAX)
        .saturating_mul(passes);
    budget
        .charge(WorkCounter::GraphNode, nodes)
        .map_err(|_| Completion::BudgetExhausted)?;
    if cancellation.is_cancelled() {
        return Err(Completion::Cancelled);
    }
    budget
        .charge(WorkCounter::GraphEdge, edges)
        .map_err(|_| Completion::BudgetExhausted)
}

enum EpochResolutionError {
    Schedule(ScheduleError),
    InvalidState,
}

fn resolve_authoritative_epoch(
    control: &BatchControl,
    accepted_base: AcceptedEpochState,
    prior_change_knowledge: BTreeMap<ChangeHash, PriorChangeKnowledge>,
    ancestry: &[ControlEnvelope],
    controls: &BTreeMap<EventId, BatchControl>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<EpochEvaluationResult, EpochResolutionError> {
    let Some(selected) = control.envelope.clone() else {
        let epoch_inputs = control
            .changes
            .iter()
            .filter(|change| {
                !accepted_base
                    .accepted_closure()
                    .contains(&change.candidate.change_hash)
            })
            .map(|change| EpochCandidate {
                candidate: change.candidate.clone(),
                semantically_valid: change.legacy_eligible
                    && change
                        .candidate
                        .dependencies
                        .iter()
                        .all(|dependency| !prior_change_knowledge.contains_key(dependency)),
                canonical_control: !control.frozen,
            });
        let mut dispositions = resolve_epoch(
            epoch_inputs,
            accepted_base.accepted_closure().clone(),
            budget,
            cancellation,
        )
        .map_err(EpochResolutionError::Schedule)?;
        let mut all_candidates = accepted_base.accepted_candidates().clone();
        all_candidates.extend(control.changes.iter().filter_map(|change| {
            (!accepted_base
                .accepted_closure()
                .contains(&change.candidate.change_hash))
            .then_some((change.candidate.change_hash, change.candidate.clone()))
        }));
        let eligible = all_candidates
            .values()
            .filter(|candidate| {
                accepted_base
                    .accepted_closure()
                    .contains(&candidate.change_hash)
                    || dispositions.get(&candidate.change_hash)
                        == Some(&ProtocolDisposition::Accepted)
            })
            .cloned()
            .collect::<Vec<_>>();
        let graph = build_graph(eligible.clone(), accepted_base.accepted_closure().clone())
            .map_err(|_| EpochResolutionError::InvalidState)?;
        let quarantine =
            quarantine_equivocation_descendants(eligible, &graph, budget, cancellation).map_err(
                |error| match error {
                    QuarantineError::BudgetExhausted => {
                        EpochResolutionError::Schedule(ScheduleError::BudgetExhausted)
                    }
                    QuarantineError::Cancelled => {
                        EpochResolutionError::Schedule(ScheduleError::Cancelled)
                    }
                    QuarantineError::Alert(_) => EpochResolutionError::InvalidState,
                },
            )?;
        for hash in &quarantine.quarantined {
            dispositions.insert(*hash, ProtocolDisposition::Excluded);
        }
        let mut accepted_candidates = accepted_base.accepted_candidates().clone();
        accepted_candidates.extend(control.changes.iter().filter_map(|change| {
            if accepted_base
                .accepted_closure()
                .contains(&change.candidate.change_hash)
            {
                return None;
            }
            (dispositions.get(&change.candidate.change_hash)
                == Some(&ProtocolDisposition::Accepted))
            .then_some((change.candidate.change_hash, change.candidate.clone()))
        }));
        let accepted_closure = accepted_candidates.keys().copied().collect::<BTreeSet<_>>();
        let depended_on = accepted_candidates
            .values()
            .flat_map(|candidate| candidate.dependencies.iter().copied())
            .filter(|hash| accepted_closure.contains(hash))
            .collect::<BTreeSet<_>>();
        let frontier_heads = accepted_closure.difference(&depended_on).copied().collect();
        let materialized = (accepted_closure == *accepted_base.accepted_closure())
            .then(|| accepted_base.materialized().cloned())
            .flatten();
        return EpochEvaluationResult::new(
            accepted_closure,
            frontier_heads,
            accepted_candidates,
            dispositions,
            quarantine.alerts,
            materialized,
        )
        .map_err(|_| EpochResolutionError::InvalidState);
    };
    let canonical_ancestry = ancestry.to_vec();
    let raw_changes = controls
        .values()
        .flat_map(|control| control.changes.iter())
        .filter_map(|change| {
            change
                .raw_change
                .clone()
                .map(|raw| (change.candidate.change_hash, raw))
        })
        .collect();
    let epoch_changes = control
        .changes
        .iter()
        .filter(|change| {
            !accepted_base
                .accepted_closure()
                .contains(&change.candidate.change_hash)
        })
        .map(|change| (change.candidate.clone(), change.raw_change.clone()))
        .collect::<Vec<_>>();
    let input = EpochEvaluationInput::new_with_raw_and_prior(
        selected,
        accepted_base,
        epoch_changes,
        raw_changes,
        canonical_ancestry,
        prior_change_knowledge,
    )
    .map_err(|_| EpochResolutionError::InvalidState)?;
    evaluate_epoch(&input, budget, cancellation).map_err(|error| match error {
        EpochEvaluationError::Schedule(error) => EpochResolutionError::Schedule(error),
        EpochEvaluationError::Quarantine(QuarantineError::BudgetExhausted) => {
            EpochResolutionError::Schedule(ScheduleError::BudgetExhausted)
        }
        EpochEvaluationError::Quarantine(QuarantineError::Cancelled) => {
            EpochResolutionError::Schedule(ScheduleError::Cancelled)
        }
        EpochEvaluationError::Quarantine(QuarantineError::Alert(_))
        | EpochEvaluationError::Graph(_)
        | EpochEvaluationError::State(_) => EpochResolutionError::InvalidState,
    })
}

fn prior_change_knowledge(
    parent: Option<&EpochEvaluationResult>,
    selected_base: &BTreeSet<ChangeHash>,
    dispositions: &BTreeMap<ChangeHash, ProtocolDisposition>,
) -> BTreeMap<ChangeHash, PriorChangeKnowledge> {
    let mut knowledge = parent
        .into_iter()
        .flat_map(|result| result.accepted_state().accepted_closure())
        .filter(|hash| !selected_base.contains(hash))
        .map(|hash| (*hash, PriorChangeKnowledge::PrunedCanonicalAncestor))
        .collect::<BTreeMap<_, _>>();
    for (hash, disposition) in dispositions {
        if *disposition == ProtocolDisposition::Invalid && !selected_base.contains(hash) {
            knowledge.insert(*hash, PriorChangeKnowledge::KnownInvalid);
        }
    }
    knowledge
}

fn accepted_state_for_closure(
    accepted: &BTreeSet<ChangeHash>,
    controls: &BTreeMap<EventId, BatchControl>,
    parent: Option<&EpochEvaluationResult>,
) -> Option<AcceptedEpochState> {
    let candidates = controls
        .values()
        .flat_map(|control| control.changes.iter())
        .filter(|change| accepted.contains(&change.candidate.change_hash))
        .map(|change| (change.candidate.change_hash, change.candidate.clone()))
        .collect::<BTreeMap<_, _>>();
    if candidates.len() != accepted.len() {
        return None;
    }
    let depended_on = candidates
        .values()
        .flat_map(|candidate| candidate.dependencies.iter().copied())
        .filter(|hash| accepted.contains(hash))
        .collect::<BTreeSet<_>>();
    let heads = accepted.difference(&depended_on).copied().collect();
    let materialized = parent.and_then(|result| {
        (result.accepted_state().accepted_closure() == accepted)
            .then(|| result.accepted_state().materialized().cloned())
            .flatten()
    });
    let materialized = if accepted.is_empty() && materialized.is_none() {
        MaterializedDocumentView::empty().ok()
    } else {
        materialized
    };
    AcceptedEpochState::new(accepted.clone(), heads, candidates, materialized).ok()
}

fn applied_heads_agree(
    derived_heads: &BTreeSet<ChangeHash>,
    applied_heads: &BTreeSet<ChangeHash>,
) -> bool {
    derived_heads == applied_heads
}

fn charge_application_work(
    ordered: &[ChangeHash],
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<(), Completion> {
    for _ in ordered {
        if cancellation.is_cancelled() {
            return Err(Completion::Cancelled);
        }
        budget
            .charge(WorkCounter::ApplyChange, 1)
            .map_err(|_| Completion::BudgetExhausted)?;
    }
    if cancellation.is_cancelled() {
        return Err(Completion::Cancelled);
    }
    budget
        .charge(WorkCounter::ApplyChange, 1)
        .map_err(|_| Completion::BudgetExhausted)
}

impl BatchEvaluationReport {
    fn with_heads(mut self, heads: BTreeSet<ChangeHash>) -> Self {
        self.heads = heads;
        self
    }

    fn with_failure(mut self, failure: EvaluationFailure) -> Self {
        self.failure = Some(failure);
        self
    }
}

fn incomplete_report(
    progress: PreservedBatchProgress,
    completion: Completion,
) -> BatchEvaluationReport {
    BatchEvaluationReport {
        canonical_controls: progress.canonical_controls,
        control_dispositions: progress.control_dispositions,
        accepted_at_control: progress.accepted_at_control,
        statefully_valid_controls: progress.statefully_valid_controls,
        dispositions: progress.dispositions,
        accepted_changes: progress.accepted_changes,
        heads: BTreeSet::new(),
        materialized_document: None,
        integrity_alerts: progress.integrity_alerts,
        completion,
        failure: match completion {
            Completion::Complete => None,
            Completion::BudgetExhausted => Some(EvaluationFailure::BudgetExhausted),
            Completion::Cancelled => Some(EvaluationFailure::Cancelled),
        },
    }
}

fn derive_heads(
    accepted: &BTreeSet<ChangeHash>,
    controls: &BTreeMap<EventId, BatchControl>,
) -> BTreeSet<ChangeHash> {
    let dependencies = controls
        .values()
        .flat_map(|control| control.changes.iter())
        .filter(|change| accepted.contains(&change.candidate.change_hash))
        .flat_map(|change| change.candidate.dependencies.iter().copied())
        .collect::<BTreeSet<_>>();
    accepted.difference(&dependencies).copied().collect()
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{
        AcceptedAtControl, BatchChange, BatchControl, charge_control_closures, evaluate_batch,
    };
    use crate::automerge_adapter::decode::decode_change;
    use crate::graph::actor_state::tests::candidate;
    use crate::{
        ChangeHash, Completion, EvaluationFailure, EventId, NeverCancelled, ProtocolDisposition,
        ProtocolRevision, WorkBudget, WorkCounter,
    };

    fn control(id: u8, parent: Option<u8>, changes: Vec<BatchChange>) -> BatchControl {
        BatchControl {
            event_id: EventId::from_bytes([id; 32]),
            parent: parent.map(|byte| EventId::from_bytes([byte; 32])),
            accepted_base: BTreeSet::new(),
            frozen: false,
            changes,
            envelope: None,
        }
    }

    fn change(hash: u8, actor: u8, sequence: u64) -> BatchChange {
        let mut value = candidate(actor, sequence, sequence, 1);
        value.change_hash = ChangeHash::from_bytes([hash; 32]);
        BatchChange {
            candidate: value,
            legacy_eligible: true,
            raw_change: None,
        }
    }

    #[test]
    fn control_closure_precharge_exhaustion_is_atomic() {
        let envelope = crate::control::validate::tests::genesis();
        let event_id = envelope.event_id();
        let controls = BTreeMap::from([(
            event_id,
            BatchControl {
                event_id,
                parent: None,
                accepted_base: BTreeSet::new(),
                frozen: false,
                changes: Vec::new(),
                envelope: Some(envelope),
            },
        )]);
        let accepted = ChangeHash::from_bytes([9; 32]);
        let view = crate::control::parent_view::ParentEpochView::from_parts_for_test(
            BTreeSet::from([accepted]),
            BTreeSet::from([accepted]),
            BTreeMap::from([(accepted, BTreeSet::new())]),
            BTreeMap::new(),
            BTreeMap::new(),
        );
        let mut budget = WorkBudget::new(0, 3);
        assert_eq!(
            charge_control_closures(
                &BTreeSet::from([event_id]),
                &controls,
                &[],
                &view,
                &mut budget,
                &NeverCancelled
            ),
            Err(Completion::BudgetExhausted)
        );
        assert_eq!(budget.consumed().get(WorkCounter::GraphNode), 0);
    }

    #[test]
    fn implement_full_batch_reference_evaluator() {
        let raw = include_str!("../../../../fixtures/v1_draft/automerge_changes/basic/change.hex")
            .trim()
            .as_bytes()
            .chunks_exact(2)
            .filter_map(|pair| {
                core::str::from_utf8(pair)
                    .ok()
                    .and_then(|text| u8::from_str_radix(text, 16).ok())
            })
            .collect::<Vec<_>>();
        let decoded = decode_change(&raw, ProtocolRevision::draft_v1());
        assert!(decoded.is_ok());
        let Ok(decoded) = decoded else { return };
        let mut basic = change(1, 1, 1);
        basic.candidate.change_hash = ChangeHash::from_bytes(*decoded.hash.as_bytes());
        basic.candidate.actor = crate::ActorId::from_bytes(*decoded.actor.as_bytes());
        basic.candidate.sequence = decoded.sequence;
        basic.candidate.start_op = decoded.start_op;
        basic.candidate.operation_count = u64::try_from(decoded.operations.len()).unwrap_or(0);
        basic.raw_change = Some(raw);
        let mut basic_budget = WorkBudget::new(0, 200);
        let basic_report = evaluate_batch(
            [control(1, None, vec![basic.clone()])],
            &mut basic_budget,
            &NeverCancelled,
        );
        assert_eq!(basic_report.completion, Completion::Complete);
        assert_eq!(
            basic_report
                .accepted_at_control
                .get(&EventId::from_bytes([1; 32]))
                .map(AcceptedAtControl::accepted_closure),
            Some(&basic_report.accepted_changes),
        );
        assert!(basic_report.materialized_document.is_some());
        let consumed_items = 200 - basic_budget.remaining().1;
        let final_schedule_exhausted = evaluate_batch(
            [control(1, None, vec![basic.clone()])],
            &mut WorkBudget::new(0, consumed_items - 1),
            &NeverCancelled,
        );
        assert_eq!(
            final_schedule_exhausted.completion,
            Completion::BudgetExhausted
        );
        assert_eq!(final_schedule_exhausted.accepted_changes.len(), 1);
        assert_eq!(
            final_schedule_exhausted.canonical_controls,
            vec![EventId::from_bytes([1; 32])]
        );
        assert_eq!(
            final_schedule_exhausted.control_dispositions[&EventId::from_bytes([1; 32])],
            ProtocolDisposition::Accepted
        );
        assert_eq!(
            final_schedule_exhausted
                .accepted_at_control
                .get(&EventId::from_bytes([1; 32]))
                .map(AcceptedAtControl::accepted_closure),
            Some(&final_schedule_exhausted.accepted_changes)
        );
        assert!(final_schedule_exhausted.materialized_document.is_none());

        let mut malformed = basic.clone();
        malformed.raw_change = Some(vec![0xff]);
        let materialization_failed = evaluate_batch(
            [control(1, None, vec![malformed])],
            &mut WorkBudget::new(0, 200),
            &NeverCancelled,
        );
        assert_eq!(materialization_failed.completion, Completion::Complete);
        assert_eq!(
            materialization_failed.failure,
            Some(EvaluationFailure::Apply)
        );
        assert!(materialization_failed.materialized_document.is_none());

        let concurrent = evaluate_batch(
            [control(1, None, vec![change(1, 1, 1), change(2, 2, 1)])],
            &mut WorkBudget::new(0, 200),
            &NeverCancelled,
        );
        assert_eq!(concurrent.accepted_changes.len(), 2);

        let mut invalid = change(3, 3, 1);
        invalid.legacy_eligible = false;
        let revoked = evaluate_batch(
            [control(1, None, vec![invalid.clone()])],
            &mut WorkBudget::new(0, 200),
            &NeverCancelled,
        );
        assert_eq!(
            revoked.dispositions[&invalid.candidate.change_hash],
            ProtocolDisposition::Invalid
        );

        let mut fork_budget = WorkBudget::new(0, 200);
        let forked = evaluate_batch(
            [control(2, None, vec![]), control(1, None, vec![])],
            &mut fork_budget,
            &NeverCancelled,
        );
        assert_eq!(
            forked.canonical_controls,
            vec![EventId::from_bytes([1; 32])]
        );
        assert_eq!(forked.integrity_alerts.len(), 1);
        let fork_consumed = 200 - fork_budget.remaining().1;
        let interrupted_fork = evaluate_batch(
            [control(2, None, vec![]), control(1, None, vec![])],
            &mut WorkBudget::new(0, fork_consumed - 1),
            &NeverCancelled,
        );
        assert_eq!(interrupted_fork.completion, Completion::BudgetExhausted);
        assert_eq!(interrupted_fork.integrity_alerts, forked.integrity_alerts);
        assert_eq!(
            interrupted_fork.control_dispositions,
            forked.control_dispositions
        );

        let equivocated = evaluate_batch(
            [control(1, None, vec![change(1, 1, 1), change(2, 1, 1)])],
            &mut WorkBudget::new(0, 200),
            &NeverCancelled,
        );
        assert!(equivocated.accepted_changes.is_empty());
        assert_eq!(equivocated.integrity_alerts.len(), 1);

        let mut frozen = control(1, None, vec![change(1, 1, 1)]);
        frozen.frozen = true;
        let frozen = evaluate_batch([frozen], &mut WorkBudget::new(0, 200), &NeverCancelled);
        assert!(frozen.accepted_changes.is_empty());
        assert_eq!(
            frozen.dispositions.values().next(),
            Some(&ProtocolDisposition::Excluded)
        );

        let mut reversed = vec![control(2, None, vec![]), control(1, None, vec![basic])];
        let first = evaluate_batch(
            reversed.clone(),
            &mut WorkBudget::new(0, 200),
            &NeverCancelled,
        );
        reversed.reverse();
        let second = evaluate_batch(reversed, &mut WorkBudget::new(0, 200), &NeverCancelled);
        assert_eq!(first, second);
    }

    #[test]
    fn applied_head_agreement_is_required() {
        let accepted = BTreeSet::from([ChangeHash::from_bytes([1; 32])]);
        let controls = std::collections::BTreeMap::from([(
            EventId::from_bytes([2; 32]),
            control(2, None, vec![change(1, 1, 1)]),
        )]);
        let derived = super::derive_heads(&accepted, &controls);
        assert!(super::applied_heads_agree(&derived, &accepted));
        assert!(!super::applied_heads_agree(
            &derived,
            &BTreeSet::from([ChangeHash::from_bytes([9; 32])])
        ));
    }

    #[test]
    fn accepted_base_duplicate_is_not_readmitted() {
        let accepted = change(1, 1, 1);
        let hash = accepted.candidate.change_hash;
        let parent = control(1, None, vec![accepted.clone()]);
        let mut child = control(2, Some(1), vec![accepted]);
        child.accepted_base = BTreeSet::from([hash]);
        let report = evaluate_batch(
            [parent, child],
            &mut WorkBudget::new(0, 500),
            &NeverCancelled,
        );
        assert_eq!(report.completion, Completion::Complete);
        assert_eq!(report.accepted_changes, BTreeSet::from([hash]));
        assert_eq!(
            report.dispositions.get(&hash),
            Some(&ProtocolDisposition::Accepted)
        );
    }

    #[test]
    fn pruned_prior_dependency_is_invalid_not_pending() {
        let first = change(1, 1, 1);
        let second = change(2, 2, 1);
        let first_hash = first.candidate.change_hash;
        let second_hash = second.candidate.change_hash;
        let parent = control(1, None, vec![first, second]);
        let mut dependant = change(3, 3, 1);
        dependant.candidate.dependencies = vec![second_hash];
        let dependant_hash = dependant.candidate.change_hash;
        let mut child = control(2, Some(1), vec![dependant]);
        child.accepted_base = BTreeSet::from([first_hash]);
        let report = evaluate_batch(
            [parent, child],
            &mut WorkBudget::new(0, 1_000),
            &NeverCancelled,
        );
        assert_eq!(report.completion, Completion::Complete);
        assert_eq!(report.accepted_changes, BTreeSet::from([first_hash]));
        assert_eq!(
            report.dispositions.get(&second_hash),
            Some(&ProtocolDisposition::Excluded)
        );
        assert_eq!(
            report.dispositions.get(&dependant_hash),
            Some(&ProtocolDisposition::Invalid)
        );
    }
}
