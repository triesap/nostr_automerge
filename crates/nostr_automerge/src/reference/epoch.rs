use std::collections::{BTreeMap, BTreeSet};

use crate::graph::change_candidate::ChangeCandidate;
use crate::graph::schedule::{ScheduleError, schedule_candidates};
use crate::{CancellationCheck, ChangeHash, ProtocolDisposition, WorkBudget};

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
    let mut eligible = Vec::new();
    for input in candidates {
        let hash = input.candidate.change_hash;
        if !input.canonical_control {
            dispositions.insert(hash, ProtocolDisposition::Excluded);
        } else if !input.semantically_valid {
            dispositions.insert(hash, ProtocolDisposition::Invalid);
        } else {
            eligible.push(input.candidate);
        }
    }
    let schedule = schedule_candidates(eligible, accepted_base, budget, cancellation)?;
    for hash in schedule.ordered {
        dispositions.insert(hash, ProtocolDisposition::Accepted);
    }
    for hash in schedule.pending {
        dispositions.insert(hash, ProtocolDisposition::Pending);
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
            &mut WorkBudget::new(0, 20),
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
}
