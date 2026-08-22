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
    pub(crate) branch_states: BTreeMap<EventId, BranchEvaluationState>,
    pub(crate) branch_change_dispositions:
        BTreeMap<EventId, BTreeMap<ChangeHash, ProtocolDisposition>>,
    pub(crate) dispositions: BTreeMap<ChangeHash, ProtocolDisposition>,
    pub(crate) accepted_changes: BTreeSet<ChangeHash>,
    pub(crate) heads: BTreeSet<ChangeHash>,
    pub(crate) materialized_document: Option<Vec<u8>>,
    pub(crate) integrity_alerts: Vec<IntegrityAlert>,
    pub(crate) completion: Completion,
    pub(crate) failure: Option<EvaluationFailure>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BranchEvaluationState {
    Valid,
    Pending,
    Invalid,
}

impl BranchEvaluationState {
    pub(crate) const fn final_disposition(self, canonical: bool) -> ProtocolDisposition {
        match (self, canonical) {
            (Self::Valid, true) => ProtocolDisposition::Accepted,
            (Self::Valid, false) => ProtocolDisposition::Excluded,
            (Self::Pending, _) => ProtocolDisposition::Pending,
            (Self::Invalid, _) => ProtocolDisposition::Invalid,
        }
    }
}

pub(crate) fn evaluate_batch(
    controls: impl IntoIterator<Item = BatchControl>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> BatchEvaluationReport {
    evaluate_batch_with_prior(controls, &BTreeMap::new(), budget, cancellation)
}

pub(crate) fn evaluate_batch_with_prior(
    controls: impl IntoIterator<Item = BatchControl>,
    additional_prior: &BTreeMap<EventId, BTreeMap<ChangeHash, PriorChangeKnowledge>>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> BatchEvaluationReport {
    if cancellation.is_cancelled() {
        return no_progress_batch_report(Completion::Cancelled);
    }
    let mut collected = Vec::new();
    for control in controls {
        if cancellation.is_cancelled() {
            return no_progress_batch_report(Completion::Cancelled);
        }
        if budget.charge(WorkCounter::Control, 1).is_err() {
            return no_progress_batch_report(Completion::BudgetExhausted);
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
    let mut branch_states = BTreeMap::new();
    let mut branch_change_dispositions = BTreeMap::new();
    match evaluate_branch_table(
        &controls,
        &by_parent,
        additional_prior,
        &dispositions,
        budget,
        cancellation,
    ) {
        Ok(table) => {
            branch_states = table.states.clone();
            statefully_valid_controls = table.valid.keys().copied().collect();
            accepted_at_control = table
                .valid
                .iter()
                .map(|(event_id, branch)| {
                    (*event_id, AcceptedAtControl::from_result(&branch.epoch))
                })
                .collect();
            branch_change_dispositions = table
                .valid
                .iter()
                .map(|(event_id, branch)| (*event_id, branch.change_dispositions.clone()))
                .collect();
            for (event_id, state) in &branch_states {
                control_dispositions.insert(*event_id, state.final_disposition(false));
            }
            match derive_canonical_branch(&controls, &by_parent, &table, &dispositions) {
                Ok(canonical) => {
                    canonical_controls = canonical.controls;
                    dispositions = canonical.change_dispositions;
                    accepted_changes = canonical.accepted_changes;
                    integrity_alerts = canonical.integrity_alerts;
                    for selected in &canonical_controls {
                        control_dispositions.insert(*selected, ProtocolDisposition::Accepted);
                    }
                }
                Err(()) => failure = Some(EvaluationFailure::InvariantViolation),
            }
        }
        Err(stop) => {
            completion = stop;
        }
    }

    if let Some(failure) = failure {
        return failed_batch_report(failure);
    }
    if completion != Completion::Complete {
        return no_progress_batch_report(completion);
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
            return no_progress_batch_report(completion);
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
        return no_progress_batch_report(completion);
    }
    let materialized = if can_materialize {
        match materialize_history(&raw_changes, &ordered) {
            Ok(document) => Some(document),
            Err(_) => {
                return failed_batch_report(EvaluationFailure::Apply);
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
            return failed_batch_report(EvaluationFailure::InvariantViolation);
        }
        None => (derived_heads, None),
    };
    BatchEvaluationReport {
        canonical_controls,
        control_dispositions,
        accepted_at_control,
        statefully_valid_controls,
        branch_states,
        branch_change_dispositions,
        dispositions,
        accepted_changes,
        heads,
        materialized_document,
        integrity_alerts,
        completion,
        failure: None,
    }
}

struct ValidBranchEvaluation {
    epoch: EpochEvaluationResult,
    change_dispositions: BTreeMap<ChangeHash, ProtocolDisposition>,
    validated_base: BTreeSet<ChangeHash>,
    ancestry: Vec<ControlEnvelope>,
    prior_knowledge: BTreeMap<ChangeHash, PriorChangeKnowledge>,
}

#[derive(Default)]
struct BranchTableEvaluation {
    states: BTreeMap<EventId, BranchEvaluationState>,
    valid: BTreeMap<EventId, ValidBranchEvaluation>,
}

struct CanonicalBranchEvaluation {
    controls: Vec<EventId>,
    change_dispositions: BTreeMap<ChangeHash, ProtocolDisposition>,
    accepted_changes: BTreeSet<ChangeHash>,
    integrity_alerts: Vec<IntegrityAlert>,
}

#[derive(Default)]
struct BatchChangeMemo {
    candidates: BTreeMap<ChangeHash, ChangeCandidate>,
    raw_changes: BTreeMap<ChangeHash, Vec<u8>>,
    hashes_by_control: BTreeMap<EventId, BTreeSet<ChangeHash>>,
    controls_by_hash: BTreeMap<ChangeHash, BTreeSet<EventId>>,
}

impl BatchChangeMemo {
    fn derive(
        controls: &BTreeMap<EventId, BatchControl>,
        budget: &mut WorkBudget,
        cancellation: &impl CancellationCheck,
    ) -> Result<Self, Completion> {
        let mut memo = Self::default();
        for (control_id, control) in controls {
            for change in &control.changes {
                if cancellation.is_cancelled() {
                    return Err(Completion::Cancelled);
                }
                budget
                    .charge(WorkCounter::GraphNode, 1)
                    .map_err(|_| Completion::BudgetExhausted)?;
                budget
                    .charge(
                        WorkCounter::GraphEdge,
                        u64::try_from(change.candidate.dependencies.len()).unwrap_or(u64::MAX),
                    )
                    .map_err(|_| Completion::BudgetExhausted)?;
                let hash = change.candidate.change_hash;
                memo.candidates
                    .entry(hash)
                    .or_insert_with(|| change.candidate.clone());
                if let Some(raw) = &change.raw_change {
                    memo.raw_changes.entry(hash).or_insert_with(|| raw.clone());
                }
                memo.hashes_by_control
                    .entry(*control_id)
                    .or_default()
                    .insert(hash);
                memo.controls_by_hash
                    .entry(hash)
                    .or_default()
                    .insert(*control_id);
            }
        }
        Ok(memo)
    }
}

fn derive_canonical_branch(
    controls: &BTreeMap<EventId, BatchControl>,
    children_by_parent: &BTreeMap<Option<EventId>, BTreeSet<EventId>>,
    table: &BranchTableEvaluation,
    preliminary_change_dispositions: &BTreeMap<ChangeHash, ProtocolDisposition>,
) -> Result<CanonicalBranchEvaluation, ()> {
    let mut change_dispositions = preliminary_change_dispositions.clone();
    let mut canonical_controls = Vec::new();
    let mut accepted_changes = BTreeSet::new();
    let mut integrity_alerts = Vec::new();
    let mut parent_id = None;
    while let Some(children) = children_by_parent.get(&parent_id) {
        let outcomes = children.iter().filter_map(|event_id| {
            let control = controls.get(event_id)?;
            let state = table.states.get(event_id)?;
            let sequence = control
                .envelope
                .as_ref()
                .map_or(0, ControlEnvelope::sequence);
            Some(match state {
                BranchEvaluationState::Valid => ControlCandidateOutcome::valid(
                    *event_id,
                    control.parent,
                    sequence,
                    table
                        .valid
                        .get(event_id)
                        .map(|branch| branch.validated_base.clone())
                        .unwrap_or_default(),
                ),
                BranchEvaluationState::Pending => ControlCandidateOutcome::pending(
                    *event_id,
                    control.parent,
                    sequence,
                    crate::DiagnosticCode::registered("control.state"),
                    None,
                ),
                BranchEvaluationState::Invalid => ControlCandidateOutcome::invalid(
                    *event_id,
                    control.parent,
                    sequence,
                    crate::DiagnosticCode::registered("control.state"),
                    None,
                ),
            })
        });
        let (selection, alert) = select_valid_outcomes_with_alert(parent_id, outcomes);
        let Some(selected) = selection.selected else {
            break;
        };
        let branch = table.valid.get(&selected).ok_or(())?;
        let control = controls.get(&selected).ok_or(())?;
        canonical_controls.push(selected);
        if let Some(alert) = alert {
            integrity_alerts.push(alert);
        }
        change_dispositions.extend(branch.epoch.dispositions().clone());
        accepted_changes = branch.epoch.accepted_state().accepted_closure().clone();
        integrity_alerts.extend_from_slice(branch.epoch.integrity_alerts());
        if control.frozen {
            break;
        }
        parent_id = Some(selected);
    }
    for (control_id, branch) in &table.valid {
        if canonical_controls.contains(control_id) {
            continue;
        }
        for alert in branch.epoch.integrity_alerts() {
            if !integrity_alerts.contains(alert) {
                integrity_alerts.push(alert.clone());
            }
        }
    }
    for (hash, disposition) in &mut change_dispositions {
        if accepted_changes.contains(hash) {
            *disposition = ProtocolDisposition::Accepted;
        } else if *disposition == ProtocolDisposition::Accepted {
            *disposition = ProtocolDisposition::Excluded;
        }
    }
    Ok(CanonicalBranchEvaluation {
        controls: canonical_controls,
        change_dispositions,
        accepted_changes,
        integrity_alerts,
    })
}

fn evaluate_branch_table(
    controls: &BTreeMap<EventId, BatchControl>,
    children_by_parent: &BTreeMap<Option<EventId>, BTreeSet<EventId>>,
    additional_prior: &BTreeMap<EventId, BTreeMap<ChangeHash, PriorChangeKnowledge>>,
    preliminary_change_dispositions: &BTreeMap<ChangeHash, ProtocolDisposition>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<BranchTableEvaluation, Completion> {
    let mut table = BranchTableEvaluation::default();
    let change_memo = BatchChangeMemo::derive(controls, budget, cancellation)?;
    let mut accepted_state_cache = BTreeMap::new();
    let mut ready = children_by_parent.get(&None).cloned().unwrap_or_default();
    while let Some(event_id) = ready.pop_first() {
        if cancellation.is_cancelled() {
            return Err(Completion::Cancelled);
        }
        charge_control_transitions(1, budget).map_err(|_| Completion::BudgetExhausted)?;
        let Some(control) = controls.get(&event_id) else {
            table
                .states
                .insert(event_id, BranchEvaluationState::Invalid);
            continue;
        };
        let parent_branch = control.parent.and_then(|parent| table.valid.get(&parent));
        let inherited = control
            .parent
            .and_then(|parent| table.states.get(&parent))
            .and_then(|state| match state {
                BranchEvaluationState::Pending => Some(BranchEvaluationState::Pending),
                BranchEvaluationState::Invalid => Some(BranchEvaluationState::Invalid),
                BranchEvaluationState::Valid => None,
            });
        let validated_base = if let Some(state) = inherited {
            table.states.insert(event_id, state);
            None
        } else if let (Some(parent), Some(branch), Some(child)) = (
            control
                .parent
                .and_then(|parent| controls.get(&parent))
                .and_then(|parent| parent.envelope.as_ref()),
            parent_branch,
            control.envelope.as_ref(),
        ) {
            let mut view = ParentEpochView::from_result(&branch.epoch);
            view.extend_prior_knowledge(&branch.prior_knowledge);
            if let Some(parent_id) = control.parent
                && let Some(knowledge) = additional_prior.get(&parent_id)
            {
                view.extend_prior_knowledge(knowledge);
            }
            let singleton = BTreeSet::from([event_id]);
            charge_control_closures(
                &singleton,
                controls,
                &branch.ancestry,
                &view,
                budget,
                cancellation,
            )?;
            let ancestry = branch
                .ancestry
                .iter()
                .map(ControlEnvelope::content)
                .collect::<Vec<_>>();
            match evaluate_child(parent, child, &ancestry, &view) {
                CandidateResult::Valid => Some(
                    accepted_frontier_closure(
                        child.base_heads(),
                        view.accepted(),
                        view.dependency_index(),
                    )
                    .accepted,
                ),
                CandidateResult::Pending(_) => {
                    table
                        .states
                        .insert(event_id, BranchEvaluationState::Pending);
                    None
                }
                CandidateResult::Invalid(_) => {
                    table
                        .states
                        .insert(event_id, BranchEvaluationState::Invalid);
                    None
                }
            }
        } else if control.parent.is_none() {
            Some(if control.envelope.is_some() {
                BTreeSet::new()
            } else {
                control.accepted_base.clone()
            })
        } else if control.envelope.is_none() && parent_branch.is_some() {
            Some(control.accepted_base.clone())
        } else {
            table
                .states
                .insert(event_id, BranchEvaluationState::Invalid);
            None
        };

        if let Some(validated_base) = validated_base {
            let parent_epoch = parent_branch.map(|branch| &branch.epoch);
            let Some(accepted_base) = accepted_state_for_closure(
                &validated_base,
                &change_memo.candidates,
                parent_epoch,
                &mut accepted_state_cache,
            ) else {
                table
                    .states
                    .insert(event_id, BranchEvaluationState::Invalid);
                enqueue_children(event_id, children_by_parent, &mut ready);
                continue;
            };
            let mut knowledge = prior_change_knowledge(
                parent_epoch,
                &validated_base,
                preliminary_change_dispositions,
                event_id,
                &change_memo,
            );
            if let Some(additional) = additional_prior.get(&event_id) {
                for (hash, item) in additional {
                    knowledge.entry(*hash).or_insert(*item);
                }
            }
            let parent_ancestry = parent_branch
                .map(|branch| branch.ancestry.as_slice())
                .unwrap_or_default();
            let retained_knowledge = knowledge.clone();
            match resolve_authoritative_epoch(
                control,
                accepted_base,
                knowledge,
                parent_ancestry,
                &change_memo.raw_changes,
                budget,
                cancellation,
            ) {
                Ok(epoch) => {
                    let mut branch_change_dispositions = parent_branch
                        .map(|branch| branch.change_dispositions.clone())
                        .unwrap_or_default();
                    if let Some(parent) = parent_branch {
                        for hash in parent.epoch.accepted_state().accepted_closure() {
                            branch_change_dispositions.insert(
                                *hash,
                                if validated_base.contains(hash) {
                                    ProtocolDisposition::Accepted
                                } else {
                                    ProtocolDisposition::Excluded
                                },
                            );
                        }
                    }
                    for hash in &validated_base {
                        branch_change_dispositions.insert(*hash, ProtocolDisposition::Accepted);
                    }
                    branch_change_dispositions.extend(epoch.dispositions().clone());
                    let mut ancestry = parent_ancestry.to_vec();
                    if let Some(envelope) = control.envelope.as_ref() {
                        ancestry.push(envelope.clone());
                    }
                    table.states.insert(event_id, BranchEvaluationState::Valid);
                    table.valid.insert(
                        event_id,
                        ValidBranchEvaluation {
                            epoch,
                            change_dispositions: branch_change_dispositions,
                            validated_base,
                            ancestry,
                            prior_knowledge: retained_knowledge,
                        },
                    );
                }
                Err(EpochResolutionError::Schedule(ScheduleError::BudgetExhausted)) => {
                    return Err(Completion::BudgetExhausted);
                }
                Err(EpochResolutionError::Schedule(ScheduleError::Cancelled)) => {
                    return Err(Completion::Cancelled);
                }
                Err(EpochResolutionError::InvalidState) => {
                    table
                        .states
                        .insert(event_id, BranchEvaluationState::Invalid);
                }
            }
        }
        enqueue_children(event_id, children_by_parent, &mut ready);
    }
    Ok(table)
}

fn enqueue_children(
    parent: EventId,
    children_by_parent: &BTreeMap<Option<EventId>, BTreeSet<EventId>>,
    ready: &mut BTreeSet<EventId>,
) {
    if let Some(children) = children_by_parent.get(&Some(parent)) {
        ready.extend(children);
    }
}

pub(crate) fn propagate_control_parent_dispositions(
    parents: &BTreeMap<EventId, Option<EventId>>,
    dispositions: &mut BTreeMap<EventId, ProtocolDisposition>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<(), Completion> {
    let mut resolved = BTreeSet::new();
    for start in parents.keys().copied() {
        if resolved.contains(&start) {
            continue;
        }
        let mut path = Vec::new();
        let mut positions = BTreeMap::new();
        let mut current = start;
        loop {
            if resolved.contains(&current) {
                break;
            }
            if let Some(cycle_start) = positions.get(&current).copied() {
                for event_id in &path[cycle_start..] {
                    dispositions.insert(*event_id, ProtocolDisposition::Invalid);
                    resolved.insert(*event_id);
                }
                break;
            }
            positions.insert(current, path.len());
            path.push(current);
            let Some(parent) = parents.get(&current).copied().flatten() else {
                break;
            };
            if cancellation.is_cancelled() {
                return Err(Completion::Cancelled);
            }
            budget
                .charge(WorkCounter::Control, 1)
                .map_err(|_| Completion::BudgetExhausted)?;
            if !parents.contains_key(&parent) {
                break;
            }
            current = parent;
        }
        for child in path.into_iter().rev() {
            if resolved.contains(&child) {
                continue;
            }
            if let Some(parent) = parents.get(&child).copied().flatten() {
                match dispositions.get(&parent) {
                    Some(ProtocolDisposition::Pending) => {
                        dispositions.insert(child, ProtocolDisposition::Pending);
                    }
                    Some(
                        ProtocolDisposition::Invalid | ProtocolDisposition::UnsupportedRevision,
                    ) => {
                        dispositions.insert(child, ProtocolDisposition::Invalid);
                    }
                    Some(ProtocolDisposition::Accepted | ProtocolDisposition::Excluded) | None => {}
                }
            }
            resolved.insert(child);
        }
    }
    Ok(())
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
    raw_changes: &BTreeMap<ChangeHash, Vec<u8>>,
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
                    && change.candidate.dependencies.iter().all(|dependency| {
                        !prior_change_knowledge
                            .get(dependency)
                            .is_some_and(|knowledge| knowledge.is_known_impossible())
                    }),
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
    let epoch_changes = control
        .changes
        .iter()
        .filter(|change| {
            !accepted_base
                .accepted_closure()
                .contains(&change.candidate.change_hash)
        })
        .map(|change| change.candidate.clone())
        .collect::<Vec<_>>();
    let input = EpochEvaluationInput::new_with_borrowed_raw_and_prior(
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
    selected_control: EventId,
    memo: &BatchChangeMemo,
) -> BTreeMap<ChangeHash, PriorChangeKnowledge> {
    let mut knowledge = selected_base
        .iter()
        .map(|hash| (*hash, PriorChangeKnowledge::AcceptedInBase))
        .collect::<BTreeMap<_, _>>();
    if let Some(hashes) = memo.hashes_by_control.get(&selected_control) {
        for hash in hashes {
            knowledge
                .entry(*hash)
                .or_insert(PriorChangeKnowledge::SameEpochCandidate);
        }
    }
    knowledge.extend(
        parent
            .into_iter()
            .flat_map(|result| result.accepted_state().accepted_closure())
            .filter(|hash| !selected_base.contains(hash))
            .map(|hash| (*hash, PriorChangeKnowledge::PrunedCanonicalAncestor)),
    );
    for (hash, control_ids) in &memo.controls_by_hash {
        if control_ids
            .iter()
            .any(|control_id| *control_id != selected_control)
        {
            knowledge
                .entry(*hash)
                .or_insert(PriorChangeKnowledge::KnownOtherControl);
        }
    }
    for (hash, disposition) in dispositions {
        if *disposition == ProtocolDisposition::Invalid && !selected_base.contains(hash) {
            knowledge
                .entry(*hash)
                .or_insert(PriorChangeKnowledge::KnownInvalid);
        } else if *disposition == ProtocolDisposition::Excluded
            && !selected_base.contains(hash)
            && !knowledge.contains_key(hash)
        {
            knowledge.insert(*hash, PriorChangeKnowledge::PriorEquivocationExcluded);
        }
    }
    knowledge
}

fn accepted_state_for_closure(
    accepted: &BTreeSet<ChangeHash>,
    candidates_by_hash: &BTreeMap<ChangeHash, ChangeCandidate>,
    parent: Option<&EpochEvaluationResult>,
    cache: &mut BTreeMap<BTreeSet<ChangeHash>, AcceptedEpochState>,
) -> Option<AcceptedEpochState> {
    if let Some(cached) = cache.get(accepted) {
        return Some(cached.clone());
    }
    let candidates = accepted
        .iter()
        .filter_map(|hash| {
            candidates_by_hash
                .get(hash)
                .cloned()
                .map(|value| (*hash, value))
        })
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
    let state = AcceptedEpochState::new(accepted.clone(), heads, candidates, materialized).ok()?;
    cache.insert(accepted.clone(), state.clone());
    Some(state)
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
    pub(crate) fn referenced_branch_change_disposition(
        &self,
        control: EventId,
        hash: ChangeHash,
    ) -> Option<ProtocolDisposition> {
        self.branch_change_dispositions
            .get(&control)
            .and_then(|dispositions| dispositions.get(&hash))
            .copied()
    }
}

fn no_progress_batch_report(completion: Completion) -> BatchEvaluationReport {
    let failure = match completion {
        Completion::BudgetExhausted => EvaluationFailure::BudgetExhausted,
        Completion::Cancelled => EvaluationFailure::Cancelled,
        Completion::Complete => EvaluationFailure::InvariantViolation,
    };
    empty_batch_report(completion, Some(failure))
}

fn failed_batch_report(failure: EvaluationFailure) -> BatchEvaluationReport {
    empty_batch_report(Completion::Complete, Some(failure))
}

fn empty_batch_report(
    completion: Completion,
    failure: Option<EvaluationFailure>,
) -> BatchEvaluationReport {
    BatchEvaluationReport {
        canonical_controls: Vec::new(),
        control_dispositions: BTreeMap::new(),
        accepted_at_control: BTreeMap::new(),
        statefully_valid_controls: BTreeSet::new(),
        branch_states: BTreeMap::new(),
        branch_change_dispositions: BTreeMap::new(),
        dispositions: BTreeMap::new(),
        accepted_changes: BTreeSet::new(),
        heads: BTreeSet::new(),
        materialized_document: None,
        integrity_alerts: Vec::new(),
        completion,
        failure,
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
        AcceptedAtControl, BatchChange, BatchChangeMemo, BatchControl, BatchEvaluationReport,
        BranchEvaluationState, charge_control_closures, evaluate_batch,
        propagate_control_parent_dispositions,
    };
    use crate::automerge_adapter::decode::decode_change;
    use crate::graph::actor_state::tests::candidate;
    use crate::{
        ChangeHash, Completion, EvaluationFailure, EventId, IntegrityAlert, NeverCancelled,
        ProtocolDisposition, ProtocolRevision, WorkBudget, WorkCounter,
    };

    #[test]
    fn branch_state_maps_exhaustively_to_final_disposition() {
        assert_eq!(
            BranchEvaluationState::Valid.final_disposition(true),
            ProtocolDisposition::Accepted
        );
        assert_eq!(
            BranchEvaluationState::Valid.final_disposition(false),
            ProtocolDisposition::Excluded
        );
        assert_eq!(
            BranchEvaluationState::Pending.final_disposition(true),
            ProtocolDisposition::Pending
        );
        assert_eq!(
            BranchEvaluationState::Invalid.final_disposition(false),
            ProtocolDisposition::Invalid
        );
    }

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

    fn assert_no_progress_batch(report: &BatchEvaluationReport, completion: Completion) {
        assert_eq!(report.completion, completion);
        let expected_failure = match completion {
            Completion::BudgetExhausted => Some(EvaluationFailure::BudgetExhausted),
            Completion::Cancelled => Some(EvaluationFailure::Cancelled),
            Completion::Complete => None,
        };
        assert_ne!(completion, Completion::Complete);
        assert_eq!(report.failure, expected_failure);
        assert!(report.canonical_controls.is_empty());
        assert!(report.control_dispositions.is_empty());
        assert!(report.accepted_at_control.is_empty());
        assert!(report.statefully_valid_controls.is_empty());
        assert!(report.branch_states.is_empty());
        assert!(report.branch_change_dispositions.is_empty());
        assert!(report.dispositions.is_empty());
        assert!(report.accepted_changes.is_empty());
        assert!(report.heads.is_empty());
        assert!(report.materialized_document.is_none());
        assert!(report.integrity_alerts.is_empty());
    }

    #[test]
    fn pending_parent_state_propagates_through_descendants() {
        let parent = EventId::from_bytes([1; 32]);
        let child = EventId::from_bytes([2; 32]);
        let grandchild = EventId::from_bytes([3; 32]);
        let parents = BTreeMap::from([
            (parent, Some(EventId::from_bytes([9; 32]))),
            (child, Some(parent)),
            (grandchild, Some(child)),
        ]);
        let mut dispositions = BTreeMap::from([
            (parent, ProtocolDisposition::Pending),
            (child, ProtocolDisposition::Invalid),
            (grandchild, ProtocolDisposition::Excluded),
        ]);
        assert_eq!(
            propagate_control_parent_dispositions(
                &parents,
                &mut dispositions,
                &mut WorkBudget::new(0, 3),
                &crate::NeverCancelled,
            ),
            Ok(())
        );
        assert_eq!(
            dispositions.get(&child),
            Some(&ProtocolDisposition::Pending)
        );
        assert_eq!(
            dispositions.get(&grandchild),
            Some(&ProtocolDisposition::Pending)
        );
    }

    #[test]
    fn invalid_parent_state_propagates_through_descendants() {
        let parent = EventId::from_bytes([4; 32]);
        let child = EventId::from_bytes([5; 32]);
        let grandchild = EventId::from_bytes([6; 32]);
        let parents = BTreeMap::from([
            (parent, Some(EventId::from_bytes([8; 32]))),
            (child, Some(parent)),
            (grandchild, Some(child)),
        ]);
        let mut dispositions = BTreeMap::from([
            (parent, ProtocolDisposition::Invalid),
            (child, ProtocolDisposition::Excluded),
            (grandchild, ProtocolDisposition::Pending),
        ]);
        assert_eq!(
            propagate_control_parent_dispositions(
                &parents,
                &mut dispositions,
                &mut WorkBudget::new(0, 3),
                &crate::NeverCancelled,
            ),
            Ok(())
        );
        assert_eq!(
            dispositions.get(&child),
            Some(&ProtocolDisposition::Invalid)
        );
        assert_eq!(
            dispositions.get(&grandchild),
            Some(&ProtocolDisposition::Invalid)
        );
    }

    #[test]
    fn deep_pending_chain_reaches_a_fixed_point_independent_of_id_order() {
        let parent = EventId::from_bytes([9; 32]);
        let child = EventId::from_bytes([8; 32]);
        let grandchild = EventId::from_bytes([7; 32]);
        let parents = BTreeMap::from([
            (parent, Some(EventId::from_bytes([10; 32]))),
            (child, Some(parent)),
            (grandchild, Some(child)),
        ]);
        let mut dispositions = BTreeMap::from([
            (parent, ProtocolDisposition::Pending),
            (child, ProtocolDisposition::Excluded),
            (grandchild, ProtocolDisposition::Excluded),
        ]);
        assert_eq!(
            propagate_control_parent_dispositions(
                &parents,
                &mut dispositions,
                &mut WorkBudget::new(0, 3),
                &crate::NeverCancelled,
            ),
            Ok(())
        );
        assert!(
            [parent, child, grandchild]
                .into_iter()
                .all(|id| dispositions.get(&id) == Some(&ProtocolDisposition::Pending))
        );
    }

    #[test]
    fn deep_invalid_chain_reaches_a_fixed_point_independent_of_id_order() {
        let parent = EventId::from_bytes([12; 32]);
        let child = EventId::from_bytes([11; 32]);
        let grandchild = EventId::from_bytes([10; 32]);
        let parents = BTreeMap::from([
            (parent, Some(EventId::from_bytes([13; 32]))),
            (child, Some(parent)),
            (grandchild, Some(child)),
        ]);
        let mut dispositions = BTreeMap::from([
            (parent, ProtocolDisposition::Invalid),
            (child, ProtocolDisposition::Excluded),
            (grandchild, ProtocolDisposition::Pending),
        ]);
        assert_eq!(
            propagate_control_parent_dispositions(
                &parents,
                &mut dispositions,
                &mut WorkBudget::new(0, 3),
                &crate::NeverCancelled,
            ),
            Ok(())
        );
        assert!(
            [parent, child, grandchild]
                .into_iter()
                .all(|id| dispositions.get(&id) == Some(&ProtocolDisposition::Invalid))
        );
    }

    fn propagation_chain(length: u16) -> BTreeMap<EventId, Option<EventId>> {
        (0..length)
            .map(|index| {
                let event_id = EventId::from_bytes([(length - index) as u8; 32]);
                let parent =
                    (index > 0).then(|| EventId::from_bytes([(length - (index - 1)) as u8; 32]));
                (event_id, parent)
            })
            .collect()
    }

    #[test]
    fn parent_propagation_has_exact_linear_budget_boundaries() {
        let empty = BTreeMap::new();
        assert_eq!(
            propagate_control_parent_dispositions(
                &empty,
                &mut BTreeMap::new(),
                &mut WorkBudget::new(0, 0),
                &crate::NeverCancelled,
            ),
            Ok(())
        );

        let parents = propagation_chain(200);
        let root = EventId::from_bytes([200; 32]);
        let initial = parents
            .keys()
            .copied()
            .map(|event_id| {
                (
                    event_id,
                    if event_id == root {
                        ProtocolDisposition::Pending
                    } else {
                        ProtocolDisposition::Excluded
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        let mut exact_dispositions = initial.clone();
        let mut exact = WorkBudget::new(0, 199);
        assert_eq!(
            propagate_control_parent_dispositions(
                &parents,
                &mut exact_dispositions,
                &mut exact,
                &crate::NeverCancelled,
            ),
            Ok(())
        );
        assert_eq!(exact.consumed().get(WorkCounter::Control), 199);
        assert!(
            exact_dispositions
                .values()
                .all(|value| *value == ProtocolDisposition::Pending)
        );

        let mut insufficient_dispositions = initial;
        let mut insufficient = WorkBudget::new(0, 198);
        assert_eq!(
            propagate_control_parent_dispositions(
                &parents,
                &mut insufficient_dispositions,
                &mut insufficient,
                &crate::NeverCancelled,
            ),
            Err(Completion::BudgetExhausted)
        );
        assert_eq!(insufficient.consumed().get(WorkCounter::Control), 198);
    }

    #[test]
    fn parent_propagation_checks_every_cancellation_boundary() {
        let parents = propagation_chain(8);
        let root = EventId::from_bytes([8; 32]);
        let initial = parents
            .keys()
            .copied()
            .map(|event_id| {
                (
                    event_id,
                    if event_id == root {
                        ProtocolDisposition::Invalid
                    } else {
                        ProtocolDisposition::Excluded
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        for boundary in 0_u64..=7 {
            let checks = std::cell::Cell::new(0_u64);
            let cancellation = || {
                let current = checks.get();
                checks.set(current.saturating_add(1));
                current == boundary
            };
            let mut dispositions = initial.clone();
            let mut budget = WorkBudget::new(0, 7);
            let result = propagate_control_parent_dispositions(
                &parents,
                &mut dispositions,
                &mut budget,
                &cancellation,
            );
            if boundary < 7 {
                assert_eq!(result, Err(Completion::Cancelled));
                assert_eq!(budget.consumed().get(WorkCounter::Control), boundary);
            } else {
                assert_eq!(result, Ok(()));
                assert_eq!(budget.consumed().get(WorkCounter::Control), 7);
                assert!(
                    dispositions
                        .values()
                        .all(|value| *value == ProtocolDisposition::Invalid)
                );
            }
        }
    }

    #[test]
    fn parent_cycles_fail_closed_after_one_visit_per_edge() {
        let first = EventId::from_bytes([1; 32]);
        let second = EventId::from_bytes([2; 32]);
        let parents = BTreeMap::from([(first, Some(second)), (second, Some(first))]);
        let mut dispositions = BTreeMap::from([
            (first, ProtocolDisposition::Pending),
            (second, ProtocolDisposition::Excluded),
        ]);
        let mut budget = WorkBudget::new(0, 2);
        assert_eq!(
            propagate_control_parent_dispositions(
                &parents,
                &mut dispositions,
                &mut budget,
                &crate::NeverCancelled,
            ),
            Ok(())
        );
        assert!(
            dispositions
                .values()
                .all(|value| *value == ProtocolDisposition::Invalid)
        );
        assert_eq!(budget.consumed().get(WorkCounter::Control), 2);
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
            basic_report.branch_change_dispositions[&EventId::from_bytes([1; 32])]
                [&basic.candidate.change_hash],
            ProtocolDisposition::Accepted
        );
        assert_eq!(
            basic_report.referenced_branch_change_disposition(
                EventId::from_bytes([1; 32]),
                basic.candidate.change_hash,
            ),
            Some(ProtocolDisposition::Accepted)
        );
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
        assert_no_progress_batch(&final_schedule_exhausted, Completion::BudgetExhausted);

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
        assert_no_progress_batch(&interrupted_fork, Completion::BudgetExhausted);

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
    fn finding_075_interrupted_batch_discards_all_canonical_progress() {
        let controls = || [control(2, None, vec![]), control(1, None, vec![])];
        let mut complete_budget = WorkBudget::new(0, 200);
        let complete = evaluate_batch(controls(), &mut complete_budget, &NeverCancelled);
        assert_eq!(complete.completion, Completion::Complete);
        let consumed = 200_u64.saturating_sub(complete_budget.remaining().1);
        let interrupted = evaluate_batch(
            controls(),
            &mut WorkBudget::new(0, consumed.saturating_sub(1)),
            &NeverCancelled,
        );
        assert_no_progress_batch(&interrupted, Completion::BudgetExhausted);

        let cancelled = evaluate_batch(controls(), &mut WorkBudget::new(0, 200), &|| true);
        assert_no_progress_batch(&cancelled, Completion::Cancelled);
    }

    #[test]
    #[ignore = "expected to fail until FINDING_077 closes"]
    fn finding_077_canonical_raw_bytes_share_one_allocation() {
        let mut value = change(7, 7, 1);
        value.raw_change = Some(vec![0x5a; 64]);
        let controls =
            BTreeMap::from([(EventId::from_bytes([1; 32]), control(1, None, vec![value]))]);
        let original = controls
            .values()
            .next()
            .and_then(|control| control.changes.first())
            .and_then(|change| change.raw_change.as_ref());
        assert!(original.is_some());
        let mut budget = WorkBudget::new(64, 64);
        let retained = BatchChangeMemo::derive(&controls, &mut budget, &NeverCancelled)
            .ok()
            .and_then(|memo| memo.raw_changes.values().next().cloned());
        assert!(retained.is_some());
        let (Some(original), Some(retained)) = (original, retained) else {
            return;
        };
        assert_eq!(
            (
                core::ptr::eq(original.as_ptr(), retained.as_ptr()),
                budget.consumed().get(WorkCounter::DecodeByte),
            ),
            (true, 0),
            "FINDING_077 reproduced: canonical raw bytes are copied without byte accounting"
        );
    }

    #[test]
    fn losing_branch_preserves_pending_invalid_and_equivocation_outcomes() {
        let mut invalid = change(3, 3, 1);
        invalid.legacy_eligible = false;
        let mut pending = change(4, 4, 1);
        pending
            .candidate
            .dependencies
            .push(ChangeHash::from_bytes([99; 32]));
        let equivocation_a = change(5, 5, 1);
        let equivocation_b = change(6, 5, 1);
        let report = evaluate_batch(
            [
                control(1, None, vec![]),
                control(
                    2,
                    None,
                    vec![
                        invalid.clone(),
                        pending.clone(),
                        equivocation_a.clone(),
                        equivocation_b.clone(),
                    ],
                ),
            ],
            &mut WorkBudget::new(0, 500),
            &NeverCancelled,
        );
        let branch = &report.branch_change_dispositions[&EventId::from_bytes([2; 32])];
        assert_eq!(
            branch[&invalid.candidate.change_hash],
            ProtocolDisposition::Invalid
        );
        assert_eq!(
            branch[&pending.candidate.change_hash],
            ProtocolDisposition::Pending
        );
        assert_eq!(
            branch[&equivocation_a.candidate.change_hash],
            ProtocolDisposition::Excluded
        );
        assert_eq!(
            branch[&equivocation_b.candidate.change_hash],
            ProtocolDisposition::Excluded
        );
        assert_eq!(report.integrity_alerts.len(), 2);
        assert!(
            report
                .integrity_alerts
                .iter()
                .any(|alert| matches!(alert, IntegrityAlert::DeviceEquivocation(_)))
        );
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
    fn accepted_base_candidates_are_filtered_from_both_epoch_paths() {
        let source = include_str!("evaluate.rs");
        let exclusion = "!accepted_base\n                    .accepted_closure()\n                    .contains(&change.candidate.change_hash)";
        assert_eq!(source.matches(exclusion).count(), 1);
        let selected_exclusion = "!accepted_base\n                .accepted_closure()\n                .contains(&change.candidate.change_hash)";
        assert_eq!(source.matches(selected_exclusion).count(), 2);
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
