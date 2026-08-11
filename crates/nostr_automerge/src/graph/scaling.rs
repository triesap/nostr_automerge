use std::collections::BTreeSet;

use super::change_candidate::ChangeCandidate;
use super::dependency_graph::{GraphBuildError, build_graph};
use super::equivocation::quarantine_equivocation_descendants;
use super::schedule::schedule_candidates;
use crate::automerge_adapter::fixture::nested_map_bytes;
use crate::automerge_adapter::materialized_view::MaterializedDocumentView;
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

#[test]
fn expanded_control_actor_conflict_and_projection_models_are_bounded() {
    for size in [64_u16, 128] {
        let chain = (1..=size)
            .map(|value| {
                candidate(
                    value,
                    (value > 1).then(|| hash(value - 1)).into_iter().collect(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            measured(chain),
            (
                usize::from(size),
                0,
                u64::from(size) * 2,
                u64::from(size - 1) * 2,
            )
        );
    }

    let actor_work = |actors: u8, conflicts: bool| {
        let mut candidates = Vec::new();
        for actor in 1..=actors {
            let mut first = candidate(u16::from(actor) * 2, Vec::new());
            first.actor = ActorId::from_bytes([actor; 32]);
            candidates.push(first.clone());
            if conflicts {
                let mut conflict = first;
                conflict.change_hash = hash(u16::from(actor) * 2 + 1);
                candidates.push(conflict);
            }
        }
        let graph = build_graph(candidates.clone(), BTreeSet::new()).unwrap_or_default();
        let mut budget = WorkBudget::new(0, 1_000_000);
        let result =
            quarantine_equivocation_descendants(candidates, &graph, &mut budget, &NeverCancelled);
        assert!(result.is_ok());
        (
            budget.consumed().get(WorkCounter::GraphNode),
            budget.consumed().get(WorkCounter::GraphEdge),
        )
    };
    for conflicts in [false, true] {
        let small = actor_work(32, conflicts);
        let large = actor_work(64, conflicts);
        assert!(large.0 <= small.0 * 2 + 64, "{small:?} {large:?}");
        assert!(large.1 <= small.1 * 2 + 64, "{small:?} {large:?}");
    }

    let projection_work = |depth: usize| {
        let Some(bytes) = nested_map_bytes(depth) else {
            return 0;
        };
        let mut budget = WorkBudget::new(u64::MAX, u64::MAX);
        let view = MaterializedDocumentView::from_canonical_bytes_metered(
            bytes,
            &mut budget,
            &NeverCancelled,
        );
        assert!(view.is_ok());
        budget.consumed().get(WorkCounter::Assertion)
    };
    let shallow = projection_work(32);
    let deep = projection_work(64);
    assert!(shallow > 0 && deep > shallow);
    assert!(deep <= shallow * 5, "{shallow} {deep}");
}
