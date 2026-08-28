use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::automerge_adapter::materialized_view::MaterializedDocumentView;
use crate::graph::actor_state::{
    ActorStateError, EpochActorState, MeteredActorStateError, initialize_actor_states,
    initialize_actor_states_metered,
};
use crate::graph::change_candidate::ChangeCandidate;
use crate::{ActorId, ChangeHash, WorkCounter};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AcceptedEpochStateError {
    ClosureMismatch,
    FrontierMismatch,
    ActorState(ActorStateError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MeteredAcceptedEpochStateError<E> {
    Work(E),
    State(AcceptedEpochStateError),
}

/// Complete accepted state at one canonical control.
///
/// Fields are deliberately private so later control selection cannot assemble
/// a partial parent view from independently sourced collections.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct AcceptedEpochState {
    accepted_closure: Arc<BTreeSet<ChangeHash>>,
    frontier_heads: Arc<BTreeSet<ChangeHash>>,
    accepted_candidates: BTreeMap<ChangeHash, ChangeCandidate>,
    dependencies: BTreeMap<ChangeHash, BTreeSet<ChangeHash>>,
    actor_states: Arc<BTreeMap<ActorId, EpochActorState>>,
    writer_contributions: BTreeMap<ActorId, ChangeHash>,
    materialized: Option<MaterializedDocumentView>,
}

impl AcceptedEpochState {
    pub(crate) fn new(
        accepted_closure: BTreeSet<ChangeHash>,
        frontier_heads: BTreeSet<ChangeHash>,
        accepted_candidates: BTreeMap<ChangeHash, ChangeCandidate>,
        materialized: Option<MaterializedDocumentView>,
    ) -> Result<Self, AcceptedEpochStateError> {
        if accepted_closure != accepted_candidates.keys().copied().collect() {
            return Err(AcceptedEpochStateError::ClosureMismatch);
        }
        let dependencies = accepted_candidates
            .iter()
            .map(|(hash, candidate)| {
                (
                    *hash,
                    candidate
                        .dependencies
                        .iter()
                        .copied()
                        .collect::<BTreeSet<_>>(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        if dependencies
            .values()
            .any(|items| !items.is_subset(&accepted_closure))
        {
            return Err(AcceptedEpochStateError::ClosureMismatch);
        }
        let depended_on = dependencies
            .values()
            .flatten()
            .copied()
            .collect::<BTreeSet<_>>();
        let exact_heads = accepted_closure
            .difference(&depended_on)
            .copied()
            .collect::<BTreeSet<_>>();
        if frontier_heads != exact_heads {
            return Err(AcceptedEpochStateError::FrontierMismatch);
        }
        let actor_states = initialize_actor_states(accepted_candidates.values().cloned())
            .map_err(AcceptedEpochStateError::ActorState)?;
        let writer_contributions = actor_states
            .iter()
            .map(|(actor, state)| (*actor, state.highest_change))
            .collect();
        Ok(Self {
            accepted_closure: Arc::new(accepted_closure),
            frontier_heads: Arc::new(frontier_heads),
            accepted_candidates,
            dependencies,
            actor_states: Arc::new(actor_states),
            writer_contributions,
            materialized,
        })
    }

    pub(crate) fn new_metered<E>(
        accepted_closure: Arc<BTreeSet<ChangeHash>>,
        accepted_candidates: BTreeMap<ChangeHash, ChangeCandidate>,
        materialized: Option<MaterializedDocumentView>,
        mut charge: impl FnMut(WorkCounter) -> Result<(), E>,
    ) -> Result<Self, MeteredAcceptedEpochStateError<E>> {
        let projection =
            initialize_actor_states_metered(&accepted_closure, &accepted_candidates, &mut charge)
                .map_err(|error| match error {
                MeteredActorStateError::Work(error) => MeteredAcceptedEpochStateError::Work(error),
                MeteredActorStateError::State(error) => MeteredAcceptedEpochStateError::State(
                    AcceptedEpochStateError::ActorState(error),
                ),
            })?;
        let (frontier_heads, dependencies, actor_states, writer_contributions) =
            projection.into_accepted_state_parts();

        Ok(Self {
            accepted_closure,
            frontier_heads: Arc::new(frontier_heads),
            accepted_candidates,
            dependencies,
            actor_states: Arc::new(actor_states),
            writer_contributions,
            materialized,
        })
    }

    pub(crate) fn accepted_closure(&self) -> &BTreeSet<ChangeHash> {
        &self.accepted_closure
    }

    pub(crate) fn accepted_closure_handle(&self) -> Arc<BTreeSet<ChangeHash>> {
        Arc::clone(&self.accepted_closure)
    }

    pub(crate) fn frontier_heads(&self) -> &BTreeSet<ChangeHash> {
        &self.frontier_heads
    }

    pub(crate) fn frontier_heads_handle(&self) -> Arc<BTreeSet<ChangeHash>> {
        Arc::clone(&self.frontier_heads)
    }

    pub(crate) fn accepted_candidates(&self) -> &BTreeMap<ChangeHash, ChangeCandidate> {
        &self.accepted_candidates
    }

    pub(crate) fn dependencies(&self) -> &BTreeMap<ChangeHash, BTreeSet<ChangeHash>> {
        &self.dependencies
    }

    pub(crate) fn actor_states(&self) -> &BTreeMap<ActorId, EpochActorState> {
        &self.actor_states
    }

    pub(crate) fn actor_states_handle(&self) -> Arc<BTreeMap<ActorId, EpochActorState>> {
        Arc::clone(&self.actor_states)
    }

    pub(crate) fn writer_contributions(&self) -> &BTreeMap<ActorId, ChangeHash> {
        &self.writer_contributions
    }

    pub(crate) const fn materialized(&self) -> Option<&MaterializedDocumentView> {
        self.materialized.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::collections::{BTreeMap, BTreeSet};
    use std::sync::Arc;

    use super::{AcceptedEpochState, AcceptedEpochStateError, MeteredAcceptedEpochStateError};
    use crate::automerge_adapter::materialized_view::MaterializedDocumentView;
    use crate::graph::change_candidate::ChangeCandidate;
    use crate::{ActorId, ChangeHash, Completion, DevicePublicKey, EventId, WorkBudget};

    fn candidate(hash: u8) -> ChangeCandidate {
        ChangeCandidate {
            change_hash: ChangeHash::from_bytes([hash; 32]),
            actor: ActorId::from_bytes([1; 32]),
            sequence: 1,
            start_op: 1,
            operation_count: 1,
            dependencies: Vec::new().into(),
            control_id: EventId::from_bytes([2; 32]),
            author: DevicePublicKey::from_bytes([3; 32]),
            valid_carriers: vec![EventId::from_bytes([4; 32])].into(),
        }
    }

    type Parts = (
        BTreeSet<ChangeHash>,
        BTreeSet<ChangeHash>,
        BTreeMap<ChangeHash, ChangeCandidate>,
    );

    fn materialized() -> Option<MaterializedDocumentView> {
        MaterializedDocumentView::empty_for_test().ok()
    }

    fn parts() -> Parts {
        let change = candidate(5);
        let hash = change.change_hash;
        (
            BTreeSet::from([hash]),
            BTreeSet::from([hash]),
            BTreeMap::from([(hash, change)]),
        )
    }

    fn graph_candidate(
        hash: u8,
        actor: u8,
        sequence: u64,
        start_op: u64,
        dependencies: Vec<ChangeHash>,
    ) -> ChangeCandidate {
        ChangeCandidate {
            change_hash: ChangeHash::from_bytes([hash; 32]),
            actor: ActorId::from_bytes([actor; 32]),
            sequence,
            start_op,
            operation_count: 1,
            dependencies: dependencies.into(),
            control_id: EventId::from_bytes([2; 32]),
            author: DevicePublicKey::from_bytes([actor; 32]),
            valid_carriers: vec![EventId::from_bytes([hash; 32])].into(),
        }
    }

    fn build_with_limit(
        candidates: BTreeMap<ChangeHash, ChangeCandidate>,
        max_items: u64,
        cancel_at: Option<u64>,
    ) -> (
        Result<AcceptedEpochState, MeteredAcceptedEpochStateError<Completion>>,
        WorkBudget,
        u64,
    ) {
        let closure = Arc::new(candidates.keys().copied().collect());
        let mut budget = WorkBudget::new(0, max_items);
        let observations = Cell::new(0_u64);
        let result = AcceptedEpochState::new_metered(closure, candidates, None, |counter| {
            let current = observations.get();
            observations.set(current + 1);
            if cancel_at == Some(current) {
                return Err(Completion::Cancelled);
            }
            budget
                .charge(counter, 1)
                .map_err(|_| Completion::BudgetExhausted)
        });
        (result, budget, observations.get())
    }

    #[test]
    fn rejects_inconsistent_heads_and_derives_actor_state() {
        let (closure, heads, candidates) = parts();
        let materialized = materialized();
        assert!(materialized.is_some());
        let Some(materialized) = materialized else {
            return;
        };
        let state = AcceptedEpochState::new(
            closure.clone(),
            heads,
            candidates.clone(),
            Some(materialized.clone()),
        );
        assert!(state.is_ok());

        assert!(matches!(
            AcceptedEpochState::new(
                BTreeSet::new(),
                BTreeSet::new(),
                candidates.clone(),
                Some(materialized.clone()),
            ),
            Err(AcceptedEpochStateError::ClosureMismatch)
        ));
        assert!(matches!(
            AcceptedEpochState::new(
                closure.clone(),
                BTreeSet::new(),
                candidates.clone(),
                Some(materialized.clone()),
            ),
            Err(AcceptedEpochStateError::FrontierMismatch)
        ));
        let derived = AcceptedEpochState::new(
            closure,
            BTreeSet::from([ChangeHash::from_bytes([5; 32])]),
            candidates,
            Some(materialized),
        );
        assert!(derived.is_ok_and(|state| {
            state.actor_states().values().all(|actor| {
                actor.last_sequence == 1
                    && actor.next_op == 2
                    && actor.highest_change == ChangeHash::from_bytes([5; 32])
            })
        }));
    }

    #[test]
    fn metered_builder_has_exact_deep_wide_and_dense_boundaries() {
        let mut deep = BTreeMap::new();
        let mut prior = None;
        for index in 1_u8..=8 {
            let dependencies = prior.into_iter().collect::<Vec<_>>();
            let candidate =
                graph_candidate(index, 1, u64::from(index), u64::from(index), dependencies);
            prior = Some(candidate.change_hash);
            deep.insert(candidate.change_hash, candidate);
        }
        let (deep_exact, deep_budget, _) = build_with_limit(deep.clone(), 61, None);
        assert!(deep_exact.is_ok());
        assert_eq!(
            deep_budget.consumed().get(crate::WorkCounter::GraphNode),
            40
        );
        assert_eq!(
            deep_budget.consumed().get(crate::WorkCounter::GraphEdge),
            21
        );
        let (deep_short, deep_short_budget, _) = build_with_limit(deep, 60, None);
        assert!(matches!(
            deep_short,
            Err(MeteredAcceptedEpochStateError::Work(
                Completion::BudgetExhausted
            ))
        ));
        assert_eq!(
            deep_short_budget
                .consumed()
                .get(crate::WorkCounter::GraphNode)
                + deep_short_budget
                    .consumed()
                    .get(crate::WorkCounter::GraphEdge),
            60
        );

        let wide = (1_u8..=8)
            .map(|index| {
                let candidate = graph_candidate(index, index, 1, 1, Vec::new());
                (candidate.change_hash, candidate)
            })
            .collect::<BTreeMap<_, _>>();
        let (wide_exact, wide_budget, _) = build_with_limit(wide, 40, None);
        assert!(wide_exact.is_ok());
        assert_eq!(
            wide_budget.consumed().get(crate::WorkCounter::GraphNode),
            40
        );
        assert_eq!(wide_budget.consumed().get(crate::WorkCounter::GraphEdge), 0);

        let mut dense = BTreeMap::new();
        let mut earlier = Vec::new();
        for index in 1_u8..=8 {
            let candidate = graph_candidate(index, index, 1, u64::from(index), earlier.clone());
            earlier.push(candidate.change_hash);
            dense.insert(candidate.change_hash, candidate);
        }
        let (dense_exact, dense_budget, _) = build_with_limit(dense.clone(), 124, None);
        assert!(dense_exact.is_ok());
        assert_eq!(
            dense_budget.consumed().get(crate::WorkCounter::GraphNode),
            40
        );
        assert_eq!(
            dense_budget.consumed().get(crate::WorkCounter::GraphEdge),
            84
        );
        let (dense_cancelled, dense_cancelled_budget, observations) =
            build_with_limit(dense, 124, Some(50));
        assert!(matches!(
            dense_cancelled,
            Err(MeteredAcceptedEpochStateError::Work(Completion::Cancelled))
        ));
        assert_eq!(observations, 51);
        assert_eq!(
            dense_cancelled_budget
                .consumed()
                .get(crate::WorkCounter::GraphNode)
                + dense_cancelled_budget
                    .consumed()
                    .get(crate::WorkCounter::GraphEdge),
            50
        );
    }
}
