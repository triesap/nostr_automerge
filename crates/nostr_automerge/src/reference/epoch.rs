use std::collections::{BTreeMap, BTreeSet};

use crate::graph::change_candidate::ChangeCandidate;
use crate::graph::schedule::{ScheduleError, schedule_candidates};
use crate::{CancellationCheck, ChangeHash, ProtocolDisposition, WorkBudget, WorkCounter};

#[derive(Clone, Debug)]
pub(crate) struct EpochCandidate {
    pub(crate) candidate: ChangeCandidate,
    pub(crate) semantically_valid: bool,
    pub(crate) canonical_control: bool,
}

pub(crate) fn resolve_epoch(
    candidates: impl IntoIterator<Item = EpochCandidate>,
    accepted_base: &BTreeSet<ChangeHash>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<BTreeMap<ChangeHash, ProtocolDisposition>, ScheduleError> {
    let mut dispositions = BTreeMap::new();
    let mut dependencies = BTreeMap::new();
    let mut eligible = Vec::new();
    for input in candidates {
        if cancellation.is_cancelled() {
            return Err(ScheduleError::Cancelled);
        }
        budget
            .charge(WorkCounter::GraphNode, 1)
            .map_err(|_| ScheduleError::BudgetExhausted)?;
        let hash = input.candidate.change_hash;
        let mut candidate_dependencies = BTreeSet::new();
        for dependency in input.candidate.dependencies.iter() {
            if cancellation.is_cancelled() {
                return Err(ScheduleError::Cancelled);
            }
            budget
                .charge(WorkCounter::GraphEdge, 1)
                .map_err(|_| ScheduleError::BudgetExhausted)?;
            candidate_dependencies.insert(*dependency);
        }
        dependencies.insert(hash, candidate_dependencies);
        if !input.canonical_control {
            dispositions.insert(hash, ProtocolDisposition::Excluded);
        } else if !input.semantically_valid {
            dispositions.insert(hash, ProtocolDisposition::Invalid);
        } else {
            eligible.push(input.candidate);
        }
    }
    let schedule = schedule_candidates(eligible, accepted_base, budget, cancellation)?;
    let _missing_dependencies = schedule.missing_dependencies;
    for hash in schedule.ordered {
        dispositions.insert(hash, ProtocolDisposition::Accepted);
    }
    for hash in schedule.pending {
        dispositions.insert(hash, ProtocolDisposition::Pending);
    }
    for hash in schedule.cyclic {
        dispositions.insert(hash, ProtocolDisposition::Invalid);
    }
    loop {
        let mut updates = BTreeMap::new();
        for (hash, candidate_dependencies) in &dependencies {
            if cancellation.is_cancelled() {
                return Err(ScheduleError::Cancelled);
            }
            budget
                .charge(WorkCounter::GraphNode, 1)
                .map_err(|_| ScheduleError::BudgetExhausted)?;
            let current = dispositions.get(hash).copied();
            if !matches!(
                current,
                Some(ProtocolDisposition::Accepted | ProtocolDisposition::Pending)
            ) {
                continue;
            }
            let mut rejected_dependency = false;
            let mut all_dependencies_accepted = true;
            let mut dependency_iter = candidate_dependencies.iter();
            for _ in 0..candidate_dependencies.len() {
                if cancellation.is_cancelled() {
                    return Err(ScheduleError::Cancelled);
                }
                budget
                    .charge(WorkCounter::GraphEdge, 1)
                    .map_err(|_| ScheduleError::BudgetExhausted)?;
                let Some(dependency) = dependency_iter.next() else {
                    return Err(ScheduleError::BudgetExhausted);
                };
                rejected_dependency |= matches!(
                    dispositions.get(dependency),
                    Some(ProtocolDisposition::Invalid | ProtocolDisposition::Excluded)
                );
                all_dependencies_accepted &= accepted_base.contains(dependency)
                    || dispositions.get(dependency) == Some(&ProtocolDisposition::Accepted);
            }
            if rejected_dependency {
                updates.insert(*hash, ProtocolDisposition::Invalid);
            } else if current == Some(ProtocolDisposition::Pending) && all_dependencies_accepted {
                updates.insert(*hash, ProtocolDisposition::Accepted);
            }
        }
        if updates.is_empty() {
            break;
        }
        let update_count = updates.len();
        let mut update_iter = updates.into_iter();
        for _ in 0..update_count {
            if cancellation.is_cancelled() {
                return Err(ScheduleError::Cancelled);
            }
            budget
                .charge(WorkCounter::GraphNode, 1)
                .map_err(|_| ScheduleError::BudgetExhausted)?;
            let Some((hash, disposition)) = update_iter.next() else {
                return Err(ScheduleError::BudgetExhausted);
            };
            dispositions.insert(hash, disposition);
        }
    }
    Ok(dispositions)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{EpochCandidate, resolve_epoch};
    use crate::graph::actor_state::tests::candidate;
    use crate::{ChangeHash, NeverCancelled, ProtocolDisposition, WorkBudget};

    #[test]
    fn resolve_accepted_and_pending_changes() {
        let mut dependency = candidate(1, 1, 1, 1);
        dependency.change_hash = ChangeHash::from_bytes([1; 32]);
        let mut dependant = candidate(2, 1, 1, 1);
        dependant.change_hash = ChangeHash::from_bytes([2; 32]);
        dependant.dependencies = vec![dependency.change_hash].into();
        let mut missing = candidate(3, 1, 1, 1);
        missing.change_hash = ChangeHash::from_bytes([3; 32]);
        missing.dependencies = vec![ChangeHash::from_bytes([9; 32])].into();
        let mut invalid = candidate(4, 1, 1, 1);
        invalid.change_hash = ChangeHash::from_bytes([4; 32]);
        let mut excluded = candidate(5, 1, 1, 1);
        excluded.change_hash = ChangeHash::from_bytes([5; 32]);
        let result = resolve_epoch(
            [
                EpochCandidate {
                    candidate: dependant.clone(),
                    semantically_valid: true,
                    canonical_control: true,
                },
                EpochCandidate {
                    candidate: missing.clone(),
                    semantically_valid: true,
                    canonical_control: true,
                },
                EpochCandidate {
                    candidate: dependency.clone(),
                    semantically_valid: true,
                    canonical_control: true,
                },
                EpochCandidate {
                    candidate: invalid.clone(),
                    semantically_valid: false,
                    canonical_control: true,
                },
                EpochCandidate {
                    candidate: excluded.clone(),
                    semantically_valid: true,
                    canonical_control: false,
                },
            ],
            &BTreeSet::new(),
            &mut WorkBudget::new(0, 100),
            &NeverCancelled,
        );
        assert!(result.is_ok());
        let result = match result {
            Ok(result) => result,
            Err(_) => return,
        };
        assert_eq!(
            result[&dependency.change_hash],
            ProtocolDisposition::Accepted
        );
        assert_eq!(
            result[&dependant.change_hash],
            ProtocolDisposition::Accepted
        );
        assert_eq!(result[&missing.change_hash], ProtocolDisposition::Pending);
        assert_eq!(result[&invalid.change_hash], ProtocolDisposition::Invalid);
        assert_eq!(result[&excluded.change_hash], ProtocolDisposition::Excluded);
    }

    #[test]
    fn disposition_fixpoint_invalidates_transitive_dependants() {
        let mut invalid = candidate(1, 1, 1, 1);
        invalid.change_hash = ChangeHash::from_bytes([1; 32]);
        let mut child = candidate(2, 1, 1, 1);
        child.change_hash = ChangeHash::from_bytes([2; 32]);
        child.dependencies = vec![invalid.change_hash].into();
        let mut grandchild = candidate(3, 1, 1, 1);
        grandchild.change_hash = ChangeHash::from_bytes([3; 32]);
        grandchild.dependencies = vec![child.change_hash].into();
        let inputs = [
            EpochCandidate {
                candidate: grandchild.clone(),
                semantically_valid: true,
                canonical_control: true,
            },
            EpochCandidate {
                candidate: child.clone(),
                semantically_valid: true,
                canonical_control: true,
            },
            EpochCandidate {
                candidate: invalid.clone(),
                semantically_valid: false,
                canonical_control: true,
            },
        ];
        let result = resolve_epoch(
            inputs,
            &BTreeSet::new(),
            &mut WorkBudget::new(0, 100),
            &NeverCancelled,
        );
        assert!(result.is_ok_and(|dispositions| {
            [
                invalid.change_hash,
                child.change_hash,
                grandchild.change_hash,
            ]
            .iter()
            .all(|hash| dispositions.get(hash) == Some(&ProtocolDisposition::Invalid))
        }));
    }
}
