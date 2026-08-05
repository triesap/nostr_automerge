use std::collections::BTreeSet;

use super::change_candidate::ChangeCandidate;
use super::dependency_graph::{GraphBuildError, build_graph};
use super::schedule::schedule_candidates;
use crate::{
    ActorId, ChangeHash, DevicePublicKey, EventId, NeverCancelled, WorkBudget, WorkCounter,
};

fn hash(value: u16) -> ChangeHash {
    let mut bytes = [0; 32];
    bytes[..2].copy_from_slice(&value.to_be_bytes());
    ChangeHash::from_bytes(bytes)
}

fn candidate(value: u16, dependencies: Vec<ChangeHash>) -> ChangeCandidate {
    ChangeCandidate {
        change_hash: hash(value),
        actor: ActorId::from_bytes([1; 32]),
        sequence: u64::from(value),
        start_op: u64::from(value),
        operation_count: 1,
        dependencies,
        control_id: EventId::from_bytes([2; 32]),
        author: DevicePublicKey::from_bytes([3; 32]),
        valid_carriers: BTreeSet::from([EventId::from_bytes(*hash(value).as_bytes())]),
    }
}

fn measured(candidates: Vec<ChangeCandidate>) -> (usize, usize, u64, u64) {
    let mut budget = WorkBudget::new(0, 100_000);
    let schedule = schedule_candidates(candidates, BTreeSet::new(), &mut budget, &NeverCancelled);
    assert!(schedule.is_ok());
    let schedule = schedule.unwrap_or_default();
    (
        schedule.ordered.len(),
        schedule.pending.len() + schedule.cyclic.len(),
        budget.consumed().get(WorkCounter::GraphNode),
        budget.consumed().get(WorkCounter::GraphEdge),
    )
}

#[test]
fn graph_scaling_regression_models_are_proportional() {
    let chain = (1..=128)
        .map(|value| {
            candidate(
                value,
                (value > 1).then(|| hash(value - 1)).into_iter().collect(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(measured(chain), (128, 0, 256, 254));

    let fan_out = std::iter::once(candidate(1, vec![]))
        .chain((2..=128).map(|value| candidate(value, vec![hash(1)])))
        .collect::<Vec<_>>();
    assert_eq!(measured(fan_out), (128, 0, 256, 254));

    let fan_in = (1..=127)
        .map(|value| candidate(value, vec![]))
        .chain(std::iter::once(candidate(
            128,
            (1..=127).map(hash).collect(),
        )))
        .collect::<Vec<_>>();
    assert_eq!(measured(fan_in), (128, 0, 256, 254));

    let missing = measured(vec![candidate(1, vec![hash(999)])]);
    assert_eq!(missing, (0, 1, 3, 2));
    let cycle = measured(vec![
        candidate(1, vec![hash(2)]),
        candidate(2, vec![hash(1)]),
    ]);
    assert_eq!(cycle, (0, 2, 6, 4));

    assert_eq!(
        build_graph(
            [candidate(1, vec![]), candidate(1, vec![])],
            BTreeSet::new(),
        ),
        Err(GraphBuildError::DuplicateNode)
    );
}
