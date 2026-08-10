use std::collections::{BTreeMap, BTreeSet};

use super::change_candidate::ChangeCandidate;
use super::dependency_graph::DependencyGraph;
use crate::{CancellationCheck, ChangeHash, WorkBudget, WorkCounter};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ClosureError {
    Missing(ChangeHash),
    BudgetExhausted,
    Cancelled,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct CandidateDependencyClosure {
    pub(crate) known: BTreeSet<ChangeHash>,
    pub(crate) missing: BTreeSet<ChangeHash>,
    pub(crate) cyclic: BTreeSet<ChangeHash>,
    pub(crate) ordered: Vec<ChangeHash>,
}

pub(crate) fn candidate_dependency_closure(
    candidate: &ChangeCandidate,
    candidates: &BTreeMap<ChangeHash, ChangeCandidate>,
) -> CandidateDependencyClosure {
    let mut result = CandidateDependencyClosure::default();
    let mut pending = candidate.dependencies.clone();
    while let Some(hash) = pending.pop() {
        if !result.known.insert(hash) {
            continue;
        }
        if let Some(ancestor) = candidates.get(&hash) {
            pending.extend(ancestor.dependencies.iter().copied());
        } else {
            result.known.remove(&hash);
            result.missing.insert(hash);
        }
    }

    let mut indegrees = result
        .known
        .iter()
        .map(|hash| (*hash, 0usize))
        .collect::<BTreeMap<_, _>>();
    let mut dependants = BTreeMap::<ChangeHash, BTreeSet<ChangeHash>>::new();
    for hash in &result.known {
        if let Some(ancestor) = candidates.get(hash) {
            for dependency in ancestor
                .dependencies
                .iter()
                .filter(|dependency| result.known.contains(dependency))
            {
                if let Some(indegree) = indegrees.get_mut(hash) {
                    *indegree += 1;
                }
                dependants.entry(*dependency).or_default().insert(*hash);
            }
        }
    }
    let mut ready = indegrees
        .iter()
        .filter_map(|(hash, indegree)| (*indegree == 0).then_some(*hash))
        .collect::<BTreeSet<_>>();
    while let Some(hash) = ready.pop_first() {
        result.ordered.push(hash);
        if let Some(children) = dependants.get(&hash) {
            for child in children {
                if let Some(indegree) = indegrees.get_mut(child) {
                    *indegree -= 1;
                    if *indegree == 0 {
                        ready.insert(*child);
                    }
                }
            }
        }
    }
    let ordered = result.ordered.iter().copied().collect::<BTreeSet<_>>();
    result.cyclic = result.known.difference(&ordered).copied().collect();
    result
}

pub(crate) fn ancestor_closure(
    graph: &DependencyGraph,
    roots: impl IntoIterator<Item = ChangeHash>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<BTreeSet<ChangeHash>, ClosureError> {
    let mut closure = BTreeSet::new();
    let mut stack = roots.into_iter().collect::<Vec<_>>();
    stack.sort_unstable_by(|left, right| right.cmp(left));
    while let Some(hash) = stack.pop() {
        if cancellation.is_cancelled() {
            return Err(ClosureError::Cancelled);
        }
        if closure.contains(&hash) {
            continue;
        }
        budget
            .charge(WorkCounter::GraphNode, 1)
            .map_err(|_| ClosureError::BudgetExhausted)?;
        closure.insert(hash);
        if let Some(dependencies) = graph.nodes.get(&hash) {
            for dependency in dependencies.iter().rev() {
                if cancellation.is_cancelled() {
                    return Err(ClosureError::Cancelled);
                }
                budget
                    .charge(WorkCounter::GraphEdge, 1)
                    .map_err(|_| ClosureError::BudgetExhausted)?;
                stack.push(*dependency);
            }
        } else if !graph.accepted_base.contains(&hash) {
            return Err(ClosureError::Missing(hash));
        }
    }
    Ok(closure)
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{ClosureError, ancestor_closure, candidate_dependency_closure};
    use crate::graph::actor_state::tests::candidate;
    use crate::graph::dependency_graph::DependencyGraph;
    use crate::{ChangeHash, NeverCancelled, WorkBudget, WorkCounter};

    fn hash(value: u16) -> ChangeHash {
        let mut bytes = [0; 32];
        bytes[..2].copy_from_slice(&value.to_be_bytes());
        ChangeHash::from_bytes(bytes)
    }

    #[test]
    fn implement_iterative_ancestor_closure() {
        let mut nodes = BTreeMap::new();
        nodes.insert(hash(2), BTreeSet::from([hash(1)]));
        nodes.insert(hash(3), BTreeSet::from([hash(1)]));
        nodes.insert(hash(4), BTreeSet::from([hash(2), hash(3)]));
        for value in 5..1_000 {
            nodes.insert(hash(value), BTreeSet::from([hash(value - 1)]));
        }
        let graph = DependencyGraph {
            nodes,
            dependants: BTreeMap::new(),
            indegrees: BTreeMap::new(),
            accepted_base: BTreeSet::from([hash(1)]),
            edge_count: 999,
        };
        let mut budget = WorkBudget::new(0, 2_000);
        let closure = ancestor_closure(&graph, [hash(999)], &mut budget, &NeverCancelled);
        assert_eq!(closure.as_ref().map(BTreeSet::len), Ok(999));
        assert_eq!(budget.consumed().get(WorkCounter::GraphNode), 999);
        assert_eq!(budget.consumed().get(WorkCounter::GraphEdge), 999);
        let mut missing_budget = WorkBudget::new(0, 10);
        assert_eq!(
            ancestor_closure(&graph, [hash(1_001)], &mut missing_budget, &NeverCancelled),
            Err(ClosureError::Missing(hash(1_001)))
        );
        let mut exhausted = WorkBudget::new(0, 1);
        assert_eq!(
            ancestor_closure(&graph, [hash(2)], &mut exhausted, &NeverCancelled),
            Err(ClosureError::BudgetExhausted)
        );
        assert_eq!(exhausted.consumed().get(WorkCounter::GraphNode), 1);
        assert_eq!(exhausted.consumed().get(WorkCounter::GraphEdge), 0);
        let mut cancelled = WorkBudget::new(0, 10);
        assert_eq!(
            ancestor_closure(&graph, [hash(2)], &mut cancelled, &|| true),
            Err(ClosureError::Cancelled)
        );
    }

    #[test]
    fn candidate_dependency_closure_covers_all_graph_shapes() {
        let mut root = candidate(1, 1, 1, 1);
        root.change_hash = hash(1);
        let mut left = candidate(2, 1, 1, 1);
        left.change_hash = hash(2);
        left.dependencies = vec![root.change_hash];
        let mut right = candidate(3, 1, 1, 1);
        right.change_hash = hash(3);
        right.dependencies = vec![root.change_hash];
        let mut diamond = candidate(4, 1, 1, 1);
        diamond.change_hash = hash(4);
        diamond.dependencies = vec![left.change_hash, right.change_hash];
        let candidates = [root.clone(), left.clone(), right.clone(), diamond.clone()]
            .into_iter()
            .map(|candidate| (candidate.change_hash, candidate))
            .collect::<BTreeMap<_, _>>();
        let closure = candidate_dependency_closure(&diamond, &candidates);
        assert_eq!(closure.known, BTreeSet::from([hash(1), hash(2), hash(3)]));
        assert_eq!(closure.ordered, vec![hash(1), hash(2), hash(3)]);
        assert!(closure.missing.is_empty() && closure.cyclic.is_empty());

        let mut multiple_roots = diamond.clone();
        multiple_roots.dependencies = vec![left.change_hash, right.change_hash, hash(9)];
        let closure = candidate_dependency_closure(&multiple_roots, &candidates);
        assert_eq!(closure.missing, BTreeSet::from([hash(9)]));

        let mut cycle_left = left;
        let mut cycle_right = right;
        cycle_left.dependencies = vec![cycle_right.change_hash];
        cycle_right.dependencies = vec![cycle_left.change_hash];
        let cycle_candidates = [cycle_left.clone(), cycle_right.clone()]
            .into_iter()
            .map(|candidate| (candidate.change_hash, candidate))
            .collect::<BTreeMap<_, _>>();
        let mut cycle_root = root;
        cycle_root.dependencies = vec![cycle_left.change_hash];
        let closure = candidate_dependency_closure(&cycle_root, &cycle_candidates);
        assert_eq!(closure.cyclic, BTreeSet::from([hash(2), hash(3)]));
        assert!(closure.ordered.is_empty());
    }
}
