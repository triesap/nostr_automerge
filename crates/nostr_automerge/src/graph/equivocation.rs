use std::collections::{BTreeMap, BTreeSet};

use super::change_candidate::ChangeCandidate;
use super::dependency_graph::DependencyGraph;
use crate::integrity::{AlertError, DeviceEquivocationAlert, IntegrityAlert};
use crate::{
    ActorId, CancellationCheck, ChangeHash, EventId, ProtocolDisposition, WorkBudget, WorkCounter,
};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum QuarantineOverlayOperation {
    QuarantinedPull,
    DispositionInsert,
}

pub(crate) fn publish_quarantine_dispositions_metered<E>(
    quarantined: &BTreeSet<ChangeHash>,
    dispositions: &mut BTreeMap<ChangeHash, ProtocolDisposition>,
    mut charge: impl FnMut(WorkCounter) -> Result<(), E>,
) -> Result<(), E> {
    publish_quarantine_dispositions_observed(quarantined, dispositions, &mut charge, |_| {})
}

fn publish_quarantine_dispositions_observed<E>(
    quarantined: &BTreeSet<ChangeHash>,
    dispositions: &mut BTreeMap<ChangeHash, ProtocolDisposition>,
    charge: &mut impl FnMut(WorkCounter) -> Result<(), E>,
    mut observed: impl FnMut(QuarantineOverlayOperation),
) -> Result<(), E> {
    let mut items = quarantined.iter();
    for _ in 0..quarantined.len() {
        charge(WorkCounter::GraphNode)?;
        let hash = items.next().copied();
        observed(QuarantineOverlayOperation::QuarantinedPull);
        let Some(hash) = hash else { break };
        charge(WorkCounter::GraphNode)?;
        dispositions.insert(hash, ProtocolDisposition::Excluded);
        observed(QuarantineOverlayOperation::DispositionInsert);
    }
    Ok(())
}

