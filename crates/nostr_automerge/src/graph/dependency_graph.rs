use std::collections::{BTreeMap, BTreeSet};

use super::change_candidate::ChangeCandidate;
use crate::ChangeHash;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct DependencyGraph {
    pub(crate) nodes: BTreeMap<ChangeHash, BTreeSet<ChangeHash>>,
    pub(crate) dependants: BTreeMap<ChangeHash, BTreeSet<ChangeHash>>,
    pub(crate) indegrees: BTreeMap<ChangeHash, u64>,
    pub(crate) accepted_base: BTreeSet<ChangeHash>,
    pub(crate) edge_count: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GraphBuildError {
    DuplicateNode,
    DuplicateDependency,
    SelfDependency,
    Limit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MeteredGraphBuildError<E> {
    Work(E),
    Graph(GraphBuildError),
}

pub(crate) fn build_graph_metered<E>(
    candidates: &[ChangeCandidate],
    accepted_base: &BTreeSet<ChangeHash>,
    mut charge: impl FnMut(crate::WorkCounter) -> Result<(), E>,
) -> Result<DependencyGraph, MeteredGraphBuildError<E>> {
    let mut nodes = BTreeMap::new();
    let mut edges = 0_u64;
    let mut candidate_iter = candidates.iter();
    for _ in 0..candidates.len() {
        charge(crate::WorkCounter::GraphNode).map_err(MeteredGraphBuildError::Work)?;
        let Some(candidate) = candidate_iter.next() else {
            return Err(MeteredGraphBuildError::Graph(GraphBuildError::Limit));
        };
        let mut dependencies = BTreeSet::new();
        let mut dependency_iter = candidate.dependencies.iter();
        for _ in 0..candidate.dependencies.len() {
            charge(crate::WorkCounter::GraphEdge).map_err(MeteredGraphBuildError::Work)?;
            let Some(dependency) = dependency_iter.next().copied() else {
                return Err(MeteredGraphBuildError::Graph(GraphBuildError::Limit));
            };
            if !dependencies.insert(dependency) {
                return Err(MeteredGraphBuildError::Graph(
                    GraphBuildError::DuplicateDependency,
                ));
            }
        }
        if dependencies.contains(&candidate.change_hash) {
            return Err(MeteredGraphBuildError::Graph(
                GraphBuildError::SelfDependency,
            ));
        }
        edges = edges
            .checked_add(
                u64::try_from(dependencies.len())
                    .map_err(|_| MeteredGraphBuildError::Graph(GraphBuildError::Limit))?,
            )
            .ok_or(MeteredGraphBuildError::Graph(GraphBuildError::Limit))?;
        if nodes.insert(candidate.change_hash, dependencies).is_some() {
            return Err(MeteredGraphBuildError::Graph(
                GraphBuildError::DuplicateNode,
            ));
        }
    }

    let mut dependants = BTreeMap::<ChangeHash, BTreeSet<ChangeHash>>::new();
    let mut indegrees = BTreeMap::new();
    let mut node_iter = nodes.iter();
    for _ in 0..nodes.len() {
        charge(crate::WorkCounter::GraphNode).map_err(MeteredGraphBuildError::Work)?;
        let Some((hash, dependencies)) = node_iter.next() else {
            return Err(MeteredGraphBuildError::Graph(GraphBuildError::Limit));
        };
        dependants.entry(*hash).or_default();
        let mut indegree = 0_u64;
        let mut dependency_iter = dependencies.iter();
        for _ in 0..dependencies.len() {
            charge(crate::WorkCounter::GraphEdge).map_err(MeteredGraphBuildError::Work)?;
            let Some(dependency) = dependency_iter.next() else {
                return Err(MeteredGraphBuildError::Graph(GraphBuildError::Limit));
            };
            dependants.entry(*dependency).or_default().insert(*hash);
            if nodes.contains_key(dependency) {
                indegree = indegree
                    .checked_add(1)
                    .ok_or(MeteredGraphBuildError::Graph(GraphBuildError::Limit))?;
            }
        }
        indegrees.insert(*hash, indegree);
    }

    let mut owned_base = BTreeSet::new();
    let mut base_iter = accepted_base.iter();
    for _ in 0..accepted_base.len() {
        charge(crate::WorkCounter::GraphNode).map_err(MeteredGraphBuildError::Work)?;
        let Some(hash) = base_iter.next() else {
            return Err(MeteredGraphBuildError::Graph(GraphBuildError::Limit));
        };
        owned_base.insert(*hash);
    }
    Ok(DependencyGraph {
        nodes,
        dependants,
        indegrees,
        accepted_base: owned_base,
        edge_count: edges,
    })
}

pub(crate) fn build_graph(
    candidates: impl IntoIterator<Item = ChangeCandidate>,
    accepted_base: BTreeSet<ChangeHash>,
) -> Result<DependencyGraph, GraphBuildError> {
    build(candidates, accepted_base, None)
}

#[cfg(test)]
fn build_with_limits(
    candidates: impl IntoIterator<Item = ChangeCandidate>,
    accepted_base: BTreeSet<ChangeHash>,
    node_limit: u64,
    edge_limit: u64,
) -> Result<DependencyGraph, GraphBuildError> {
    build(candidates, accepted_base, Some((node_limit, edge_limit)))
}

fn build(
    candidates: impl IntoIterator<Item = ChangeCandidate>,
    accepted_base: BTreeSet<ChangeHash>,
    limits: Option<(u64, u64)>,
) -> Result<DependencyGraph, GraphBuildError> {
    let mut nodes = BTreeMap::new();
    let mut edges = 0_u64;
    for candidate in candidates {
        let dependencies: BTreeSet<_> = candidate.dependencies.iter().copied().collect();
        if dependencies.len() != candidate.dependencies.len() {
            return Err(GraphBuildError::DuplicateDependency);
        }
        if dependencies.contains(&candidate.change_hash) {
            return Err(GraphBuildError::SelfDependency);
        }
        edges = edges
            .checked_add(u64::try_from(dependencies.len()).map_err(|_| GraphBuildError::Limit)?)
            .ok_or(GraphBuildError::Limit)?;
        let node_count = u64::try_from(nodes.len())
            .map_err(|_| GraphBuildError::Limit)?
            .checked_add(1)
            .ok_or(GraphBuildError::Limit)?;
        if let Some((node_limit, edge_limit)) = limits
            && (edges > edge_limit || node_count > node_limit)
        {
            return Err(GraphBuildError::Limit);
        }
        if nodes.insert(candidate.change_hash, dependencies).is_some() {
            return Err(GraphBuildError::DuplicateNode);
        }
    }
    let mut dependants = nodes
        .keys()
        .copied()
        .map(|hash| (hash, BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for (hash, dependencies) in &nodes {
        for dependency in dependencies {
            dependants.entry(*dependency).or_default().insert(*hash);
        }
    }
    let indegrees = nodes
        .iter()
        .map(|(hash, dependencies)| {
            let count = dependencies
                .iter()
                .filter(|dependency| nodes.contains_key(*dependency))
                .count();
            u64::try_from(count)
                .map(|count| (*hash, count))
                .map_err(|_| GraphBuildError::Limit)
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    Ok(DependencyGraph {
        nodes,
        dependants,
        indegrees,
        accepted_base,
        edge_count: edges,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{GraphBuildError, build_graph, build_graph_metered, build_with_limits};
    use crate::graph::change_candidate::ChangeCandidate;
    use crate::{ActorId, ChangeHash, DevicePublicKey, EventId};

    fn candidate(hash: u8, dependencies: Vec<u8>) -> ChangeCandidate {
        ChangeCandidate {
            change_hash: ChangeHash::from_bytes([hash; 32]),
            actor: ActorId::from_bytes([1; 32]),
            sequence: u64::from(hash),
            start_op: u64::from(hash),
            operation_count: 1,
            dependencies: dependencies
                .into_iter()
                .map(|byte| ChangeHash::from_bytes([byte; 32]))
                .collect::<Vec<_>>()
                .into(),
            control_id: EventId::from_bytes([2; 32]),
            author: DevicePublicKey::from_bytes([3; 32]),
            valid_carriers: vec![EventId::from_bytes([hash; 32])].into(),
        }
    }

    #[test]
    fn build_deterministic_dependency_graph() {
        assert_eq!(
            build_graph([], BTreeSet::new()).map(|g| g.nodes.len()),
            Ok(0)
        );
        let graph = build_graph(
            [
                candidate(1, vec![]),
                candidate(2, vec![1]),
                candidate(3, vec![1]),
                candidate(4, vec![2, 3]),
            ],
            BTreeSet::new(),
        );
        assert!(graph.is_ok());
        let graph = match graph {
            Ok(graph) => graph,
            Err(_) => return,
        };
        assert_eq!(graph.nodes[&ChangeHash::from_bytes([4; 32])].len(), 2);
        assert_eq!(graph.edge_count, 4);
        assert_eq!(
            graph.indegrees,
            BTreeMap::from([
                (ChangeHash::from_bytes([1; 32]), 0),
                (ChangeHash::from_bytes([2; 32]), 1),
                (ChangeHash::from_bytes([3; 32]), 1),
                (ChangeHash::from_bytes([4; 32]), 2),
            ])
        );
        assert_eq!(
            graph.dependants[&ChangeHash::from_bytes([1; 32])],
            BTreeSet::from([
                ChangeHash::from_bytes([2; 32]),
                ChangeHash::from_bytes([3; 32]),
            ])
        );
        assert_eq!(
            graph.dependants[&ChangeHash::from_bytes([2; 32])],
            BTreeSet::from([ChangeHash::from_bytes([4; 32])])
        );
        assert_eq!(
            build_graph([candidate(1, vec![1])], BTreeSet::new()),
            Err(GraphBuildError::SelfDependency)
        );
        assert_eq!(
            build_graph([candidate(2, vec![1, 1])], BTreeSet::new()),
            Err(GraphBuildError::DuplicateDependency)
        );
        assert_eq!(
            build_with_limits([candidate(1, vec![])], BTreeSet::new(), 0, 0),
            Err(GraphBuildError::Limit)
        );
    }

    #[test]
    fn ordinary_graph_limits_are_not_checkpoint_limits() {
        let ordinary = build_graph(
            [candidate(1, vec![]), candidate(2, vec![1])],
            BTreeSet::new(),
        );
        assert!(ordinary.is_ok());
        assert_eq!(
            build_with_limits(
                [candidate(1, vec![]), candidate(2, vec![1])],
                BTreeSet::new(),
                1,
                1,
            ),
            Err(GraphBuildError::Limit)
        );
    }

    #[test]
    fn metered_graph_matches_unmetered_topology_and_charges_each_pass() {
        let candidates = vec![
            candidate(1, vec![]),
            candidate(2, vec![1]),
            candidate(3, vec![1]),
            candidate(4, vec![2, 3]),
        ];
        let expected = build_graph(candidates.clone(), BTreeSet::new());
        let mut charges = Vec::new();
        let measured = build_graph_metered(&candidates, &BTreeSet::new(), |counter| {
            charges.push(counter);
            Ok::<_, ()>(())
        });
        assert_eq!(measured.map_err(|_| GraphBuildError::Limit), expected);
        assert_eq!(
            charges
                .iter()
                .filter(|counter| **counter == crate::WorkCounter::GraphNode)
                .count(),
            8
        );
        assert_eq!(
            charges
                .iter()
                .filter(|counter| **counter == crate::WorkCounter::GraphEdge)
                .count(),
            8
        );
    }
}
