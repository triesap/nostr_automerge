use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::ChangeHash;
use crate::automerge_adapter::materialized_view::MaterializedDocumentView;
use crate::control::ancestry::ControlAncestry;
use crate::control::authorize::any_control_member_metered;
use crate::control::epoch_state::{AcceptedEpochState, MeteredAcceptedEpochStateError};
use crate::control::validate::ControlEnvelope;
use crate::graph::actor_state::{
    EpochActorState, MeteredActorStateError, initialize_actor_states_metered,
};
use crate::graph::change_candidate::ChangeCandidate;
use crate::graph::closure::{CandidateClosureError, candidate_dependency_closure};
use crate::graph::dependency_graph::{
    GraphBuildError, MeteredGraphBuildError, build_graph_metered,
};
use crate::graph::epoch::{EpochAncestry, classify_epoch_ancestry_metered};
use crate::graph::equivocation::{
    QuarantineError, publish_quarantine_dispositions_metered, quarantine_equivocation_descendants,
};
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

pub(crate) fn publish_epoch_result_metered(
    accepted_state: Arc<AcceptedEpochState>,
    dispositions: BTreeMap<ChangeHash, ProtocolDisposition>,
    integrity_alerts: Vec<IntegrityAlert>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<EpochEvaluationResult, ScheduleError> {
    publish_epoch_result_observed(
        accepted_state,
        dispositions,
        integrity_alerts,
        &mut |counter| charge_epoch_item(counter, budget, cancellation),
        || {},
    )
}

fn publish_epoch_result_observed<E>(
    accepted_state: Arc<AcceptedEpochState>,
    dispositions: BTreeMap<ChangeHash, ProtocolDisposition>,
    integrity_alerts: Vec<IntegrityAlert>,
    charge: &mut impl FnMut(WorkCounter) -> Result<(), E>,
    observed: impl FnOnce(),
) -> Result<EpochEvaluationResult, E> {
    charge(WorkCounter::GraphNode)?;
    let result =
        EpochEvaluationResult::from_shared_state(accepted_state, dispositions, integrity_alerts);
    observed();
    Ok(result)
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
    let mut epoch_candidates = epoch_storage_operation(
        WorkCounter::GraphNode,
        EpochStorageOperation::EpochCandidateVectorConstruction,
        &mut |counter| charge_epoch_item(counter, budget, cancellation),
        &mut |_| {},
        Vec::new,
    )
    .map_err(EpochEvaluationError::Schedule)?;
    let mut candidate_iter = input.candidate_changes().values();
    for _ in 0..input.candidate_changes().len() {
        let candidate = epoch_storage_operation(
            WorkCounter::GraphNode,
            EpochStorageOperation::EpochCandidatePull,
            &mut |counter| charge_epoch_item(counter, budget, cancellation),
            &mut |_| {},
            || candidate_iter.next(),
        )
        .map_err(EpochEvaluationError::Schedule)?;
        let Some(candidate) = candidate else {
            return Err(EpochEvaluationError::State(
                crate::control::epoch_state::AcceptedEpochStateError::ClosureMismatch,
            ));
        };
        let candidate = epoch_storage_operation(
            WorkCounter::GraphNode,
            EpochStorageOperation::EpochCandidateClone,
            &mut |counter| charge_epoch_item(counter, budget, cancellation),
            &mut |_| {},
            || candidate.clone(),
        )
        .map_err(EpochEvaluationError::Schedule)?;
        if terminal {
            epoch_storage_operation(
                WorkCounter::GraphNode,
                EpochStorageOperation::EpochCandidatePush,
                &mut |counter| charge_epoch_item(counter, budget, cancellation),
                &mut |_| {},
                || {
                    epoch_candidates.push(EpochCandidate {
                        candidate,
                        semantically_valid: false,
                        canonical_control: false,
                    });
                },
            )
            .map_err(EpochEvaluationError::Schedule)?;
            continue;
        }
        let authorized = any_control_member_metered(
            &selected.content().members,
            |member| {
                member.actor == candidate.actor
                    && member.device == candidate.author
                    && member.roles.contains(&Role::Write)
            },
            |counter| charge_epoch_item(counter, budget, cancellation),
        )
        .map_err(EpochEvaluationError::Schedule)?;
        if !authorized {
            epoch_storage_operation(
                WorkCounter::GraphNode,
                EpochStorageOperation::EpochCandidatePush,
                &mut |counter| charge_epoch_item(counter, budget, cancellation),
                &mut |_| {},
                || {
                    epoch_candidates.push(EpochCandidate {
                        candidate,
                        semantically_valid: false,
                        canonical_control: true,
                    });
                },
            )
            .map_err(EpochEvaluationError::Schedule)?;
            continue;
        }
        let closure =
            candidate_dependency_closure(&candidate, &all_candidates, budget, cancellation)
                .map_err(|error| EpochEvaluationError::Schedule(closure_schedule_error(error)))?;
        let complete_closure = closure.missing.is_empty().then_some(&closure.known);
        let actor_counter_frontier_valid = if let Some(known) = complete_closure {
            match initialize_actor_states_metered(known, &all_candidates, |counter| {
                charge_epoch_item(counter, budget, cancellation)
            }) {
                Ok(projection) => match projection.candidate_semantics_decision_metered(
                    &candidate,
                    input.accepted_base().frontier_heads(),
                    |counter| charge_epoch_item(counter, budget, cancellation),
                ) {
                    Ok(()) => true,
                    Err(MeteredActorStateError::Work(error)) => {
                        return Err(EpochEvaluationError::Schedule(error));
                    }
                    Err(MeteredActorStateError::State(_)) => false,
                },
                Err(MeteredActorStateError::Work(error)) => {
                    return Err(EpochEvaluationError::Schedule(error));
                }
                Err(MeteredActorStateError::State(_)) => false,
            }
        } else {
            true
        };
        let ancestry = classify_epoch_ancestry_metered(
            input.accepted_base().frontier_heads(),
            &closure.known,
            &closure.missing,
            |counter| {
                charge_epoch_item(counter, budget, cancellation)
                    .map_err(EpochEvaluationError::Schedule)
            },
        )?;
        let ancestry_valid = !matches!(ancestry, EpochAncestry::InvalidOmission);
        let prior_dependencies_valid = prior_dependencies_valid_metered(
            input.prior_change_knowledge(),
            &candidate.dependencies,
            &closure.missing,
            |counter| {
                charge_epoch_item(counter, budget, cancellation)
                    .map_err(EpochEvaluationError::Schedule)
            },
        )?;
        let prior_semantics_valid =
            actor_counter_frontier_valid && ancestry_valid && prior_dependencies_valid;
        let application_valid = if !prior_semantics_valid {
            false
        } else if !closure.missing.is_empty() {
            true
        } else if !closure.cyclic.is_empty() {
            false
        } else {
            let raw = epoch_storage_operation(
                WorkCounter::GraphNode,
                EpochStorageOperation::RawChangeLookup,
                &mut |counter| charge_epoch_item(counter, budget, cancellation),
                &mut |_| {},
                || input.raw_changes().get(&candidate.change_hash),
            )
            .map_err(EpochEvaluationError::Schedule)?;
            if let Some(raw) = raw {
                let mut closure_raw = epoch_storage_operation(
                    WorkCounter::GraphNode,
                    EpochStorageOperation::RawClosureMapConstruction,
                    &mut |counter| charge_epoch_item(counter, budget, cancellation),
                    &mut |_| {},
                    BTreeMap::new,
                )
                .map_err(EpochEvaluationError::Schedule)?;
                let mut known_hashes = closure.known.iter();
                for _ in 0..closure.known.len() {
                    let hash = epoch_storage_operation(
                        WorkCounter::GraphNode,
                        EpochStorageOperation::RawClosureHashPull,
                        &mut |counter| charge_epoch_item(counter, budget, cancellation),
                        &mut |_| {},
                        || known_hashes.next(),
                    )
                    .map_err(EpochEvaluationError::Schedule)?;
                    let Some(hash) = hash else { break };
                    let value = epoch_storage_operation(
                        WorkCounter::GraphNode,
                        EpochStorageOperation::RawChangeLookup,
                        &mut |counter| charge_epoch_item(counter, budget, cancellation),
                        &mut |_| {},
                        || input.raw_changes().get(hash),
                    )
                    .map_err(EpochEvaluationError::Schedule)?;
                    let Some(value) = value else {
                        continue;
                    };
                    let value = epoch_storage_operation(
                        WorkCounter::GraphNode,
                        EpochStorageOperation::RawArcClone,
                        &mut |counter| charge_epoch_item(counter, budget, cancellation),
                        &mut |_| {},
                        || Arc::clone(value),
                    )
                    .map_err(EpochEvaluationError::Schedule)?;
                    epoch_storage_operation(
                        WorkCounter::GraphNode,
                        EpochStorageOperation::RawClosureInsert,
                        &mut |counter| charge_epoch_item(counter, budget, cancellation),
                        &mut |_| {},
                        || closure_raw.insert(*hash, value),
                    )
                    .map_err(EpochEvaluationError::Schedule)?;
                }
                let closure_complete = epoch_storage_operation(
                    WorkCounter::GraphNode,
                    EpochStorageOperation::RawClosureLengthComparison,
                    &mut |counter| charge_epoch_item(counter, budget, cancellation),
                    &mut |_| {},
                    || closure_raw.len() == closure.known.len(),
                )
                .map_err(EpochEvaluationError::Schedule)?;
                if !closure_complete {
                    false
                } else {
                    let mut dependencies = epoch_storage_operation(
                        WorkCounter::GraphNode,
                        EpochStorageOperation::DependencySetConstruction,
                        &mut |counter| charge_epoch_item(counter, budget, cancellation),
                        &mut |_| {},
                        BTreeSet::new,
                    )
                    .map_err(EpochEvaluationError::Schedule)?;
                    let mut declared_dependencies = candidate.dependencies.iter();
                    for _ in 0..candidate.dependencies.len() {
                        let dependency = epoch_storage_operation(
                            WorkCounter::GraphEdge,
                            EpochStorageOperation::DependencyPull,
                            &mut |counter| charge_epoch_item(counter, budget, cancellation),
                            &mut |_| {},
                            || declared_dependencies.next().copied(),
                        )
                        .map_err(EpochEvaluationError::Schedule)?;
                        let Some(dependency) = dependency else { break };
                        epoch_storage_operation(
                            WorkCounter::GraphEdge,
                            EpochStorageOperation::DependencyInsert,
                            &mut |counter| charge_epoch_item(counter, budget, cancellation),
                            &mut |_| {},
                            || dependencies.insert(dependency),
                        )
                        .map_err(EpochEvaluationError::Schedule)?;
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
            }
        };
        let semantically_valid = prior_semantics_valid && application_valid;
        epoch_storage_operation(
            WorkCounter::GraphNode,
            EpochStorageOperation::EpochCandidatePush,
            &mut |counter| charge_epoch_item(counter, budget, cancellation),
            &mut |_| {},
            || {
                epoch_candidates.push(EpochCandidate {
                    candidate,
                    semantically_valid,
                    canonical_control: true,
                });
            },
        )
        .map_err(EpochEvaluationError::Schedule)?;
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
    publish_quarantine_dispositions_metered(
        &quarantine.quarantined,
        &mut dispositions,
        |counter| charge_epoch_item(counter, budget, cancellation),
    )
    .map_err(EpochEvaluationError::Schedule)?;
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
        charge_epoch_item(WorkCounter::GraphNode, budget, cancellation)
            .map_err(EpochEvaluationError::Schedule)?;
        let accepted_base = input.accepted_base_handle();
        return publish_epoch_result_metered(
            accepted_base,
            dispositions,
            quarantine.alerts,
            budget,
            cancellation,
        )
        .map_err(EpochEvaluationError::Schedule);
    }
    charge_epoch_item(WorkCounter::GraphNode, budget, cancellation)
        .map_err(EpochEvaluationError::Schedule)?;
    let accepted_closure = Arc::new(accepted_closure);
    let accepted_state =
        AcceptedEpochState::new_metered(accepted_closure, accepted_candidates, None, |counter| {
            charge_epoch_item(counter, budget, cancellation)
        })
        .map_err(|error| match error {
            MeteredAcceptedEpochStateError::Work(error) => EpochEvaluationError::Schedule(error),
            MeteredAcceptedEpochStateError::State(error) => EpochEvaluationError::State(error),
        })?;
    charge_epoch_item(WorkCounter::GraphNode, budget, cancellation)
        .map_err(EpochEvaluationError::Schedule)?;
    let accepted_state = Arc::new(accepted_state);
    publish_epoch_result_metered(
        accepted_state,
        dispositions,
        quarantine.alerts,
        budget,
        cancellation,
    )
    .map_err(EpochEvaluationError::Schedule)
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EpochStorageOperation {
    EpochCandidateVectorConstruction,
    EpochCandidatePull,
    EpochCandidateClone,
    EpochCandidatePush,
    RawChangeLookup,
    RawClosureMapConstruction,
    RawClosureHashPull,
    RawArcClone,
    RawClosureInsert,
    RawClosureLengthComparison,
    DependencySetConstruction,
    DependencyPull,
    DependencyInsert,
    CandidateMapConstruction,
    CandidateEntryPull,
    CandidateClone,
    CandidateInsert,
    EligibleVectorConstruction,
    AcceptedBaseLookup,
    DispositionLookup,
    EligibleCandidateClone,
    EligiblePush,
    AcceptedClosureConstruction,
    AcceptedCandidatesConstruction,
    AcceptedClosureInsert,
    AcceptedCandidateClone,
    AcceptedCandidateInsert,
}

fn epoch_storage_operation<E, T>(
    counter: WorkCounter,
    operation: EpochStorageOperation,
    charge: &mut impl FnMut(WorkCounter) -> Result<(), E>,
    observed: &mut impl FnMut(EpochStorageOperation),
    target: impl FnOnce() -> T,
) -> Result<T, E> {
    charge(counter)?;
    let value = target();
    observed(operation);
    Ok(value)
}

pub(crate) fn clone_candidate_maps_metered(
    base: &BTreeMap<ChangeHash, ChangeCandidate>,
    local: &BTreeMap<ChangeHash, ChangeCandidate>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<BTreeMap<ChangeHash, ChangeCandidate>, ScheduleError> {
    clone_candidate_maps_observed(
        base,
        local,
        &mut |counter| charge_epoch_item(counter, budget, cancellation),
        |_| {},
    )
}

fn clone_candidate_maps_observed<E>(
    base: &BTreeMap<ChangeHash, ChangeCandidate>,
    local: &BTreeMap<ChangeHash, ChangeCandidate>,
    charge: &mut impl FnMut(WorkCounter) -> Result<(), E>,
    mut observed: impl FnMut(EpochStorageOperation),
) -> Result<BTreeMap<ChangeHash, ChangeCandidate>, E> {
    let mut result = epoch_storage_operation(
        WorkCounter::GraphNode,
        EpochStorageOperation::CandidateMapConstruction,
        charge,
        &mut observed,
        BTreeMap::new,
    )?;
    for source in [base, local] {
        let mut iter = source.iter();
        for _ in 0..source.len() {
            let entry = epoch_storage_operation(
                WorkCounter::GraphNode,
                EpochStorageOperation::CandidateEntryPull,
                charge,
                &mut observed,
                || iter.next(),
            )?;
            let Some((hash, candidate)) = entry else {
                break;
            };
            let candidate = epoch_storage_operation(
                WorkCounter::GraphNode,
                EpochStorageOperation::CandidateClone,
                charge,
                &mut observed,
                || candidate.clone(),
            )?;
            epoch_storage_operation(
                WorkCounter::GraphNode,
                EpochStorageOperation::CandidateInsert,
                charge,
                &mut observed,
                || result.insert(*hash, candidate),
            )?;
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
    collect_eligible_candidates_observed(
        candidates,
        accepted_base,
        dispositions,
        &mut |counter| charge_epoch_item(counter, budget, cancellation),
        |_| {},
    )
}

fn collect_eligible_candidates_observed<E>(
    candidates: &BTreeMap<ChangeHash, ChangeCandidate>,
    accepted_base: &BTreeSet<ChangeHash>,
    dispositions: &BTreeMap<ChangeHash, ProtocolDisposition>,
    charge: &mut impl FnMut(WorkCounter) -> Result<(), E>,
    mut observed: impl FnMut(EpochStorageOperation),
) -> Result<Vec<ChangeCandidate>, E> {
    let mut result = epoch_storage_operation(
        WorkCounter::GraphNode,
        EpochStorageOperation::EligibleVectorConstruction,
        charge,
        &mut observed,
        Vec::new,
    )?;
    let mut iter = candidates.iter();
    for _ in 0..candidates.len() {
        let entry = epoch_storage_operation(
            WorkCounter::GraphNode,
            EpochStorageOperation::CandidateEntryPull,
            charge,
            &mut observed,
            || iter.next(),
        )?;
        let Some((hash, candidate)) = entry else {
            break;
        };
        let accepted = epoch_storage_operation(
            WorkCounter::GraphNode,
            EpochStorageOperation::AcceptedBaseLookup,
            charge,
            &mut observed,
            || accepted_base.contains(hash),
        )?;
        let accepted = if accepted {
            true
        } else {
            epoch_storage_operation(
                WorkCounter::GraphNode,
                EpochStorageOperation::DispositionLookup,
                charge,
                &mut observed,
                || dispositions.get(hash) == Some(&ProtocolDisposition::Accepted),
            )?
        };
        if accepted {
            let candidate = epoch_storage_operation(
                WorkCounter::GraphNode,
                EpochStorageOperation::EligibleCandidateClone,
                charge,
                &mut observed,
                || candidate.clone(),
            )?;
            epoch_storage_operation(
                WorkCounter::GraphNode,
                EpochStorageOperation::EligiblePush,
                charge,
                &mut observed,
                || result.push(candidate),
            )?;
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
    project_accepted_candidates_observed(
        candidates,
        accepted_base,
        dispositions,
        &mut |counter| charge_epoch_item(counter, budget, cancellation),
        |_| {},
    )
}

fn project_accepted_candidates_observed<E>(
    candidates: &BTreeMap<ChangeHash, ChangeCandidate>,
    accepted_base: &BTreeSet<ChangeHash>,
    dispositions: &BTreeMap<ChangeHash, ProtocolDisposition>,
    charge: &mut impl FnMut(WorkCounter) -> Result<(), E>,
    mut observed: impl FnMut(EpochStorageOperation),
) -> Result<(BTreeSet<ChangeHash>, BTreeMap<ChangeHash, ChangeCandidate>), E> {
    let mut accepted_closure = epoch_storage_operation(
        WorkCounter::GraphNode,
        EpochStorageOperation::AcceptedClosureConstruction,
        charge,
        &mut observed,
        BTreeSet::new,
    )?;
    let mut accepted_candidates = epoch_storage_operation(
        WorkCounter::GraphNode,
        EpochStorageOperation::AcceptedCandidatesConstruction,
        charge,
        &mut observed,
        BTreeMap::new,
    )?;
    let mut iter = candidates.iter();
    for _ in 0..candidates.len() {
        let entry = epoch_storage_operation(
            WorkCounter::GraphNode,
            EpochStorageOperation::CandidateEntryPull,
            charge,
            &mut observed,
            || iter.next(),
        )?;
        let Some((hash, candidate)) = entry else {
            break;
        };
        let accepted = epoch_storage_operation(
            WorkCounter::GraphNode,
            EpochStorageOperation::AcceptedBaseLookup,
            charge,
            &mut observed,
            || accepted_base.contains(hash),
        )?;
        let accepted = if accepted {
            true
        } else {
            epoch_storage_operation(
                WorkCounter::GraphNode,
                EpochStorageOperation::DispositionLookup,
                charge,
                &mut observed,
                || dispositions.get(hash) == Some(&ProtocolDisposition::Accepted),
            )?
        };
        let excluded = epoch_storage_operation(
            WorkCounter::GraphNode,
            EpochStorageOperation::DispositionLookup,
            charge,
            &mut observed,
            || dispositions.get(hash) == Some(&ProtocolDisposition::Excluded),
        )?;
        if accepted && !excluded {
            epoch_storage_operation(
                WorkCounter::GraphNode,
                EpochStorageOperation::AcceptedClosureInsert,
                charge,
                &mut observed,
                || accepted_closure.insert(*hash),
            )?;
            let candidate = epoch_storage_operation(
                WorkCounter::GraphNode,
                EpochStorageOperation::AcceptedCandidateClone,
                charge,
                &mut observed,
                || candidate.clone(),
            )?;
            epoch_storage_operation(
                WorkCounter::GraphNode,
                EpochStorageOperation::AcceptedCandidateInsert,
                charge,
                &mut observed,
                || accepted_candidates.insert(*hash, candidate),
            )?;
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
        EpochStorageOperation, PriorChangeKnowledge, PriorKnowledgeState,
        clone_candidate_maps_metered, clone_candidate_maps_observed,
        collect_eligible_candidates_metered, collect_eligible_candidates_observed, evaluate_epoch,
        prior_dependencies_valid_metered, project_accepted_candidates_metered,
        project_accepted_candidates_observed, publish_epoch_result_observed,
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
        control_with_members(base_heads, Vec::new(), true)
    }

    fn control_with_members(
        base_heads: Vec<ChangeHash>,
        members: Vec<DeviceGrant>,
        terminal: bool,
    ) -> ControlEnvelope {
        let controller = ControllerPublicKey::from_bytes([1; 32]);
        let coordinate = DocumentCoordinate::new(controller, DocumentId::from_bytes([2; 32]));
        ControlEnvelope::from_validated(ValidatedControlCarrier::for_test(
            EventId::from_bytes([3; 32]),
            controller,
            coordinate,
            None,
            ValidatedControlContent {
                base_heads,
                members,
                predecessor: None,
                sequence: 0,
                successor: None,
                terminal,
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
        let production = source
            .split("#[cfg(test)]\nmod tests")
            .next()
            .unwrap_or(source);
        assert!(
            !production.contains("selected.content().members.iter().any"),
            "unmetered epoch writer authorization scan remains"
        );
        assert!(production.contains("any_control_member_metered("));
    }

    #[test]
    fn epoch_writer_refusal_precedes_dependency_work_and_preserves_typed_stops() {
        let author = DevicePublicKey::from_bytes([7; 32]);
        let coordinate = DocumentCoordinate::new(
            ControllerPublicKey::from_bytes([1; 32]),
            DocumentId::from_bytes([2; 32]),
        );
        let denied_device = DevicePublicKey::from_bytes([8; 32]);
        let denied_control = control_with_members(
            Vec::new(),
            vec![DeviceGrant {
                account: None,
                actor: ActorId::derive(coordinate, denied_device),
                device: denied_device,
                roles: vec![crate::types::role::Role::Checkpoint],
            }],
            false,
        );
        let candidate = ChangeCandidate {
            change_hash: ChangeHash::from_bytes([5; 32]),
            actor: ActorId::from_bytes([6; 32]),
            sequence: 1,
            start_op: 1,
            operation_count: 1,
            dependencies: vec![ChangeHash::from_bytes([9; 32])].into(),
            control_id: EventId::from_bytes([3; 32]),
            author,
            valid_carriers: Arc::from([]),
        };
        let input = EpochEvaluationInput::new(
            denied_control,
            empty_state(),
            [candidate.clone()],
            Vec::new(),
        );
        assert!(input.is_ok());
        let Ok(input) = input else { return };

        let mut ample = WorkBudget::new(1_000, 1_000);
        let result = evaluate_epoch(&input, &mut ample, &NeverCancelled);
        assert!(result.is_ok_and(|result| {
            result.dispositions().get(&candidate.change_hash) == Some(&ProtocolDisposition::Invalid)
        }));
        assert_eq!(ample.consumed().get(WorkCounter::Control), 3);
        assert_eq!(ample.consumed().get(WorkCounter::GraphEdge), 3);
        assert_eq!(ample.consumed().get(WorkCounter::ApplyChange), 0);

        for (capacity, expected_control) in [(7, 0), (8, 1), (9, 2)] {
            let mut budget = WorkBudget::new(1_000, capacity);
            let result = evaluate_epoch(&input, &mut budget, &NeverCancelled);
            assert!(matches!(
                result,
                Err(super::EpochEvaluationError::Schedule(
                    crate::graph::schedule::ScheduleError::BudgetExhausted
                ))
            ));
            assert_eq!(
                budget.consumed().get(WorkCounter::Control),
                expected_control
            );
            assert_eq!(budget.consumed().get(WorkCounter::GraphEdge), 0);
        }

        for (cancel_at, expected_control) in [(7, 0), (8, 1), (9, 2)] {
            let calls = std::cell::Cell::new(0_u64);
            let cancellation = || {
                let current = calls.get();
                calls.set(current.saturating_add(1));
                current == cancel_at
            };
            let mut budget = WorkBudget::new(1_000, 1_000);
            let result = evaluate_epoch(&input, &mut budget, &cancellation);
            assert!(matches!(
                result,
                Err(super::EpochEvaluationError::Schedule(
                    crate::graph::schedule::ScheduleError::Cancelled
                ))
            ));
            assert_eq!(
                budget.consumed().get(WorkCounter::Control),
                expected_control
            );
            assert_eq!(budget.consumed().get(WorkCounter::GraphEdge), 0);
        }
    }

    #[test]
    fn finding_100_quarantine_overlay_work_reproduction() {
        let selected_source = include_str!("epoch_engine.rs");
        let fallback_source = include_str!("evaluate.rs");
        let direct_loop = concat!("for hash in &quarantine.", "quarantined");
        assert!(
            !selected_source.contains(direct_loop)
                && !fallback_source.contains(direct_loop)
                && selected_source.contains("publish_quarantine_dispositions_metered(")
                && fallback_source.contains("publish_quarantine_dispositions_metered("),
            "unmetered selected and fallback quarantine overlays remain"
        );
    }

    #[test]
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
        let unmetered_capacity = concat!("Vec::with_capacity(input.", "candidate_changes().len())");
        assert!(
            !source.contains(unmetered_capacity),
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

        let mut exact = WorkBudget::new(0, 25);
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
        assert_eq!(exact.consumed().get(WorkCounter::GraphNode), 24);
        assert_eq!(exact.consumed().get(WorkCounter::GraphEdge), 1);
    }

    #[test]
    fn candidate_projections_charge_before_each_owned_entry() {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum Stop {
            BudgetExhausted,
            Cancelled,
        }
        type ProjectionRun = (
            BTreeMap<ChangeHash, ChangeCandidate>,
            Vec<ChangeCandidate>,
            (BTreeSet<ChangeHash>, BTreeMap<ChangeHash, ChangeCandidate>),
        );
        fn run<F>(
            base: &BTreeMap<ChangeHash, ChangeCandidate>,
            local: &BTreeMap<ChangeHash, ChangeCandidate>,
            accepted: &BTreeSet<ChangeHash>,
            dispositions: &BTreeMap<ChangeHash, ProtocolDisposition>,
            charge: &mut F,
            observed: &mut Vec<EpochStorageOperation>,
        ) -> Result<ProjectionRun, Stop>
        where
            F: FnMut(WorkCounter) -> Result<(), Stop>,
        {
            let mut epoch_candidates = super::epoch_storage_operation(
                WorkCounter::GraphNode,
                EpochStorageOperation::EpochCandidateVectorConstruction,
                charge,
                &mut |operation| observed.push(operation),
                Vec::new,
            )?;
            let mut candidate_values = base.values();
            let candidate = super::epoch_storage_operation(
                WorkCounter::GraphNode,
                EpochStorageOperation::EpochCandidatePull,
                charge,
                &mut |operation| observed.push(operation),
                || candidate_values.next(),
            )?;
            if let Some(candidate) = candidate {
                let candidate = super::epoch_storage_operation(
                    WorkCounter::GraphNode,
                    EpochStorageOperation::EpochCandidateClone,
                    charge,
                    &mut |operation| observed.push(operation),
                    || candidate.clone(),
                )?;
                super::epoch_storage_operation(
                    WorkCounter::GraphNode,
                    EpochStorageOperation::EpochCandidatePush,
                    charge,
                    &mut |operation| observed.push(operation),
                    || {
                        epoch_candidates.push(crate::reference::epoch::EpochCandidate {
                            candidate,
                            semantically_valid: true,
                            canonical_control: true,
                        });
                    },
                )?;
            }
            let all = clone_candidate_maps_observed(base, local, charge, |operation| {
                observed.push(operation);
            })?;
            let eligible = collect_eligible_candidates_observed(
                &all,
                accepted,
                dispositions,
                charge,
                |operation| observed.push(operation),
            )?;
            let projected = project_accepted_candidates_observed(
                &all,
                accepted,
                dispositions,
                charge,
                |operation| observed.push(operation),
            )?;
            Ok((all, eligible, projected))
        }
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
        let accepted = BTreeSet::from([first.change_hash]);
        let dispositions = BTreeMap::from([(second.change_hash, ProtocolDisposition::Accepted)]);

        let mut operations = Vec::new();
        let complete = run(
            &base,
            &local,
            &accepted,
            &dispositions,
            &mut |_| Ok(()),
            &mut operations,
        );
        assert!(complete.as_ref().is_ok_and(|(all, eligible, projected)| {
            all.len() == 2
                && eligible.len() == 2
                && projected.0.len() == 2
                && projected.1.len() == 2
        }));
        assert_eq!(operations.len(), 36);
        for operation in [
            EpochStorageOperation::EpochCandidateVectorConstruction,
            EpochStorageOperation::EpochCandidatePull,
            EpochStorageOperation::EpochCandidateClone,
            EpochStorageOperation::EpochCandidatePush,
            EpochStorageOperation::CandidateMapConstruction,
            EpochStorageOperation::CandidateEntryPull,
            EpochStorageOperation::CandidateClone,
            EpochStorageOperation::CandidateInsert,
            EpochStorageOperation::EligibleVectorConstruction,
            EpochStorageOperation::AcceptedBaseLookup,
            EpochStorageOperation::DispositionLookup,
            EpochStorageOperation::EligibleCandidateClone,
            EpochStorageOperation::EligiblePush,
            EpochStorageOperation::AcceptedClosureConstruction,
            EpochStorageOperation::AcceptedCandidatesConstruction,
            EpochStorageOperation::AcceptedClosureInsert,
            EpochStorageOperation::AcceptedCandidateClone,
            EpochStorageOperation::AcceptedCandidateInsert,
        ] {
            assert!(operations.contains(&operation));
        }
        for allowance in 0..operations.len() {
            for stop in [Stop::BudgetExhausted, Stop::Cancelled] {
                let mut successful = 0_usize;
                let mut observed = Vec::new();
                let result = run(
                    &base,
                    &local,
                    &accepted,
                    &dispositions,
                    &mut |_| {
                        if successful == allowance {
                            return Err(stop);
                        }
                        successful += 1;
                        Ok(())
                    },
                    &mut observed,
                );
                assert_eq!(result, Err(stop));
                assert_eq!(successful, allowance);
                assert_eq!(observed, operations[..allowance]);
            }
        }

        let mut short = WorkBudget::new(0, 1);
        assert_eq!(
            clone_candidate_maps_metered(&base, &local, &mut short, &NeverCancelled),
            Err(crate::graph::schedule::ScheduleError::BudgetExhausted)
        );
        assert_eq!(short.consumed().get(WorkCounter::GraphNode), 1);

        let mut exact = WorkBudget::new(0, 32);
        let all = clone_candidate_maps_metered(&base, &local, &mut exact, &NeverCancelled);
        let Ok(all) = all else {
            return;
        };
        let eligible = collect_eligible_candidates_metered(
            &all,
            &accepted,
            &dispositions,
            &mut exact,
            &NeverCancelled,
        );
        assert_eq!(eligible.as_ref().map(Vec::len), Ok(2));
        let projected = project_accepted_candidates_metered(
            &all,
            &accepted,
            &dispositions,
            &mut exact,
            &NeverCancelled,
        );
        assert!(
            projected
                .is_ok_and(|(closure, candidates)| { closure.len() == 2 && candidates.len() == 2 })
        );
        assert_eq!(exact.consumed().get(WorkCounter::GraphNode), 32);
    }

    #[test]
    fn epoch_result_publication_is_charged_immediately_before_construction() {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum Stop {
            BudgetExhausted,
            Cancelled,
        }
        for stop in [Stop::BudgetExhausted, Stop::Cancelled] {
            let observed = std::cell::Cell::new(0_usize);
            let result = publish_epoch_result_observed(
                Arc::new(empty_state()),
                BTreeMap::new(),
                Vec::new(),
                &mut |_| Err(stop),
                || observed.set(observed.get() + 1),
            );
            assert!(matches!(result, Err(error) if error == stop));
            assert_eq!(observed.get(), 0);
        }
        let observed = std::cell::Cell::new(0_usize);
        let complete = publish_epoch_result_observed(
            Arc::new(empty_state()),
            BTreeMap::new(),
            Vec::new(),
            &mut |_| Ok::<_, Stop>(()),
            || observed.set(observed.get() + 1),
        );
        assert!(complete.is_ok());
        assert_eq!(observed.get(), 1);
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
