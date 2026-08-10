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
    accepted_base: BTreeSet<ChangeHash>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<BTreeMap<ChangeHash, ProtocolDisposition>, ScheduleError> {
    let mut dispositions = BTreeMap::new();
    let mut dependencies = BTreeMap::new();
    let mut eligible = Vec::new();
    for input in candidates {
        let hash = input.candidate.change_hash;
        dependencies.insert(
            hash,
            input
                .candidate
                .dependencies
                .iter()
                .copied()
                .collect::<BTreeSet<_>>(),
        );
        if !input.canonical_control {
            dispositions.insert(hash, ProtocolDisposition::Excluded);
        } else if !input.semantically_valid {
            dispositions.insert(hash, ProtocolDisposition::Invalid);
        } else {
            eligible.push(input.candidate);
        }
    }
    let schedule = schedule_candidates(eligible, accepted_base.clone(), budget, cancellation)?;
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
        let before = dispositions.clone();
        for (hash, candidate_dependencies) in &dependencies {
            if cancellation.is_cancelled() {
                return Err(ScheduleError::Cancelled);
            }
            budget
                .charge(WorkCounter::GraphNode, 1)
                .map_err(|_| ScheduleError::BudgetExhausted)?;
            let current = before.get(hash).copied();
            if !matches!(
                current,
                Some(ProtocolDisposition::Accepted | ProtocolDisposition::Pending)
            ) {
                continue;
            }
            if candidate_dependencies.iter().any(|dependency| {
                matches!(
                    before.get(dependency),
                    Some(ProtocolDisposition::Invalid | ProtocolDisposition::Excluded)
                )
            }) {
                dispositions.insert(*hash, ProtocolDisposition::Invalid);
            } else if current == Some(ProtocolDisposition::Pending)
                && candidate_dependencies.iter().all(|dependency| {
                    accepted_base.contains(dependency)
                        || before.get(dependency) == Some(&ProtocolDisposition::Accepted)
                })
            {
                dispositions.insert(*hash, ProtocolDisposition::Accepted);
            }
        }
        if dispositions == before {
            break;
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
        dependant.dependencies = vec![dependency.change_hash];
        let mut missing = candidate(3, 1, 1, 1);
        missing.change_hash = ChangeHash::from_bytes([3; 32]);
        missing.dependencies = vec![ChangeHash::from_bytes([9; 32])];
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
            BTreeSet::new(),
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
        child.dependencies = vec![invalid.change_hash];
        let mut grandchild = candidate(3, 1, 1, 1);
        grandchild.change_hash = ChangeHash::from_bytes([3; 32]);
        grandchild.dependencies = vec![child.change_hash];
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
            BTreeSet::new(),
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
