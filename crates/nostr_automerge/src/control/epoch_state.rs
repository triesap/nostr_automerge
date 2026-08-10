use std::collections::{BTreeMap, BTreeSet};

use crate::automerge_adapter::materialized_view::MaterializedDocumentView;
use crate::graph::actor_state::{ActorStateError, EpochActorState, initialize_actor_states};
use crate::graph::change_candidate::ChangeCandidate;
use crate::{ActorId, ChangeHash};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AcceptedEpochStateError {
    ClosureMismatch,
    FrontierMismatch,
    ActorState(ActorStateError),
    ActorStateMismatch,
    WriterContributionMismatch,
}

/// Complete accepted state at one canonical control.
///
/// Fields are deliberately private so later control selection cannot assemble
/// a partial parent view from independently sourced collections.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct AcceptedEpochState {
    accepted_closure: BTreeSet<ChangeHash>,
    frontier_heads: BTreeSet<ChangeHash>,
    accepted_candidates: BTreeMap<ChangeHash, ChangeCandidate>,
    dependencies: BTreeMap<ChangeHash, BTreeSet<ChangeHash>>,
    actor_states: BTreeMap<ActorId, EpochActorState>,
    writer_contributions: BTreeMap<ActorId, ChangeHash>,
    materialized: MaterializedDocumentView,
}

impl AcceptedEpochState {
    pub(crate) fn new(
        accepted_closure: BTreeSet<ChangeHash>,
        frontier_heads: BTreeSet<ChangeHash>,
        accepted_candidates: BTreeMap<ChangeHash, ChangeCandidate>,
        actor_states: BTreeMap<ActorId, EpochActorState>,
        writer_contributions: BTreeMap<ActorId, ChangeHash>,
        materialized: MaterializedDocumentView,
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
        let derived_actor_states = initialize_actor_states(accepted_candidates.values().cloned())
            .map_err(AcceptedEpochStateError::ActorState)?;
        if actor_states != derived_actor_states {
            return Err(AcceptedEpochStateError::ActorStateMismatch);
        }
        if writer_contributions.iter().any(|(actor, hash)| {
            actor_states
                .get(actor)
                .is_none_or(|state| state.highest_change != *hash)
        }) {
            return Err(AcceptedEpochStateError::WriterContributionMismatch);
        }
        Ok(Self {
            accepted_closure,
            frontier_heads,
            accepted_candidates,
            dependencies,
            actor_states,
            writer_contributions,
            materialized,
        })
    }

    pub(crate) fn accepted_closure(&self) -> &BTreeSet<ChangeHash> {
        &self.accepted_closure
    }

    pub(crate) fn frontier_heads(&self) -> &BTreeSet<ChangeHash> {
        &self.frontier_heads
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

    pub(crate) fn writer_contributions(&self) -> &BTreeMap<ActorId, ChangeHash> {
        &self.writer_contributions
    }

    pub(crate) const fn materialized(&self) -> &MaterializedDocumentView {
        &self.materialized
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use automerge::Automerge;

    use super::{AcceptedEpochState, AcceptedEpochStateError};
    use crate::automerge_adapter::materialized_view::MaterializedDocumentView;
    use crate::graph::actor_state::EpochActorState;
    use crate::graph::change_candidate::ChangeCandidate;
    use crate::{ActorId, ChangeHash, DevicePublicKey, EventId};

    fn candidate(hash: u8) -> ChangeCandidate {
        ChangeCandidate {
            change_hash: ChangeHash::from_bytes([hash; 32]),
            actor: ActorId::from_bytes([1; 32]),
            sequence: 1,
            start_op: 1,
            operation_count: 1,
            dependencies: Vec::new(),
            control_id: EventId::from_bytes([2; 32]),
            author: DevicePublicKey::from_bytes([3; 32]),
            valid_carriers: BTreeSet::from([EventId::from_bytes([4; 32])]),
        }
    }

    type Parts = (
        BTreeSet<ChangeHash>,
        BTreeSet<ChangeHash>,
        BTreeMap<ChangeHash, ChangeCandidate>,
        BTreeMap<ActorId, EpochActorState>,
        BTreeMap<ActorId, ChangeHash>,
    );

    fn materialized() -> Option<MaterializedDocumentView> {
        let document = Automerge::new();
        MaterializedDocumentView::from_canonical_bytes(document.save_nocompress()).ok()
    }

    fn parts() -> Parts {
        let change = candidate(5);
        let hash = change.change_hash;
        let actor = change.actor;
        (
            BTreeSet::from([hash]),
            BTreeSet::from([hash]),
            BTreeMap::from([(hash, change)]),
            BTreeMap::from([(
                actor,
                EpochActorState {
                    last_sequence: 1,
                    next_op: 2,
                    highest_change: hash,
                },
            )]),
            BTreeMap::from([(actor, hash)]),
        )
    }

    #[test]
    fn rejects_inconsistent_heads_closure_and_actor_state() {
        let (closure, heads, candidates, actors, writers) = parts();
        let materialized = materialized();
        assert!(materialized.is_some());
        let Some(materialized) = materialized else {
            return;
        };
        let state = AcceptedEpochState::new(
            closure.clone(),
            heads,
            candidates.clone(),
            actors.clone(),
            writers.clone(),
            materialized.clone(),
        );
        assert!(state.is_ok());

        assert!(matches!(
            AcceptedEpochState::new(
                BTreeSet::new(),
                BTreeSet::new(),
                candidates.clone(),
                actors.clone(),
                writers.clone(),
                materialized.clone(),
            ),
            Err(AcceptedEpochStateError::ClosureMismatch)
        ));
        assert!(matches!(
            AcceptedEpochState::new(
                closure.clone(),
                BTreeSet::new(),
                candidates.clone(),
                actors.clone(),
                writers.clone(),
                materialized.clone(),
            ),
            Err(AcceptedEpochStateError::FrontierMismatch)
        ));
        assert!(matches!(
            AcceptedEpochState::new(
                closure,
                BTreeSet::from([ChangeHash::from_bytes([5; 32])]),
                candidates,
                BTreeMap::new(),
                writers,
                materialized,
            ),
            Err(AcceptedEpochStateError::ActorStateMismatch)
        ));
    }
}
