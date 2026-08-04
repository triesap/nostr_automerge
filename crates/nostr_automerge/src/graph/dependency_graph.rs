use std::collections::{BTreeMap, BTreeSet};

use super::change_candidate::ChangeCandidate;
use crate::{ChangeHash, ProtocolRevision};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct DependencyGraph {
    pub(crate) nodes: BTreeMap<ChangeHash, BTreeSet<ChangeHash>>,
    pub(crate) accepted_base: BTreeSet<ChangeHash>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GraphBuildError {
    DuplicateNode,
    DuplicateDependency,
    SelfDependency,
    Limit,
}

pub(crate) fn build_graph(
    candidates: impl IntoIterator<Item = ChangeCandidate>,
    accepted_base: BTreeSet<ChangeHash>,
) -> Result<DependencyGraph, GraphBuildError> {
    let limits = ProtocolRevision::draft_v1().limits();
    build_with_limits(
        candidates,
        accepted_base,
        limits.checkpoint_changes.get(),
        limits.checkpoint_dependency_edges.get(),
    )
}

fn build_with_limits(
    candidates: impl IntoIterator<Item = ChangeCandidate>,
    accepted_base: BTreeSet<ChangeHash>,
    node_limit: u64,
    edge_limit: u64,
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
        if edges > edge_limit
            || u64::try_from(nodes.len()).map_err(|_| GraphBuildError::Limit)? >= node_limit
        {
            return Err(GraphBuildError::Limit);
        }
        if nodes.insert(candidate.change_hash, dependencies).is_some() {
            return Err(GraphBuildError::DuplicateNode);
        }
    }
    Ok(DependencyGraph {
        nodes,
        accepted_base,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{GraphBuildError, build_graph, build_with_limits};
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
                .collect(),
            control_id: EventId::from_bytes([2; 32]),
            author: DevicePublicKey::from_bytes([3; 32]),
            valid_carriers: BTreeSet::from([EventId::from_bytes([hash; 32])]),
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
}
