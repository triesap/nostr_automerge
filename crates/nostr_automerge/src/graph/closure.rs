use std::collections::BTreeSet;

use super::dependency_graph::DependencyGraph;
use crate::{CancellationCheck, ChangeHash, WorkBudget, WorkCounter};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ClosureError {
    Missing(ChangeHash),
    BudgetExhausted,
    Cancelled,
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

    use super::{ClosureError, ancestor_closure};
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
            accepted_base: BTreeSet::from([hash(1)]),
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
}
