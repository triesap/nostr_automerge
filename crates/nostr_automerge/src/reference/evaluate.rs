use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::automerge_adapter::document::{
    AppliedDocument, ExactApplyError, materialize_history_metered,
};
use crate::automerge_adapter::materialized_view::MaterializedDocumentView;
use crate::control::ancestry::ControlAncestry;
use crate::control::candidate::{CandidateResult, evaluate_child_metered};
use crate::control::epoch_state::{AcceptedEpochState, MeteredAcceptedEpochStateError};
use crate::control::parent_view::{ParentEpochView, ParentEpochViewBuildError};
use crate::control::validate::ControlEnvelope;
use crate::graph::change_candidate::ChangeCandidate;
use crate::graph::dependency_graph::{MeteredGraphBuildError, build_graph_metered};
use crate::graph::equivocation::QuarantineError;
use crate::graph::equivocation::quarantine_equivocation_descendants;
use crate::graph::schedule::{ScheduleError, schedule_candidates};
use crate::reference::branch_state::PersistentDeltaMap;
use crate::reference::epoch::{EpochCandidate, resolve_epoch};
use crate::reference::epoch_engine::{
    AcceptedAtControl, EpochEvaluationError, EpochEvaluationInput, EpochEvaluationResult,
    MeteredEpochEvaluationInputError, PriorChangeKnowledge, PriorKnowledgeState,
    clone_candidate_maps_metered, collect_eligible_candidates_metered, evaluate_epoch,
    project_accepted_candidates_metered,
};
use crate::{
    CancellationCheck, ChangeHash, Completion, ControllerEquivocationAlert, EvaluationFailure,
    EventId, IntegrityAlert, ProtocolDisposition, WorkBudget, WorkCounter,
};

