use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::ChangeHash;
use crate::automerge_adapter::materialized_view::MaterializedDocumentView;
use crate::control::ancestry::ControlAncestry;
use crate::control::epoch_state::{AcceptedEpochState, MeteredAcceptedEpochStateError};
use crate::control::validate::ControlEnvelope;
use crate::graph::actor_state::{
    EpochActorState, MeteredActorStateError, apply_empty_counter, apply_nonempty_counter,
    initialize_actor_states_metered, validate_actor_predecessor,
};
use crate::graph::change_candidate::ChangeCandidate;
use crate::graph::closure::{CandidateClosureError, candidate_dependency_closure};
use crate::graph::dependency_graph::{
    GraphBuildError, MeteredGraphBuildError, build_graph_metered,
};
use crate::graph::epoch::{EpochAncestry, validate_epoch_ancestry};
use crate::graph::equivocation::{QuarantineError, quarantine_equivocation_descendants};
use crate::graph::schedule::ScheduleError;
use crate::reference::apply::apply_exact_closure_metered;
use crate::reference::branch_state::PersistentDeltaMap;
use crate::reference::epoch::{EpochCandidate, resolve_epoch};
use crate::types::role::Role;
use crate::{ActorId, IntegrityAlert, ProtocolDisposition};
use crate::{CancellationCheck, WorkBudget, WorkCounter};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EpochEvaluationInputError {
    BaseFrontierMismatch,
    AncestryMismatch,
    CandidateControlMismatch,
    DuplicateCandidate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MeteredEpochEvaluationInputError<E> {
    Work(E),
    Input(EpochEvaluationInputError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EpochEvaluationError {
    Schedule(ScheduleError),
    Graph(GraphBuildError),
    Quarantine(QuarantineError),
    State(crate::control::epoch_state::AcceptedEpochStateError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PriorChangeKnowledge {
    AcceptedInBase,
    SameEpochCandidate,
    PrunedCanonicalAncestor,
    KnownOtherControl,
    KnownInvalid,
    KnownUnsupported,
    PriorEquivocationExcluded,
    Unknown,
}

pub(crate) type PriorKnowledgeState = PersistentDeltaMap<ChangeHash, PriorChangeKnowledge>;

impl PriorChangeKnowledge {
    pub(crate) const fn is_known_impossible(self) -> bool {
        matches!(
            self,
            Self::PrunedCanonicalAncestor
                | Self::KnownOtherControl
                | Self::KnownInvalid
                | Self::KnownUnsupported
                | Self::PriorEquivocationExcluded
        )
    }
}

/// Complete trusted input for evaluating one selected control epoch.
///
/// The accepted base is carried as one invariant-checked state object. Change
/// semantic validity is deliberately absent and must be derived by evaluation.
#[derive(Clone)]
pub(crate) struct EpochEvaluationInput<'a> {
    selected_control: ControlEnvelope,
    accepted_base: Arc<AcceptedEpochState>,
    candidate_changes: BTreeMap<ChangeHash, ChangeCandidate>,
    raw_changes: Cow<'a, BTreeMap<ChangeHash, Arc<[u8]>>>,
    canonical_ancestry: ControlAncestry,
    prior_change_knowledge: PriorKnowledgeState,
}

impl EpochEvaluationInput<'static> {
    pub(crate) fn new(
        selected_control: ControlEnvelope,
        accepted_base: AcceptedEpochState,
        candidate_changes: impl IntoIterator<Item = ChangeCandidate>,
        canonical_ancestry: Vec<ControlEnvelope>,
    ) -> Result<Self, EpochEvaluationInputError> {
        Self::new_with_raw(
            selected_control,
            accepted_base,
            candidate_changes
                .into_iter()
                .map(|candidate| (candidate, None)),
            BTreeMap::new(),
            canonical_ancestry,
        )
    }

    pub(crate) fn new_with_raw(
        selected_control: ControlEnvelope,
        accepted_base: AcceptedEpochState,
        candidate_changes: impl IntoIterator<Item = (ChangeCandidate, Option<Arc<[u8]>>)>,
        raw_changes: BTreeMap<ChangeHash, Arc<[u8]>>,
        canonical_ancestry: Vec<ControlEnvelope>,
    ) -> Result<Self, EpochEvaluationInputError> {
        Self::new_with_raw_and_prior(
            selected_control,
            accepted_base,
            candidate_changes,
            raw_changes,
            canonical_ancestry,
            BTreeMap::new(),
        )
    }

    pub(crate) fn new_with_raw_and_prior(
        selected_control: ControlEnvelope,
        accepted_base: AcceptedEpochState,
        candidate_changes: impl IntoIterator<Item = (ChangeCandidate, Option<Arc<[u8]>>)>,
        mut raw_changes: BTreeMap<ChangeHash, Arc<[u8]>>,
        canonical_ancestry: Vec<ControlEnvelope>,
        prior_change_knowledge: BTreeMap<ChangeHash, PriorChangeKnowledge>,
    ) -> Result<Self, EpochEvaluationInputError> {
        let declared_heads = selected_control.base_heads().collect::<BTreeSet<_>>();
        if declared_heads != *accepted_base.frontier_heads() {
            return Err(EpochEvaluationInputError::BaseFrontierMismatch);
        }
        let canonical_ancestry = ControlAncestry::from_ordered(canonical_ancestry)
            .map_err(|()| EpochEvaluationInputError::AncestryMismatch)?;
        if selected_control.parent() != canonical_ancestry.last_event_id() {
            return Err(EpochEvaluationInputError::AncestryMismatch);
        }
        let selected_id = selected_control.event_id();
        let mut candidates = BTreeMap::new();
        for (candidate, raw) in candidate_changes {
            if candidate.control_id != selected_id {
                return Err(EpochEvaluationInputError::CandidateControlMismatch);
            }
            let hash = candidate.change_hash;
            if candidates.insert(hash, candidate).is_some() {
                return Err(EpochEvaluationInputError::DuplicateCandidate);
            }
            if let Some(raw) = raw {
                raw_changes.insert(hash, raw);
            }
        }
        Ok(Self {
            selected_control,
            accepted_base: Arc::new(accepted_base),
            candidate_changes: candidates,
            raw_changes: Cow::Owned(raw_changes),
            canonical_ancestry,
            prior_change_knowledge: PriorKnowledgeState::from_local(prior_change_knowledge),
        })
    }
}

impl<'a> EpochEvaluationInput<'a> {
    pub(crate) fn new_with_metered_candidate_map(
        selected_control: ControlEnvelope,
        accepted_base: Arc<AcceptedEpochState>,
        candidate_changes: BTreeMap<ChangeHash, ChangeCandidate>,
        raw_changes: &'a BTreeMap<ChangeHash, Arc<[u8]>>,
        canonical_ancestry: ControlAncestry,
        prior_change_knowledge: PriorKnowledgeState,
        mut charge: impl FnMut(WorkCounter) -> Result<(), ScheduleError>,
    ) -> Result<Self, MeteredEpochEvaluationInputError<ScheduleError>> {
        let mut declared_heads = BTreeSet::new();
        let mut head_iter = selected_control.content().base_heads.iter();
        for _ in 0..selected_control.content().base_heads.len() {
            charge(WorkCounter::GraphNode).map_err(MeteredEpochEvaluationInputError::Work)?;
            let Some(head) = head_iter.next() else {
                return Err(MeteredEpochEvaluationInputError::Input(
                    EpochEvaluationInputError::BaseFrontierMismatch,
                ));
            };
            declared_heads.insert(*head);
        }
        if declared_heads != *accepted_base.frontier_heads() {
            return Err(MeteredEpochEvaluationInputError::Input(
                EpochEvaluationInputError::BaseFrontierMismatch,
            ));
        }
        if selected_control.parent() != canonical_ancestry.last_event_id() {
            return Err(MeteredEpochEvaluationInputError::Input(
                EpochEvaluationInputError::AncestryMismatch,
            ));
        }
        let selected_id = selected_control.event_id();
        let mut candidate_iter = candidate_changes.iter();
        for _ in 0..candidate_changes.len() {
            charge(WorkCounter::GraphNode).map_err(MeteredEpochEvaluationInputError::Work)?;
            let Some((hash, candidate)) = candidate_iter.next() else {
                return Err(MeteredEpochEvaluationInputError::Input(
                    EpochEvaluationInputError::DuplicateCandidate,
                ));
            };
            if candidate.change_hash != *hash || candidate.control_id != selected_id {
                return Err(MeteredEpochEvaluationInputError::Input(
                    EpochEvaluationInputError::CandidateControlMismatch,
                ));
            }
        }
        Ok(Self {
            selected_control,
            accepted_base,
            candidate_changes,
            raw_changes: Cow::Borrowed(raw_changes),
            canonical_ancestry,
            prior_change_knowledge,
        })
    }

    pub(crate) fn new_with_borrowed_raw_and_prior(
        selected_control: ControlEnvelope,
        accepted_base: Arc<AcceptedEpochState>,
        candidate_changes: impl IntoIterator<Item = ChangeCandidate>,
        raw_changes: &'a BTreeMap<ChangeHash, Arc<[u8]>>,
        canonical_ancestry: ControlAncestry,
        prior_change_knowledge: PriorKnowledgeState,
    ) -> Result<Self, EpochEvaluationInputError> {
        let declared_heads = selected_control.base_heads().collect::<BTreeSet<_>>();
        if declared_heads != *accepted_base.frontier_heads() {
            return Err(EpochEvaluationInputError::BaseFrontierMismatch);
        }
        if selected_control.parent() != canonical_ancestry.last_event_id() {
            return Err(EpochEvaluationInputError::AncestryMismatch);
        }
        let selected_id = selected_control.event_id();
        let mut candidates = BTreeMap::new();
        for candidate in candidate_changes {
            if candidate.control_id != selected_id {
                return Err(EpochEvaluationInputError::CandidateControlMismatch);
            }
            let hash = candidate.change_hash;
            if candidates.insert(hash, candidate).is_some() {
                return Err(EpochEvaluationInputError::DuplicateCandidate);
            }
        }
        Ok(Self {
            selected_control,
            accepted_base,
            candidate_changes: candidates,
            raw_changes: Cow::Borrowed(raw_changes),
            canonical_ancestry,
            prior_change_knowledge,
        })
    }

    pub(crate) const fn selected_control(&self) -> &ControlEnvelope {
        &self.selected_control
    }

    pub(crate) fn accepted_base(&self) -> &AcceptedEpochState {
        &self.accepted_base
    }

    pub(crate) fn accepted_base_handle(&self) -> Arc<AcceptedEpochState> {
        Arc::clone(&self.accepted_base)
    }

    pub(crate) const fn candidate_changes(&self) -> &BTreeMap<ChangeHash, ChangeCandidate> {
        &self.candidate_changes
    }

    pub(crate) fn raw_changes(&self) -> &BTreeMap<ChangeHash, Arc<[u8]>> {
        &self.raw_changes
    }

    pub(crate) const fn canonical_ancestry(&self) -> &ControlAncestry {
        &self.canonical_ancestry
    }

    pub(crate) const fn prior_change_knowledge(&self) -> &PriorKnowledgeState {
        &self.prior_change_knowledge
    }
}

/// Complete authoritative output of evaluating one selected control epoch.
#[derive(Clone)]
pub(crate) struct EpochEvaluationResult {
    accepted_state: Arc<AcceptedEpochState>,
    dispositions: BTreeMap<ChangeHash, ProtocolDisposition>,
    integrity_alerts: Vec<IntegrityAlert>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AcceptedAtControl {
    accepted_closure: Arc<BTreeSet<ChangeHash>>,
    frontier_heads: Arc<BTreeSet<ChangeHash>>,
    actor_states: Arc<BTreeMap<ActorId, EpochActorState>>,
}

impl AcceptedAtControl {
    pub(crate) fn from_result(result: &EpochEvaluationResult) -> Self {
        Self {
            accepted_closure: result.accepted_state().accepted_closure_handle(),
            frontier_heads: result.accepted_state().frontier_heads_handle(),
            actor_states: result.accepted_state().actor_states_handle(),
        }
    }

    pub(crate) fn accepted_closure(&self) -> &BTreeSet<ChangeHash> {
        &self.accepted_closure
    }

    pub(crate) fn frontier_heads(&self) -> &BTreeSet<ChangeHash> {
        &self.frontier_heads
    }

    pub(crate) fn actor_states(&self) -> &BTreeMap<ActorId, EpochActorState> {
        &self.actor_states
    }

    #[cfg(test)]
    pub(crate) fn for_test(accepted_closure: BTreeSet<ChangeHash>) -> Self {
        Self {
            frontier_heads: Arc::new(accepted_closure.clone()),
            accepted_closure: Arc::new(accepted_closure),
            actor_states: Arc::new(BTreeMap::new()),
        }
    }
}

impl EpochEvaluationResult {
    pub(crate) fn new(
        accepted_closure: BTreeSet<ChangeHash>,
        frontier_heads: BTreeSet<ChangeHash>,
        accepted_candidates: BTreeMap<ChangeHash, ChangeCandidate>,
        dispositions: BTreeMap<ChangeHash, ProtocolDisposition>,
        integrity_alerts: Vec<IntegrityAlert>,
        materialized: Option<MaterializedDocumentView>,
    ) -> Result<Self, crate::control::epoch_state::AcceptedEpochStateError> {
        let accepted_state = AcceptedEpochState::new(
            accepted_closure,
            frontier_heads,
            accepted_candidates,
            materialized,
        )?;
        Ok(Self {
            accepted_state: Arc::new(accepted_state),
            dispositions,
            integrity_alerts,
        })
    }

    pub(crate) fn from_shared_state(
        accepted_state: Arc<AcceptedEpochState>,
        dispositions: BTreeMap<ChangeHash, ProtocolDisposition>,
        integrity_alerts: Vec<IntegrityAlert>,
    ) -> Self {
        Self {
            accepted_state,
            dispositions,
            integrity_alerts,
        }
    }

    pub(crate) fn accepted_state(&self) -> &AcceptedEpochState {
        &self.accepted_state
    }

    pub(crate) fn accepted_state_handle(&self) -> Arc<AcceptedEpochState> {
        Arc::clone(&self.accepted_state)
    }

    pub(crate) const fn dispositions(&self) -> &BTreeMap<ChangeHash, ProtocolDisposition> {
        &self.dispositions
    }

    pub(crate) fn integrity_alerts(&self) -> &[IntegrityAlert] {
        &self.integrity_alerts
    }
}

pub(crate) fn evaluate_epoch(
    input: &EpochEvaluationInput,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<EpochEvaluationResult, EpochEvaluationError> {
    let selected = input.selected_control();
    let terminal = selected.content().terminal;
    let all_candidates = clone_candidate_maps_metered(
        input.accepted_base().accepted_candidates(),
        input.candidate_changes(),
        budget,
        cancellation,
    )
    .map_err(EpochEvaluationError::Schedule)?;
    let mut epoch_candidates = Vec::with_capacity(input.candidate_changes().len());
    let mut candidate_iter = input.candidate_changes().values();
    for _ in 0..input.candidate_changes().len() {
        charge_epoch_item(WorkCounter::GraphNode, budget, cancellation)
            .map_err(EpochEvaluationError::Schedule)?;
        let Some(candidate) = candidate_iter.next().cloned() else {
            return Err(EpochEvaluationError::State(
                crate::control::epoch_state::AcceptedEpochStateError::ClosureMismatch,
            ));
        };
        let authorized = selected.content().members.iter().any(|member| {
            member.actor == candidate.actor
                && member.device == candidate.author
                && member.roles.contains(&Role::Write)
        });
        let closure =
            candidate_dependency_closure(&candidate, &all_candidates, budget, cancellation)
                .map_err(|error| EpochEvaluationError::Schedule(closure_schedule_error(error)))?;
        let complete_closure = closure.missing.is_empty().then_some(&closure.known);
        let actor_sequence_valid = complete_closure.is_none_or(|known| {
            validate_actor_predecessor(&candidate, known, &all_candidates).is_ok()
        });
        let actor_counter_valid = if let Some(known) = complete_closure {
            let projection =
                match initialize_actor_states_metered(known, &all_candidates, |counter| {
                    charge_epoch_item(counter, budget, cancellation)
                }) {
                    Ok(projection) => Some(projection),
                    Err(MeteredActorStateError::Work(error)) => {
                        return Err(EpochEvaluationError::Schedule(error));
                    }
                    Err(MeteredActorStateError::State(_)) => None,
                };
            projection.is_some_and(|projection| {
                let mut states = projection.actor_states;
                if candidate.operation_count == 0 {
                    let mut current_heads = projection.frontier_heads;
                    current_heads.extend(
                        input
                            .accepted_base()
                            .frontier_heads()
                            .difference(known)
                            .copied(),
                    );
                    apply_empty_counter(&mut states, &candidate, &current_heads).is_ok()
                } else {
                    apply_nonempty_counter(&mut states, &candidate).is_ok()
                }
            })
        } else {
            true
        };
        let ancestry_valid = !matches!(
            validate_epoch_ancestry(
                input.accepted_base().frontier_heads(),
                &closure.known,
                &closure.missing,
            ),
            EpochAncestry::InvalidOmission(_)
        );
        let prior_dependencies_valid = prior_dependencies_valid_metered(
            input.prior_change_knowledge(),
            &candidate.dependencies,
            &closure.missing,
            |counter| {
                charge_epoch_item(counter, budget, cancellation)
                    .map_err(EpochEvaluationError::Schedule)
            },
        )?;
        let prior_semantics_valid = authorized
            && actor_sequence_valid
            && actor_counter_valid
            && ancestry_valid
            && prior_dependencies_valid;
        let application_valid = if !prior_semantics_valid {
            false
        } else if !closure.missing.is_empty() {
            true
        } else if !closure.cyclic.is_empty() {
            false
        } else if let Some(raw) = input.raw_changes().get(&candidate.change_hash) {
            let mut closure_raw = BTreeMap::new();
            for hash in &closure.known {
                charge_epoch_item(WorkCounter::GraphNode, budget, cancellation)
                    .map_err(EpochEvaluationError::Schedule)?;
                let Some(value) = input.raw_changes().get(hash) else {
                    continue;
                };
                closure_raw.insert(*hash, Arc::clone(value));
            }
            if closure_raw.len() != closure.known.len() {
                false
            } else {
                let mut dependencies = BTreeSet::new();
                for dependency in candidate.dependencies.iter() {
                    charge_epoch_item(WorkCounter::GraphEdge, budget, cancellation)
                        .map_err(EpochEvaluationError::Schedule)?;
                    dependencies.insert(*dependency);
                }
                match apply_exact_closure_metered(
                    &closure_raw,
                    &closure.ordered,
                    candidate.change_hash,
                    raw,
                    &dependencies,
                    budget,
                    cancellation,
                ) {
                    Ok(_) => true,
                    Err(crate::automerge_adapter::document::ExactApplyError::Budget) => {
                        return Err(EpochEvaluationError::Schedule(
                            ScheduleError::BudgetExhausted,
                        ));
                    }
                    Err(crate::automerge_adapter::document::ExactApplyError::Cancelled) => {
                        return Err(EpochEvaluationError::Schedule(ScheduleError::Cancelled));
                    }
                    Err(_) => false,
                }
            }
        } else {
            false
        };
        let semantically_valid = prior_semantics_valid && application_valid;
        epoch_candidates.push(EpochCandidate {
            candidate,
            semantically_valid,
            canonical_control: !terminal,
        });
    }
    let mut dispositions = resolve_epoch(
        epoch_candidates,
        input.accepted_base().accepted_closure(),
        budget,
        cancellation,
    )
    .map_err(EpochEvaluationError::Schedule)?;
    let eligible = collect_eligible_candidates_metered(
        &all_candidates,
        input.accepted_base().accepted_closure(),
        &dispositions,
        budget,
        cancellation,
    )
    .map_err(EpochEvaluationError::Schedule)?;
    let graph = build_graph_metered(
        &eligible,
        input.accepted_base().accepted_closure(),
        |counter| charge_epoch_item(counter, budget, cancellation),
    )
    .map_err(|error| match error {
        MeteredGraphBuildError::Work(error) => EpochEvaluationError::Schedule(error),
        MeteredGraphBuildError::Graph(error) => EpochEvaluationError::Graph(error),
    })?;
    let quarantine = quarantine_equivocation_descendants(eligible, &graph, budget, cancellation)
        .map_err(EpochEvaluationError::Quarantine)?;
    for hash in &quarantine.quarantined {
        dispositions.insert(*hash, ProtocolDisposition::Excluded);
    }
    let (accepted_closure, accepted_candidates) = project_accepted_candidates_metered(
        &all_candidates,
        input.accepted_base().accepted_closure(),
        &dispositions,
        budget,
        cancellation,
    )
    .map_err(EpochEvaluationError::Schedule)?;
    if metered_hash_sets_equal(
        &accepted_closure,
        input.accepted_base().accepted_closure(),
        budget,
        cancellation,
    )? {
        return Ok(EpochEvaluationResult::from_shared_state(
            input.accepted_base_handle(),
            dispositions,
            quarantine.alerts,
        ));
    }
    let accepted_closure = Arc::new(accepted_closure);
    let accepted_state =
        AcceptedEpochState::new_metered(accepted_closure, accepted_candidates, None, |counter| {
            charge_epoch_item(counter, budget, cancellation)
        })
        .map_err(|error| match error {
            MeteredAcceptedEpochStateError::Work(error) => EpochEvaluationError::Schedule(error),
            MeteredAcceptedEpochStateError::State(error) => EpochEvaluationError::State(error),
        })?;
    Ok(EpochEvaluationResult::from_shared_state(
        Arc::new(accepted_state),
        dispositions,
        quarantine.alerts,
    ))
}

fn prior_dependencies_valid_metered<E>(
    prior: &PriorKnowledgeState,
    declared: &[ChangeHash],
    missing: &BTreeSet<ChangeHash>,
    mut visit: impl FnMut(WorkCounter) -> Result<(), E>,
) -> Result<bool, E> {
    let mut declared_items = declared.iter();
    for _ in 0..declared.len() {
        visit(WorkCounter::GraphEdge)?;
        let Some(dependency) = declared_items.next() else {
            return Ok(false);
        };
        if prior
            .get_metered(dependency, || visit(WorkCounter::GraphNode))?
            .is_some_and(|knowledge| knowledge.is_known_impossible())
        {
            return Ok(false);
        }
    }
    let mut missing_items = missing.iter();
    for _ in 0..missing.len() {
        visit(WorkCounter::GraphEdge)?;
        let Some(dependency) = missing_items.next() else {
            return Ok(false);
        };
        if prior
            .get_metered(dependency, || visit(WorkCounter::GraphNode))?
            .is_some_and(|knowledge| knowledge.is_known_impossible())
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn metered_hash_sets_equal(
    left: &BTreeSet<ChangeHash>,
    right: &BTreeSet<ChangeHash>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<bool, EpochEvaluationError> {
    if left.len() != right.len() {
        return Ok(false);
    }
    let mut left_iter = left.iter();
    let mut right_iter = right.iter();
    for _ in 0..left.len() {
        charge_epoch_item(WorkCounter::GraphNode, budget, cancellation)
            .map_err(EpochEvaluationError::Schedule)?;
        if left_iter.next() != right_iter.next() {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(crate) fn clone_candidate_maps_metered(
    base: &BTreeMap<ChangeHash, ChangeCandidate>,
    local: &BTreeMap<ChangeHash, ChangeCandidate>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<BTreeMap<ChangeHash, ChangeCandidate>, ScheduleError> {
    let mut result = BTreeMap::new();
    for source in [base, local] {
        let mut iter = source.iter();
        for _ in 0..source.len() {
            charge_epoch_item(WorkCounter::GraphNode, budget, cancellation)?;
            let Some((hash, candidate)) = iter.next() else {
                return Err(ScheduleError::BudgetExhausted);
            };
            result.insert(*hash, candidate.clone());
        }
    }
    Ok(result)
}

pub(crate) fn collect_eligible_candidates_metered(
    candidates: &BTreeMap<ChangeHash, ChangeCandidate>,
    accepted_base: &BTreeSet<ChangeHash>,
    dispositions: &BTreeMap<ChangeHash, ProtocolDisposition>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<Vec<ChangeCandidate>, ScheduleError> {
    let mut result = Vec::new();
    let mut iter = candidates.iter();
    for _ in 0..candidates.len() {
        charge_epoch_item(WorkCounter::GraphNode, budget, cancellation)?;
        let Some((hash, candidate)) = iter.next() else {
            return Err(ScheduleError::BudgetExhausted);
        };
        if accepted_base.contains(hash)
            || dispositions.get(hash) == Some(&ProtocolDisposition::Accepted)
        {
            result.push(candidate.clone());
        }
    }
    Ok(result)
}

pub(crate) fn project_accepted_candidates_metered(
    candidates: &BTreeMap<ChangeHash, ChangeCandidate>,
    accepted_base: &BTreeSet<ChangeHash>,
    dispositions: &BTreeMap<ChangeHash, ProtocolDisposition>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<(BTreeSet<ChangeHash>, BTreeMap<ChangeHash, ChangeCandidate>), ScheduleError> {
    let mut accepted_closure = BTreeSet::new();
    let mut accepted_candidates = BTreeMap::new();
    let mut iter = candidates.iter();
    for _ in 0..candidates.len() {
        charge_epoch_item(WorkCounter::GraphNode, budget, cancellation)?;
        let Some((hash, candidate)) = iter.next() else {
            return Err(ScheduleError::BudgetExhausted);
        };
        if (accepted_base.contains(hash)
            || dispositions.get(hash) == Some(&ProtocolDisposition::Accepted))
            && dispositions.get(hash) != Some(&ProtocolDisposition::Excluded)
        {
            accepted_closure.insert(*hash);
            accepted_candidates.insert(*hash, candidate.clone());
        }
    }
    Ok((accepted_closure, accepted_candidates))
}

fn charge_epoch_item(
    counter: WorkCounter,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<(), ScheduleError> {
    if cancellation.is_cancelled() {
        return Err(ScheduleError::Cancelled);
    }
    budget
        .charge(counter, 1)
        .map_err(|_| ScheduleError::BudgetExhausted)
}

const fn closure_schedule_error(error: CandidateClosureError) -> ScheduleError {
    match error {
        CandidateClosureError::BudgetExhausted => ScheduleError::BudgetExhausted,
        CandidateClosureError::Cancelled => ScheduleError::Cancelled,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::sync::Arc;

    use super::{
        AcceptedAtControl, EpochEvaluationInput, EpochEvaluationInputError, EpochEvaluationResult,
        PriorChangeKnowledge, PriorKnowledgeState, clone_candidate_maps_metered,
        collect_eligible_candidates_metered, evaluate_epoch, prior_dependencies_valid_metered,
        project_accepted_candidates_metered,
    };
    use crate::automerge_adapter::materialized_view::MaterializedDocumentView;
    use crate::carrier::control::{DeviceGrant, ValidatedControlCarrier, ValidatedControlContent};
    use crate::control::epoch_state::{AcceptedEpochState, AcceptedEpochStateError};
    use crate::control::validate::ControlEnvelope;
    use crate::graph::actor_state::EpochActorState;
    use crate::graph::change_candidate::ChangeCandidate;
    use crate::{
        ActorId, ChangeHash, Completion, ControllerPublicKey, DevicePublicKey, DocumentCoordinate,
        DocumentId, EventId, NeverCancelled, ProtocolDisposition, WorkBudget, WorkCounter,
    };

    #[test]
    fn dependency_lookup_charges_before_outer_reads_and_persistent_nodes() {
        const DEPTH: u8 = 64;
        let target = ChangeHash::from_bytes([0; 32]);
        let mut prior = PriorKnowledgeState::from(BTreeMap::from([(
            target,
            PriorChangeKnowledge::KnownInvalid,
        )]));
        for value in 1..DEPTH {
            prior = prior.extend_local(BTreeMap::from([(
                ChangeHash::from_bytes([value; 32]),
                PriorChangeKnowledge::KnownOtherControl,
            )]));
        }
        let exact = 1_usize + usize::from(DEPTH);
        for (declared, missing) in [
            (vec![target], BTreeSet::new()),
            (Vec::new(), BTreeSet::from([target])),
        ] {
            for completion in [Completion::BudgetExhausted, Completion::Cancelled] {
                for capacity in 0..=exact + 1 {
                    let mut observed = Vec::new();
                    let result =
                        prior_dependencies_valid_metered(&prior, &declared, &missing, |counter| {
                            if observed.len() == capacity {
                                return Err(completion);
                            }
                            observed.push(counter);
                            Ok(())
                        });
                    if capacity < exact {
                        assert_eq!(result, Err(completion));
                        assert_eq!(observed.len(), capacity);
                    } else {
                        assert_eq!(result, Ok(false));
                        let mut expected = vec![WorkCounter::GraphEdge];
                        expected.extend(vec![WorkCounter::GraphNode; usize::from(DEPTH)]);
                        assert_eq!(observed, expected);
                    }
                }
            }
        }
    }

    fn control(base_heads: Vec<ChangeHash>) -> ControlEnvelope {
        let controller = ControllerPublicKey::from_bytes([1; 32]);
        let coordinate = DocumentCoordinate::new(controller, DocumentId::from_bytes([2; 32]));
        ControlEnvelope::from_validated(ValidatedControlCarrier::for_test(
            EventId::from_bytes([3; 32]),
            controller,
            coordinate,
            None,
            ValidatedControlContent {
                base_heads,
                members: Vec::new(),
                predecessor: None,
                sequence: 0,
                successor: None,
                terminal: true,
            },
        ))
    }

    #[allow(clippy::expect_used)]
    fn empty_state() -> AcceptedEpochState {
        AcceptedEpochState::new(
            BTreeSet::new(),
            BTreeSet::new(),
            BTreeMap::new(),
            MaterializedDocumentView::empty_for_test().ok(),
        )
        .expect("consistent empty accepted state")
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn accepts_only_complete_trusted_epoch_input() {
        let input = EpochEvaluationInput::new(control(Vec::new()), empty_state(), [], Vec::new())
            .expect("complete genesis input");
        assert_eq!(
            input.selected_control().event_id(),
            EventId::from_bytes([3; 32])
        );
        assert!(input.accepted_base().accepted_closure().is_empty());
        assert!(input.candidate_changes().is_empty());
        assert!(input.canonical_ancestry().is_empty());
        let evaluated = evaluate_epoch(&input, &mut WorkBudget::new(1_000, 1_000), &NeverCancelled);
        assert!(evaluated.is_ok());
        assert!(evaluated.is_ok_and(|result| {
            result.accepted_state().accepted_closure().is_empty()
                && result.accepted_state().materialized().is_some()
        }));

        let head = ChangeHash::from_bytes([4; 32]);
        assert!(matches!(
            EpochEvaluationInput::new(control(vec![head]), empty_state(), [], Vec::new()),
            Err(EpochEvaluationInputError::BaseFrontierMismatch)
        ));
    }

    #[test]
    fn only_known_impossible_dependency_states_invalidate() {
        for usable in [
            PriorChangeKnowledge::AcceptedInBase,
            PriorChangeKnowledge::SameEpochCandidate,
            PriorChangeKnowledge::Unknown,
        ] {
            assert!(!usable.is_known_impossible());
        }
        for impossible in [
            PriorChangeKnowledge::PrunedCanonicalAncestor,
            PriorChangeKnowledge::KnownOtherControl,
            PriorChangeKnowledge::KnownInvalid,
            PriorChangeKnowledge::KnownUnsupported,
            PriorChangeKnowledge::PriorEquivocationExcluded,
        ] {
            assert!(impossible.is_known_impossible());
        }
    }

    #[test]
    #[ignore = "remediation v12 expected failure: unmetered writer authorization scan"]
    fn finding_100_epoch_writer_authorization_work_reproduction() {
        let coordinate = DocumentCoordinate::new(
            ControllerPublicKey::from_bytes([1; 32]),
            DocumentId::from_bytes([2; 32]),
        );
        let author = DevicePublicKey::from_bytes([200; 32]);
        let actor = ActorId::derive(coordinate, author);
        let member = |value: u8, roles: Vec<crate::types::role::Role>| {
            let device = DevicePublicKey::from_bytes([value; 32]);
            DeviceGrant {
                account: None,
                actor: ActorId::derive(coordinate, device),
                device,
                roles,
            }
        };
        let matches = |members: &[DeviceGrant]| {
            members.iter().any(|entry| {
                entry.actor == actor
                    && entry.device == author
                    && entry.roles.contains(&crate::types::role::Role::Write)
            })
        };

        let absent = (1..=64)
            .map(|value| member(value, vec![crate::types::role::Role::Checkpoint]))
            .collect::<Vec<_>>();
        assert!(!matches(&absent));

        let mut early = absent.clone();
        early.insert(0, member(200, vec![crate::types::role::Role::Write]));
        assert!(matches(&early));

        let mut final_match = absent;
        final_match.push(member(200, vec![crate::types::role::Role::Write]));
        assert!(matches(&final_match));

        let source = include_str!("epoch_engine.rs");
        assert!(
            !source.contains("selected.content().members.iter().any"),
            "unmetered epoch writer authorization scan remains"
        );
    }

    #[test]
    #[ignore = "remediation v12 expected failure: quarantine overlays are not metered"]
    fn finding_100_quarantine_overlay_work_reproduction() {
        let selected_source = include_str!("epoch_engine.rs");
        let fallback_source = include_str!("evaluate.rs");
        assert!(
            !selected_source.contains("for hash in &quarantine.quarantined")
                && !fallback_source.contains("for hash in &quarantine.quarantined"),
            "unmetered selected and fallback quarantine overlays remain"
        );
    }

    #[test]
    #[ignore = "remediation v12 expected failure: target work precedes the first charge"]
    fn finding_100_zero_post_stop_work_reproduction() {
        let item = ChangeCandidate {
            change_hash: ChangeHash::from_bytes([5; 32]),
            actor: ActorId::from_bytes([6; 32]),
            sequence: 1,
            start_op: 1,
            operation_count: 1,
            dependencies: Vec::new().into(),
            control_id: EventId::from_bytes([3; 32]),
            author: DevicePublicKey::from_bytes([7; 32]),
            valid_carriers: Arc::from([]),
        };
        let input =
            EpochEvaluationInput::new(control(Vec::new()), empty_state(), [item], Vec::new());
        assert!(input.is_ok());
        let Ok(input) = input else { return };
        let result = evaluate_epoch(&input, &mut WorkBudget::new(0, 0), &NeverCancelled);
        assert!(matches!(
            result,
            Err(super::EpochEvaluationError::Schedule(
                crate::graph::schedule::ScheduleError::BudgetExhausted
            ))
        ));

        let source = include_str!("epoch_engine.rs");
        assert!(
            !source.contains("Vec::with_capacity(input.candidate_changes().len())"),
            "unmetered target preparation remains before the first stop"
        );
    }

    #[test]
    fn actor_reconstruction_is_item_metered_before_each_operation() {
        let candidate = ChangeCandidate {
            change_hash: ChangeHash::from_bytes([5; 32]),
            actor: ActorId::from_bytes([6; 32]),
            sequence: 1,
            start_op: 1,
            operation_count: 1,
            dependencies: Vec::new().into(),
            control_id: EventId::from_bytes([3; 32]),
            author: DevicePublicKey::from_bytes([7; 32]),
            valid_carriers: std::sync::Arc::from([]),
        };
        let closure = BTreeSet::from([candidate.change_hash]);
        let candidates = BTreeMap::from([(candidate.change_hash, candidate)]);
        let mut exhausted = WorkBudget::new(0, 1);
        let short = crate::graph::actor_state::initialize_actor_states_metered(
            &closure,
            &candidates,
            |counter| {
                exhausted
                    .charge(counter, 1)
                    .map_err(|_| crate::graph::schedule::ScheduleError::BudgetExhausted)
            },
        );
        assert!(matches!(
            short,
            Err(crate::graph::actor_state::MeteredActorStateError::Work(
                crate::graph::schedule::ScheduleError::BudgetExhausted
            ))
        ));
        assert_eq!(exhausted.consumed().get(WorkCounter::GraphNode), 1);

        let mut exact = WorkBudget::new(0, 2);
        let result = crate::graph::actor_state::initialize_actor_states_metered(
            &closure,
            &candidates,
            |counter| {
                exact
                    .charge(counter, 1)
                    .map_err(|_| crate::graph::schedule::ScheduleError::BudgetExhausted)
            },
        );
        assert!(result.is_ok());
        assert_eq!(exact.consumed().get(WorkCounter::GraphNode), 2);
    }

    #[test]
    fn candidate_projections_charge_before_each_owned_entry() {
        let first = ChangeCandidate {
            change_hash: ChangeHash::from_bytes([5; 32]),
            actor: ActorId::from_bytes([6; 32]),
            sequence: 1,
            start_op: 1,
            operation_count: 1,
            dependencies: Vec::new().into(),
            control_id: EventId::from_bytes([3; 32]),
            author: DevicePublicKey::from_bytes([7; 32]),
            valid_carriers: std::sync::Arc::from([]),
        };
        let mut second = first.clone();
        second.change_hash = ChangeHash::from_bytes([8; 32]);
        let base = BTreeMap::from([(first.change_hash, first.clone())]);
        let local = BTreeMap::from([(second.change_hash, second.clone())]);

        let mut short = WorkBudget::new(0, 1);
        assert_eq!(
            clone_candidate_maps_metered(&base, &local, &mut short, &NeverCancelled),
            Err(crate::graph::schedule::ScheduleError::BudgetExhausted)
        );
        assert_eq!(short.consumed().get(WorkCounter::GraphNode), 1);

        let mut exact = WorkBudget::new(0, 6);
        let all = clone_candidate_maps_metered(&base, &local, &mut exact, &NeverCancelled);
        let Ok(all) = all else {
            return;
        };
        let dispositions = BTreeMap::from([(second.change_hash, ProtocolDisposition::Accepted)]);
        let eligible = collect_eligible_candidates_metered(
            &all,
            &BTreeSet::from([first.change_hash]),
            &dispositions,
            &mut exact,
            &NeverCancelled,
        );
        assert_eq!(eligible.as_ref().map(Vec::len), Ok(2));
        let projected = project_accepted_candidates_metered(
            &all,
            &BTreeSet::from([first.change_hash]),
            &dispositions,
            &mut exact,
            &NeverCancelled,
        );
        assert!(
            projected
                .is_ok_and(|(closure, candidates)| { closure.len() == 2 && candidates.len() == 2 })
        );
        assert_eq!(exact.consumed().get(WorkCounter::GraphNode), 6);
    }

    #[test]
    fn derives_actor_state_before_input_construction() {
        let hash = ChangeHash::from_bytes([5; 32]);
        let actor = ActorId::from_bytes([6; 32]);
        let candidate = ChangeCandidate {
            change_hash: hash,
            actor,
            sequence: 1,
            start_op: 1,
            operation_count: 1,
            dependencies: Vec::new().into(),
            control_id: EventId::from_bytes([3; 32]),
            author: DevicePublicKey::from_bytes([7; 32]),
            valid_carriers: vec![EventId::from_bytes([8; 32])].into(),
        };
        let accepted = AcceptedEpochState::new(
            BTreeSet::from([hash]),
            BTreeSet::from([hash]),
            BTreeMap::from([(hash, candidate)]),
            MaterializedDocumentView::empty_for_test().ok(),
        );
        assert!(accepted.is_ok_and(|state| {
            state.actor_states().get(&actor).is_some_and(|actor_state| {
                actor_state.last_sequence == 1
                    && actor_state.next_op == 2
                    && actor_state.highest_change == hash
            })
        }));
    }

    #[test]
    fn epoch_result_requires_complete_consistent_accepted_state() {
        let hash = ChangeHash::from_bytes([9; 32]);
        let actor = ActorId::from_bytes([10; 32]);
        let candidate = ChangeCandidate {
            change_hash: hash,
            actor,
            sequence: 1,
            start_op: 1,
            operation_count: 1,
            dependencies: Vec::new().into(),
            control_id: EventId::from_bytes([3; 32]),
            author: DevicePublicKey::from_bytes([11; 32]),
            valid_carriers: vec![EventId::from_bytes([12; 32])].into(),
        };
        let actors = BTreeMap::from([(
            actor,
            EpochActorState {
                last_sequence: 1,
                next_op: 2,
                highest_change: hash,
            },
        )]);
        let dispositions = BTreeMap::from([(hash, ProtocolDisposition::Accepted)]);
        let result = EpochEvaluationResult::new(
            BTreeSet::from([hash]),
            BTreeSet::from([hash]),
            BTreeMap::from([(hash, candidate.clone())]),
            dispositions.clone(),
            Vec::new(),
            MaterializedDocumentView::empty_for_test().ok(),
        );
        assert!(result.is_ok());
        let Ok(result) = result else {
            return;
        };
        assert_eq!(
            result.accepted_state().frontier_heads(),
            &BTreeSet::from([hash])
        );
        assert_eq!(result.accepted_state().actor_states(), &actors);
        assert_eq!(result.dispositions(), &dispositions);
        assert!(result.integrity_alerts().is_empty());
        let snapshot = AcceptedAtControl::from_result(&result);
        assert!(Arc::ptr_eq(
            &snapshot.accepted_closure,
            &result.accepted_state().accepted_closure_handle()
        ));
        assert!(Arc::ptr_eq(
            &snapshot.frontier_heads,
            &result.accepted_state().frontier_heads_handle()
        ));
        assert!(Arc::ptr_eq(
            &snapshot.actor_states,
            &result.accepted_state().actor_states_handle()
        ));
        assert_eq!(snapshot.accepted_closure(), &BTreeSet::from([hash]));
        assert_eq!(snapshot.frontier_heads(), &BTreeSet::from([hash]));
        assert_eq!(snapshot.actor_states(), &actors);

        let bad_head = EpochEvaluationResult::new(
            BTreeSet::from([hash]),
            BTreeSet::from([ChangeHash::from_bytes([13; 32])]),
            BTreeMap::from([(hash, candidate.clone())]),
            dispositions.clone(),
            Vec::new(),
            None,
        );
        assert!(matches!(
            bad_head,
            Err(AcceptedEpochStateError::FrontierMismatch)
        ));

        let derived = EpochEvaluationResult::new(
            BTreeSet::from([hash]),
            BTreeSet::from([hash]),
            BTreeMap::from([(hash, candidate)]),
            dispositions,
            Vec::new(),
            None,
        );
        assert!(derived.is_ok_and(|result| result.accepted_state().actor_states() == &actors));
    }
}
