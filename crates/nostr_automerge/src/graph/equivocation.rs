use std::collections::{BTreeMap, BTreeSet};

use super::change_candidate::ChangeCandidate;
use crate::{ActorId, ChangeHash, EventId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EquivocationGroup {
    pub(crate) actor: ActorId,
    pub(crate) first_sequence: u64,
    pub(crate) conflicting_changes: BTreeSet<ChangeHash>,
    pub(crate) carrier_event_ids: BTreeSet<EventId>,
}

pub(crate) fn detect_equivocations(
    candidates: impl IntoIterator<Item = ChangeCandidate>,
) -> Vec<EquivocationGroup> {
    let mut groups = BTreeMap::<(ActorId, u64), BTreeMap<ChangeHash, BTreeSet<EventId>>>::new();
    for candidate in candidates {
        groups
            .entry((candidate.actor, candidate.sequence))
            .or_default()
            .entry(candidate.change_hash)
            .or_default()
            .extend(candidate.valid_carriers);
    }
    let mut first_by_actor = BTreeMap::<ActorId, EquivocationGroup>::new();
    for ((actor, sequence), changes) in groups {
        if changes.len() < 2 || first_by_actor.contains_key(&actor) {
            continue;
        }
        first_by_actor.insert(
            actor,
            EquivocationGroup {
                actor,
                first_sequence: sequence,
                conflicting_changes: changes.keys().copied().collect(),
                carrier_event_ids: changes.into_values().flatten().collect(),
            },
        );
    }
    first_by_actor.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::detect_equivocations;
    use crate::graph::actor_state::tests::candidate;
    use crate::{ChangeHash, EventId};

    #[test]
    fn detect_device_equivocation_groups() {
        let first = candidate(1, 1, 1, 1);
        assert!(detect_equivocations([first.clone()]).is_empty());
        assert!(detect_equivocations([first.clone(), first.clone()]).is_empty());

        let mut conflict = first.clone();
        conflict.change_hash = ChangeHash::from_bytes([2; 32]);
        conflict.valid_carriers = [EventId::from_bytes([8; 32])].into();
        let mut third = conflict.clone();
        third.change_hash = ChangeHash::from_bytes([3; 32]);
        let mut later = conflict.clone();
        later.sequence = 2;
        later.change_hash = ChangeHash::from_bytes([4; 32]);
        let groups = detect_equivocations([later, third, conflict, first]);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].first_sequence, 1);
        assert_eq!(groups[0].conflicting_changes.len(), 3);
        assert!(
            groups[0]
                .carrier_event_ids
                .contains(&EventId::from_bytes([8; 32]))
        );
    }
}
