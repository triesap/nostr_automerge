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
    EmptyChange,
    NonEmptyChange,
    DependencyFrontier,
}

pub(crate) fn apply_empty_counter(
    states: &mut BTreeMap<ActorId, EpochActorState>,
    candidate: &ChangeCandidate,
    current_heads: &std::collections::BTreeSet<ChangeHash>,
) -> Result<(), ActorStateError> {
    if candidate.operation_count != 0 {
        return Err(ActorStateError::NonEmptyChange);
    }
    if candidate
        .dependencies
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        != *current_heads
    {
        return Err(ActorStateError::DependencyFrontier);
    }
    let (last_sequence, next_op) = states
        .get(&candidate.actor)
        .map_or((0, 1), |state| (state.last_sequence, state.next_op));
    if candidate.sequence
        != last_sequence
            .checked_add(1)
            .ok_or(ActorStateError::SequenceGap)?
    {
        return Err(ActorStateError::SequenceGap);
    }
    if candidate.start_op != next_op {
        return Err(ActorStateError::OperationCounter);
    }
    states.insert(
        candidate.actor,
        EpochActorState {
            last_sequence: candidate.sequence,
            next_op,
            highest_change: candidate.change_hash,
        },
    );
    Ok(())
}

pub(crate) fn apply_nonempty_counter(
    states: &mut BTreeMap<ActorId, EpochActorState>,
    candidate: &ChangeCandidate,
) -> Result<(), ActorStateError> {
    if candidate.operation_count == 0 {
        return Err(ActorStateError::EmptyChange);
    }
    let (last_sequence, next_op) = states
        .get(&candidate.actor)
        .map_or((0, 1), |state| (state.last_sequence, state.next_op));
    if candidate.sequence
        != last_sequence
            .checked_add(1)
            .ok_or(ActorStateError::SequenceGap)?
    {
        return Err(ActorStateError::SequenceGap);
    }
    if candidate.start_op != next_op {
        return Err(ActorStateError::OperationCounter);
    }
    let next_op = next_op
        .checked_add(candidate.operation_count)
        .ok_or(ActorStateError::OperationCounter)?;
    states.insert(
        candidate.actor,
        EpochActorState {
            last_sequence: candidate.sequence,
            next_op,
            highest_change: candidate.change_hash,
        },
    );
    Ok(())
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

    use super::{
        ActorStateError, EpochActorState, apply_empty_counter, apply_nonempty_counter,
        initialize_actor_states, validate_actor_predecessor,
    };
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

    #[test]
    fn validate_next_op_for_nonempty_changes() {
        let actor = ActorId::from_bytes([1; 32]);
        let mut states = BTreeMap::new();
        let first = candidate(1, 1, 1, 2);
        assert_eq!(apply_nonempty_counter(&mut states, &first), Ok(()));
        assert_eq!(states[&actor].next_op, 3);
        let second = candidate(1, 2, 3, 1);
        assert_eq!(apply_nonempty_counter(&mut states, &second), Ok(()));

        let mut gap = candidate(1, 3, 5, 1);
        assert_eq!(
            apply_nonempty_counter(&mut states.clone(), &gap),
            Err(ActorStateError::OperationCounter)
        );
        gap.start_op = 2;
        assert_eq!(
            apply_nonempty_counter(&mut states.clone(), &gap),
            Err(ActorStateError::OperationCounter)
        );
        let mut overflow_states = BTreeMap::from([(
            actor,
            EpochActorState {
                last_sequence: 1,
                next_op: u64::MAX,
                highest_change: first.change_hash,
            },
        )]);
        let overflow = candidate(1, 2, u64::MAX, 1);
        assert_eq!(
            apply_nonempty_counter(&mut overflow_states, &overflow),
            Err(ActorStateError::OperationCounter)
        );
        assert_eq!(
            apply_nonempty_counter(&mut states, &candidate(2, 1, 1, 1)),
            Ok(())
        );
    }

    #[test]
    fn validate_empty_merge_change_counters() {
        let mut states = BTreeMap::new();
        let first = candidate(1, 1, 1, 2);
        assert_eq!(apply_nonempty_counter(&mut states, &first), Ok(()));
        let first_head = ChangeHash::from_bytes([7; 32]);
        let mut empty = candidate(1, 2, 3, 0);
        empty.change_hash = ChangeHash::from_bytes([2; 32]);
        empty.dependencies = vec![first_head];
        assert_eq!(
            apply_empty_counter(&mut states, &empty, &BTreeSet::from([first_head])),
            Ok(())
        );
        assert_eq!(states[&ActorId::from_bytes([1; 32])].next_op, 3);

        let mut second_empty = candidate(1, 3, 3, 0);
        second_empty.change_hash = ChangeHash::from_bytes([3; 32]);
        second_empty.dependencies = vec![empty.change_hash];
        assert_eq!(
            apply_empty_counter(
                &mut states,
                &second_empty,
                &BTreeSet::from([empty.change_hash])
            ),
            Ok(())
        );
        let mut wrong_start = candidate(1, 4, 4, 0);
        wrong_start.dependencies = vec![second_empty.change_hash];
        assert_eq!(
            apply_empty_counter(
                &mut states.clone(),
                &wrong_start,
                &BTreeSet::from([second_empty.change_hash])
            ),
            Err(ActorStateError::OperationCounter)
        );
        assert_eq!(
            apply_empty_counter(&mut states, &wrong_start, &BTreeSet::new()),
            Err(ActorStateError::DependencyFrontier)
        );
    }
}
