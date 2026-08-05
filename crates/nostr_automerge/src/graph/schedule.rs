use std::collections::{BTreeMap, BTreeSet};

use super::change_candidate::ChangeCandidate;
use crate::{CancellationCheck, ChangeHash, WorkBudget};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Schedule {
    pub(crate) ordered: Vec<ChangeHash>,
    pub(crate) pending: BTreeSet<ChangeHash>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScheduleError {
    BudgetExhausted,
    Cancelled,
}

pub(crate) fn schedule_candidates(
    candidates: impl IntoIterator<Item = ChangeCandidate>,
    accepted_base: BTreeSet<ChangeHash>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<Schedule, ScheduleError> {
    let mut remaining = candidates
        .into_iter()
        .map(|candidate| (candidate.change_hash, candidate))
        .collect::<BTreeMap<_, _>>();
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
        for dependency in &candidate.dependencies {
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
        budget
            .charge_items(1)
            .map_err(|_| ScheduleError::BudgetExhausted)?;
        let Some(hash) = ready.pop_first() else {
            break;
        };
        remaining.remove(&hash);
        ordered.push(hash);
        if let Some(children) = dependants.get(&hash) {
            for child in children {
                if let Some(count) = unresolved.get_mut(child) {
                    *count -= 1;
                    if *count == 0 {
                        ready.insert(*child);
                    }
                }
            }
        }
    }
    Ok(Schedule {
        ordered,
        pending: remaining.into_keys().collect(),
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{ScheduleError, schedule_candidates};
    use crate::graph::actor_state::tests::candidate;
    use crate::{ChangeHash, NeverCancelled, WorkBudget};

    #[test]
    fn implement_deterministic_candidate_scheduling() {
        let mut low = candidate(1, 1, 1, 1);
        low.change_hash = ChangeHash::from_bytes([1; 32]);
        let mut high = candidate(2, 1, 1, 1);
        high.change_hash = ChangeHash::from_bytes([2; 32]);
        let mut dependant = candidate(1, 2, 2, 1);
        dependant.change_hash = ChangeHash::from_bytes([3; 32]);
        dependant.dependencies = vec![high.change_hash];
        let evaluate = |candidates| {
            schedule_candidates(
                candidates,
                BTreeSet::new(),
                &mut WorkBudget::new(0, 10),
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
        let mut fan_out = candidate(3, 1, 1, 1);
        fan_out.change_hash = ChangeHash::from_bytes([4; 32]);
        fan_out.dependencies = vec![low.change_hash];
        let mut fan_in = candidate(4, 1, 1, 1);
        fan_in.change_hash = ChangeHash::from_bytes([5; 32]);
        fan_in.dependencies = vec![dependant.change_hash, fan_out.change_hash];
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
}
