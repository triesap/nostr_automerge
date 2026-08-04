use std::collections::BTreeMap;

use super::change_candidate::ChangeCandidate;
use crate::{ActorId, ChangeHash};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EpochActorState {
    pub(crate) last_sequence: u64,
    pub(crate) next_op: u64,
    pub(crate) highest_change: ChangeHash,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ActorStateError {
    SequenceGap,
    Equivocation,
    OperationCounter,
    MissingPredecessor,
    ParallelPredecessor,
    SequenceRollback,
}

pub(crate) fn validate_actor_predecessor(
    candidate: &ChangeCandidate,
    closure: &std::collections::BTreeSet<ChangeHash>,
    accepted: &BTreeMap<ChangeHash, ChangeCandidate>,
) -> Result<(), ActorStateError> {
    let same_actor = closure
        .iter()
        .filter_map(|hash| accepted.get(hash))
        .filter(|change| change.actor == candidate.actor)
        .collect::<Vec<_>>();
    if same_actor
        .iter()
        .any(|change| change.sequence >= candidate.sequence)
    {
        return Err(ActorStateError::SequenceRollback);
    }
    if candidate.sequence == 1 {
        return if same_actor.is_empty() {
            Ok(())
        } else {
            Err(ActorStateError::SequenceRollback)
        };
    }
    let expected = candidate.sequence - 1;
    let predecessors = same_actor
        .iter()
        .filter(|change| change.sequence == expected)
        .count();
    match predecessors {
        1 => Ok(()),
        0 => Err(ActorStateError::MissingPredecessor),
        _ => Err(ActorStateError::ParallelPredecessor),
    }
}

pub(crate) fn initialize_actor_states(
    accepted_base: impl IntoIterator<Item = ChangeCandidate>,
) -> Result<BTreeMap<ActorId, EpochActorState>, ActorStateError> {
    let mut changes = accepted_base.into_iter().collect::<Vec<_>>();
    changes.sort_by_key(|candidate| (candidate.actor, candidate.sequence, candidate.change_hash));
    let mut states = BTreeMap::<ActorId, EpochActorState>::new();
    for candidate in changes {
        let expected_sequence = match states.get(&candidate.actor) {
            Some(state) => state
                .last_sequence
                .checked_add(1)
                .ok_or(ActorStateError::SequenceGap)?,
            None => 1,
        };
        if candidate.sequence < expected_sequence {
            return Err(ActorStateError::Equivocation);
        }
        if candidate.sequence != expected_sequence {
            return Err(ActorStateError::SequenceGap);
        }
        let next_op = states
            .get(&candidate.actor)
            .map_or(1, |state| state.next_op);
        let advanced = if candidate.operation_count == 0 {
            if candidate.start_op != next_op {
                return Err(ActorStateError::OperationCounter);
            }
            next_op
        } else {
            if candidate.start_op != next_op {
                return Err(ActorStateError::OperationCounter);
            }
            next_op
                .checked_add(candidate.operation_count)
                .ok_or(ActorStateError::OperationCounter)?
        };
        states.insert(
            candidate.actor,
            EpochActorState {
                last_sequence: candidate.sequence,
                next_op: advanced,
                highest_change: candidate.change_hash,
            },
        );
    }
    Ok(states)
}

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::BTreeSet;

    use super::{ActorStateError, initialize_actor_states, validate_actor_predecessor};
    use crate::graph::change_candidate::ChangeCandidate;
    use crate::{ActorId, ChangeHash, DevicePublicKey, EventId};
    use std::collections::BTreeMap;

    pub(crate) fn candidate(actor: u8, sequence: u64, start: u64, count: u64) -> ChangeCandidate {
        ChangeCandidate {
            change_hash: ChangeHash::from_bytes([u8::try_from(sequence).unwrap_or_default(); 32]),
            actor: ActorId::from_bytes([actor; 32]),
            sequence,
            start_op: start,
            operation_count: count,
            dependencies: Vec::new(),
            control_id: EventId::from_bytes([9; 32]),
            author: DevicePublicKey::from_bytes([actor; 32]),
            valid_carriers: BTreeSet::from([EventId::from_bytes([actor; 32])]),
        }
    }

    #[test]
    fn initialize_actor_state_from_epoch_base() {
        let states = initialize_actor_states([
            candidate(2, 1, 1, 0),
            candidate(1, 2, 3, 0),
            candidate(1, 1, 1, 2),
        ]);
        assert!(states.is_ok());
        let states = match states {
            Ok(states) => states,
            Err(_) => return,
        };
        assert_eq!(states[&ActorId::from_bytes([1; 32])].last_sequence, 2);
        assert_eq!(states[&ActorId::from_bytes([1; 32])].next_op, 3);
        assert_eq!(
            initialize_actor_states([candidate(1, 2, 1, 1)]),
            Err(ActorStateError::SequenceGap)
        );
        let mut conflict = candidate(1, 1, 1, 1);
        conflict.change_hash = ChangeHash::from_bytes([8; 32]);
        assert_eq!(
            initialize_actor_states([candidate(1, 1, 1, 1), conflict]),
            Err(ActorStateError::Equivocation)
        );
    }

    #[test]
    fn validate_actor_predecessor_sequence() {
        let first = candidate(1, 1, 1, 1);
        let mut second = candidate(1, 2, 2, 1);
        second.change_hash = ChangeHash::from_bytes([2; 32]);
        let accepted = BTreeMap::from([(first.change_hash, first.clone())]);
        assert_eq!(
            validate_actor_predecessor(&second, &BTreeSet::from([first.change_hash]), &accepted),
            Ok(())
        );
        assert_eq!(
            validate_actor_predecessor(&second, &BTreeSet::new(), &accepted),
            Err(ActorStateError::MissingPredecessor)
        );
        let mut conflict = first.clone();
        conflict.change_hash = ChangeHash::from_bytes([8; 32]);
        let conflicts = BTreeMap::from([
            (first.change_hash, first.clone()),
            (conflict.change_hash, conflict),
        ]);
        assert_eq!(
            validate_actor_predecessor(
                &second,
                &BTreeSet::from([first.change_hash, ChangeHash::from_bytes([8; 32])]),
                &conflicts,
            ),
            Err(ActorStateError::ParallelPredecessor)
        );
        assert_eq!(
            validate_actor_predecessor(&first, &BTreeSet::from([first.change_hash]), &accepted),
            Err(ActorStateError::SequenceRollback)
        );
    }
}
