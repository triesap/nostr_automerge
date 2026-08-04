#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ActorCounters {
    pub(crate) sequence: u64,
    pub(crate) next_op: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CounterInput {
    pub(crate) sequence: u64,
    pub(crate) start_op: u64,
    pub(crate) operation_count: u64,
    pub(crate) predecessor_sequence_in_closure: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CounterError {
    Sequence,
    StartOp,
    MissingPredecessorSequence,
    Overflow,
}

pub(crate) fn transition(
    previous: Option<ActorCounters>,
    input: CounterInput,
) -> Result<ActorCounters, CounterError> {
    let expected_sequence = match previous {
        Some(previous) => previous
            .sequence
            .checked_add(1)
            .ok_or(CounterError::Overflow)?,
        None => 1,
    };
    if input.sequence != expected_sequence {
        return Err(CounterError::Sequence);
    }
    if previous.is_some() && !input.predecessor_sequence_in_closure {
        return Err(CounterError::MissingPredecessorSequence);
    }

    let expected_start = previous.map_or(1, |value| value.next_op);
    if input.start_op != expected_start {
        return Err(CounterError::StartOp);
    }
    let next_op = if input.operation_count == 0 {
        expected_start
    } else {
        input
            .start_op
            .checked_add(input.operation_count)
            .ok_or(CounterError::Overflow)?
    };
    Ok(ActorCounters {
        sequence: input.sequence,
        next_op,
    })
}

#[cfg(test)]
mod tests {
    use super::{ActorCounters, CounterError, CounterInput, transition};

    fn input(sequence: u64, start_op: u64, operation_count: u64) -> CounterInput {
        CounterInput {
            sequence,
            start_op,
            operation_count,
            predecessor_sequence_in_closure: sequence == 1,
        }
    }

    #[test]
    fn implement_checked_actor_counter_transitions() {
        assert_eq!(
            transition(None, input(1, 1, 3)),
            Ok(ActorCounters {
                sequence: 1,
                next_op: 4,
            })
        );

        let previous = ActorCounters {
            sequence: 1,
            next_op: 4,
        };
        let mut subsequent = input(2, 4, 2);
        subsequent.predecessor_sequence_in_closure = true;
        assert_eq!(
            transition(Some(previous), subsequent),
            Ok(ActorCounters {
                sequence: 2,
                next_op: 6,
            })
        );

        let mut empty = input(2, 4, 0);
        empty.predecessor_sequence_in_closure = true;
        assert_eq!(
            transition(Some(previous), empty),
            Ok(ActorCounters {
                sequence: 2,
                next_op: 4,
            })
        );

        for invalid in [input(0, 4, 1), input(1, 4, 1), input(3, 4, 1)] {
            assert_eq!(
                transition(Some(previous), invalid),
                Err(CounterError::Sequence)
            );
        }
        let mut missing_predecessor = input(2, 4, 1);
        missing_predecessor.predecessor_sequence_in_closure = false;
        assert_eq!(
            transition(Some(previous), missing_predecessor),
            Err(CounterError::MissingPredecessorSequence)
        );
        let mut wrong_start = input(2, 5, 1);
        wrong_start.predecessor_sequence_in_closure = true;
        assert_eq!(
            transition(Some(previous), wrong_start),
            Err(CounterError::StartOp)
        );
        assert_eq!(transition(None, input(1, 0, 1)), Err(CounterError::StartOp));

        let exhausted_sequence = ActorCounters {
            sequence: u64::MAX,
            next_op: 1,
        };
        assert_eq!(
            transition(Some(exhausted_sequence), input(1, 1, 1)),
            Err(CounterError::Overflow)
        );
        let near_op_limit = ActorCounters {
            sequence: 1,
            next_op: u64::MAX,
        };
        let mut overflowing = input(2, u64::MAX, 1);
        overflowing.predecessor_sequence_in_closure = true;
        assert_eq!(
            transition(Some(near_op_limit), overflowing),
            Err(CounterError::Overflow)
        );
    }
}