pub(crate) fn detect_equivocations<I>(
    candidates: I,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<Vec<EquivocationGroup>, QuarantineError>
where
    I: IntoIterator<Item = ChangeCandidate>,
    I::IntoIter: ExactSizeIterator,
{
    let mut groups = BTreeMap::<(ActorId, u64), BTreeMap<ChangeHash, BTreeSet<EventId>>>::new();
    let mut candidate_items = candidates.into_iter();
    for _ in 0..candidate_items.len() {
        charge_quarantine_work(budget, cancellation, WorkCounter::GraphNode)?;
        let Some(candidate) = candidate_items.next() else {
            return Err(QuarantineError::BudgetExhausted);
        };
        let sequence_group = groups
            .entry((candidate.actor, candidate.sequence))
            .or_default();
        let carrier_group = sequence_group.entry(candidate.change_hash).or_default();
        let mut carriers = candidate.valid_carriers.iter();
        for _ in 0..candidate.valid_carriers.len() {
            charge_quarantine_work(budget, cancellation, WorkCounter::GraphEdge)?;
            let Some(event_id) = carriers.next() else {
                return Err(QuarantineError::BudgetExhausted);
            };
            carrier_group.insert(*event_id);
        }
    }
    let mut first_by_actor = BTreeMap::<ActorId, EquivocationGroup>::new();
    let mut group_items = groups.into_iter();
    for _ in 0..group_items.len() {
        charge_quarantine_work(budget, cancellation, WorkCounter::GraphNode)?;
        let Some(((actor, sequence), changes)) = group_items.next() else {
            return Err(QuarantineError::BudgetExhausted);
        };
        if changes.len() < 2 {
            continue;
        }
        charge_quarantine_work(budget, cancellation, WorkCounter::GraphNode)?;
        if first_by_actor.contains_key(&actor) {
            continue;
        }
        let mut conflicting_changes = BTreeSet::new();
        let mut carrier_event_ids = BTreeSet::new();
        let mut changes_items = changes.iter();
        for _ in 0..changes.len() {
            charge_quarantine_work(budget, cancellation, WorkCounter::GraphNode)?;
            let Some((hash, carriers)) = changes_items.next() else {
                return Err(QuarantineError::BudgetExhausted);
            };
            conflicting_changes.insert(*hash);
            let mut carrier_items = carriers.iter();
            for _ in 0..carriers.len() {
                charge_quarantine_work(budget, cancellation, WorkCounter::GraphEdge)?;
                let Some(event_id) = carrier_items.next() else {
                    return Err(QuarantineError::BudgetExhausted);
                };
                carrier_event_ids.insert(*event_id);
            }
        }
        charge_quarantine_work(budget, cancellation, WorkCounter::GraphNode)?;
        first_by_actor.insert(
            actor,
            EquivocationGroup {
                actor,
                first_sequence: sequence,
                conflicting_changes,
                carrier_event_ids,
            },
        );
    }
    let mut detected = Vec::new();
    let mut actor_items = first_by_actor.into_values();
    for _ in 0..actor_items.len() {
        charge_quarantine_work(budget, cancellation, WorkCounter::GraphNode)?;
        let Some(group) = actor_items.next() else {
            return Err(QuarantineError::BudgetExhausted);
        };
        detected.push(group);
    }
    Ok(detected)
}

pub(crate) fn quarantine_equivocation_descendants<I>(
    inputs: I,
    graph: &DependencyGraph,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<QuarantineResult, QuarantineError>
where
    I: IntoIterator<Item = ChangeCandidate>,
    I::IntoIter: ExactSizeIterator,
{
    let mut candidates = BTreeMap::new();
    let mut candidates_by_actor = BTreeMap::<ActorId, Vec<(u64, ChangeHash)>>::new();
    let mut input_items = inputs.into_iter();
    for _ in 0..input_items.len() {
        charge_quarantine_work(budget, cancellation, WorkCounter::GraphNode)?;
        let Some(candidate) = input_items.next() else {
            return Err(QuarantineError::BudgetExhausted);
        };
        let actor_candidates = candidates_by_actor.entry(candidate.actor).or_default();
        actor_candidates.push((candidate.sequence, candidate.change_hash));
        candidates.insert(candidate.change_hash, candidate);
    }
    let groups = detect_equivocations(candidates.values().cloned(), budget, cancellation)?;
    let mut quarantined = BTreeSet::new();
    let mut alerts = Vec::new();

    let mut group_items = groups.into_iter();
    for _ in 0..group_items.len() {
        charge_quarantine_work(budget, cancellation, WorkCounter::GraphNode)?;
        let Some(group) = group_items.next() else {
            return Err(QuarantineError::BudgetExhausted);
        };
        let mut affected = BTreeSet::new();
        let mut conflicting_items = group.conflicting_changes.iter();
        for _ in 0..group.conflicting_changes.len() {
            charge_quarantine_work(budget, cancellation, WorkCounter::GraphNode)?;
            let Some(hash) = conflicting_items.next() else {
                return Err(QuarantineError::BudgetExhausted);
            };
            affected.insert(*hash);
        }
        charge_quarantine_work(budget, cancellation, WorkCounter::GraphNode)?;
        if let Some(actor_candidates) = candidates_by_actor.get(&group.actor) {
            let mut actor_items = actor_candidates.iter();
            for _ in 0..actor_candidates.len() {
                charge_quarantine_work(budget, cancellation, WorkCounter::GraphNode)?;
                let Some((sequence, hash)) = actor_items.next() else {
                    return Err(QuarantineError::BudgetExhausted);
                };
                if *sequence > group.first_sequence {
                    affected.insert(*hash);
                }
            }
        }
        let mut queue = Vec::new();
        let mut affected_items = affected.iter();
        for _ in 0..affected.len() {
            charge_quarantine_work(budget, cancellation, WorkCounter::GraphNode)?;
            let Some(hash) = affected_items.next() else {
                return Err(QuarantineError::BudgetExhausted);
            };
            queue.push(*hash);
        }
        while !queue.is_empty() {
            charge_quarantine_work(budget, cancellation, WorkCounter::GraphNode)?;
            let Some(hash) = queue.pop() else {
                return Err(QuarantineError::BudgetExhausted);
            };
            if let Some(dependants) = graph.dependants.get(&hash) {
                let mut dependant_items = dependants.iter().rev();
                for _ in 0..dependants.len() {
                    charge_quarantine_work(budget, cancellation, WorkCounter::GraphEdge)?;
                    let Some(dependant) = dependant_items.next() else {
                        return Err(QuarantineError::BudgetExhausted);
                    };
                    if affected.insert(*dependant) {
                        queue.push(*dependant);
                    }
                }
            }
        }
        let mut descendants = Vec::new();
        let mut affected_items = affected.iter();
        for _ in 0..affected.len() {
            charge_quarantine_work(budget, cancellation, WorkCounter::GraphNode)?;
            let Some(hash) = affected_items.next() else {
                return Err(QuarantineError::BudgetExhausted);
            };
            if !group.conflicting_changes.contains(hash) {
                descendants.push(*hash);
            }
        }
        let mut conflicting_changes = Vec::new();
        let mut conflicting_items = group.conflicting_changes.iter();
        for _ in 0..group.conflicting_changes.len() {
            charge_quarantine_work(budget, cancellation, WorkCounter::GraphNode)?;
            let Some(hash) = conflicting_items.next() else {
                return Err(QuarantineError::BudgetExhausted);
            };
            conflicting_changes.push(*hash);
        }
        if group.first_sequence == 0 {
            return Err(QuarantineError::Alert(AlertError));
        }
        alerts.push(IntegrityAlert::DeviceEquivocation(
            DeviceEquivocationAlert::from_validated_parts(
                group.actor,
                group.first_sequence,
                conflicting_changes,
                descendants,
            ),
        ));
        let mut affected_items = affected.into_iter();
        for _ in 0..affected_items.len() {
            charge_quarantine_work(budget, cancellation, WorkCounter::GraphNode)?;
            let Some(hash) = affected_items.next() else {
                return Err(QuarantineError::BudgetExhausted);
            };
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
    if cancellation.is_cancelled() {
        return Err(QuarantineError::Cancelled);
    }
    budget
        .charge(counter, 1)
        .map_err(|_| QuarantineError::BudgetExhausted)
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{
        QuarantineOverlayOperation, detect_equivocations, publish_quarantine_dispositions_observed,
        quarantine_equivocation_descendants as quarantine_descendants,
    };
    use crate::graph::actor_state::tests::candidate;
    use crate::graph::dependency_graph::build_graph;
    use crate::integrity::IntegrityAlert;
    use crate::{
        ChangeHash, EventId, NeverCancelled, ProtocolDisposition, WorkBudget, WorkCounter,
    };

    #[test]
    fn quarantine_overlay_charges_each_pull_and_insert_before_work() {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum Stop {
            BudgetExhausted,
            Cancelled,
        }
        let quarantined = BTreeSet::from([
            ChangeHash::from_bytes([1; 32]),
            ChangeHash::from_bytes([2; 32]),
            ChangeHash::from_bytes([3; 32]),
        ]);
        let mut ample_dispositions = BTreeMap::new();
        let mut trace = Vec::new();
        let ample = publish_quarantine_dispositions_observed(
            &quarantined,
            &mut ample_dispositions,
            &mut |_| Ok::<_, Stop>(()),
            |operation| trace.push(operation),
        );
        assert_eq!(ample, Ok(()));
        assert_eq!(
            trace,
            [
                QuarantineOverlayOperation::QuarantinedPull,
                QuarantineOverlayOperation::DispositionInsert,
                QuarantineOverlayOperation::QuarantinedPull,
                QuarantineOverlayOperation::DispositionInsert,
                QuarantineOverlayOperation::QuarantinedPull,
                QuarantineOverlayOperation::DispositionInsert,
            ]
        );
        assert!(
            ample_dispositions
                .values()
                .all(|value| { *value == ProtocolDisposition::Excluded })
        );

        for allowance in 0..trace.len() {
            for stop in [Stop::BudgetExhausted, Stop::Cancelled] {
                let mut successful = 0_usize;
                let mut observed = Vec::new();
                let mut dispositions = BTreeMap::new();
                let result = publish_quarantine_dispositions_observed(
                    &quarantined,
                    &mut dispositions,
                    &mut |_| {
                        if successful == allowance {
                            return Err(stop);
                        }
                        successful += 1;
                        Ok(())
                    },
                    |operation| observed.push(operation),
                );
                assert_eq!(result, Err(stop));
                assert_eq!(successful, allowance);
                assert_eq!(observed, trace[..allowance]);
                assert_eq!(
                    dispositions.len(),
                    allowance / 2,
                    "no insertion before its charge"
                );
            }
        }
    }

    #[test]
    fn detect_device_equivocation_groups() {
        let first = candidate(1, 1, 1, 1);
        let detect = |candidates| {
            detect_equivocations(candidates, &mut WorkBudget::new(0, 1_000), &NeverCancelled)
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
            &mut WorkBudget::new(0, 1_000),
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
        later_same_actor.dependencies = vec![first.change_hash].into();
        let mut cross_actor = candidate(2, 1, 1, 1);
        cross_actor.change_hash = ChangeHash::from_bytes([4; 32]);
        cross_actor.dependencies = vec![later_same_actor.change_hash].into();
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
        let mut budget = WorkBudget::new(0, 1_000);
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
                &mut WorkBudget::new(0, 1_000),
                &NeverCancelled,
            ),
            Ok(result)
        );

        let mut deep = independent;
        deep.change_hash = ChangeHash::from_bytes([6; 32]);
        deep.dependencies = vec![cross_actor.change_hash].into();
        let mut extended = input;
        extended.push(deep.clone());
        let graph = build_graph(extended.clone(), BTreeSet::new());
        assert!(graph.is_ok());
        let Ok(graph) = graph else { return };
        let extended_result = quarantine_descendants(
            extended.clone(),
            &graph,
            &mut WorkBudget::new(0, 1_000),
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
            quarantine_descendants(extended, &graph, &mut WorkBudget::new(0, 1_000), &|| true,),
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
        dependent.dependencies = vec![first.change_hash].into();
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
            &mut WorkBudget::new(0, 1_000),
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

    #[test]
    fn quarantine_traversal_has_exact_prefix_and_cancellation_boundaries() {
        let first = candidate(1, 1, 1, 1);
        let mut conflict = first.clone();
        conflict.change_hash = ChangeHash::from_bytes([2; 32]);
        let mut dependent = candidate(1, 2, 2, 1);
        dependent.change_hash = ChangeHash::from_bytes([3; 32]);
        dependent.dependencies = vec![first.change_hash].into();
        let input = vec![dependent, conflict, first];
        let graph = build_graph(input.clone(), BTreeSet::new());
        assert!(graph.is_ok());
        let Ok(graph) = graph else { return };

        let mut ample = WorkBudget::new(0, 10_000);
        let expected = quarantine_descendants(input.clone(), &graph, &mut ample, &NeverCancelled);
        assert!(expected.is_ok());
        let expected_items = ample
            .consumed()
            .get(WorkCounter::GraphNode)
            .checked_add(ample.consumed().get(WorkCounter::GraphEdge));
        assert!(expected_items.is_some_and(|count| count > 0));
        let Some(expected_items) = expected_items else {
            return;
        };

        for capacity in [expected_items - 1, expected_items, expected_items + 1] {
            let mut budget = WorkBudget::new(0, capacity);
            let result =
                quarantine_descendants(input.clone(), &graph, &mut budget, &NeverCancelled);
            if capacity < expected_items {
                assert_eq!(result, Err(super::QuarantineError::BudgetExhausted));
            } else {
                assert_eq!(result, expected);
            }
            assert_eq!(
                budget
                    .consumed()
                    .get(WorkCounter::GraphNode)
                    .checked_add(budget.consumed().get(WorkCounter::GraphEdge)),
                Some(capacity.min(expected_items))
            );
        }

        for cancel_at in 0..expected_items {
            let observations = std::cell::Cell::new(0_u64);
            let cancellation = || {
                if observations.get() == cancel_at {
                    true
                } else {
                    observations.set(observations.get().saturating_add(1));
                    false
                }
            };
            let result = quarantine_descendants(
                input.clone(),
                &graph,
                &mut WorkBudget::new(0, expected_items + 1),
                &cancellation,
            );
            assert_eq!(result, Err(super::QuarantineError::Cancelled));
            assert_eq!(observations.get(), cancel_at);
        }
    }
}
