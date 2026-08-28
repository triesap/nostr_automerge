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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CandidateClosureError {
    BudgetExhausted,
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CandidateClosureOperation {
    ResultConstruction,
    PendingStackConstruction,
    DependencyPull,
    PendingPush,
    PendingPull,
    KnownLookup,
    KnownInsert,
    CandidateLookup,
    KnownRemove,
    MissingInsert,
    IndegreeMapConstruction,
    KnownPull,
    IndegreeInsert,
    DependencyKnownComparison,
    IndegreeLookup,
    IndegreeIncrement,
    DependantMapConstruction,
    DependantLookup,
    DependantBucketInsert,
    DependantInsert,
    ReadySetConstruction,
    IndegreePull,
    ReadinessComparison,
    ReadyInsert,
    ReadyPull,
    OrderedPush,
    DependantChildrenLookup,
    DependantPull,
    IndegreeDecrement,
    OrderedSetConstruction,
    OrderedPull,
    OrderedInsert,
    OrderedMembershipComparison,
    CyclicInsert,
    ResultPublication,
}

pub(crate) fn candidate_dependency_closure(
    candidate: &ChangeCandidate,
    candidates: &BTreeMap<ChangeHash, ChangeCandidate>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<CandidateDependencyClosure, CandidateClosureError> {
    candidate_dependency_closure_observed(
        candidate,
        candidates,
        |counter| charge_closure_work(budget, cancellation, counter),
        |_| {},
    )
}

fn candidate_dependency_closure_observed<E>(
    candidate: &ChangeCandidate,
    candidates: &BTreeMap<ChangeHash, ChangeCandidate>,
    mut charge: impl FnMut(WorkCounter) -> Result<(), E>,
    mut observed: impl FnMut(CandidateClosureOperation),
) -> Result<CandidateDependencyClosure, E> {
    let mut result = closure_operation(
        WorkCounter::GraphNode,
        CandidateClosureOperation::ResultConstruction,
        &mut charge,
        &mut observed,
        CandidateDependencyClosure::default,
    )?;
    let mut pending = closure_operation(
        WorkCounter::GraphEdge,
        CandidateClosureOperation::PendingStackConstruction,
        &mut charge,
        &mut observed,
        Vec::new,
    )?;
    let mut roots = candidate.dependencies.iter();
    for _ in 0..candidate.dependencies.len() {
        let dependency = closure_operation(
            WorkCounter::GraphEdge,
            CandidateClosureOperation::DependencyPull,
            &mut charge,
            &mut observed,
            || roots.next().copied(),
        )?;
        let Some(dependency) = dependency else { break };
        closure_operation(
            WorkCounter::GraphEdge,
            CandidateClosureOperation::PendingPush,
            &mut charge,
            &mut observed,
            || pending.push(dependency),
        )?;
    }
    loop {
        let hash = closure_operation(
            WorkCounter::GraphNode,
            CandidateClosureOperation::PendingPull,
            &mut charge,
            &mut observed,
            || pending.pop(),
        )?;
        let Some(hash) = hash else { break };
        let known = closure_operation(
            WorkCounter::GraphNode,
            CandidateClosureOperation::KnownLookup,
            &mut charge,
            &mut observed,
            || result.known.contains(&hash),
        )?;
        if known {
            continue;
        }
        closure_operation(
            WorkCounter::GraphNode,
            CandidateClosureOperation::KnownInsert,
            &mut charge,
            &mut observed,
            || result.known.insert(hash),
        )?;
        let ancestor = closure_operation(
            WorkCounter::GraphNode,
            CandidateClosureOperation::CandidateLookup,
            &mut charge,
            &mut observed,
            || candidates.get(&hash),
        )?;
        if let Some(ancestor) = ancestor {
            let mut dependencies = ancestor.dependencies.iter();
            for _ in 0..ancestor.dependencies.len() {
                let dependency = closure_operation(
                    WorkCounter::GraphEdge,
                    CandidateClosureOperation::DependencyPull,
                    &mut charge,
                    &mut observed,
                    || dependencies.next().copied(),
                )?;
                let Some(dependency) = dependency else { break };
                closure_operation(
                    WorkCounter::GraphEdge,
                    CandidateClosureOperation::PendingPush,
                    &mut charge,
                    &mut observed,
                    || pending.push(dependency),
                )?;
            }
        } else {
            closure_operation(
                WorkCounter::GraphNode,
                CandidateClosureOperation::KnownRemove,
                &mut charge,
                &mut observed,
                || result.known.remove(&hash),
            )?;
            closure_operation(
                WorkCounter::GraphNode,
                CandidateClosureOperation::MissingInsert,
                &mut charge,
                &mut observed,
                || result.missing.insert(hash),
            )?;
        }
    }

    let mut indegrees = closure_operation(
        WorkCounter::GraphNode,
        CandidateClosureOperation::IndegreeMapConstruction,
        &mut charge,
        &mut observed,
        BTreeMap::new,
    )?;
    let mut known = result.known.iter();
    for _ in 0..result.known.len() {
        let hash = closure_operation(
            WorkCounter::GraphNode,
            CandidateClosureOperation::KnownPull,
            &mut charge,
            &mut observed,
            || known.next().copied(),
        )?;
        let Some(hash) = hash else { break };
        closure_operation(
            WorkCounter::GraphNode,
            CandidateClosureOperation::IndegreeInsert,
            &mut charge,
            &mut observed,
            || indegrees.insert(hash, 0_usize),
        )?;
    }
    let mut dependants = closure_operation(
        WorkCounter::GraphEdge,
        CandidateClosureOperation::DependantMapConstruction,
        &mut charge,
        &mut observed,
        BTreeMap::<ChangeHash, BTreeSet<ChangeHash>>::new,
    )?;
    let mut known = result.known.iter();
    for _ in 0..result.known.len() {
        let hash = closure_operation(
            WorkCounter::GraphNode,
            CandidateClosureOperation::KnownPull,
            &mut charge,
            &mut observed,
            || known.next().copied(),
        )?;
        let Some(hash) = hash else { break };
        let ancestor = closure_operation(
            WorkCounter::GraphNode,
            CandidateClosureOperation::CandidateLookup,
            &mut charge,
            &mut observed,
            || candidates.get(&hash),
        )?;
        let Some(ancestor) = ancestor else { continue };
        let mut dependencies = ancestor.dependencies.iter();
        for _ in 0..ancestor.dependencies.len() {
            let dependency = closure_operation(
                WorkCounter::GraphEdge,
                CandidateClosureOperation::DependencyPull,
                &mut charge,
                &mut observed,
                || dependencies.next().copied(),
            )?;
            let Some(dependency) = dependency else { break };
            let is_known = closure_operation(
                WorkCounter::GraphEdge,
                CandidateClosureOperation::DependencyKnownComparison,
                &mut charge,
                &mut observed,
                || result.known.contains(&dependency),
            )?;
            if !is_known {
                continue;
            }
            let indegree = closure_operation(
                WorkCounter::GraphNode,
                CandidateClosureOperation::IndegreeLookup,
                &mut charge,
                &mut observed,
                || indegrees.get_mut(&hash),
            )?;
            if let Some(indegree) = indegree {
                closure_operation(
                    WorkCounter::GraphNode,
                    CandidateClosureOperation::IndegreeIncrement,
                    &mut charge,
                    &mut observed,
                    || *indegree = indegree.saturating_add(1),
                )?;
            }
            let has_bucket = closure_operation(
                WorkCounter::GraphEdge,
                CandidateClosureOperation::DependantLookup,
                &mut charge,
                &mut observed,
                || dependants.contains_key(&dependency),
            )?;
            if !has_bucket {
                closure_operation(
                    WorkCounter::GraphEdge,
                    CandidateClosureOperation::DependantBucketInsert,
                    &mut charge,
                    &mut observed,
                    || dependants.insert(dependency, BTreeSet::new()),
                )?;
            }
            let children = closure_operation(
                WorkCounter::GraphEdge,
                CandidateClosureOperation::DependantLookup,
                &mut charge,
                &mut observed,
                || dependants.get_mut(&dependency),
            )?;
            if let Some(children) = children {
                closure_operation(
                    WorkCounter::GraphEdge,
                    CandidateClosureOperation::DependantInsert,
                    &mut charge,
                    &mut observed,
                    || children.insert(hash),
                )?;
            }
        }
    }
    let mut ready = closure_operation(
        WorkCounter::GraphNode,
        CandidateClosureOperation::ReadySetConstruction,
        &mut charge,
        &mut observed,
        BTreeSet::new,
    )?;
    let mut indegree_items = indegrees.iter();
    for _ in 0..indegrees.len() {
        let item = closure_operation(
            WorkCounter::GraphNode,
            CandidateClosureOperation::IndegreePull,
            &mut charge,
            &mut observed,
            || {
                indegree_items
                    .next()
                    .map(|(hash, indegree)| (*hash, *indegree))
            },
        )?;
        let Some((hash, indegree)) = item else { break };
        let is_ready = closure_operation(
            WorkCounter::GraphNode,
            CandidateClosureOperation::ReadinessComparison,
            &mut charge,
            &mut observed,
            || indegree == 0,
        )?;
        if is_ready {
            closure_operation(
                WorkCounter::GraphNode,
                CandidateClosureOperation::ReadyInsert,
                &mut charge,
                &mut observed,
                || ready.insert(hash),
            )?;
        }
    }
    loop {
        let hash = closure_operation(
            WorkCounter::GraphNode,
            CandidateClosureOperation::ReadyPull,
            &mut charge,
            &mut observed,
            || ready.pop_first(),
        )?;
        let Some(hash) = hash else { break };
        closure_operation(
            WorkCounter::GraphNode,
            CandidateClosureOperation::OrderedPush,
            &mut charge,
            &mut observed,
            || result.ordered.push(hash),
        )?;
        let children = closure_operation(
            WorkCounter::GraphEdge,
            CandidateClosureOperation::DependantChildrenLookup,
            &mut charge,
            &mut observed,
            || dependants.get(&hash),
        )?;
        let Some(children) = children else { continue };
        let mut children = children.iter();
        let child_count = children.len();
        for _ in 0..child_count {
            let child = closure_operation(
                WorkCounter::GraphEdge,
                CandidateClosureOperation::DependantPull,
                &mut charge,
                &mut observed,
                || children.next().copied(),
            )?;
            let Some(child) = child else { break };
            let indegree = closure_operation(
                WorkCounter::GraphNode,
                CandidateClosureOperation::IndegreeLookup,
                &mut charge,
                &mut observed,
                || indegrees.get_mut(&child),
            )?;
            let Some(indegree) = indegree else { continue };
            closure_operation(
                WorkCounter::GraphNode,
                CandidateClosureOperation::IndegreeDecrement,
                &mut charge,
                &mut observed,
                || *indegree = indegree.saturating_sub(1),
            )?;
            let is_ready = closure_operation(
                WorkCounter::GraphNode,
                CandidateClosureOperation::ReadinessComparison,
                &mut charge,
                &mut observed,
                || *indegree == 0,
            )?;
            if is_ready {
                closure_operation(
                    WorkCounter::GraphNode,
                    CandidateClosureOperation::ReadyInsert,
                    &mut charge,
                    &mut observed,
                    || ready.insert(child),
                )?;
            }
        }
    }
    let mut ordered = closure_operation(
        WorkCounter::GraphNode,
        CandidateClosureOperation::OrderedSetConstruction,
        &mut charge,
        &mut observed,
        BTreeSet::new,
    )?;
    let mut ordered_items = result.ordered.iter();
    for _ in 0..result.ordered.len() {
        let hash = closure_operation(
            WorkCounter::GraphNode,
            CandidateClosureOperation::OrderedPull,
            &mut charge,
            &mut observed,
            || ordered_items.next().copied(),
        )?;
        let Some(hash) = hash else { break };
        closure_operation(
            WorkCounter::GraphNode,
            CandidateClosureOperation::OrderedInsert,
            &mut charge,
            &mut observed,
            || ordered.insert(hash),
        )?;
    }
    let mut known = result.known.iter();
    for _ in 0..result.known.len() {
        let hash = closure_operation(
            WorkCounter::GraphNode,
            CandidateClosureOperation::KnownPull,
            &mut charge,
            &mut observed,
            || known.next().copied(),
        )?;
        let Some(hash) = hash else { break };
        let is_ordered = closure_operation(
            WorkCounter::GraphNode,
            CandidateClosureOperation::OrderedMembershipComparison,
            &mut charge,
            &mut observed,
            || ordered.contains(&hash),
        )?;
        if !is_ordered {
            closure_operation(
                WorkCounter::GraphNode,
                CandidateClosureOperation::CyclicInsert,
                &mut charge,
                &mut observed,
                || result.cyclic.insert(hash),
            )?;
        }
    }
    closure_operation(
        WorkCounter::GraphNode,
        CandidateClosureOperation::ResultPublication,
        &mut charge,
        &mut observed,
        || result,
    )
}

fn closure_operation<T, E>(
    counter: WorkCounter,
    operation: CandidateClosureOperation,
    charge: &mut impl FnMut(WorkCounter) -> Result<(), E>,
    observed: &mut impl FnMut(CandidateClosureOperation),
    target: impl FnOnce() -> T,
) -> Result<T, E> {
    charge(counter)?;
    let value = target();
    observed(operation);
    Ok(value)
}

fn charge_closure_work(
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
    counter: WorkCounter,
) -> Result<(), CandidateClosureError> {
    if cancellation.is_cancelled() {
        return Err(CandidateClosureError::Cancelled);
    }
    budget
        .charge(counter, 1)
        .map_err(|_| CandidateClosureError::BudgetExhausted)
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

    use super::{
        CandidateClosureError, CandidateClosureOperation, ClosureError, ancestor_closure,
        candidate_dependency_closure, candidate_dependency_closure_observed,
    };
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
        left.dependencies = vec![root.change_hash].into();
        let mut right = candidate(3, 1, 1, 1);
        right.change_hash = hash(3);
        right.dependencies = vec![root.change_hash].into();
        let mut diamond = candidate(4, 1, 1, 1);
        diamond.change_hash = hash(4);
        diamond.dependencies = vec![left.change_hash, right.change_hash].into();
        let candidates = [root.clone(), left.clone(), right.clone(), diamond.clone()]
            .into_iter()
            .map(|candidate| (candidate.change_hash, candidate))
            .collect::<BTreeMap<_, _>>();
        let mut budget = WorkBudget::new(0, 10_000);
        let closure =
            candidate_dependency_closure(&diamond, &candidates, &mut budget, &NeverCancelled);
        let Ok(closure) = closure else {
            return;
        };
        assert_eq!(closure.known, BTreeSet::from([hash(1), hash(2), hash(3)]));
        assert_eq!(closure.ordered, vec![hash(1), hash(2), hash(3)]);
        assert!(closure.missing.is_empty() && closure.cyclic.is_empty());

        let mut multiple_roots = diamond.clone();
        multiple_roots.dependencies = vec![left.change_hash, right.change_hash, hash(9)].into();
        let closure = candidate_dependency_closure(
            &multiple_roots,
            &candidates,
            &mut WorkBudget::new(0, 10_000),
            &NeverCancelled,
        );
        let Ok(closure) = closure else {
            return;
        };
        assert_eq!(closure.missing, BTreeSet::from([hash(9)]));

        let mut cycle_left = left;
        let mut cycle_right = right;
        cycle_left.dependencies = vec![cycle_right.change_hash].into();
        cycle_right.dependencies = vec![cycle_left.change_hash].into();
        let cycle_candidates = [cycle_left.clone(), cycle_right.clone()]
            .into_iter()
            .map(|candidate| (candidate.change_hash, candidate))
            .collect::<BTreeMap<_, _>>();
        let mut cycle_root = root;
        cycle_root.dependencies = vec![cycle_left.change_hash].into();
        let closure = candidate_dependency_closure(
            &cycle_root,
            &cycle_candidates,
            &mut WorkBudget::new(0, 10_000),
            &NeverCancelled,
        );
        let Ok(closure) = closure else {
            return;
        };
        assert_eq!(closure.cyclic, BTreeSet::from([hash(2), hash(3)]));
        assert!(closure.ordered.is_empty());

        assert_eq!(
            candidate_dependency_closure(
                &diamond,
                &candidates,
                &mut WorkBudget::new(0, 0),
                &NeverCancelled,
            ),
            Err(CandidateClosureError::BudgetExhausted)
        );
        assert_eq!(
            candidate_dependency_closure(
                &diamond,
                &candidates,
                &mut WorkBudget::new(0, 100),
                &|| true,
            ),
            Err(CandidateClosureError::Cancelled)
        );
    }

    #[test]
    fn candidate_dependency_closure_charges_immediately_before_every_target_operation() {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum Stop {
            BudgetExhausted,
            Cancelled,
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum Trace {
            Charge(WorkCounter),
            Operation(CandidateClosureOperation),
        }

        let mut first = candidate(1, 1, 1, 1);
        first.change_hash = hash(1);
        let mut second = candidate(2, 1, 1, 1);
        second.change_hash = hash(2);
        second.dependencies = vec![first.change_hash].into();
        let mut third = candidate(3, 1, 1, 1);
        third.change_hash = hash(3);
        third.dependencies = vec![first.change_hash].into();
        let mut cycle_left = candidate(5, 1, 1, 1);
        cycle_left.change_hash = hash(5);
        cycle_left.dependencies = vec![hash(6)].into();
        let mut cycle_right = candidate(6, 1, 1, 1);
        cycle_right.change_hash = hash(6);
        cycle_right.dependencies = vec![hash(5)].into();
        let candidates = [
            first.clone(),
            second.clone(),
            third.clone(),
            cycle_left,
            cycle_right,
        ]
        .into_iter()
        .map(|candidate| (candidate.change_hash, candidate))
        .collect::<BTreeMap<_, _>>();
        let mut query = candidate(4, 1, 1, 1);
        query.change_hash = hash(4);
        query.dependencies = vec![second.change_hash, third.change_hash, hash(5), hash(9)].into();

        let trace = std::cell::RefCell::new(Vec::new());
        let ample = candidate_dependency_closure_observed(
            &query,
            &candidates,
            |counter| {
                trace.borrow_mut().push(Trace::Charge(counter));
                Ok::<_, Stop>(())
            },
            |operation| trace.borrow_mut().push(Trace::Operation(operation)),
        );
        assert!(ample.is_ok());
        let Ok(ample) = ample else { return };
        let trace = trace.into_inner();
        assert!(!trace.is_empty() && trace.len().is_multiple_of(2));
        for pair in trace.chunks_exact(2) {
            assert!(matches!(pair, [Trace::Charge(_), Trace::Operation(_)]));
        }
        let operations = trace
            .iter()
            .filter_map(|item| match item {
                Trace::Operation(operation) => Some(*operation),
                Trace::Charge(_) => None,
            })
            .collect::<Vec<_>>();
        for required in [
            CandidateClosureOperation::ResultConstruction,
            CandidateClosureOperation::PendingStackConstruction,
            CandidateClosureOperation::DependencyPull,
            CandidateClosureOperation::PendingPush,
            CandidateClosureOperation::PendingPull,
            CandidateClosureOperation::KnownLookup,
            CandidateClosureOperation::KnownInsert,
            CandidateClosureOperation::CandidateLookup,
            CandidateClosureOperation::KnownRemove,
            CandidateClosureOperation::MissingInsert,
            CandidateClosureOperation::IndegreeMapConstruction,
            CandidateClosureOperation::KnownPull,
            CandidateClosureOperation::IndegreeInsert,
            CandidateClosureOperation::DependencyKnownComparison,
            CandidateClosureOperation::IndegreeLookup,
            CandidateClosureOperation::IndegreeIncrement,
            CandidateClosureOperation::DependantMapConstruction,
            CandidateClosureOperation::DependantLookup,
            CandidateClosureOperation::DependantBucketInsert,
            CandidateClosureOperation::DependantInsert,
            CandidateClosureOperation::ReadySetConstruction,
            CandidateClosureOperation::IndegreePull,
            CandidateClosureOperation::ReadinessComparison,
            CandidateClosureOperation::ReadyInsert,
            CandidateClosureOperation::ReadyPull,
            CandidateClosureOperation::OrderedPush,
            CandidateClosureOperation::DependantChildrenLookup,
            CandidateClosureOperation::DependantPull,
            CandidateClosureOperation::IndegreeDecrement,
            CandidateClosureOperation::OrderedSetConstruction,
            CandidateClosureOperation::OrderedPull,
            CandidateClosureOperation::OrderedInsert,
            CandidateClosureOperation::OrderedMembershipComparison,
            CandidateClosureOperation::CyclicInsert,
            CandidateClosureOperation::ResultPublication,
        ] {
            assert!(operations.contains(&required), "missing {required:?}");
        }

        for allowance in 0..operations.len() {
            let mut successful = 0_usize;
            let mut observed = Vec::new();
            let result = candidate_dependency_closure_observed(
                &query,
                &candidates,
                |_| {
                    if successful == allowance {
                        return Err(Stop::BudgetExhausted);
                    }
                    successful += 1;
                    Ok(())
                },
                |operation| observed.push(operation),
            );
            assert_eq!(result, Err(Stop::BudgetExhausted), "budget {allowance}");
            assert_eq!(successful, allowance, "budget {allowance}");
            assert_eq!(observed, operations[..allowance], "budget {allowance}");

            let mut successful = 0_usize;
            let mut observed = Vec::new();
            let result = candidate_dependency_closure_observed(
                &query,
                &candidates,
                |_| {
                    if successful == allowance {
                        return Err(Stop::Cancelled);
                    }
                    successful += 1;
                    Ok(())
                },
                |operation| observed.push(operation),
            );
            assert_eq!(result, Err(Stop::Cancelled), "cancel {allowance}");
            assert_eq!(successful, allowance, "cancel {allowance}");
            assert_eq!(observed, operations[..allowance], "cancel {allowance}");
        }

        let mut completed = 0_usize;
        let exact = candidate_dependency_closure_observed(
            &query,
            &candidates,
            |_| {
                completed += 1;
                Ok::<_, Stop>(())
            },
            |_| {},
        );
        assert_eq!(exact, Ok(ample));
        assert_eq!(completed, operations.len());
    }

    #[test]
    fn candidate_dependency_closure_scales_across_deep_wide_cycle_and_missing_graphs() {
        let mut candidates = BTreeMap::new();
        for value in 1_u16..=64 {
            let Ok(actor) = u8::try_from(value) else {
                return;
            };
            let mut item = candidate(actor, 1, 1, 1);
            item.change_hash = hash(value);
            if value > 1 {
                item.dependencies = vec![hash(value - 1)].into();
            }
            candidates.insert(item.change_hash, item);
        }
        let mut deep = candidate(65, 1, 1, 1);
        deep.change_hash = hash(65);
        deep.dependencies = vec![hash(64)].into();
        let closure = candidate_dependency_closure(
            &deep,
            &candidates,
            &mut WorkBudget::new(0, 100_000),
            &NeverCancelled,
        );
        assert!(closure.is_ok());
        let Ok(closure) = closure else { return };
        assert_eq!(closure.known.len(), 64);
        assert_eq!(closure.ordered, (1..=64).map(hash).collect::<Vec<_>>());

        let missing = (100..164).map(hash).collect::<Vec<_>>();
        let mut wide = candidate(66, 1, 1, 1);
        wide.change_hash = hash(66);
        wide.dependencies = missing.clone().into();
        let closure = candidate_dependency_closure(
            &wide,
            &candidates,
            &mut WorkBudget::new(0, 100_000),
            &NeverCancelled,
        );
        assert!(closure.is_ok());
        let Ok(closure) = closure else { return };
        assert_eq!(closure.missing, missing.into_iter().collect());
        assert!(closure.known.is_empty() && closure.ordered.is_empty());

        let mut cycle_left = candidate(200, 1, 1, 1);
        cycle_left.change_hash = hash(200);
        cycle_left.dependencies = vec![hash(201)].into();
        let mut cycle_right = candidate(201, 1, 1, 1);
        cycle_right.change_hash = hash(201);
        cycle_right.dependencies = vec![hash(200)].into();
        let cycle_candidates = [cycle_left, cycle_right]
            .into_iter()
            .map(|candidate| (candidate.change_hash, candidate))
            .collect::<BTreeMap<_, _>>();
        let mut cycle = candidate(202, 1, 1, 1);
        cycle.change_hash = hash(202);
        cycle.dependencies = vec![hash(200)].into();
        let closure = candidate_dependency_closure(
            &cycle,
            &cycle_candidates,
            &mut WorkBudget::new(0, 10_000),
            &NeverCancelled,
        );
        assert!(closure.is_ok());
        let Ok(closure) = closure else { return };
        assert_eq!(closure.cyclic, BTreeSet::from([hash(200), hash(201)]));
        assert!(closure.ordered.is_empty());
    }

    #[test]
    fn finding_100_dependency_closure_work_reproduction() {
        let dependencies = (1..=64).map(hash).collect::<Vec<_>>();
        let mut root = candidate(65, 1, 1, 1);
        root.change_hash = hash(65);
        root.dependencies = dependencies.clone().into();
        let closure = candidate_dependency_closure(
            &root,
            &BTreeMap::new(),
            &mut WorkBudget::new(0, 10_000),
            &NeverCancelled,
        );
        assert!(closure.is_ok_and(|value| {
            value.known.is_empty()
                && value.missing == dependencies.into_iter().collect::<BTreeSet<_>>()
                && value.ordered.is_empty()
        }));

        let source = include_str!("closure.rs");
        let prohibited = [
            ["Vec::with_", "capacity(candidate.dependencies.len())"].concat(),
            [".collect::<Result<", "BTreeMap<_, _>, _>>()"].concat(),
            [".collect::<Result<", "Vec<_>, _>>()?"].concat(),
            ["while let Some(hash) = pending.", "pop()"].concat(),
            ["result.known.", "difference(&ordered)"].concat(),
        ];
        assert!(
            prohibited.iter().all(|fragment| !source.contains(fragment)),
            "unmetered dependency-closure preparation remains"
        );
    }
}
