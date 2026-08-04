use std::collections::{BTreeMap, BTreeSet};

use crate::{ActorId, ChangeHash};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ActorState {
    pub(crate) sequence: u64,
    pub(crate) operation_counter: u64,
    pub(crate) highest_change: ChangeHash,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ParentEpochView {
    accepted: BTreeSet<ChangeHash>,
    heads: BTreeSet<ChangeHash>,
    dependencies: BTreeMap<ChangeHash, BTreeSet<ChangeHash>>,
    actors: BTreeMap<ActorId, ActorState>,
    writer_contributions: BTreeMap<ActorId, ChangeHash>,
}

impl ParentEpochView {
    pub(crate) fn new(
        accepted: BTreeSet<ChangeHash>,
        heads: BTreeSet<ChangeHash>,
        dependencies: BTreeMap<ChangeHash, BTreeSet<ChangeHash>>,
        actors: BTreeMap<ActorId, ActorState>,
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

    pub(crate) fn heads(&self) -> &BTreeSet<ChangeHash> {
        &self.heads
    }

    pub(crate) fn dependencies(&self, hash: &ChangeHash) -> Option<&BTreeSet<ChangeHash>> {
        self.dependencies.get(hash)
    }

    pub(crate) fn actor_state(&self, actor: &ActorId) -> Option<ActorState> {
        self.actors.get(actor).copied()
    }

    pub(crate) fn writer_contribution(&self, actor: &ActorId) -> Option<ChangeHash> {
        self.writer_contributions.get(actor).copied()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{ActorState, ParentEpochView};
    use crate::{ActorId, ChangeHash};

    #[test]
    fn define_parent_accepted_history_query_interface() {
        let actor = ActorId::from_bytes([1; 32]);
        let change = ChangeHash::from_bytes([2; 32]);
        let dependency = ChangeHash::from_bytes([3; 32]);
        let state = ActorState {
            sequence: 4,
            operation_counter: 9,
            highest_change: change,
        };
        let view = ParentEpochView::new(
            BTreeSet::from([change, dependency]),
            BTreeSet::from([change]),
            BTreeMap::from([(change, BTreeSet::from([dependency]))]),
            BTreeMap::from([(actor, state)]),
            BTreeMap::from([(actor, change)]),
        );
        assert!(view.contains(&change));
        assert_eq!(view.heads(), &BTreeSet::from([change]));
        assert_eq!(
            view.dependencies(&change),
            Some(&BTreeSet::from([dependency]))
        );
        assert_eq!(view.actor_state(&actor), Some(state));
        assert_eq!(view.writer_contribution(&actor), Some(change));
    }
}
