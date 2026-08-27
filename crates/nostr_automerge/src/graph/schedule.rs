use std::borrow::Borrow;
use std::collections::{BTreeMap, BTreeSet};

use super::change_candidate::ChangeCandidate;
use crate::{CancellationCheck, ChangeHash, WorkBudget, WorkCounter};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Schedule {
    pub(crate) ordered: Vec<ChangeHash>,
    pub(crate) pending: BTreeSet<ChangeHash>,
    pub(crate) missing_dependencies: BTreeSet<ChangeHash>,
    pub(crate) cyclic: BTreeSet<ChangeHash>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScheduleError {
    BudgetExhausted,
    Cancelled,
}

pub(crate) fn schedule_candidates(
    candidates: impl IntoIterator<Item = ChangeCandidate>,
    accepted_base: impl Borrow<BTreeSet<ChangeHash>>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<Schedule, ScheduleError> {
    let accepted_base = accepted_base.borrow();
    let mut remaining = BTreeMap::new();
    for candidate in candidates {
        if cancellation.is_cancelled() {
            return Err(ScheduleError::Cancelled);
        }
        budget
            .charge(WorkCounter::GraphNode, 1)
            .map_err(|_| ScheduleError::BudgetExhausted)?;
        remaining.insert(candidate.change_hash, candidate);
    }
    let candidate_hashes = remaining.keys().copied().collect::<BTreeSet<_>>();
    let mut unresolved = BTreeMap::new();
    let mut dependants = BTreeMap::<ChangeHash, BTreeSet<ChangeHash>>::new();
    for (hash, candidate) in &remaining {
        let count = candidate
            .dependencies
            .iter()
            .filter(|dependency| !accepted_base.contains(dependency))
            .count();
        unresolved.insert(*hash, count);
        for dependency in candidate.dependencies.iter() {
            if cancellation.is_cancelled() {
                return Err(ScheduleError::Cancelled);
            }
            budget
                .charge(WorkCounter::GraphEdge, 1)
                .map_err(|_| ScheduleError::BudgetExhausted)?;
            if candidate_hashes.contains(dependency) {
                dependants.entry(*dependency).or_default().insert(*hash);
            }
        }
    }
    let mut ready = unresolved
        .iter()
        .filter_map(|(hash, count)| (*count == 0).then_some(*hash))
        .collect::<BTreeSet<_>>();
    let mut ordered = Vec::new();
    loop {
        if cancellation.is_cancelled() {
            return Err(ScheduleError::Cancelled);
        }
        let Some(hash) = ready.pop_first() else {
            break;
        };
        budget
            .charge(WorkCounter::GraphNode, 1)
            .map_err(|_| ScheduleError::BudgetExhausted)?;
        remaining.remove(&hash);
        ordered.push(hash);
        if let Some(children) = dependants.get(&hash) {
            for child in children {
                if cancellation.is_cancelled() {
                    return Err(ScheduleError::Cancelled);
                }
                budget
                    .charge(WorkCounter::GraphEdge, 1)
                    .map_err(|_| ScheduleError::BudgetExhausted)?;
                if let Some(count) = unresolved.get_mut(child) {
                    *count -= 1;
                    if *count == 0 {
                        ready.insert(*child);
                    }
                }
            }
        }
    }
    let mut missing_dependencies = BTreeSet::new();
    for candidate in remaining.values() {
        for dependency in candidate.dependencies.iter() {
            if cancellation.is_cancelled() {
                return Err(ScheduleError::Cancelled);
            }
            budget
                .charge(WorkCounter::GraphEdge, 1)
                .map_err(|_| ScheduleError::BudgetExhausted)?;
            if !candidate_hashes.contains(dependency) && !accepted_base.contains(dependency) {
                missing_dependencies.insert(*dependency);
            }
        }
    }
    let mut pending = BTreeSet::new();
    for (hash, candidate) in &remaining {
        if cancellation.is_cancelled() {
            return Err(ScheduleError::Cancelled);
        }
        budget
            .charge(WorkCounter::GraphNode, 1)
            .map_err(|_| ScheduleError::BudgetExhausted)?;
        if candidate
            .dependencies
            .iter()
            .any(|dependency| missing_dependencies.contains(dependency))
        {
            pending.insert(*hash);
        }
    }
    let mut blocked = pending.iter().copied().collect::<Vec<_>>();
    while let Some(hash) = blocked.pop() {
        if let Some(children) = dependants.get(&hash) {
            for child in children {
                if cancellation.is_cancelled() {
                    return Err(ScheduleError::Cancelled);
                }
                budget
                    .charge(WorkCounter::GraphEdge, 1)
                    .map_err(|_| ScheduleError::BudgetExhausted)?;
                if remaining.contains_key(child) && pending.insert(*child) {
                    blocked.push(*child);
                }
            }
        }
    }
    let mut cyclic = BTreeSet::new();
    for hash in remaining.keys() {
        if cancellation.is_cancelled() {
            return Err(ScheduleError::Cancelled);
        }
        budget
            .charge(WorkCounter::GraphNode, 1)
            .map_err(|_| ScheduleError::BudgetExhausted)?;
        if !pending.contains(hash) {
            cyclic.insert(*hash);
        }
    }
    Ok(Schedule {
        ordered,
        pending,
        missing_dependencies,
        cyclic,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{ScheduleError, schedule_candidates};
    use crate::graph::actor_state::tests::candidate;
    use crate::{ChangeHash, NeverCancelled, WorkBudget, WorkCounter};

    #[test]
    fn implement_deterministic_candidate_scheduling() {
        let mut low = candidate(1, 1, 1, 1);
        low.change_hash = ChangeHash::from_bytes([1; 32]);
        let mut high = candidate(2, 1, 1, 1);
        high.change_hash = ChangeHash::from_bytes([2; 32]);
        let mut dependant = candidate(1, 2, 2, 1);
        dependant.change_hash = ChangeHash::from_bytes([3; 32]);
        dependant.dependencies = vec![high.change_hash].into();
        let evaluate = |candidates| {
            schedule_candidates(
                candidates,
                BTreeSet::new(),
                &mut WorkBudget::new(0, 30),
                &NeverCancelled,
            )
        };
        let first = evaluate(vec![dependant.clone(), high.clone(), low.clone()]);
        let second = evaluate(vec![low.clone(), high.clone(), dependant.clone()]);
        assert_eq!(first, second);
        assert_eq!(
            first.map(|schedule| schedule.ordered),
            Ok(vec![
                low.change_hash,
                high.change_hash,
                dependant.change_hash
            ])
        );
        let mut measured = WorkBudget::new(0, 10);
        let schedule = schedule_candidates(
            [high.clone(), dependant.clone()],
            BTreeSet::new(),
            &mut measured,
            &NeverCancelled,
        );
        assert!(schedule.is_ok());
        assert_eq!(measured.consumed().get(WorkCounter::GraphNode), 4);
        assert_eq!(measured.consumed().get(WorkCounter::GraphEdge), 2);
        let missing_hash = ChangeHash::from_bytes([9; 32]);
        let mut missing = high.clone();
        missing.dependencies = vec![missing_hash].into();
        let missing_schedule = evaluate(vec![missing]).map(|schedule| {
            (
                schedule.pending,
                schedule.missing_dependencies,
                schedule.cyclic,
            )
        });
        assert_eq!(
            missing_schedule,
            Ok((
                BTreeSet::from([high.change_hash]),
                BTreeSet::from([missing_hash]),
                BTreeSet::new(),
            ))
        );
        let mut cycle_low = low.clone();
        cycle_low.dependencies = vec![high.change_hash].into();
        let mut cycle_high = high.clone();
        cycle_high.dependencies = vec![low.change_hash].into();
        assert_eq!(
            evaluate(vec![cycle_high, cycle_low])
                .map(|schedule| (schedule.pending, schedule.cyclic)),
            Ok((
                BTreeSet::new(),
                BTreeSet::from([low.change_hash, high.change_hash]),
            ))
        );
        let mut fan_out = candidate(3, 1, 1, 1);
        fan_out.change_hash = ChangeHash::from_bytes([4; 32]);
        fan_out.dependencies = vec![low.change_hash].into();
        let mut fan_in = candidate(4, 1, 1, 1);
        fan_in.change_hash = ChangeHash::from_bytes([5; 32]);
        fan_in.dependencies = vec![dependant.change_hash, fan_out.change_hash].into();
        assert_eq!(
            evaluate(vec![
                fan_in.clone(),
                fan_out.clone(),
                dependant.clone(),
                high.clone(),
                low.clone(),
            ])
            .map(|schedule| schedule.ordered),
            Ok(vec![
                ChangeHash::from_bytes([1; 32]),
                ChangeHash::from_bytes([2; 32]),
                ChangeHash::from_bytes([3; 32]),
                ChangeHash::from_bytes([4; 32]),
                ChangeHash::from_bytes([5; 32]),
            ])
        );
        assert_eq!(
            schedule_candidates(
                [low.clone()],
                BTreeSet::new(),
                &mut WorkBudget::new(0, 0),
                &NeverCancelled,
            ),
            Err(ScheduleError::BudgetExhausted)
        );
        assert_eq!(
            schedule_candidates([low], BTreeSet::new(), &mut WorkBudget::new(0, 10), &|| {
                true
            },),
            Err(ScheduleError::Cancelled)
        );
    }

    #[test]
    fn dependency_cycle_is_invalid() {
        let evaluate = |candidates| {
            schedule_candidates(
                candidates,
                BTreeSet::new(),
                &mut WorkBudget::new(0, 100),
                &NeverCancelled,
            )
        };
        let mut left = candidate(1, 1, 1, 1);
        left.change_hash = ChangeHash::from_bytes([1; 32]);
        let mut right = candidate(2, 1, 1, 1);
        right.change_hash = ChangeHash::from_bytes([2; 32]);
        left.dependencies = vec![right.change_hash].into();
        right.dependencies = vec![left.change_hash].into();
        let mut descendant = candidate(3, 1, 1, 1);
        descendant.change_hash = ChangeHash::from_bytes([3; 32]);
        descendant.dependencies = vec![right.change_hash].into();
        assert_eq!(
            evaluate(vec![descendant.clone(), right.clone(), left.clone()])
                .map(|schedule| schedule.cyclic),
            Ok(BTreeSet::from([
                left.change_hash,
                right.change_hash,
                descendant.change_hash,
            ]))
        );

        let mut third = candidate(3, 1, 1, 1);
        third.change_hash = ChangeHash::from_bytes([4; 32]);
        right.dependencies = vec![third.change_hash].into();
        third.dependencies = vec![left.change_hash].into();
        assert_eq!(
            evaluate(vec![third.clone(), right, left.clone()]).map(|schedule| schedule.cyclic),
            Ok(BTreeSet::from([
                left.change_hash,
                ChangeHash::from_bytes([2; 32]),
                third.change_hash,
            ]))
        );

        let mut self_cycle = candidate(4, 1, 1, 1);
        self_cycle.change_hash = ChangeHash::from_bytes([5; 32]);
        self_cycle.dependencies = vec![self_cycle.change_hash].into();
        assert_eq!(
            evaluate(vec![self_cycle.clone()]).map(|schedule| schedule.cyclic),
            Ok(BTreeSet::from([self_cycle.change_hash]))
        );
    }

    #[test]
    #[ignore = "remediation v12 expected failure: schedule readiness is not fully metered"]
    fn finding_100_schedule_readiness_work_reproduction() {
        let mut inputs = (1..=64)
            .rev()
            .map(|value| {
                let mut item = candidate(value, 1, 1, 1);
                item.change_hash = ChangeHash::from_bytes([value; 32]);
                item
            })
            .collect::<Vec<_>>();
        let expected = (1..=64)
            .map(|value| ChangeHash::from_bytes([value; 32]))
            .collect::<Vec<_>>();
        let scheduled = schedule_candidates(
            inputs.drain(..),
            BTreeSet::new(),
            &mut WorkBudget::new(0, 10_000),
            &NeverCancelled,
        );
        assert_eq!(scheduled.map(|value| value.ordered), Ok(expected));

        let source = include_str!("schedule.rs");
        assert!(
            !source.contains("remaining.keys().copied().collect::<BTreeSet<_>>()")
                && !source.contains(".collect::<BTreeSet<_>>();")
                && !source.contains("pending.iter().copied().collect::<Vec<_>>()"),
            "unmetered schedule readiness and pop preparation remains"
        );
    }

    #[test]
    #[ignore = "remediation v12 expected failure: schedule publication is not separately metered"]
    fn finding_100_schedule_publication_work_reproduction() {
        let item = candidate(1, 1, 1, 1);
        let result = schedule_candidates(
            [item.clone()],
            BTreeSet::new(),
            &mut WorkBudget::new(0, 10),
            &NeverCancelled,
        );
        assert_eq!(
            result.map(|value| value.ordered),
            Ok(vec![item.change_hash])
        );

        let source = include_str!("schedule.rs");
        assert!(
            !source.contains("remaining.insert(candidate.change_hash, candidate);")
                && !source.contains("ordered.push(hash);")
                && !source.contains("Ok(Schedule {"),
            "unmetered schedule insertion and result publication remains"
        );
    }
}
