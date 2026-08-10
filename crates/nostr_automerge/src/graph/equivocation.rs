use std::collections::{BTreeMap, BTreeSet};

use super::change_candidate::ChangeCandidate;
use super::dependency_graph::DependencyGraph;
use crate::integrity::{AlertError, DeviceEquivocationAlert, IntegrityAlert};
use crate::{ActorId, CancellationCheck, ChangeHash, EventId, WorkBudget, WorkCounter};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EquivocationGroup {
    pub(crate) actor: ActorId,
    pub(crate) first_sequence: u64,
    pub(crate) conflicting_changes: BTreeSet<ChangeHash>,
    pub(crate) carrier_event_ids: BTreeSet<EventId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct QuarantineResult {
    pub(crate) quarantined: BTreeSet<ChangeHash>,
    pub(crate) alerts: Vec<IntegrityAlert>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum QuarantineError {
    Alert(AlertError),
    BudgetExhausted,
    Cancelled,
}

impl From<AlertError> for QuarantineError {
    fn from(error: AlertError) -> Self {
        Self::Alert(error)
    }
}

pub(crate) fn detect_equivocations(
    candidates: impl IntoIterator<Item = ChangeCandidate>,
) -> Vec<EquivocationGroup> {
    let mut groups = BTreeMap::<(ActorId, u64), BTreeMap<ChangeHash, BTreeSet<EventId>>>::new();
    for candidate in candidates {
        groups
            .entry((candidate.actor, candidate.sequence))
            .or_default()
            .entry(candidate.change_hash)
            .or_default()
            .extend(candidate.valid_carriers);
    }
    let mut first_by_actor = BTreeMap::<ActorId, EquivocationGroup>::new();
    for ((actor, sequence), changes) in groups {
        if changes.len() < 2 || first_by_actor.contains_key(&actor) {
            continue;
        }
        first_by_actor.insert(
            actor,
            EquivocationGroup {
                actor,
                first_sequence: sequence,
                conflicting_changes: changes.keys().copied().collect(),
                carrier_event_ids: changes.into_values().flatten().collect(),
            },
        );
    }
    first_by_actor.into_values().collect()
}

pub(crate) fn quarantine_equivocation_descendants(
    inputs: impl IntoIterator<Item = ChangeCandidate>,
    graph: &DependencyGraph,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<QuarantineResult, QuarantineError> {
    let mut candidates = BTreeMap::new();
    for candidate in inputs {
        if cancellation.is_cancelled() {
            return Err(QuarantineError::Cancelled);
        }
        budget
            .charge(WorkCounter::GraphNode, 1)
            .map_err(|_| QuarantineError::BudgetExhausted)?;
        candidates.insert(candidate.change_hash, candidate);
    }
    let groups = detect_equivocations(candidates.values().cloned());
    let mut quarantined = BTreeSet::new();
    let mut alerts = Vec::with_capacity(groups.len());

    for group in groups {
        let mut affected = group.conflicting_changes.clone();
        affected.extend(
            candidates
                .values()
                .filter(|candidate| {
                    candidate.actor == group.actor && candidate.sequence > group.first_sequence
                })
                .map(|candidate| candidate.change_hash),
        );
        let mut queue = affected.iter().copied().collect::<Vec<_>>();
        while let Some(hash) = queue.pop() {
            if cancellation.is_cancelled() {
                return Err(QuarantineError::Cancelled);
            }
            budget
                .charge(WorkCounter::GraphNode, 1)
                .map_err(|_| QuarantineError::BudgetExhausted)?;
            if let Some(dependants) = graph.dependants.get(&hash) {
                for dependant in dependants.iter().rev() {
                    if cancellation.is_cancelled() {
                        return Err(QuarantineError::Cancelled);
                    }
                    budget
                        .charge(WorkCounter::GraphEdge, 1)
                        .map_err(|_| QuarantineError::BudgetExhausted)?;
                    if affected.insert(*dependant) {
                        queue.push(*dependant);
                    }
                }
            }
        }
        let descendants = affected
            .difference(&group.conflicting_changes)
            .copied()
            .collect::<Vec<_>>();
        alerts.push(IntegrityAlert::DeviceEquivocation(
            DeviceEquivocationAlert::new(
                group.actor,
                group.first_sequence,
                group.conflicting_changes.iter().copied().collect(),
                descendants,
            )?,
        ));
        quarantined.extend(affected);
    }

    Ok(QuarantineResult {
        quarantined,
        alerts,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{
        detect_equivocations, quarantine_equivocation_descendants as quarantine_descendants,
    };
    use crate::graph::actor_state::tests::candidate;
    use crate::graph::dependency_graph::build_graph;
    use crate::integrity::IntegrityAlert;
    use crate::{ChangeHash, EventId, NeverCancelled, WorkBudget, WorkCounter};

    #[test]
    fn detect_device_equivocation_groups() {
        let first = candidate(1, 1, 1, 1);
        assert!(detect_equivocations([first.clone()]).is_empty());
        assert!(detect_equivocations([first.clone(), first.clone()]).is_empty());

        let mut conflict = first.clone();
        conflict.change_hash = ChangeHash::from_bytes([2; 32]);
        conflict.valid_carriers = [EventId::from_bytes([8; 32])].into();
        let mut third = conflict.clone();
        third.change_hash = ChangeHash::from_bytes([3; 32]);
        let mut later = conflict.clone();
        later.sequence = 2;
        later.change_hash = ChangeHash::from_bytes([4; 32]);
        let groups = detect_equivocations([later, third, conflict, first]);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].first_sequence, 1);
        assert_eq!(groups[0].conflicting_changes.len(), 3);
        assert!(
            groups[0]
                .carrier_event_ids
                .contains(&EventId::from_bytes([8; 32]))
        );
    }

    #[test]
    fn first_conflicting_sequence_wins() {
        let first = candidate(1, 1, 1, 1);
        let mut first_conflict = first.clone();
        first_conflict.change_hash = ChangeHash::from_bytes([2; 32]);
        let mut later = candidate(1, 2, 2, 1);
        later.change_hash = ChangeHash::from_bytes([3; 32]);
        let mut later_conflict = later.clone();
        later_conflict.change_hash = ChangeHash::from_bytes([4; 32]);
        let groups =
            detect_equivocations([later_conflict, first_conflict.clone(), later, first.clone()]);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].first_sequence, 1);
        assert_eq!(
            groups[0].conflicting_changes,
            BTreeSet::from([first.change_hash, first_conflict.change_hash])
        );
    }

    #[test]
    fn quarantine_equivocation_descendants() {
        let first = candidate(1, 1, 1, 1);
        let mut conflict = first.clone();
        conflict.change_hash = ChangeHash::from_bytes([2; 32]);
        let mut later_same_actor = candidate(1, 2, 2, 1);
        later_same_actor.change_hash = ChangeHash::from_bytes([3; 32]);
        later_same_actor.dependencies = vec![first.change_hash];
        let mut cross_actor = candidate(2, 1, 1, 1);
        cross_actor.change_hash = ChangeHash::from_bytes([4; 32]);
        cross_actor.dependencies = vec![later_same_actor.change_hash];
        let mut independent = candidate(3, 1, 1, 1);
        independent.change_hash = ChangeHash::from_bytes([5; 32]);
        let input = vec![
            independent.clone(),
            cross_actor.clone(),
            later_same_actor.clone(),
            conflict.clone(),
            first.clone(),
        ];
        let graph = build_graph(input.clone(), BTreeSet::new());
        assert!(graph.is_ok());
        let Ok(graph) = graph else { return };
        let mut budget = WorkBudget::new(0, 100);
        let result = quarantine_descendants(input.clone(), &graph, &mut budget, &NeverCancelled);
        assert!(result.is_ok());
        let Ok(result) = result else { return };
        assert_eq!(
            result.quarantined,
            BTreeSet::from([
                first.change_hash,
                conflict.change_hash,
                later_same_actor.change_hash,
                cross_actor.change_hash,
            ])
        );
        assert!(!result.quarantined.contains(&independent.change_hash));
        assert_eq!(result.alerts.len(), 1);
        assert_eq!(budget.consumed().get(WorkCounter::GraphNode), 9);
        assert_eq!(budget.consumed().get(WorkCounter::GraphEdge), 2);
        assert!(matches!(
            result.alerts[0],
            IntegrityAlert::DeviceEquivocation(_)
        ));
        let IntegrityAlert::DeviceEquivocation(alert) = &result.alerts[0] else {
            return;
        };
        assert_eq!(alert.first_sequence(), 1);
        assert_eq!(
            alert.affected_descendants(),
            &[later_same_actor.change_hash, cross_actor.change_hash]
        );

        let mut reversed = input.clone();
        reversed.reverse();
        assert_eq!(
            quarantine_descendants(
                reversed,
                &graph,
                &mut WorkBudget::new(0, 100),
                &NeverCancelled,
            ),
            Ok(result)
        );

        let mut deep = independent;
        deep.change_hash = ChangeHash::from_bytes([6; 32]);
        deep.dependencies = vec![cross_actor.change_hash];
        let mut extended = input;
        extended.push(deep.clone());
        let graph = build_graph(extended.clone(), BTreeSet::new());
        assert!(graph.is_ok());
        let Ok(graph) = graph else { return };
        let extended_result = quarantine_descendants(
            extended.clone(),
            &graph,
            &mut WorkBudget::new(0, 100),
            &NeverCancelled,
        );
        assert!(
            extended_result
                .as_ref()
                .is_ok_and(|result| result.quarantined.contains(&deep.change_hash))
        );
        assert!(matches!(
            quarantine_descendants(
                extended.clone(),
                &graph,
                &mut WorkBudget::new(0, 0),
                &NeverCancelled,
            ),
            Err(super::QuarantineError::BudgetExhausted)
        ));
        assert!(matches!(
            quarantine_descendants(extended, &graph, &mut WorkBudget::new(0, 100), &|| true,),
            Err(super::QuarantineError::Cancelled)
        ));
    }
}
