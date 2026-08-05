use std::collections::{BTreeMap, BTreeSet};

use super::dependency_graph::DependencyGraph;
use crate::ChangeHash;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Topology {
    pub(crate) order: Vec<ChangeHash>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TopologyError {
    PendingMissing(Vec<ChangeHash>),
    Cycle(Vec<ChangeHash>),
}

pub(crate) fn validate_topology(graph: &DependencyGraph) -> Result<Topology, TopologyError> {
    let missing = graph
        .nodes
        .values()
        .flatten()
        .filter(|dependency| {
            !graph.nodes.contains_key(*dependency) && !graph.accepted_base.contains(*dependency)
        })
        .copied()
        .collect::<BTreeSet<_>>();
    if !missing.is_empty() {
        return Err(TopologyError::PendingMissing(missing.into_iter().collect()));
    }

    let mut indegree = BTreeMap::new();
    let mut dependants = BTreeMap::<ChangeHash, BTreeSet<ChangeHash>>::new();
    for (hash, dependencies) in &graph.nodes {
        let count = dependencies
            .iter()
            .filter(|dependency| graph.nodes.contains_key(*dependency))
            .count();
        indegree.insert(*hash, count);
        for dependency in dependencies {
            if graph.nodes.contains_key(dependency) {
                dependants.entry(*dependency).or_default().insert(*hash);
            }
        }
    }
    let mut ready = indegree
        .iter()
        .filter_map(|(hash, degree)| (*degree == 0).then_some(*hash))
        .collect::<BTreeSet<_>>();
    let mut order = Vec::with_capacity(graph.nodes.len());
    while let Some(hash) = ready.pop_first() {
        order.push(hash);
        if let Some(children) = dependants.get(&hash) {
            for child in children {
                if let Some(degree) = indegree.get_mut(child) {
                    *degree -= 1;
                    if *degree == 0 {
                        ready.insert(*child);
                    }
                }
            }
        }
    }
    if order.len() != graph.nodes.len() {
        let ordered = order.iter().copied().collect::<BTreeSet<_>>();
        let cyclic = graph
            .nodes
            .keys()
            .filter(|hash| !ordered.contains(hash))
            .copied()
            .collect();
        return Err(TopologyError::Cycle(cyclic));
    }
    Ok(Topology { order })
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{TopologyError, validate_topology};
    use crate::ChangeHash;
    use crate::graph::dependency_graph::DependencyGraph;

    fn hash(byte: u8) -> ChangeHash {
        ChangeHash::from_bytes([byte; 32])
    }

    #[test]
    fn detect_cycles_and_malformed_dependencies() {
        let valid = DependencyGraph {
            nodes: BTreeMap::from([
                (hash(1), BTreeSet::new()),
                (hash(2), BTreeSet::from([hash(1)])),
            ]),
            dependants: BTreeMap::new(),
            accepted_base: BTreeSet::new(),
            edge_count: 1,
        };
        assert_eq!(
            validate_topology(&valid).map(|topology| topology.order),
            Ok(vec![hash(1), hash(2)])
        );
        let missing = DependencyGraph {
            nodes: BTreeMap::from([(hash(2), BTreeSet::from([hash(9)]))]),
            dependants: BTreeMap::new(),
            accepted_base: BTreeSet::new(),
            edge_count: 1,
        };
        assert_eq!(
            validate_topology(&missing),
            Err(TopologyError::PendingMissing(vec![hash(9)]))
        );
        let cycle = DependencyGraph {
            nodes: BTreeMap::from([
                (hash(1), BTreeSet::from([hash(2)])),
                (hash(2), BTreeSet::from([hash(1)])),
            ]),
            dependants: BTreeMap::new(),
            accepted_base: BTreeSet::new(),
            edge_count: 2,
        };
        assert_eq!(
            validate_topology(&cycle),
            Err(TopologyError::Cycle(vec![hash(1), hash(2)]))
        );
        let self_cycle = DependencyGraph {
            nodes: BTreeMap::from([(hash(3), BTreeSet::from([hash(3)]))]),
            dependants: BTreeMap::new(),
            accepted_base: BTreeSet::new(),
            edge_count: 1,
        };
        assert_eq!(
            validate_topology(&self_cycle),
            Err(TopologyError::Cycle(vec![hash(3)]))
        );
    }
}
