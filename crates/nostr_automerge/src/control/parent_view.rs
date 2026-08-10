use std::collections::{BTreeMap, BTreeSet};

use crate::control::epoch_state::AcceptedEpochState;
use crate::graph::actor_state::EpochActorState;
use crate::{ActorId, ChangeHash};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ParentEpochView {
    accepted: BTreeSet<ChangeHash>,
    heads: BTreeSet<ChangeHash>,
    dependencies: BTreeMap<ChangeHash, BTreeSet<ChangeHash>>,
    actors: BTreeMap<ActorId, EpochActorState>,
    writer_contributions: BTreeMap<ActorId, ChangeHash>,
}

impl ParentEpochView {
    pub(crate) fn from_accepted_state(state: &AcceptedEpochState) -> Self {
        Self {
            accepted: state.accepted_closure().clone(),
            heads: state.frontier_heads().clone(),
            dependencies: state.dependencies().clone(),
            actors: state.actor_states().clone(),
            writer_contributions: state.writer_contributions().clone(),
        }
    }

    #[cfg(test)]
    pub(crate) fn from_parts_for_test(
        accepted: BTreeSet<ChangeHash>,
        heads: BTreeSet<ChangeHash>,
        dependencies: BTreeMap<ChangeHash, BTreeSet<ChangeHash>>,
        actors: BTreeMap<ActorId, EpochActorState>,
        writer_contributions: BTreeMap<ActorId, ChangeHash>,
    ) -> Self {
        Self {
            accepted,
            heads,
            dependencies,
            actors,
            writer_contributions,
        }
    }

    pub(crate) fn contains(&self, hash: &ChangeHash) -> bool {
        self.accepted.contains(hash)
    }

    pub(crate) fn accepted(&self) -> &BTreeSet<ChangeHash> {
        &self.accepted
    }

    pub(crate) fn heads(&self) -> &BTreeSet<ChangeHash> {
        &self.heads
    }

    pub(crate) fn dependencies(&self, hash: &ChangeHash) -> Option<&BTreeSet<ChangeHash>> {
        self.dependencies.get(hash)
    }

    pub(crate) fn dependency_index(&self) -> &BTreeMap<ChangeHash, BTreeSet<ChangeHash>> {
        &self.dependencies
    }

    pub(crate) fn actor_state(&self, actor: &ActorId) -> Option<EpochActorState> {
        self.actors.get(actor).copied()
    }

    pub(crate) fn writer_contribution(&self, actor: &ActorId) -> Option<ChangeHash> {
        self.writer_contributions.get(actor).copied()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use automerge::Automerge;

    use super::ParentEpochView;
    use crate::automerge_adapter::materialized_view::MaterializedDocumentView;
    use crate::control::epoch_state::AcceptedEpochState;
    use crate::graph::actor_state::EpochActorState;
    use crate::graph::change_candidate::ChangeCandidate;
    use crate::{ActorId, ChangeHash, DevicePublicKey, EventId};

    fn candidate(actor: u8, hash: u8) -> ChangeCandidate {
        ChangeCandidate {
            change_hash: ChangeHash::from_bytes([hash; 32]),
            actor: ActorId::from_bytes([actor; 32]),
            sequence: 1,
            start_op: 1,
            operation_count: 1,
            dependencies: Vec::new(),
            control_id: EventId::from_bytes([7; 32]),
            author: DevicePublicKey::from_bytes([actor; 32]),
            valid_carriers: BTreeSet::from([EventId::from_bytes([hash; 32])]),
        }
    }

    #[test]
    fn define_parent_accepted_history_query_interface() {
        let first = candidate(1, 2);
        let second = candidate(3, 4);
        let first_actor = first.actor;
        let second_actor = second.actor;
        let first_hash = first.change_hash;
        let second_hash = second.change_hash;
        let candidates = BTreeMap::from([(first_hash, first), (second_hash, second)]);
        let actors = BTreeMap::from([
            (
                first_actor,
                EpochActorState {
                    last_sequence: 1,
                    next_op: 2,
                    highest_change: first_hash,
                },
            ),
            (
                second_actor,
                EpochActorState {
                    last_sequence: 1,
                    next_op: 2,
                    highest_change: second_hash,
                },
            ),
        ]);
        let document = Automerge::new();
        let materialized =
            MaterializedDocumentView::from_canonical_bytes(document.save_nocompress());
        assert!(materialized.is_ok());
        let Ok(materialized) = materialized else {
            return;
        };
        let state = AcceptedEpochState::new(
            BTreeSet::from([first_hash, second_hash]),
            BTreeSet::from([first_hash, second_hash]),
            candidates,
            actors.clone(),
            BTreeMap::from([(first_actor, first_hash), (second_actor, second_hash)]),
            materialized,
        );
        assert!(state.is_ok());
        let Ok(state) = state else {
            return;
        };
        let view = ParentEpochView::from_accepted_state(&state);
        assert!(view.contains(&first_hash));
        assert_eq!(view.heads(), &BTreeSet::from([first_hash, second_hash]));
        assert_eq!(view.dependencies(&first_hash), Some(&BTreeSet::new()));
        assert_eq!(
            view.actor_state(&first_actor),
            actors.get(&first_actor).copied()
        );
        assert_eq!(view.writer_contribution(&second_actor), Some(second_hash));
    }
}
