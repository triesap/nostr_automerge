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
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<Vec<EquivocationGroup>, QuarantineError> {
    let mut groups = BTreeMap::<(ActorId, u64), BTreeMap<ChangeHash, BTreeSet<EventId>>>::new();
    for candidate in candidates {
        charge_quarantine_work(budget, cancellation, WorkCounter::GraphNode)?;
        let carrier_count = u64::try_from(candidate.valid_carriers.len())
            .map_err(|_| QuarantineError::BudgetExhausted)?;
        charge_quarantine_amount(budget, cancellation, WorkCounter::GraphEdge, carrier_count)?;
        groups
            .entry((candidate.actor, candidate.sequence))
            .or_default()
            .entry(candidate.change_hash)
            .or_default()
            .extend(candidate.valid_carriers);
    }
    let mut first_by_actor = BTreeMap::<ActorId, EquivocationGroup>::new();
    for ((actor, sequence), changes) in groups {
        charge_quarantine_work(budget, cancellation, WorkCounter::GraphNode)?;
        if changes.len() < 2 || first_by_actor.contains_key(&actor) {
            continue;
        }
        let conflicting_count =
            u64::try_from(changes.len()).map_err(|_| QuarantineError::BudgetExhausted)?;
        let carrier_count = changes.values().try_fold(0_u64, |total, carriers| {
            u64::try_from(carriers.len())
                .ok()
                .and_then(|count| total.checked_add(count))
        });
        let carrier_count = carrier_count.ok_or(QuarantineError::BudgetExhausted)?;
        charge_quarantine_amount(
            budget,
            cancellation,
            WorkCounter::GraphNode,
            conflicting_count,
        )?;
        charge_quarantine_amount(budget, cancellation, WorkCounter::GraphEdge, carrier_count)?;
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
    let count =
        u64::try_from(first_by_actor.len()).map_err(|_| QuarantineError::BudgetExhausted)?;
    charge_quarantine_amount(budget, cancellation, WorkCounter::GraphNode, count)?;
    Ok(first_by_actor.into_values().collect())
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
    let groups = detect_equivocations(candidates.values().cloned(), budget, cancellation)?;
    let mut quarantined = BTreeSet::new();
    let mut alerts = Vec::with_capacity(groups.len());

    for group in groups {
        let mut affected = BTreeSet::new();
        for hash in &group.conflicting_changes {
            charge_quarantine_work(budget, cancellation, WorkCounter::GraphNode)?;
            affected.insert(*hash);
        }
        for candidate in candidates.values() {
            charge_quarantine_work(budget, cancellation, WorkCounter::GraphNode)?;
            if candidate.actor == group.actor && candidate.sequence > group.first_sequence {
                affected.insert(candidate.change_hash);
            }
        }
        let mut queue = Vec::with_capacity(affected.len());
        for hash in &affected {
            charge_quarantine_work(budget, cancellation, WorkCounter::GraphNode)?;
            queue.push(*hash);
        }
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
        let mut descendants = Vec::new();
        for hash in affected.difference(&group.conflicting_changes) {
            charge_quarantine_work(budget, cancellation, WorkCounter::GraphNode)?;
            descendants.push(*hash);
        }
        let mut conflicting_changes = Vec::with_capacity(group.conflicting_changes.len());
        for hash in &group.conflicting_changes {
            charge_quarantine_work(budget, cancellation, WorkCounter::GraphNode)?;
            conflicting_changes.push(*hash);
        }
        alerts.push(IntegrityAlert::DeviceEquivocation(
            DeviceEquivocationAlert::new(
                group.actor,
                group.first_sequence,
                conflicting_changes,
                descendants,
            )?,
        ));
        for hash in affected {
            charge_quarantine_work(budget, cancellation, WorkCounter::GraphNode)?;
            quarantined.insert(hash);
        }
    }

    Ok(QuarantineResult {
        quarantined,
        alerts,
    })
}

fn charge_quarantine_work(
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
    counter: WorkCounter,
) -> Result<(), QuarantineError> {
    charge_quarantine_amount(budget, cancellation, counter, 1)
}

fn charge_quarantine_amount(
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
    counter: WorkCounter,
    amount: u64,
) -> Result<(), QuarantineError> {
    if cancellation.is_cancelled() {
        return Err(QuarantineError::Cancelled);
    }
    budget
        .charge(counter, amount)
        .map_err(|_| QuarantineError::BudgetExhausted)
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
        let detect = |candidates| {
            detect_equivocations(candidates, &mut WorkBudget::new(0, 100), &NeverCancelled)
                .unwrap_or_default()
        };
        assert!(detect(vec![first.clone()]).is_empty());
        assert!(detect(vec![first.clone(), first.clone()]).is_empty());

        let mut conflict = first.clone();
        conflict.change_hash = ChangeHash::from_bytes([2; 32]);
        conflict.valid_carriers = [EventId::from_bytes([8; 32])].into();
        let mut third = conflict.clone();
        third.change_hash = ChangeHash::from_bytes([3; 32]);
        let mut later = conflict.clone();
        later.sequence = 2;
        later.change_hash = ChangeHash::from_bytes([4; 32]);
        let groups = detect(vec![later, third, conflict, first]);
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
        let groups = detect_equivocations(
            [later_conflict, first_conflict.clone(), later, first.clone()],
            &mut WorkBudget::new(0, 100),
            &NeverCancelled,
        )
        .unwrap_or_default();
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
        assert!(budget.consumed().get(WorkCounter::GraphNode) > 9);
        assert!(budget.consumed().get(WorkCounter::GraphEdge) >= 2);
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

    #[test]
    fn quarantine_later_actor_changes() {
        let first = candidate(7, 1, 1, 1);
        let mut conflict = first.clone();
        conflict.change_hash = ChangeHash::from_bytes([2; 32]);
        let mut dependent = candidate(7, 2, 2, 1);
        dependent.change_hash = ChangeHash::from_bytes([3; 32]);
        dependent.dependencies = vec![first.change_hash];
        let mut independent = candidate(7, 3, 3, 1);
        independent.change_hash = ChangeHash::from_bytes([4; 32]);
        let inputs = vec![
            independent.clone(),
            dependent.clone(),
            conflict.clone(),
            first.clone(),
        ];
        let graph = build_graph(inputs.clone(), BTreeSet::new());
        assert!(graph.is_ok());
        let Ok(graph) = graph else { return };
        let result = quarantine_descendants(
            inputs,
            &graph,
            &mut WorkBudget::new(0, 100),
            &NeverCancelled,
        );
        assert!(result.is_ok_and(|result| {
            [
                first.change_hash,
                conflict.change_hash,
                dependent.change_hash,
                independent.change_hash,
            ]
            .iter()
            .all(|hash| result.quarantined.contains(hash))
        }));
    }
}