#[derive(Clone, Debug)]
pub(crate) struct BatchChange {
    pub(crate) candidate: ChangeCandidate,
    /// Eligibility hint used only by the envelope-free unit-test adapter.
    /// Stateful public evaluation derives every semantic outcome independently.
    pub(crate) legacy_eligible: bool,
    pub(crate) raw_change: Option<Arc<[u8]>>,
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

struct InitialBatchMaps {
    controls: BTreeMap<EventId, BatchControl>,
    root_controls: BTreeSet<EventId>,
    children_by_parent: BTreeMap<EventId, BTreeSet<EventId>>,
    control_dispositions: BTreeMap<EventId, ProtocolDisposition>,
    change_dispositions: BTreeMap<ChangeHash, ProtocolDisposition>,
    change_memo: BatchChangeMemo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InitialMapBuildError<E> {
    Work(E),
    Invariant,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BatchEvaluationReport {
    pub(crate) canonical_controls: Vec<EventId>,
    pub(crate) control_dispositions: BTreeMap<EventId, ProtocolDisposition>,
    pub(crate) accepted_at_control: BTreeMap<EventId, AcceptedAtControl>,
    pub(crate) statefully_valid_controls: BTreeSet<EventId>,
    pub(crate) branch_states: BTreeMap<EventId, BranchEvaluationState>,
    pub(crate) branch_change_dispositions:
        BTreeMap<EventId, PersistentDeltaMap<ChangeHash, ProtocolDisposition>>,
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
    evaluate_batch_with_prior_and_control_dispositions(
        controls,
        &BTreeMap::new(),
        BTreeMap::new(),
        budget,
        cancellation,
    )
}

pub(crate) fn evaluate_batch_with_prior(
    controls: impl IntoIterator<Item = BatchControl>,
    additional_prior: &BTreeMap<EventId, BTreeMap<ChangeHash, PriorChangeKnowledge>>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> BatchEvaluationReport {
    evaluate_batch_with_prior_and_control_dispositions(
        controls,
        additional_prior,
        BTreeMap::new(),
        budget,
        cancellation,
    )
}

pub(crate) fn evaluate_batch_with_prior_and_control_dispositions(
    controls: impl IntoIterator<Item = BatchControl>,
    additional_prior: &BTreeMap<EventId, BTreeMap<ChangeHash, PriorChangeKnowledge>>,
    initial_control_dispositions: BTreeMap<EventId, ProtocolDisposition>,
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
    let initial =
        match prepare_initial_maps_metered(collected, initial_control_dispositions, |counter| {
            charge_prior_knowledge_item(counter, budget, cancellation)
        }) {
            Ok(initial) => initial,
            Err(InitialMapBuildError::Work(completion)) => {
                return no_progress_batch_report(completion);
            }
            Err(InitialMapBuildError::Invariant) => {
                return failed_batch_report(EvaluationFailure::InvariantViolation);
            }
        };
    let InitialBatchMaps {
        controls,
        root_controls,
        children_by_parent: by_parent,
        control_dispositions,
        change_dispositions: mut dispositions,
        change_memo,
    } = initial;
    let mut table = match evaluate_branch_table(
        BranchTableInputs {
            controls: &controls,
            root_controls: &root_controls,
            children_by_parent: &by_parent,
            additional_prior,
            preliminary_change_dispositions: &dispositions,
        },
        control_dispositions,
        change_memo,
        budget,
        cancellation,
    ) {
        Ok(table) => table,
        Err(BranchTableError::Stop(stop)) => return no_progress_batch_report(stop),
        Err(BranchTableError::Invariant) => {
            return failed_batch_report(EvaluationFailure::InvariantViolation);
        }
    };
    let canonical = match derive_canonical_branch(
        &controls,
        &root_controls,
        &by_parent,
        &mut table,
        &dispositions,
        |counter| charge_prior_knowledge_item(counter, budget, cancellation),
    ) {
        Ok(canonical) => canonical,
        Err(CanonicalBranchError::Work(stop)) => return no_progress_batch_report(stop),
        Err(CanonicalBranchError::Invariant) => {
            return failed_batch_report(EvaluationFailure::InvariantViolation);
        }
    };
    let BranchTableEvaluation {
        states: branch_states,
        valid: _,
        control_dispositions,
        accepted_at_control,
        statefully_valid_controls,
        branch_change_dispositions,
        change_memo,
    } = table;
    let CanonicalBranchEvaluation {
        controls: canonical_controls,
        change_dispositions,
        accepted_changes,
        integrity_alerts,
    } = canonical;
    dispositions = change_dispositions;

    let accepted_projection =
        match project_accepted_candidate_maps_metered(&accepted_changes, &change_memo, |counter| {
            charge_prior_knowledge_item(counter, budget, cancellation)
        }) {
            Ok(projection) => projection,
            Err(AcceptedCandidateProjectionError::Work(stop)) => {
                return no_progress_batch_report(stop);
            }
            Err(AcceptedCandidateProjectionError::Invariant) => {
                return failed_batch_report(EvaluationFailure::InvariantViolation);
            }
        };
    let ordered = match schedule_candidates(
        accepted_projection.candidates,
        BTreeSet::new(),
        budget,
        cancellation,
    ) {
        Ok(schedule) => schedule.ordered,
        Err(error) => {
            let completion = match error {
                ScheduleError::BudgetExhausted => Completion::BudgetExhausted,
                ScheduleError::Cancelled => Completion::Cancelled,
            };
            return no_progress_batch_report(completion);
        }
    };
    let raw_changes = accepted_projection.raw_changes;
    let can_materialize = raw_changes.len() == accepted_changes.len();
    let materialized = if can_materialize {
        match materialize_history_metered(&raw_changes, &ordered, budget, cancellation) {
            Ok(document) => Some(document),
            Err(ExactApplyError::Budget) => {
                return no_progress_batch_report(Completion::BudgetExhausted);
            }
            Err(ExactApplyError::Cancelled) => {
                return no_progress_batch_report(Completion::Cancelled);
            }
            Err(_) => {
                return failed_batch_report(EvaluationFailure::Apply);
            }
        }
    } else {
        None
    };
    let derived_heads = match derive_heads_metered(&accepted_changes, &change_memo, |counter| {
        charge_prior_knowledge_item(counter, budget, cancellation)
    }) {
        Ok(heads) => heads,
        Err(HeadDerivationError::Work(stop)) => {
            return no_progress_batch_report(stop);
        }
        Err(HeadDerivationError::Invariant) => {
            return failed_batch_report(EvaluationFailure::InvariantViolation);
        }
    };
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
        completion: Completion::Complete,
        failure: None,
    }
}

fn prepare_initial_maps_metered<E>(
    collected: Vec<BatchControl>,
    mut control_dispositions: BTreeMap<EventId, ProtocolDisposition>,
    mut visit: impl FnMut(WorkCounter) -> Result<(), E>,
) -> Result<InitialBatchMaps, InitialMapBuildError<E>> {
    let control_count = collected.len();
    let mut controls = BTreeMap::new();
    let mut change_dispositions = BTreeMap::new();
    let mut change_memo = BatchChangeMemo::default();
    let mut collected_items = collected.into_iter();
    for _ in 0..control_count {
        visit(WorkCounter::Control).map_err(InitialMapBuildError::Work)?;
        let Some(control) = collected_items.next() else {
            return Err(InitialMapBuildError::Invariant);
        };
        change_memo.record_control_metered(&control, &mut change_dispositions, &mut visit)?;
        visit(WorkCounter::Control).map_err(InitialMapBuildError::Work)?;
        controls.insert(control.event_id, control);
    }

    let mut root_controls = BTreeSet::new();
    let mut children_by_parent = BTreeMap::<EventId, BTreeSet<EventId>>::new();
    let mut parent_items = controls.values();
    for _ in 0..controls.len() {
        visit(WorkCounter::Control).map_err(InitialMapBuildError::Work)?;
        let Some(control) = parent_items.next() else {
            return Err(InitialMapBuildError::Invariant);
        };
        if let Some(parent) = control.parent {
            children_by_parent
                .entry(parent)
                .or_default()
                .insert(control.event_id);
        } else {
            root_controls.insert(control.event_id);
        }
    }

    let mut control_ids = controls.keys();
    for _ in 0..controls.len() {
        visit(WorkCounter::Control).map_err(InitialMapBuildError::Work)?;
        let Some(event_id) = control_ids.next() else {
            return Err(InitialMapBuildError::Invariant);
        };
        control_dispositions.insert(*event_id, ProtocolDisposition::Excluded);
    }

    Ok(InitialBatchMaps {
        controls,
        root_controls,
        children_by_parent,
        control_dispositions,
        change_dispositions,
        change_memo,
    })
}

fn clone_hash_set_metered<E>(
    source: &BTreeSet<ChangeHash>,
    mut visit: impl FnMut(WorkCounter) -> Result<(), E>,
) -> Result<BTreeSet<ChangeHash>, BranchDeltaError<E>> {
    let mut cloned = BTreeSet::new();
    let mut items = source.iter();
    for _ in 0..source.len() {
        visit(WorkCounter::GraphNode).map_err(BranchDeltaError::Work)?;
        let Some(hash) = items.next() else {
            return Err(BranchDeltaError::Invariant);
        };
        visit(WorkCounter::GraphNode).map_err(BranchDeltaError::Work)?;
        cloned.insert(*hash);
    }
    Ok(cloned)
}

struct ValidBranchEvaluation {
    epoch: EpochEvaluationResult,
    change_dispositions: PersistentDeltaMap<ChangeHash, ProtocolDisposition>,
    validated_base: BTreeSet<ChangeHash>,
    ancestry: ControlAncestry,
    prior_knowledge: PriorKnowledgeState,
}

#[derive(Default)]
struct BranchTableEvaluation {
    states: BTreeMap<EventId, BranchEvaluationState>,
    valid: BTreeMap<EventId, ValidBranchEvaluation>,
    control_dispositions: BTreeMap<EventId, ProtocolDisposition>,
    accepted_at_control: BTreeMap<EventId, AcceptedAtControl>,
    statefully_valid_controls: BTreeSet<EventId>,
    branch_change_dispositions:
        BTreeMap<EventId, PersistentDeltaMap<ChangeHash, ProtocolDisposition>>,
    change_memo: BatchChangeMemo,
}

struct CanonicalBranchEvaluation {
    controls: Vec<EventId>,
    change_dispositions: BTreeMap<ChangeHash, ProtocolDisposition>,
    accepted_changes: BTreeSet<ChangeHash>,
    integrity_alerts: Vec<IntegrityAlert>,
}

struct AcceptedCandidateProjection {
    candidates: Vec<ChangeCandidate>,
    raw_changes: BTreeMap<ChangeHash, Arc<[u8]>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AcceptedCandidateProjectionError<E> {
    Work(E),
    Invariant,
}

#[derive(Default)]
struct BatchChangeMemo {
    candidates: BTreeMap<ChangeHash, ChangeCandidate>,
    raw_changes: BTreeMap<ChangeHash, Arc<[u8]>>,
    hashes_by_control: BTreeMap<EventId, BTreeSet<ChangeHash>>,
    controls_by_hash: BTreeMap<ChangeHash, BTreeSet<EventId>>,
}

fn project_accepted_candidate_maps_metered<E>(
    accepted: &BTreeSet<ChangeHash>,
    memo: &BatchChangeMemo,
    mut visit: impl FnMut(WorkCounter) -> Result<(), E>,
) -> Result<AcceptedCandidateProjection, AcceptedCandidateProjectionError<E>> {
    let mut candidates = Vec::new();
    let mut raw_changes = BTreeMap::new();
    let mut accepted_items = accepted.iter();
    for _ in 0..accepted.len() {
        visit(WorkCounter::GraphNode).map_err(AcceptedCandidateProjectionError::Work)?;
        let Some(hash) = accepted_items.next() else {
            return Err(AcceptedCandidateProjectionError::Invariant);
        };
        visit(WorkCounter::GraphNode).map_err(AcceptedCandidateProjectionError::Work)?;
        let Some(candidate) = memo.candidates.get(hash) else {
            return Err(AcceptedCandidateProjectionError::Invariant);
        };
        visit(WorkCounter::GraphNode).map_err(AcceptedCandidateProjectionError::Work)?;
        candidates.push(candidate.clone());
        visit(WorkCounter::GraphNode).map_err(AcceptedCandidateProjectionError::Work)?;
        if let Some(raw) = memo.raw_changes.get(hash) {
            visit(WorkCounter::GraphNode).map_err(AcceptedCandidateProjectionError::Work)?;
            if raw_changes.insert(*hash, Arc::clone(raw)).is_some() {
                return Err(AcceptedCandidateProjectionError::Invariant);
            }
        }
    }
    Ok(AcceptedCandidateProjection {
        candidates,
        raw_changes,
    })
}

impl BatchChangeMemo {
    fn record_control_metered<E>(
        &mut self,
        control: &BatchControl,
        change_dispositions: &mut BTreeMap<ChangeHash, ProtocolDisposition>,
        mut visit: impl FnMut(WorkCounter) -> Result<(), E>,
    ) -> Result<(), InitialMapBuildError<E>> {
        let mut change_items = control.changes.iter();
        for _ in 0..control.changes.len() {
            visit(WorkCounter::GraphNode).map_err(InitialMapBuildError::Work)?;
            let Some(change) = change_items.next() else {
                return Err(InitialMapBuildError::Invariant);
            };
            let mut dependencies = change.candidate.dependencies.iter();
            for _ in 0..change.candidate.dependencies.len() {
                visit(WorkCounter::GraphEdge).map_err(InitialMapBuildError::Work)?;
                if dependencies.next().is_none() {
                    return Err(InitialMapBuildError::Invariant);
                }
            }
            let hash = change.candidate.change_hash;
            visit(WorkCounter::GraphNode).map_err(InitialMapBuildError::Work)?;
            self.candidates
                .entry(hash)
                .or_insert_with(|| change.candidate.clone());
            if let Some(raw) = &change.raw_change {
                visit(WorkCounter::GraphNode).map_err(InitialMapBuildError::Work)?;
                self.raw_changes
                    .entry(hash)
                    .or_insert_with(|| Arc::clone(raw));
            }
            visit(WorkCounter::GraphNode).map_err(InitialMapBuildError::Work)?;
            let hashes = self.hashes_by_control.entry(control.event_id).or_default();
            visit(WorkCounter::GraphNode).map_err(InitialMapBuildError::Work)?;
            hashes.insert(hash);
            visit(WorkCounter::GraphNode).map_err(InitialMapBuildError::Work)?;
            let controls = self.controls_by_hash.entry(hash).or_default();
            visit(WorkCounter::GraphNode).map_err(InitialMapBuildError::Work)?;
            controls.insert(control.event_id);
            visit(WorkCounter::GraphNode).map_err(InitialMapBuildError::Work)?;
            change_dispositions.insert(hash, ProtocolDisposition::Excluded);
        }
        Ok(())
    }

    fn derive(
        controls: &BTreeMap<EventId, BatchControl>,
        budget: &mut WorkBudget,
        cancellation: &impl CancellationCheck,
    ) -> Result<Self, Completion> {
        let mut memo = Self::default();
        let mut control_items = controls.iter();
        for _ in 0..controls.len() {
            charge_prior_knowledge_item(WorkCounter::Control, budget, cancellation)?;
            let Some((control_id, control)) = control_items.next() else {
                return Err(Completion::BudgetExhausted);
            };
            debug_assert_eq!(*control_id, control.event_id);
            let mut dispositions = BTreeMap::new();
            memo.record_control_metered(control, &mut dispositions, |counter| {
                charge_prior_knowledge_item(counter, budget, cancellation)
            })
            .map_err(|error| match error {
                InitialMapBuildError::Work(stop) => stop,
                InitialMapBuildError::Invariant => Completion::BudgetExhausted,
            })?;
        }
        Ok(memo)
    }
}

fn derive_canonical_branch<E>(
    controls: &BTreeMap<EventId, BatchControl>,
    root_controls: &BTreeSet<EventId>,
    children_by_parent: &BTreeMap<EventId, BTreeSet<EventId>>,
    table: &mut BranchTableEvaluation,
    preliminary_change_dispositions: &BTreeMap<ChangeHash, ProtocolDisposition>,
    mut visit: impl FnMut(WorkCounter) -> Result<(), E>,
) -> Result<CanonicalBranchEvaluation, CanonicalBranchError<E>> {
    let mut change_dispositions = BTreeMap::new();
    let mut preliminary_items = preliminary_change_dispositions.iter();
    for _ in 0..preliminary_change_dispositions.len() {
        visit(WorkCounter::GraphNode).map_err(CanonicalBranchError::Work)?;
        let Some((hash, disposition)) = preliminary_items.next() else {
            return Err(CanonicalBranchError::Invariant);
        };
        change_dispositions.insert(*hash, *disposition);
    }
    let mut canonical_controls = Vec::new();
    let mut canonical_control_ids = BTreeSet::new();
    let mut integrity_alerts = Vec::new();
    let mut seen_alerts = BTreeSet::new();
    let mut parent_id = None;
    let mut children = root_controls;
    let mut final_accepted = None;
    loop {
        let mut valid_children = Vec::new();
        let mut child_items = children.iter();
        for _ in 0..children.len() {
            visit(WorkCounter::Control).map_err(CanonicalBranchError::Work)?;
            let Some(event_id) = child_items.next() else {
                return Err(CanonicalBranchError::Invariant);
            };
            if table.states.get(event_id) == Some(&BranchEvaluationState::Valid) {
                valid_children.push(*event_id);
            }
        }
        visit(WorkCounter::Control).map_err(CanonicalBranchError::Work)?;
        let Some(selected) = valid_children.first().copied() else {
            break;
        };
        if table
            .control_dispositions
            .insert(selected, ProtocolDisposition::Accepted)
            .is_none()
        {
            return Err(CanonicalBranchError::Invariant);
        }
        let branch = table
            .valid
            .get(&selected)
            .ok_or(CanonicalBranchError::Invariant)?;
        let control = controls
            .get(&selected)
            .ok_or(CanonicalBranchError::Invariant)?;
        canonical_controls.push(selected);
        canonical_control_ids.insert(selected);
        final_accepted = Some(branch.epoch.accepted_state().accepted_closure_handle());
        if valid_children.len() > 1 {
            let alert = IntegrityAlert::ControllerEquivocation(
                ControllerEquivocationAlert::from_validated_parts(
                    parent_id,
                    valid_children,
                    selected,
                ),
            );
            seen_alerts.insert(alert.clone());
            integrity_alerts.push(alert);
        }
        let dispositions = branch.epoch.dispositions();
        let mut disposition_items = dispositions.iter();
        for _ in 0..dispositions.len() {
            visit(WorkCounter::GraphNode).map_err(CanonicalBranchError::Work)?;
            let Some((hash, disposition)) = disposition_items.next() else {
                return Err(CanonicalBranchError::Invariant);
            };
            change_dispositions.insert(*hash, *disposition);
        }
        let alerts = branch.epoch.integrity_alerts();
        let mut alert_items = alerts.iter();
        for _ in 0..alerts.len() {
            visit(WorkCounter::Control).map_err(CanonicalBranchError::Work)?;
            let Some(alert) = alert_items.next() else {
                return Err(CanonicalBranchError::Invariant);
            };
            if seen_alerts.insert(alert.clone()) {
                integrity_alerts.push(alert.clone());
            }
        }
        if control.frozen {
            break;
        }
        parent_id = Some(selected);
        visit(WorkCounter::Control).map_err(CanonicalBranchError::Work)?;
        let Some(next_children) = children_by_parent.get(&selected) else {
            break;
        };
        children = next_children;
    }
    let mut valid_items = table.valid.iter();
    for _ in 0..table.valid.len() {
        visit(WorkCounter::Control).map_err(CanonicalBranchError::Work)?;
        let Some((control_id, branch)) = valid_items.next() else {
            return Err(CanonicalBranchError::Invariant);
        };
        if canonical_control_ids.contains(control_id) {
            continue;
        }
        let alerts = branch.epoch.integrity_alerts();
        let mut alert_items = alerts.iter();
        for _ in 0..alerts.len() {
            visit(WorkCounter::Control).map_err(CanonicalBranchError::Work)?;
            let Some(alert) = alert_items.next() else {
                return Err(CanonicalBranchError::Invariant);
            };
            if seen_alerts.insert(alert.clone()) {
                integrity_alerts.push(alert.clone());
            }
        }
    }
    let mut accepted_changes = BTreeSet::new();
    if let Some(accepted) = final_accepted {
        let mut accepted_items = accepted.iter();
        for _ in 0..accepted.len() {
            visit(WorkCounter::GraphNode).map_err(CanonicalBranchError::Work)?;
            let Some(hash) = accepted_items.next() else {
                return Err(CanonicalBranchError::Invariant);
            };
            accepted_changes.insert(*hash);
        }
    }
    let disposition_count = change_dispositions.len();
    let mut disposition_items = change_dispositions.iter_mut();
    for _ in 0..disposition_count {
        visit(WorkCounter::GraphNode).map_err(CanonicalBranchError::Work)?;
        let Some((hash, disposition)) = disposition_items.next() else {
            return Err(CanonicalBranchError::Invariant);
        };
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CanonicalBranchError<E> {
    Work(E),
    Invariant,
}

enum BranchTableError {
    Stop(Completion),
    Invariant,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum BranchDeltaError<E> {
    Work(E),
    Invariant,
}

impl From<Completion> for BranchTableError {
    fn from(value: Completion) -> Self {
        Self::Stop(value)
    }
}

fn branch_delta_error(error: BranchDeltaError<Completion>) -> BranchTableError {
    match error {
        BranchDeltaError::Work(stop) => BranchTableError::Stop(stop),
        BranchDeltaError::Invariant => BranchTableError::Invariant,
    }
}

fn extend_prior_knowledge_metered<E>(
    parent: &PriorKnowledgeState,
    mut local: BTreeMap<ChangeHash, PriorChangeKnowledge>,
    additional: Option<&BTreeMap<ChangeHash, PriorChangeKnowledge>>,
    mut visit: impl FnMut(WorkCounter) -> Result<(), E>,
) -> Result<PriorKnowledgeState, BranchDeltaError<E>> {
    if let Some(additional) = additional {
        let mut items = additional.iter();
        for _ in 0..additional.len() {
            visit(WorkCounter::GraphNode).map_err(BranchDeltaError::Work)?;
            let Some((hash, item)) = items.next() else {
                return Err(BranchDeltaError::Invariant);
            };
            visit(WorkCounter::GraphNode).map_err(BranchDeltaError::Work)?;
            local.entry(*hash).or_insert(*item);
        }
    }
    parent
        .extend_prepared_metered(local, |_| visit(WorkCounter::GraphNode))
        .map_err(BranchDeltaError::Work)
}

fn extend_branch_dispositions_metered<E>(
    parent: &PersistentDeltaMap<ChangeHash, ProtocolDisposition>,
    parent_accepted: Option<&BTreeSet<ChangeHash>>,
    validated_base: &BTreeSet<ChangeHash>,
    epoch_dispositions: &BTreeMap<ChangeHash, ProtocolDisposition>,
    mut visit: impl FnMut(WorkCounter) -> Result<(), E>,
) -> Result<PersistentDeltaMap<ChangeHash, ProtocolDisposition>, BranchDeltaError<E>> {
    let mut local = BTreeMap::new();
    if let Some(parent_accepted) = parent_accepted {
        let mut items = parent_accepted.iter();
        for _ in 0..parent_accepted.len() {
            visit(WorkCounter::GraphNode).map_err(BranchDeltaError::Work)?;
            let Some(hash) = items.next() else {
                return Err(BranchDeltaError::Invariant);
            };
            let disposition = if validated_base.contains(hash) {
                ProtocolDisposition::Accepted
            } else {
                ProtocolDisposition::Excluded
            };
            if parent
                .get_metered(hash, || visit(WorkCounter::GraphNode))
                .map_err(BranchDeltaError::Work)?
                != Some(&disposition)
            {
                visit(WorkCounter::GraphNode).map_err(BranchDeltaError::Work)?;
                local.insert(*hash, disposition);
            }
        }
    }
    let mut validated_items = validated_base.iter();
    for _ in 0..validated_base.len() {
        visit(WorkCounter::GraphNode).map_err(BranchDeltaError::Work)?;
        let Some(hash) = validated_items.next() else {
            return Err(BranchDeltaError::Invariant);
        };
        if parent
            .get_metered(hash, || visit(WorkCounter::GraphNode))
            .map_err(BranchDeltaError::Work)?
            != Some(&ProtocolDisposition::Accepted)
        {
            visit(WorkCounter::GraphNode).map_err(BranchDeltaError::Work)?;
            local.insert(*hash, ProtocolDisposition::Accepted);
        }
    }
    let mut epoch_items = epoch_dispositions.iter();
    for _ in 0..epoch_dispositions.len() {
        visit(WorkCounter::GraphNode).map_err(BranchDeltaError::Work)?;
        let Some((hash, disposition)) = epoch_items.next() else {
            return Err(BranchDeltaError::Invariant);
        };
        if parent
            .get_metered(hash, || visit(WorkCounter::GraphNode))
            .map_err(BranchDeltaError::Work)?
            != Some(disposition)
        {
            visit(WorkCounter::GraphNode).map_err(BranchDeltaError::Work)?;
            local.insert(*hash, *disposition);
        }
    }
    parent
        .extend_prepared_metered(local, |_| visit(WorkCounter::GraphNode))
        .map_err(BranchDeltaError::Work)
}

fn insert_branch_state_metered<E>(
    states: &mut BTreeMap<EventId, BranchEvaluationState>,
    control_dispositions: &mut BTreeMap<EventId, ProtocolDisposition>,
    event_id: EventId,
    state: BranchEvaluationState,
    mut visit: impl FnMut(WorkCounter) -> Result<(), E>,
) -> Result<(), BranchDeltaError<E>> {
    visit(WorkCounter::Control).map_err(BranchDeltaError::Work)?;
    if states.insert(event_id, state).is_some()
        || control_dispositions
            .insert(event_id, state.final_disposition(false))
            .is_none()
    {
        return Err(BranchDeltaError::Invariant);
    }
    Ok(())
}

fn insert_valid_branch_metered<E>(
    table: &mut BranchTableEvaluation,
    event_id: EventId,
    branch: ValidBranchEvaluation,
    mut visit: impl FnMut(WorkCounter) -> Result<(), E>,
) -> Result<(), BranchDeltaError<E>> {
    visit(WorkCounter::Control).map_err(BranchDeltaError::Work)?;
    let accepted_projection = AcceptedAtControl::from_result(&branch.epoch);
    let branch_change_dispositions = branch.change_dispositions.clone();
    if table
        .states
        .insert(event_id, BranchEvaluationState::Valid)
        .is_some()
        || table
            .control_dispositions
            .insert(event_id, ProtocolDisposition::Excluded)
            .is_none()
        || !table.statefully_valid_controls.insert(event_id)
        || table
            .accepted_at_control
            .insert(event_id, accepted_projection)
            .is_some()
        || table
            .branch_change_dispositions
            .insert(event_id, branch_change_dispositions)
            .is_some()
        || table.valid.insert(event_id, branch).is_some()
    {
        return Err(BranchDeltaError::Invariant);
    }
    Ok(())
}

struct BranchTableInputs<'a> {
    controls: &'a BTreeMap<EventId, BatchControl>,
    root_controls: &'a BTreeSet<EventId>,
    children_by_parent: &'a BTreeMap<EventId, BTreeSet<EventId>>,
    additional_prior: &'a BTreeMap<EventId, BTreeMap<ChangeHash, PriorChangeKnowledge>>,
    preliminary_change_dispositions: &'a BTreeMap<ChangeHash, ProtocolDisposition>,
}

fn evaluate_branch_table(
    inputs: BranchTableInputs<'_>,
    control_dispositions: BTreeMap<EventId, ProtocolDisposition>,
    change_memo: BatchChangeMemo,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<BranchTableEvaluation, BranchTableError> {
    let BranchTableInputs {
        controls,
        root_controls,
        children_by_parent,
        additional_prior,
        preliminary_change_dispositions,
    } = inputs;
    let mut table = BranchTableEvaluation {
        control_dispositions,
        change_memo,
        ..BranchTableEvaluation::default()
    };
    let mut accepted_state_cache = BTreeMap::new();
    let mut ready = BTreeSet::new();
    let mut root_items = root_controls.iter();
    for _ in 0..root_controls.len() {
        charge_prior_knowledge_item(WorkCounter::Control, budget, cancellation)?;
        let Some(root) = root_items.next() else {
            return Err(BranchTableError::Invariant);
        };
        ready.insert(*root);
    }
    loop {
        charge_prior_knowledge_item(WorkCounter::Control, budget, cancellation)?;
        let Some(event_id) = ready.pop_first() else {
            break;
        };
        charge_prior_knowledge_item(WorkCounter::Control, budget, cancellation)?;
        let Some(control) = controls.get(&event_id) else {
            insert_branch_state_metered(
                &mut table.states,
                &mut table.control_dispositions,
                event_id,
                BranchEvaluationState::Invalid,
                |counter| charge_prior_knowledge_item(counter, budget, cancellation),
            )
            .map_err(branch_delta_error)?;
            continue;
        };
        let parent_branch = if let Some(parent) = control.parent {
            charge_prior_knowledge_item(WorkCounter::Control, budget, cancellation)?;
            table.valid.get(&parent)
        } else {
            None
        };
        let inherited = if let Some(parent) = control.parent {
            charge_prior_knowledge_item(WorkCounter::Control, budget, cancellation)?;
            table.states.get(&parent).and_then(|state| match state {
                BranchEvaluationState::Pending => Some(BranchEvaluationState::Pending),
                BranchEvaluationState::Invalid => Some(BranchEvaluationState::Invalid),
                BranchEvaluationState::Valid => None,
            })
        } else {
            None
        };
        let validated_base = if let Some(state) = inherited {
            insert_branch_state_metered(
                &mut table.states,
                &mut table.control_dispositions,
                event_id,
                state,
                |counter| charge_prior_knowledge_item(counter, budget, cancellation),
            )
            .map_err(branch_delta_error)?;
            None
        } else if let (Some(parent), Some(branch), Some(child)) = (
            control
                .parent
                .and_then(|parent| controls.get(&parent))
                .and_then(|parent| parent.envelope.as_ref()),
            parent_branch,
            control.envelope.as_ref(),
        ) {
            let mut view = ParentEpochView::from_result_metered(&branch.epoch, || {
                charge_prior_knowledge_item(WorkCounter::GraphNode, budget, cancellation)
            })
            .map_err(|error| match error {
                ParentEpochViewBuildError::Work(stop) => BranchTableError::Stop(stop),
                ParentEpochViewBuildError::Invariant => BranchTableError::Invariant,
            })?;
            view.share_inherited_prior(&branch.prior_knowledge);
            if let Some(parent_id) = control.parent
                && let Some(knowledge) = additional_prior.get(&parent_id)
            {
                view.set_additional_prior_knowledge_metered(knowledge, || {
                    charge_prior_knowledge_item(WorkCounter::GraphNode, budget, cancellation)
                })
                .map_err(|error| match error {
                    ParentEpochViewBuildError::Work(stop) => BranchTableError::Stop(stop),
                    ParentEpochViewBuildError::Invariant => BranchTableError::Invariant,
                })?;
            }
            let mut visit = |counter| charge_prior_knowledge_item(counter, budget, cancellation);
            match evaluate_child_metered(parent, child, &branch.ancestry, &view, &mut visit)? {
                CandidateResult::Valid => Some(
                    crate::control::frontier::accepted_frontier_closure_metered(
                        &child.content().base_heads,
                        view.accepted(),
                        view.dependency_index(),
                        |counter| charge_prior_knowledge_item(counter, budget, cancellation),
                    )?
                    .accepted,
                ),
                CandidateResult::Pending(_) => {
                    insert_branch_state_metered(
                        &mut table.states,
                        &mut table.control_dispositions,
                        event_id,
                        BranchEvaluationState::Pending,
                        |counter| charge_prior_knowledge_item(counter, budget, cancellation),
                    )
                    .map_err(branch_delta_error)?;
                    None
                }
                CandidateResult::Invalid(_) => {
                    insert_branch_state_metered(
                        &mut table.states,
                        &mut table.control_dispositions,
                        event_id,
                        BranchEvaluationState::Invalid,
                        |counter| charge_prior_knowledge_item(counter, budget, cancellation),
                    )
                    .map_err(branch_delta_error)?;
                    None
                }
            }
        } else if control.parent.is_none() {
            Some(if control.envelope.is_some() {
                BTreeSet::new()
            } else {
                clone_hash_set_metered(&control.accepted_base, |counter| {
                    charge_prior_knowledge_item(counter, budget, cancellation)
                })
                .map_err(branch_delta_error)?
            })
        } else if control.envelope.is_none() && parent_branch.is_some() {
            Some(
                clone_hash_set_metered(&control.accepted_base, |counter| {
                    charge_prior_knowledge_item(counter, budget, cancellation)
                })
                .map_err(branch_delta_error)?,
            )
        } else {
            insert_branch_state_metered(
                &mut table.states,
                &mut table.control_dispositions,
                event_id,
                BranchEvaluationState::Invalid,
                |counter| charge_prior_knowledge_item(counter, budget, cancellation),
            )
            .map_err(branch_delta_error)?;
            None
        };

        if let Some(validated_base) = validated_base {
            let parent_epoch = parent_branch.map(|branch| &branch.epoch);
            let Some(accepted_base) = accepted_state_for_closure(
                &validated_base,
                &table.change_memo.candidates,
                parent_epoch,
                &mut accepted_state_cache,
                budget,
                cancellation,
            )?
            else {
                insert_branch_state_metered(
                    &mut table.states,
                    &mut table.control_dispositions,
                    event_id,
                    BranchEvaluationState::Invalid,
                    |counter| charge_prior_knowledge_item(counter, budget, cancellation),
                )
                .map_err(branch_delta_error)?;
                enqueue_children(
                    event_id,
                    children_by_parent,
                    &mut ready,
                    budget,
                    cancellation,
                )?;
                continue;
            };
            let local_knowledge = prior_change_knowledge(
                parent_epoch,
                &validated_base,
                preliminary_change_dispositions,
                event_id,
                &table.change_memo,
                budget,
                cancellation,
            )?;
            let parent_knowledge = parent_branch
                .map(|branch| branch.prior_knowledge.clone())
                .unwrap_or_default();
            let knowledge = extend_prior_knowledge_metered(
                &parent_knowledge,
                local_knowledge,
                additional_prior.get(&event_id),
                |counter| charge_prior_knowledge_item(counter, budget, cancellation),
            )
            .map_err(branch_delta_error)?;
            let parent_ancestry = parent_branch
                .map(|branch| branch.ancestry.clone())
                .unwrap_or_default();
            let ancestry = if let Some(envelope) = control.envelope.as_ref() {
                let Ok(ancestry) = parent_ancestry.push_checked(envelope.clone()) else {
                    insert_branch_state_metered(
                        &mut table.states,
                        &mut table.control_dispositions,
                        event_id,
                        BranchEvaluationState::Invalid,
                        |counter| charge_prior_knowledge_item(counter, budget, cancellation),
                    )
                    .map_err(branch_delta_error)?;
                    enqueue_children(
                        event_id,
                        children_by_parent,
                        &mut ready,
                        budget,
                        cancellation,
                    )?;
                    continue;
                };
                ancestry
            } else {
                parent_ancestry.clone()
            };
            let retained_knowledge = knowledge.clone();
            match resolve_authoritative_epoch(
                control,
                accepted_base,
                knowledge,
                parent_ancestry,
                &table.change_memo.raw_changes,
                budget,
                cancellation,
            ) {
                Ok(epoch) => {
                    let parent_change_dispositions = parent_branch
                        .map(|branch| branch.change_dispositions.clone())
                        .unwrap_or_default();
                    let branch_change_dispositions = extend_branch_dispositions_metered(
                        &parent_change_dispositions,
                        parent_branch
                            .map(|parent| parent.epoch.accepted_state().accepted_closure()),
                        &validated_base,
                        epoch.dispositions(),
                        |counter| charge_prior_knowledge_item(counter, budget, cancellation),
                    )
                    .map_err(branch_delta_error)?;
                    insert_valid_branch_metered(
                        &mut table,
                        event_id,
                        ValidBranchEvaluation {
                            epoch,
                            change_dispositions: branch_change_dispositions,
                            validated_base,
                            ancestry,
                            prior_knowledge: retained_knowledge,
                        },
                        |counter| charge_prior_knowledge_item(counter, budget, cancellation),
                    )
                    .map_err(branch_delta_error)?;
                }
                Err(EpochResolutionError::Schedule(ScheduleError::BudgetExhausted)) => {
                    return Err(Completion::BudgetExhausted.into());
                }
                Err(EpochResolutionError::Schedule(ScheduleError::Cancelled)) => {
                    return Err(Completion::Cancelled.into());
                }
                Err(EpochResolutionError::InvalidState) => {
                    insert_branch_state_metered(
                        &mut table.states,
                        &mut table.control_dispositions,
                        event_id,
                        BranchEvaluationState::Invalid,
                        |counter| charge_prior_knowledge_item(counter, budget, cancellation),
                    )
                    .map_err(branch_delta_error)?;
                }
            }
        }
        enqueue_children(
            event_id,
            children_by_parent,
            &mut ready,
            budget,
            cancellation,
        )?;
    }
    Ok(table)
}

fn enqueue_children(
    parent: EventId,
    children_by_parent: &BTreeMap<EventId, BTreeSet<EventId>>,
    ready: &mut BTreeSet<EventId>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<(), BranchTableError> {
    charge_prior_knowledge_item(WorkCounter::Control, budget, cancellation)?;
    if let Some(children) = children_by_parent.get(&parent) {
        let mut child_items = children.iter();
        for _ in 0..children.len() {
            charge_prior_knowledge_item(WorkCounter::Control, budget, cancellation)?;
            let Some(child) = child_items.next() else {
                return Err(BranchTableError::Invariant);
            };
            ready.insert(*child);
        }
    }
    Ok(())
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

enum EpochResolutionError {
    Schedule(ScheduleError),
    InvalidState,
}

fn collect_control_candidates_metered(
    control: &BatchControl,
    accepted_base: &BTreeSet<ChangeHash>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<BTreeMap<ChangeHash, ChangeCandidate>, Completion> {
    let mut candidates = BTreeMap::new();
    let mut changes = control.changes.iter();
    for _ in 0..control.changes.len() {
        charge_prior_knowledge_item(WorkCounter::GraphNode, budget, cancellation)?;
        let Some(change) = changes.next() else {
            return Err(Completion::BudgetExhausted);
        };
        let hash = change.candidate.change_hash;
        if !accepted_base.contains(&hash) {
            candidates.insert(hash, change.candidate.clone());
        }
    }
    Ok(candidates)
}

fn resolve_authoritative_epoch(
    control: &BatchControl,
    accepted_base: Arc<AcceptedEpochState>,
    prior_change_knowledge: PriorKnowledgeState,
    ancestry: ControlAncestry,
    raw_changes: &BTreeMap<ChangeHash, Arc<[u8]>>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<EpochEvaluationResult, EpochResolutionError> {
    let Some(selected) = control.envelope.clone() else {
        let local_candidates = collect_control_candidates_metered(
            control,
            accepted_base.accepted_closure(),
            budget,
            cancellation,
        )
        .map_err(epoch_resolution_stop)?;
        let mut epoch_inputs = Vec::with_capacity(local_candidates.len());
        let mut change_iter = control.changes.iter();
        for _ in 0..control.changes.len() {
            charge_prior_knowledge_item(WorkCounter::GraphNode, budget, cancellation)
                .map_err(epoch_resolution_stop)?;
            let Some(change) = change_iter.next() else {
                return Err(EpochResolutionError::InvalidState);
            };
            let candidate = &change.candidate;
            if accepted_base
                .accepted_closure()
                .contains(&candidate.change_hash)
            {
                continue;
            }
            let mut dependencies_valid = true;
            let mut dependency_iter = candidate.dependencies.iter();
            for _ in 0..candidate.dependencies.len() {
                charge_prior_knowledge_item(WorkCounter::GraphEdge, budget, cancellation)
                    .map_err(epoch_resolution_stop)?;
                let Some(dependency) = dependency_iter.next() else {
                    return Err(EpochResolutionError::InvalidState);
                };
                if prior_change_knowledge
                    .get_metered(dependency, || {
                        charge_prior_knowledge_item(WorkCounter::GraphNode, budget, cancellation)
                            .map_err(epoch_resolution_stop)
                    })?
                    .is_some_and(|knowledge| knowledge.is_known_impossible())
                {
                    dependencies_valid = false;
                }
            }
            epoch_inputs.push(EpochCandidate {
                candidate: candidate.clone(),
                semantically_valid: change.legacy_eligible && dependencies_valid,
                canonical_control: !control.frozen,
            });
        }
        let mut dispositions = resolve_epoch(
            epoch_inputs,
            accepted_base.accepted_closure(),
            budget,
            cancellation,
        )
        .map_err(EpochResolutionError::Schedule)?;
        let all_candidates = clone_candidate_maps_metered(
            accepted_base.accepted_candidates(),
            &local_candidates,
            budget,
            cancellation,
        )
        .map_err(EpochResolutionError::Schedule)?;
        let eligible = collect_eligible_candidates_metered(
            &all_candidates,
            accepted_base.accepted_closure(),
            &dispositions,
            budget,
            cancellation,
        )
        .map_err(EpochResolutionError::Schedule)?;
        let graph = build_graph_metered(&eligible, accepted_base.accepted_closure(), |counter| {
            charge_prior_knowledge_item(counter, budget, cancellation).map_err(|stop| match stop {
                Completion::BudgetExhausted => ScheduleError::BudgetExhausted,
                Completion::Cancelled => ScheduleError::Cancelled,
                Completion::Complete => ScheduleError::BudgetExhausted,
            })
        })
        .map_err(|error| match error {
            MeteredGraphBuildError::Work(stop) => EpochResolutionError::Schedule(stop),
            MeteredGraphBuildError::Graph(_) => EpochResolutionError::InvalidState,
        })?;
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
        let (accepted_closure, accepted_candidates) = project_accepted_candidates_metered(
            &all_candidates,
            accepted_base.accepted_closure(),
            &dispositions,
            budget,
            cancellation,
        )
        .map_err(EpochResolutionError::Schedule)?;
        if metered_hash_sets_equal(
            &accepted_closure,
            accepted_base.accepted_closure(),
            budget,
            cancellation,
        )
        .map_err(epoch_resolution_stop)?
        {
            return Ok(EpochEvaluationResult::from_shared_state(
                accepted_base,
                dispositions,
                quarantine.alerts,
            ));
        }
        let accepted_state = AcceptedEpochState::new_metered(
            Arc::new(accepted_closure),
            accepted_candidates,
            None,
            |counter| charge_prior_knowledge_item(counter, budget, cancellation),
        )
        .map_err(|error| match error {
            MeteredAcceptedEpochStateError::Work(stop) => epoch_resolution_stop(stop),
            MeteredAcceptedEpochStateError::State(_) => EpochResolutionError::InvalidState,
        })?;
        return Ok(EpochEvaluationResult::from_shared_state(
            Arc::new(accepted_state),
            dispositions,
            quarantine.alerts,
        ));
    };
    let epoch_changes = collect_control_candidates_metered(
        control,
        accepted_base.accepted_closure(),
        budget,
        cancellation,
    )
    .map_err(epoch_resolution_stop)?;
    let input = EpochEvaluationInput::new_with_metered_candidate_map(
        selected,
        accepted_base,
        epoch_changes,
        raw_changes,
        ancestry,
        prior_change_knowledge,
        |counter| {
            charge_prior_knowledge_item(counter, budget, cancellation).map_err(|stop| match stop {
                Completion::BudgetExhausted => ScheduleError::BudgetExhausted,
                Completion::Cancelled => ScheduleError::Cancelled,
                Completion::Complete => ScheduleError::BudgetExhausted,
            })
        },
    )
    .map_err(|error| match error {
        MeteredEpochEvaluationInputError::Work(stop) => EpochResolutionError::Schedule(stop),
        MeteredEpochEvaluationInputError::Input(_) => EpochResolutionError::InvalidState,
    })?;
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

const fn epoch_resolution_stop(stop: Completion) -> EpochResolutionError {
    match stop {
        Completion::BudgetExhausted => {
            EpochResolutionError::Schedule(ScheduleError::BudgetExhausted)
        }
        Completion::Cancelled => EpochResolutionError::Schedule(ScheduleError::Cancelled),
        Completion::Complete => EpochResolutionError::InvalidState,
    }
}

fn prior_change_knowledge(
    parent: Option<&EpochEvaluationResult>,
    selected_base: &BTreeSet<ChangeHash>,
    dispositions: &BTreeMap<ChangeHash, ProtocolDisposition>,
    selected_control: EventId,
    memo: &BatchChangeMemo,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<BTreeMap<ChangeHash, PriorChangeKnowledge>, Completion> {
    let mut knowledge = BTreeMap::new();
    for hash in selected_base {
        charge_prior_knowledge_item(WorkCounter::GraphNode, budget, cancellation)?;
        knowledge.insert(*hash, PriorChangeKnowledge::AcceptedInBase);
    }
    if let Some(hashes) = memo.hashes_by_control.get(&selected_control) {
        for hash in hashes {
            charge_prior_knowledge_item(WorkCounter::GraphNode, budget, cancellation)?;
            knowledge
                .entry(*hash)
                .or_insert(PriorChangeKnowledge::SameEpochCandidate);
        }
    }
    if let Some(parent) = parent {
        for hash in parent.accepted_state().accepted_closure() {
            charge_prior_knowledge_item(WorkCounter::GraphNode, budget, cancellation)?;
            if !selected_base.contains(hash) {
                knowledge.insert(*hash, PriorChangeKnowledge::PrunedCanonicalAncestor);
            }
        }
    }
    for (hash, control_ids) in &memo.controls_by_hash {
        charge_prior_knowledge_item(WorkCounter::GraphNode, budget, cancellation)?;
        let mut other_control = false;
        for control_id in control_ids {
            charge_prior_knowledge_item(WorkCounter::Control, budget, cancellation)?;
            if *control_id != selected_control {
                other_control = true;
                break;
            }
        }
        if other_control {
            knowledge
                .entry(*hash)
                .or_insert(PriorChangeKnowledge::KnownOtherControl);
        }
    }
    for (hash, disposition) in dispositions {
        charge_prior_knowledge_item(WorkCounter::GraphNode, budget, cancellation)?;
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
    Ok(knowledge)
}

fn charge_prior_knowledge_item(
    counter: WorkCounter,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<(), Completion> {
    if cancellation.is_cancelled() {
        return Err(Completion::Cancelled);
    }
    budget
        .charge(counter, 1)
        .map_err(|_| Completion::BudgetExhausted)
}

fn accepted_state_for_closure(
    accepted: &BTreeSet<ChangeHash>,
    candidates_by_hash: &BTreeMap<ChangeHash, ChangeCandidate>,
    parent: Option<&EpochEvaluationResult>,
    cache: &mut BTreeMap<Arc<BTreeSet<ChangeHash>>, Arc<AcceptedEpochState>>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<Option<Arc<AcceptedEpochState>>, Completion> {
    let mut cache_key = BTreeSet::new();
    for hash in accepted {
        charge_prior_knowledge_item(WorkCounter::GraphNode, budget, cancellation)?;
        cache_key.insert(*hash);
    }
    let cache_key = Arc::new(cache_key);
    if let Some(cached) = cache.get(cache_key.as_ref()) {
        return Ok(Some(Arc::clone(cached)));
    }
    if let Some(parent) = parent
        && metered_hash_sets_equal(
            parent.accepted_state().accepted_closure(),
            accepted,
            budget,
            cancellation,
        )?
    {
        let shared = parent.accepted_state_handle();
        charge_prior_knowledge_item(WorkCounter::GraphNode, budget, cancellation)?;
        cache.insert(Arc::clone(&cache_key), Arc::clone(&shared));
        return Ok(Some(shared));
    }
    let mut candidates = BTreeMap::new();
    for hash in accepted {
        charge_prior_knowledge_item(WorkCounter::GraphNode, budget, cancellation)?;
        let Some(candidate) = candidates_by_hash.get(hash) else {
            return Ok(None);
        };
        candidates.insert(*hash, candidate.clone());
    }
    let materialized = if accepted.is_empty() {
        MaterializedDocumentView::empty().ok()
    } else {
        None
    };
    let state = match AcceptedEpochState::new_metered(
        Arc::clone(&cache_key),
        candidates,
        materialized,
        |counter| charge_prior_knowledge_item(counter, budget, cancellation),
    ) {
        Ok(state) => state,
        Err(MeteredAcceptedEpochStateError::Work(stop)) => return Err(stop),
        Err(MeteredAcceptedEpochStateError::State(_)) => return Ok(None),
    };
    let state = Arc::new(state);
    charge_prior_knowledge_item(WorkCounter::GraphNode, budget, cancellation)?;
    cache.insert(cache_key, Arc::clone(&state));
    Ok(Some(state))
}

fn metered_hash_sets_equal(
    left: &BTreeSet<ChangeHash>,
    right: &BTreeSet<ChangeHash>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<bool, Completion> {
    if left.len() != right.len() {
        return Ok(false);
    }
    let mut left_iter = left.iter();
    let mut right_iter = right.iter();
    for _ in 0..left.len() {
        charge_prior_knowledge_item(WorkCounter::GraphNode, budget, cancellation)?;
        if left_iter.next() != right_iter.next() {
            return Ok(false);
        }
    }
    Ok(true)
}

fn applied_heads_agree(
    derived_heads: &BTreeSet<ChangeHash>,
    applied_heads: &BTreeSet<ChangeHash>,
) -> bool {
    derived_heads == applied_heads
}

impl BatchEvaluationReport {
    pub(crate) fn referenced_branch_change_disposition_metered<E>(
        &self,
        control: EventId,
        hash: ChangeHash,
        mut visit: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<ProtocolDisposition>, E> {
        visit()?;
        let Some(dispositions) = self.branch_change_dispositions.get(&control) else {
            return Ok(None);
        };
        Ok(dispositions.get_metered(&hash, visit)?.copied())
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HeadDerivationError<E> {
    Work(E),
    Invariant,
}

fn derive_heads_metered<E>(
    accepted: &BTreeSet<ChangeHash>,
    memo: &BatchChangeMemo,
    mut visit: impl FnMut(WorkCounter) -> Result<(), E>,
) -> Result<BTreeSet<ChangeHash>, HeadDerivationError<E>> {
    let mut dependencies = BTreeSet::new();
    let mut accepted_items = accepted.iter();
    for _ in 0..accepted.len() {
        visit(WorkCounter::GraphNode).map_err(HeadDerivationError::Work)?;
        let Some(hash) = accepted_items.next() else {
            return Err(HeadDerivationError::Invariant);
        };
        visit(WorkCounter::GraphNode).map_err(HeadDerivationError::Work)?;
        let Some(candidate) = memo.candidates.get(hash) else {
            return Err(HeadDerivationError::Invariant);
        };
        let mut dependency_items = candidate.dependencies.iter();
        for _ in 0..candidate.dependencies.len() {
            visit(WorkCounter::GraphEdge).map_err(HeadDerivationError::Work)?;
            let Some(dependency) = dependency_items.next() else {
                return Err(HeadDerivationError::Invariant);
            };
            visit(WorkCounter::GraphNode).map_err(HeadDerivationError::Work)?;
            dependencies.insert(*dependency);
        }
    }

    let mut heads = BTreeSet::new();
    let mut accepted_items = accepted.iter();
    for _ in 0..accepted.len() {
        visit(WorkCounter::GraphNode).map_err(HeadDerivationError::Work)?;
        let Some(hash) = accepted_items.next() else {
            return Err(HeadDerivationError::Invariant);
        };
        visit(WorkCounter::GraphNode).map_err(HeadDerivationError::Work)?;
        if dependencies.contains(hash) {
            continue;
        }
        visit(WorkCounter::GraphNode).map_err(HeadDerivationError::Work)?;
        heads.insert(*hash);
    }
    Ok(heads)
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::collections::{BTreeMap, BTreeSet};
    use std::sync::Arc;

    use super::{
        AcceptedAtControl, BatchChange, BatchChangeMemo, BatchControl, BatchEvaluationReport,
        BranchDeltaError, BranchEvaluationState, BranchTableEvaluation, InitialMapBuildError,
        PersistentDeltaMap, PriorChangeKnowledge, PriorKnowledgeState, ValidBranchEvaluation,
        accepted_state_for_closure, charge_prior_knowledge_item, clone_hash_set_metered,
        derive_canonical_branch, derive_heads_metered, empty_batch_report, evaluate_batch,
        extend_branch_dispositions_metered, extend_prior_knowledge_metered,
        insert_branch_state_metered, insert_valid_branch_metered, prepare_initial_maps_metered,
        prior_change_knowledge, project_accepted_candidate_maps_metered,
        propagate_control_parent_dispositions,
    };

    fn empty_valid_branch() -> ValidBranchEvaluation {
        ValidBranchEvaluation {
            epoch: super::EpochEvaluationResult::new(
                BTreeSet::new(),
                BTreeSet::new(),
                BTreeMap::new(),
                BTreeMap::new(),
                Vec::new(),
                None,
            )
            .unwrap_or_else(|_| unreachable!("empty epoch result is valid")),
            change_dispositions: PersistentDeltaMap::default(),
            validated_base: BTreeSet::new(),
            ancestry: super::ControlAncestry::default(),
            prior_knowledge: PriorKnowledgeState::default(),
        }
    }

    #[test]
    fn referenced_disposition_lookup_exposes_every_persistent_node() {
        const DEPTH: u8 = 64;
        let control = EventId::from_bytes([90; 32]);
        let target = ChangeHash::from_bytes([0; 32]);
        let mut dispositions =
            PersistentDeltaMap::from(BTreeMap::from([(target, ProtocolDisposition::Accepted)]));
        for value in 1..DEPTH {
            dispositions = dispositions.extend_local(BTreeMap::from([(
                ChangeHash::from_bytes([value; 32]),
                ProtocolDisposition::Excluded,
            )]));
        }
        let mut report = empty_batch_report(Completion::Complete, None);
        report
            .branch_change_dispositions
            .insert(control, dispositions);
        let exact = 1_usize + usize::from(DEPTH);

        for completion in [Completion::BudgetExhausted, Completion::Cancelled] {
            for capacity in 0..=exact + 1 {
                let observed = Cell::new(0_usize);
                let result =
                    report.referenced_branch_change_disposition_metered(control, target, || {
                        if observed.get() == capacity {
                            return Err(completion);
                        }
                        observed.set(observed.get() + 1);
                        Ok(())
                    });
                if capacity < exact {
                    assert_eq!(result, Err(completion));
                    assert_eq!(observed.get(), capacity);
                } else {
                    assert_eq!(result, Ok(Some(ProtocolDisposition::Accepted)));
                    assert_eq!(observed.get(), exact);
                }
            }
        }
    }

    #[test]
    fn branch_local_extension_owns_preparation_and_publication() {
        const DEPTH: u8 = 64;
        let target = ChangeHash::from_bytes([0; 32]);
        let mut parent_prior = PriorKnowledgeState::from(BTreeMap::from([(
            target,
            PriorChangeKnowledge::KnownInvalid,
        )]));
        let mut parent_dispositions =
            PersistentDeltaMap::from(BTreeMap::from([(target, ProtocolDisposition::Excluded)]));
        for value in 1..DEPTH {
            let hash = ChangeHash::from_bytes([value; 32]);
            parent_prior = parent_prior.extend_local(BTreeMap::from([(
                hash,
                PriorChangeKnowledge::KnownOtherControl,
            )]));
            parent_dispositions = parent_dispositions
                .extend_local(BTreeMap::from([(hash, ProtocolDisposition::Excluded)]));
        }

        let additional = BTreeMap::from([(target, PriorChangeKnowledge::AcceptedInBase)]);
        let prior_exact = 2_usize + 1 + usize::from(DEPTH) + 1;
        for completion in [Completion::BudgetExhausted, Completion::Cancelled] {
            for capacity in 0..=prior_exact + 1 {
                let observed = Cell::new(0_usize);
                let result = extend_prior_knowledge_metered(
                    &parent_prior,
                    BTreeMap::new(),
                    Some(&additional),
                    |_| {
                        if observed.get() == capacity {
                            return Err(completion);
                        }
                        observed.set(observed.get() + 1);
                        Ok(())
                    },
                );
                if capacity < prior_exact {
                    assert!(
                        matches!(result, Err(BranchDeltaError::Work(stop)) if stop == completion)
                    );
                    assert_eq!(observed.get(), capacity);
                    assert_eq!(
                        parent_prior.get(&target),
                        Some(&PriorChangeKnowledge::KnownInvalid)
                    );
                } else {
                    assert!(
                        matches!(result, Ok(ref state) if state.get(&target) == Some(&PriorChangeKnowledge::AcceptedInBase))
                    );
                    assert_eq!(observed.get(), prior_exact);
                }
            }
        }

        let validated = BTreeSet::from([target]);
        let disposition_exact = 2 * (1_usize + usize::from(DEPTH) + 1);
        for completion in [Completion::BudgetExhausted, Completion::Cancelled] {
            for capacity in 0..=disposition_exact + 1 {
                let observed = Cell::new(0_usize);
                let result = extend_branch_dispositions_metered(
                    &parent_dispositions,
                    None,
                    &validated,
                    &BTreeMap::new(),
                    |_| {
                        if observed.get() == capacity {
                            return Err(completion);
                        }
                        observed.set(observed.get() + 1);
                        Ok(())
                    },
                );
                if capacity < disposition_exact {
                    assert!(
                        matches!(result, Err(BranchDeltaError::Work(stop)) if stop == completion)
                    );
                    assert_eq!(observed.get(), capacity);
                    assert_eq!(
                        parent_dispositions.get(&target),
                        Some(&ProtocolDisposition::Excluded)
                    );
                } else {
                    assert!(
                        matches!(result, Ok(ref state) if state.get(&target) == Some(&ProtocolDisposition::Accepted))
                    );
                    assert_eq!(observed.get(), disposition_exact);
                }
            }
        }
    }
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

    #[test]
    fn accepted_state_cache_is_shared_and_charged_per_key_and_insert() {
        let mut retained = candidate(1, 1, 1, 1);
        retained.change_hash = ChangeHash::from_bytes([7; 32]);
        let accepted = BTreeSet::from([retained.change_hash]);
        let candidates = BTreeMap::from([(retained.change_hash, retained)]);
        let mut cache = BTreeMap::new();
        let mut exact = WorkBudget::new(0, 17);

        let first = accepted_state_for_closure(
            &accepted,
            &candidates,
            None,
            &mut cache,
            &mut exact,
            &NeverCancelled,
        )
        .ok()
        .flatten();
        let second = accepted_state_for_closure(
            &accepted,
            &candidates,
            None,
            &mut cache,
            &mut exact,
            &NeverCancelled,
        )
        .ok()
        .flatten();
        assert!(matches!((&first, &second), (Some(left), Some(right)) if Arc::ptr_eq(left, right)));
        assert_eq!(exact.consumed().get(WorkCounter::GraphNode), 17);

        let mut insufficient_cache = BTreeMap::new();
        let mut insufficient = WorkBudget::new(0, 15);
        assert!(matches!(
            accepted_state_for_closure(
                &accepted,
                &candidates,
                None,
                &mut insufficient_cache,
                &mut insufficient,
                &NeverCancelled,
            ),
            Err(Completion::BudgetExhausted)
        ));
        assert_eq!(insufficient.consumed().get(WorkCounter::GraphNode), 15);
        assert!(insufficient_cache.is_empty());

        let mut cancelled_cache = BTreeMap::new();
        let mut cancelled = WorkBudget::new(0, 16);
        assert!(matches!(
            accepted_state_for_closure(
                &accepted,
                &candidates,
                None,
                &mut cancelled_cache,
                &mut cancelled,
                &|| true,
            ),
            Err(Completion::Cancelled)
        ));
        assert_eq!(cancelled.consumed().get(WorkCounter::GraphNode), 0);
    }

    #[test]
    fn prior_knowledge_is_charged_per_item_before_access() {
        let selected_control = EventId::from_bytes([7; 32]);
        let other_control = EventId::from_bytes([8; 32]);
        let accepted = ChangeHash::from_bytes([1; 32]);
        let same_epoch = ChangeHash::from_bytes([2; 32]);
        let other = ChangeHash::from_bytes([3; 32]);
        let invalid = ChangeHash::from_bytes([4; 32]);
        let excluded = ChangeHash::from_bytes([5; 32]);
        let memo = BatchChangeMemo {
            hashes_by_control: BTreeMap::from([(selected_control, BTreeSet::from([same_epoch]))]),
            controls_by_hash: BTreeMap::from([(other, BTreeSet::from([other_control]))]),
            ..BatchChangeMemo::default()
        };
        let dispositions = BTreeMap::from([
            (invalid, ProtocolDisposition::Invalid),
            (excluded, ProtocolDisposition::Excluded),
        ]);
        let selected_base = BTreeSet::from([accepted]);

        let mut exact = WorkBudget::new(0, 6);
        let knowledge = prior_change_knowledge(
            None,
            &selected_base,
            &dispositions,
            selected_control,
            &memo,
            &mut exact,
            &NeverCancelled,
        );
        assert!(knowledge.is_ok());
        let knowledge = knowledge.unwrap_or_default();
        assert_eq!(exact.consumed().get(WorkCounter::GraphNode), 5);
        assert_eq!(exact.consumed().get(WorkCounter::Control), 1);
        assert_eq!(
            knowledge.get(&accepted),
            Some(&PriorChangeKnowledge::AcceptedInBase)
        );
        assert_eq!(
            knowledge.get(&same_epoch),
            Some(&PriorChangeKnowledge::SameEpochCandidate)
        );
        assert_eq!(
            knowledge.get(&other),
            Some(&PriorChangeKnowledge::KnownOtherControl)
        );
        assert_eq!(
            knowledge.get(&invalid),
            Some(&PriorChangeKnowledge::KnownInvalid)
        );
        assert_eq!(
            knowledge.get(&excluded),
            Some(&PriorChangeKnowledge::PriorEquivocationExcluded)
        );

        let mut insufficient = WorkBudget::new(0, 5);
        assert_eq!(
            prior_change_knowledge(
                None,
                &selected_base,
                &dispositions,
                selected_control,
                &memo,
                &mut insufficient,
                &NeverCancelled,
            ),
            Err(Completion::BudgetExhausted)
        );
        assert_eq!(insufficient.consumed().get(WorkCounter::GraphNode), 4);
        assert_eq!(insufficient.consumed().get(WorkCounter::Control), 1);

        let mut cancelled = WorkBudget::new(0, 6);
        assert_eq!(
            prior_change_knowledge(
                None,
                &selected_base,
                &dispositions,
                selected_control,
                &memo,
                &mut cancelled,
                &|| true,
            ),
            Err(Completion::Cancelled)
        );
        assert_eq!(cancelled.consumed().get(WorkCounter::GraphNode), 0);
        assert_eq!(cancelled.consumed().get(WorkCounter::Control), 0);
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

    #[test]
    fn canonical_branch_projection_charges_before_every_owned_operation() {
        let first = EventId::from_bytes([1; 32]);
        let second = EventId::from_bytes([2; 32]);
        let hash = ChangeHash::from_bytes([3; 32]);
        let fixture = || {
            let controls = BTreeMap::from([
                (first, control(1, None, Vec::new())),
                (second, control(2, None, Vec::new())),
            ]);
            let roots = BTreeSet::from([first, second]);
            let children = BTreeMap::new();
            let table = BranchTableEvaluation {
                states: BTreeMap::from([
                    (first, BranchEvaluationState::Valid),
                    (second, BranchEvaluationState::Valid),
                ]),
                valid: BTreeMap::from([
                    (first, empty_valid_branch()),
                    (second, empty_valid_branch()),
                ]),
                control_dispositions: BTreeMap::from([
                    (first, ProtocolDisposition::Excluded),
                    (second, ProtocolDisposition::Excluded),
                ]),
                ..BranchTableEvaluation::default()
            };
            let dispositions = BTreeMap::from([(hash, ProtocolDisposition::Accepted)]);
            (controls, roots, children, table, dispositions)
        };
        let (controls, roots, children, mut table, dispositions) = fixture();
        let observed = Cell::new(0_usize);
        let ample = derive_canonical_branch(
            &controls,
            &roots,
            &children,
            &mut table,
            &dispositions,
            |_| {
                observed.set(observed.get() + 1);
                Ok::<(), Completion>(())
            },
        )
        .unwrap_or_else(|_| unreachable!("ample projection"));
        let exact = observed.get();
        assert_eq!(ample.controls, vec![first]);
        assert_eq!(
            ample.change_dispositions.get(&hash),
            Some(&ProtocolDisposition::Excluded)
        );
        assert_eq!(ample.integrity_alerts.len(), 1);
        assert_eq!(
            table.control_dispositions.get(&first),
            Some(&ProtocolDisposition::Accepted)
        );

        for stop in [Completion::BudgetExhausted, Completion::Cancelled] {
            for capacity in 0..=exact + 1 {
                let (controls, roots, children, mut table, dispositions) = fixture();
                let observed = Cell::new(0_usize);
                let result = derive_canonical_branch(
                    &controls,
                    &roots,
                    &children,
                    &mut table,
                    &dispositions,
                    |_| {
                        if observed.get() == capacity {
                            return Err(stop);
                        }
                        observed.set(observed.get() + 1);
                        Ok(())
                    },
                );
                if capacity < exact {
                    assert!(
                        matches!(result, Err(super::CanonicalBranchError::Work(value)) if value == stop)
                    );
                    assert_eq!(observed.get(), capacity);
                } else {
                    assert!(result.is_ok());
                    assert_eq!(observed.get(), exact);
                }
            }
        }
    }

    #[test]
    fn initial_evaluator_maps_charge_before_every_item() {
        let controls = || {
            vec![
                control(1, None, vec![change(11, 1, 1)]),
                control(2, Some(1), vec![change(12, 2, 1)]),
            ]
        };
        let retained = EventId::from_bytes([99; 32]);
        let initial_dispositions = || {
            BTreeMap::from([
                (EventId::from_bytes([1; 32]), ProtocolDisposition::Accepted),
                (retained, ProtocolDisposition::Invalid),
            ])
        };
        let mut expected = Vec::new();
        for _ in 0..2 {
            expected.push(WorkCounter::Control);
            expected.extend([WorkCounter::GraphNode; 7]);
            expected.push(WorkCounter::Control);
        }
        expected.extend([WorkCounter::Control; 4]);

        for stop in [Completion::BudgetExhausted, Completion::Cancelled] {
            for capacity in 0..=expected.len() + 1 {
                let observed = Cell::new(0_usize);
                let mut counters = Vec::new();
                let result =
                    prepare_initial_maps_metered(controls(), initial_dispositions(), |counter| {
                        if observed.get() == capacity {
                            return Err(stop);
                        }
                        counters.push(counter);
                        observed.set(observed.get() + 1);
                        Ok(())
                    });
                if capacity < expected.len() {
                    assert!(
                        matches!(result, Err(InitialMapBuildError::Work(value)) if value == stop)
                    );
                    assert_eq!(observed.get(), capacity);
                    assert_eq!(counters, expected[..capacity]);
                } else {
                    assert!(result.is_ok());
                    if let Ok(maps) = result {
                        assert_eq!(observed.get(), expected.len());
                        assert_eq!(counters, expected);
                        assert_eq!(maps.controls.len(), 2);
                        assert_eq!(maps.root_controls.len(), 1);
                        assert_eq!(maps.children_by_parent.len(), 1);
                        assert_eq!(maps.control_dispositions.len(), 3);
                        assert_eq!(
                            maps.control_dispositions.get(&EventId::from_bytes([1; 32])),
                            Some(&ProtocolDisposition::Excluded)
                        );
                        assert_eq!(
                            maps.control_dispositions.get(&retained),
                            Some(&ProtocolDisposition::Invalid)
                        );
                        assert_eq!(maps.change_dispositions.len(), 2);
                    }
                }
            }
        }
    }

    #[test]
    fn branch_table_publication_charges_before_each_insert() {
        let event_id = EventId::from_bytes([19; 32]);
        for stop in [Completion::BudgetExhausted, Completion::Cancelled] {
            let table = || BranchTableEvaluation {
                control_dispositions: BTreeMap::from([(event_id, ProtocolDisposition::Excluded)]),
                ..BranchTableEvaluation::default()
            };
            let mut stopped = table();
            assert_eq!(
                insert_branch_state_metered(
                    &mut stopped.states,
                    &mut stopped.control_dispositions,
                    event_id,
                    BranchEvaluationState::Invalid,
                    |_| Err(stop),
                ),
                Err(BranchDeltaError::Work(stop))
            );
            assert!(stopped.states.is_empty());
            assert_eq!(
                stopped.control_dispositions.get(&event_id),
                Some(&ProtocolDisposition::Excluded)
            );

            for capacity in 0..=2 {
                let mut table = table();
                let observed = Cell::new(0_usize);
                let result =
                    insert_valid_branch_metered(&mut table, event_id, empty_valid_branch(), |_| {
                        if observed.get() == capacity {
                            return Err(stop);
                        }
                        observed.set(observed.get() + 1);
                        Ok(())
                    });
                if capacity < 1 {
                    assert_eq!(result, Err(BranchDeltaError::Work(stop)));
                    assert_eq!(observed.get(), capacity);
                    assert!(table.states.is_empty());
                    assert!(table.valid.is_empty());
                } else {
                    assert_eq!(result, Ok(()));
                    assert_eq!(observed.get(), 1);
                    assert_eq!(table.states.len(), 1);
                    assert_eq!(table.valid.len(), 1);
                    assert_eq!(table.accepted_at_control.len(), 1);
                    assert_eq!(table.statefully_valid_controls.len(), 1);
                    assert_eq!(table.branch_change_dispositions.len(), 1);
                }
            }
        }
    }

    #[test]
    fn accepted_base_projection_charges_before_every_read_and_insert() {
        let source = BTreeSet::from([
            ChangeHash::from_bytes([21; 32]),
            ChangeHash::from_bytes([22; 32]),
            ChangeHash::from_bytes([23; 32]),
        ]);
        let exact = source.len() * 2;
        for stop in [Completion::BudgetExhausted, Completion::Cancelled] {
            for capacity in 0..=exact + 1 {
                let observed = Cell::new(0_usize);
                let result = clone_hash_set_metered(&source, |_| {
                    if observed.get() == capacity {
                        return Err(stop);
                    }
                    observed.set(observed.get() + 1);
                    Ok(())
                });
                if capacity < exact {
                    assert!(matches!(result, Err(BranchDeltaError::Work(value)) if value == stop));
                    assert_eq!(observed.get(), capacity);
                } else {
                    assert_eq!(result, Ok(source.clone()));
                    assert_eq!(observed.get(), exact);
                }
            }
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
    fn control_closure_traversal_stops_before_each_node_operation() {
        let accepted = ChangeHash::from_bytes([9; 32]);
        let view = crate::control::parent_view::ParentEpochView::from_parts_for_test(
            BTreeSet::from([accepted]),
            BTreeSet::from([accepted]),
            BTreeMap::from([(accepted, BTreeSet::new())]),
            BTreeMap::new(),
            BTreeMap::new(),
        );
        let mut budget = WorkBudget::new(0, 1);
        assert_eq!(
            crate::control::frontier::accepted_frontier_closure_metered(
                &[accepted],
                view.accepted(),
                view.dependency_index(),
                |counter| charge_prior_knowledge_item(counter, &mut budget, &NeverCancelled)
            ),
            Err(Completion::BudgetExhausted)
        );
        assert_eq!(budget.consumed().get(WorkCounter::GraphNode), 1);
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
        basic.raw_change = Some(raw.into());
        let mut basic_budget = WorkBudget::new(10_000, 1_000);
        let basic_report = evaluate_batch(
            [control(1, None, vec![basic.clone()])],
            &mut basic_budget,
            &NeverCancelled,
        );
        assert_eq!(basic_report.completion, Completion::Complete);
        assert_eq!(
            basic_report.branch_change_dispositions[&EventId::from_bytes([1; 32])]
                .get(&basic.candidate.change_hash),
            Some(&ProtocolDisposition::Accepted)
        );
        let disposition_visits = Cell::new(0_u8);
        assert_eq!(
            basic_report.referenced_branch_change_disposition_metered(
                EventId::from_bytes([1; 32]),
                basic.candidate.change_hash,
                || {
                    disposition_visits.set(disposition_visits.get() + 1);
                    Ok::<(), ()>(())
                },
            ),
            Ok(Some(ProtocolDisposition::Accepted))
        );
        assert_eq!(disposition_visits.get(), 2);
        assert_eq!(
            basic_report
                .accepted_at_control
                .get(&EventId::from_bytes([1; 32]))
                .map(AcceptedAtControl::accepted_closure),
            Some(&basic_report.accepted_changes),
        );
        assert!(basic_report.materialized_document.is_some());
        let consumed_items = 1_000 - basic_budget.remaining().1;
        let final_schedule_exhausted = evaluate_batch(
            [control(1, None, vec![basic.clone()])],
            &mut WorkBudget::new(10_000, consumed_items - 1),
            &NeverCancelled,
        );
        assert_no_progress_batch(&final_schedule_exhausted, Completion::BudgetExhausted);

        let mut malformed = basic.clone();
        malformed.raw_change = Some(vec![0xff].into());
        let materialization_failed = evaluate_batch(
            [control(1, None, vec![malformed])],
            &mut WorkBudget::new(10_000, 1_000),
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
    fn finding_077_canonical_raw_bytes_share_one_allocation() {
        let mut value = change(7, 7, 1);
        value.raw_change = Some(vec![0x5a; 64].into());
        let controls =
            BTreeMap::from([(EventId::from_bytes([1; 32]), control(1, None, vec![value]))]);
        let original = controls
            .values()
            .next()
            .and_then(|control| control.changes.first())
            .and_then(|change| change.raw_change.as_ref());
        assert!(original.is_some());
        let mut budget = WorkBudget::new(64, 1_000);
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
    fn batch_memo_clones_only_shared_candidate_payload_handles() {
        let mut value = change(7, 7, 1);
        value.candidate.dependencies = (0_u8..64)
            .map(|byte| ChangeHash::from_bytes([byte; 32]))
            .collect::<Vec<_>>()
            .into();
        value.candidate.valid_carriers = (64_u8..=96)
            .map(|byte| EventId::from_bytes([byte; 32]))
            .collect::<Vec<_>>()
            .into();
        let dependency_payload = value.candidate.dependencies.clone();
        let carrier_payload = value.candidate.valid_carriers.clone();
        let controls =
            BTreeMap::from([(EventId::from_bytes([1; 32]), control(1, None, vec![value]))]);
        let mut budget = WorkBudget::new(0, 1_000);
        let retained = BatchChangeMemo::derive(&controls, &mut budget, &NeverCancelled)
            .ok()
            .and_then(|memo| memo.candidates.into_values().next());
        let Some(retained) = retained else {
            return;
        };
        assert!(Arc::ptr_eq(&dependency_payload, &retained.dependencies));
        assert!(Arc::ptr_eq(&carrier_payload, &retained.valid_carriers));
    }

    #[test]
    fn batch_memo_sharing_starts_only_after_exact_graph_charges() {
        let controls = || {
            let mut value = change(7, 7, 1);
            value.candidate.dependencies = (0_u8..64)
                .map(|byte| ChangeHash::from_bytes([byte; 32]))
                .collect::<Vec<_>>()
                .into();
            let dependencies = value.candidate.dependencies.clone();
            (
                BTreeMap::from([(EventId::from_bytes([1; 32]), control(1, None, vec![value]))]),
                dependencies,
            )
        };

        let (insufficient_controls, insufficient_payload) = controls();
        let mut insufficient = WorkBudget::new(0, 71);
        assert_eq!(
            BatchChangeMemo::derive(&insufficient_controls, &mut insufficient, &NeverCancelled,)
                .map(|_| ()),
            Err(Completion::BudgetExhausted)
        );
        assert_eq!(Arc::strong_count(&insufficient_payload), 2);

        for limit in [72_u64, 73] {
            let (exact_controls, exact_payload) = controls();
            let mut budget = WorkBudget::new(0, limit);
            let memo = BatchChangeMemo::derive(&exact_controls, &mut budget, &NeverCancelled);
            assert!(memo.is_ok(), "limit:{limit}");
            let memo = memo.ok();
            assert_eq!(Arc::strong_count(&exact_payload), 3, "limit:{limit}");
            assert_eq!(budget.remaining().1, limit - 72, "limit:{limit}");
            drop(memo);
        }

        let (cancelled_controls, cancelled_payload) = controls();
        let mut cancelled = WorkBudget::new(0, 72);
        assert_eq!(
            BatchChangeMemo::derive(&cancelled_controls, &mut cancelled, &|| true).map(|_| ()),
            Err(Completion::Cancelled)
        );
        assert_eq!(Arc::strong_count(&cancelled_payload), 2);
        assert_eq!(cancelled.consumed().get(WorkCounter::GraphNode), 0);
    }

    #[test]
    fn accepted_candidate_and_raw_projection_charges_each_owned_operation() {
        let value = change(7, 7, 1);
        let hash = value.candidate.change_hash;
        let raw: Arc<[u8]> = vec![0x5a; 64].into();
        let memo = BatchChangeMemo {
            candidates: BTreeMap::from([(hash, value.candidate)]),
            raw_changes: BTreeMap::from([(hash, Arc::clone(&raw))]),
            hashes_by_control: BTreeMap::new(),
            controls_by_hash: BTreeMap::new(),
        };
        let accepted = BTreeSet::from([hash]);

        for stop_at in 0..5 {
            let visits = Cell::new(0_usize);
            let result = project_accepted_candidate_maps_metered(&accepted, &memo, |_| {
                let current = visits.get();
                if current == stop_at {
                    return Err(());
                }
                visits.set(current + 1);
                Ok(())
            });
            assert!(result.is_err(), "stop_at:{stop_at}");
            assert_eq!(visits.get(), stop_at, "stop_at:{stop_at}");
        }

        let visits = Cell::new(0_usize);
        let projection = project_accepted_candidate_maps_metered(&accepted, &memo, |_| {
            visits.set(visits.get() + 1);
            Ok::<(), ()>(())
        })
        .unwrap_or_else(|_| unreachable!("complete projection is valid"));
        assert_eq!(visits.get(), 5);
        assert_eq!(projection.candidates.len(), 1);
        assert_eq!(projection.raw_changes.get(&hash), Some(&raw));
    }

    #[test]
    fn losing_branch_preserves_pending_invalid_and_equivocation_outcomes() {
        let mut invalid = change(3, 3, 1);
        invalid.legacy_eligible = false;
        let mut pending = change(4, 4, 1);
        pending.candidate.dependencies = pending
            .candidate
            .dependencies
            .iter()
            .copied()
            .chain([ChangeHash::from_bytes([99; 32])])
            .collect::<Vec<_>>()
            .into();
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
            branch.get(&invalid.candidate.change_hash),
            Some(&ProtocolDisposition::Invalid)
        );
        assert_eq!(
            branch.get(&pending.candidate.change_hash),
            Some(&ProtocolDisposition::Pending)
        );
        assert_eq!(
            branch.get(&equivocation_a.candidate.change_hash),
            Some(&ProtocolDisposition::Excluded)
        );
        assert_eq!(
            branch.get(&equivocation_b.candidate.change_hash),
            Some(&ProtocolDisposition::Excluded)
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
        let candidate = change(1, 1, 1).candidate;
        let accepted = BTreeSet::from([candidate.change_hash]);
        let memo = BatchChangeMemo {
            candidates: BTreeMap::from([(candidate.change_hash, candidate)]),
            ..BatchChangeMemo::default()
        };
        let derived = derive_heads_metered(&accepted, &memo, |_| Ok::<(), ()>(()))
            .unwrap_or_else(|_| unreachable!("complete head derivation is valid"));
        assert!(super::applied_heads_agree(&derived, &accepted));
        assert!(!super::applied_heads_agree(
            &derived,
            &BTreeSet::from([ChangeHash::from_bytes([9; 32])])
        ));
    }

    #[test]
    fn head_derivation_charges_deep_forked_and_duplicate_inputs_exactly() {
        let mut first = change(1, 1, 1).candidate;
        let mut second = change(2, 1, 2).candidate;
        let mut third = change(3, 2, 1).candidate;
        let mut fourth = change(4, 1, 3).candidate;
        first.dependencies = Vec::new().into();
        second.dependencies = vec![first.change_hash].into();
        third.dependencies = vec![first.change_hash].into();
        fourth.dependencies = vec![second.change_hash, third.change_hash].into();
        let accepted = BTreeSet::from([
            first.change_hash,
            second.change_hash,
            third.change_hash,
            fourth.change_hash,
        ]);
        let expected_heads = BTreeSet::from([fourth.change_hash]);
        let memo = BatchChangeMemo {
            candidates: [first, second, third, fourth]
                .into_iter()
                .map(|candidate| (candidate.change_hash, candidate))
                .collect(),
            ..BatchChangeMemo::default()
        };
        let expected = [
            WorkCounter::GraphNode,
            WorkCounter::GraphNode,
            WorkCounter::GraphNode,
            WorkCounter::GraphNode,
            WorkCounter::GraphEdge,
            WorkCounter::GraphNode,
            WorkCounter::GraphNode,
            WorkCounter::GraphNode,
            WorkCounter::GraphEdge,
            WorkCounter::GraphNode,
            WorkCounter::GraphNode,
            WorkCounter::GraphNode,
            WorkCounter::GraphEdge,
            WorkCounter::GraphNode,
            WorkCounter::GraphEdge,
            WorkCounter::GraphNode,
            WorkCounter::GraphNode,
            WorkCounter::GraphNode,
            WorkCounter::GraphNode,
            WorkCounter::GraphNode,
            WorkCounter::GraphNode,
            WorkCounter::GraphNode,
            WorkCounter::GraphNode,
            WorkCounter::GraphNode,
            WorkCounter::GraphNode,
        ];

        for capacity in [expected.len() - 1, expected.len(), expected.len() + 1] {
            let visits = Cell::new(0_usize);
            let mut budget = WorkBudget::new(0, u64::try_from(capacity).unwrap_or(u64::MAX));
            let result = derive_heads_metered(&accepted, &memo, |counter| {
                assert_eq!(counter, expected[visits.get()]);
                budget
                    .charge(counter, 1)
                    .map_err(|_| Completion::BudgetExhausted)?;
                visits.set(visits.get() + 1);
                Ok(())
            });
            if capacity < expected.len() {
                assert_eq!(
                    result,
                    Err(super::HeadDerivationError::Work(
                        Completion::BudgetExhausted
                    ))
                );
                assert_eq!(visits.get(), capacity);
            } else {
                assert_eq!(result, Ok(expected_heads.clone()));
                assert_eq!(visits.get(), expected.len());
            }
        }

        for cancel_at in 0..expected.len() {
            let visits = Cell::new(0_usize);
            let result = derive_heads_metered(&accepted, &memo, |counter| {
                assert_eq!(counter, expected[visits.get()]);
                if visits.get() == cancel_at {
                    return Err(Completion::Cancelled);
                }
                visits.set(visits.get() + 1);
                Ok(())
            });
            assert_eq!(
                result,
                Err(super::HeadDerivationError::Work(Completion::Cancelled))
            );
            assert_eq!(visits.get(), cancel_at);
        }

        let empty_visits = Cell::new(0_usize);
        assert_eq!(
            derive_heads_metered(&BTreeSet::new(), &memo, |_| {
                empty_visits.set(empty_visits.get() + 1);
                Ok::<(), ()>(())
            }),
            Ok(BTreeSet::new())
        );
        assert_eq!(empty_visits.get(), 0);
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
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        assert_eq!(
            production
                .matches("collect_control_candidates_metered(")
                .count(),
            3,
            "one closed helper plus the legacy and selected epoch call sites"
        );
        assert!(production.contains("if !accepted_base.contains(&hash)"));
    }

    #[test]
    fn pruned_prior_dependency_is_invalid_not_pending() {
        let first = change(1, 1, 1);
        let second = change(2, 2, 1);
        let first_hash = first.candidate.change_hash;
        let second_hash = second.candidate.change_hash;
        let parent = control(1, None, vec![first, second]);
        let mut dependant = change(3, 3, 1);
        dependant.candidate.dependencies = vec![second_hash].into();
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
