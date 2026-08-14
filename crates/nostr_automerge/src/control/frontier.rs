use std::collections::{BTreeMap, BTreeSet};

use crate::ChangeHash;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ParentFrontierReference {
    AcceptedUnderParent,
    PendingUnderParent,
    Missing,
    InvalidUnderParent,
    ExcludedUnderParent,
    Unsupported,
    OtherControl,
    Unknown,
}

impl ParentFrontierReference {
    pub(crate) const fn dependent_disposition(self) -> Option<crate::ProtocolDisposition> {
        match self {
            Self::AcceptedUnderParent => None,
            Self::PendingUnderParent | Self::Missing | Self::Unknown => {
                Some(crate::ProtocolDisposition::Pending)
            }
            Self::InvalidUnderParent
            | Self::ExcludedUnderParent
            | Self::Unsupported
            | Self::OtherControl => Some(crate::ProtocolDisposition::Invalid),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct FrontierClosure {
    pub(crate) accepted: BTreeSet<ChangeHash>,
    pub(crate) missing: BTreeSet<ChangeHash>,
    pub(crate) out_of_parent: BTreeSet<ChangeHash>,
}

/// Resolve a frontier iteratively against accepted parent history.
pub(crate) fn accepted_frontier_closure(
    frontier: impl IntoIterator<Item = ChangeHash>,
    accepted_parent: &BTreeSet<ChangeHash>,
    known_dependencies: &BTreeMap<ChangeHash, BTreeSet<ChangeHash>>,
) -> FrontierClosure {
    let mut result = FrontierClosure::default();
    let mut visited = BTreeSet::new();
    let mut stack = frontier.into_iter().collect::<Vec<_>>();
    while let Some(hash) = stack.pop() {
        if !visited.insert(hash) {
            continue;
        }
        let Some(dependencies) = known_dependencies.get(&hash) else {
            result.missing.insert(hash);
            continue;
        };
        if !accepted_parent.contains(&hash) {
            result.out_of_parent.insert(hash);
            continue;
        }
        result.accepted.insert(hash);
        stack.extend(dependencies.iter().copied());
    }
    result
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{FrontierClosure, ParentFrontierReference, accepted_frontier_closure};
    use crate::{ChangeHash, ProtocolDisposition};

    fn hash(value: u8) -> ChangeHash {
        ChangeHash::from_bytes([value; 32])
    }

    #[test]
    fn resolves_chain_branch_and_fan_in_closures() {
        let dependencies = BTreeMap::from([
            (hash(1), BTreeSet::new()),
            (hash(2), BTreeSet::from([hash(1)])),
            (hash(3), BTreeSet::from([hash(1)])),
            (hash(4), BTreeSet::from([hash(2), hash(3)])),
        ]);
        let accepted = dependencies.keys().copied().collect::<BTreeSet<_>>();
        assert_eq!(
            accepted_frontier_closure([hash(2)], &accepted, &dependencies).accepted,
            BTreeSet::from([hash(1), hash(2)])
        );
        assert_eq!(
            accepted_frontier_closure([hash(2), hash(3)], &accepted, &dependencies).accepted,
            BTreeSet::from([hash(1), hash(2), hash(3)])
        );
        assert_eq!(
            accepted_frontier_closure([hash(4)], &accepted, &dependencies).accepted,
            accepted
        );
    }

    #[test]
    fn separates_missing_evidence_from_out_of_parent_references() {
        let dependencies = BTreeMap::from([
            (hash(1), BTreeSet::new()),
            (hash(2), BTreeSet::from([hash(1)])),
            (hash(8), BTreeSet::new()),
        ]);
        let result = accepted_frontier_closure(
            [hash(2), hash(8), hash(9)],
            &BTreeSet::from([hash(1), hash(2)]),
            &dependencies,
        );
        assert_eq!(
            result,
            FrontierClosure {
                accepted: BTreeSet::from([hash(1), hash(2)]),
                missing: BTreeSet::from([hash(9)]),
                out_of_parent: BTreeSet::from([hash(8)]),
            }
        );
    }

    #[test]
    fn base_head_knowledge_has_exhaustive_dependent_outcomes() {
        let cases = [
            (ParentFrontierReference::AcceptedUnderParent, None),
            (
                ParentFrontierReference::PendingUnderParent,
                Some(ProtocolDisposition::Pending),
            ),
            (
                ParentFrontierReference::Missing,
                Some(ProtocolDisposition::Pending),
            ),
            (
                ParentFrontierReference::InvalidUnderParent,
                Some(ProtocolDisposition::Invalid),
            ),
            (
                ParentFrontierReference::ExcludedUnderParent,
                Some(ProtocolDisposition::Invalid),
            ),
            (
                ParentFrontierReference::Unsupported,
                Some(ProtocolDisposition::Invalid),
            ),
            (
                ParentFrontierReference::OtherControl,
                Some(ProtocolDisposition::Invalid),
            ),
            (
                ParentFrontierReference::Unknown,
                Some(ProtocolDisposition::Pending),
            ),
        ];
        for (state, expected) in cases {
            assert_eq!(state.dependent_disposition(), expected);
        }
    }
}
