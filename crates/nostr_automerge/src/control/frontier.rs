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

#[cfg(test)]
pub(crate) fn reasoned_frontier_disposition(
    frontier: impl IntoIterator<Item = ChangeHash>,
    mut knowledge: impl FnMut(&ChangeHash) -> ParentFrontierReference,
) -> Option<crate::ProtocolDisposition> {
    let mut pending = false;
    for hash in frontier {
        match knowledge(&hash).dependent_disposition() {
            Some(crate::ProtocolDisposition::Invalid) => {
                return Some(crate::ProtocolDisposition::Invalid);
            }
            Some(crate::ProtocolDisposition::Pending) => pending = true,
            Some(
                crate::ProtocolDisposition::Accepted
                | crate::ProtocolDisposition::Excluded
                | crate::ProtocolDisposition::UnsupportedRevision,
            )
            | None => {}
        }
    }
    pending.then_some(crate::ProtocolDisposition::Pending)
}

pub(crate) fn reasoned_frontier_disposition_metered<E>(
    frontier: &[ChangeHash],
    mut knowledge: impl FnMut(
        &ChangeHash,
        &mut dyn FnMut(crate::WorkCounter) -> Result<(), E>,
    ) -> Result<ParentFrontierReference, E>,
    visit: &mut impl FnMut(crate::WorkCounter) -> Result<(), E>,
) -> Result<Option<crate::ProtocolDisposition>, E> {
    let mut pending = false;
    let mut index = 0;
    while index < frontier.len() {
        visit(crate::WorkCounter::GraphNode)?;
        let hash = frontier[index];
        index += 1;
        match knowledge(&hash, visit)?.dependent_disposition() {
            Some(crate::ProtocolDisposition::Invalid) => {
                return Ok(Some(crate::ProtocolDisposition::Invalid));
            }
            Some(crate::ProtocolDisposition::Pending) => pending = true,
            Some(
                crate::ProtocolDisposition::Accepted
                | crate::ProtocolDisposition::Excluded
                | crate::ProtocolDisposition::UnsupportedRevision,
            )
            | None => {}
        }
    }
    Ok(pending.then_some(crate::ProtocolDisposition::Pending))
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct FrontierClosure {
    pub(crate) accepted: BTreeSet<ChangeHash>,
    pub(crate) missing: BTreeSet<ChangeHash>,
    pub(crate) out_of_parent: BTreeSet<ChangeHash>,
}

/// Resolve a frontier iteratively against accepted parent history.
#[cfg(test)]
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

pub(crate) fn accepted_frontier_closure_metered<E>(
    frontier: &[ChangeHash],
    accepted_parent: &BTreeSet<ChangeHash>,
    known_dependencies: &BTreeMap<ChangeHash, BTreeSet<ChangeHash>>,
    visit: impl FnMut(crate::WorkCounter) -> Result<(), E>,
) -> Result<FrontierClosure, E> {
    accepted_frontier_closure_metered_with(
        frontier,
        accepted_parent,
        known_dependencies,
        visit,
        |_| false,
    )
    .map(|(closure, _)| closure)
}

pub(crate) fn accepted_frontier_closure_antichain_metered<E>(
    head: &[ChangeHash],
    full_frontier: &[ChangeHash],
    accepted_parent: &BTreeSet<ChangeHash>,
    known_dependencies: &BTreeMap<ChangeHash, BTreeSet<ChangeHash>>,
    visit: impl FnMut(crate::WorkCounter) -> Result<(), E>,
) -> Result<(FrontierClosure, bool), E> {
    accepted_frontier_closure_metered_with(
        head,
        accepted_parent,
        known_dependencies,
        visit,
        |ancestor| ancestor != head[0] && full_frontier.contains(&ancestor),
    )
}

fn accepted_frontier_closure_metered_with<E>(
    frontier: &[ChangeHash],
    accepted_parent: &BTreeSet<ChangeHash>,
    known_dependencies: &BTreeMap<ChangeHash, BTreeSet<ChangeHash>>,
    mut visit: impl FnMut(crate::WorkCounter) -> Result<(), E>,
    mut accepted_observer: impl FnMut(ChangeHash) -> bool,
) -> Result<(FrontierClosure, bool), E> {
    visit(crate::WorkCounter::GraphNode)?;
    let mut result = FrontierClosure::default();
    let mut observed = false;
    let mut visited = BTreeSet::new();
    let mut stack = Vec::new();
    let mut frontier_index = 0;
    while frontier_index < frontier.len() {
        visit(crate::WorkCounter::GraphNode)?;
        stack.push(frontier[frontier_index]);
        frontier_index += 1;
    }
    while !stack.is_empty() {
        visit(crate::WorkCounter::GraphNode)?;
        let Some(hash) = stack.pop() else {
            break;
        };
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
        observed |= accepted_observer(hash);
        result.accepted.insert(hash);
        let mut dependencies = dependencies.iter();
        let dependency_count = dependencies.len();
        for _ in 0..dependency_count {
            visit(crate::WorkCounter::GraphEdge)?;
            let Some(dependency) = dependencies.next() else {
                break;
            };
            stack.push(*dependency);
        }
    }
    Ok((result, observed))
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{
        FrontierClosure, ParentFrontierReference, accepted_frontier_closure,
        accepted_frontier_closure_metered, reasoned_frontier_disposition,
        reasoned_frontier_disposition_metered,
    };
    use crate::{ChangeHash, Completion, ProtocolDisposition, WorkCounter};

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
    fn metered_closure_charges_before_every_node_and_edge_operation() {
        let dependencies = BTreeMap::from([
            (hash(1), BTreeSet::new()),
            (hash(2), BTreeSet::from([hash(1)])),
        ]);
        let accepted = dependencies.keys().copied().collect::<BTreeSet<_>>();
        let expected = vec![
            WorkCounter::GraphNode,
            WorkCounter::GraphNode,
            WorkCounter::GraphNode,
            WorkCounter::GraphEdge,
            WorkCounter::GraphNode,
        ];
        let mut observed = Vec::new();
        assert_eq!(
            accepted_frontier_closure_metered(&[hash(2)], &accepted, &dependencies, |counter| {
                observed.push(counter);
                Ok::<_, Completion>(())
            }),
            Ok(accepted_frontier_closure(
                [hash(2)],
                &accepted,
                &dependencies
            ))
        );
        assert_eq!(observed, expected);

        for boundary in 0..expected.len() {
            let mut observed = Vec::new();
            assert_eq!(
                accepted_frontier_closure_metered(
                    &[hash(2)],
                    &accepted,
                    &dependencies,
                    |counter| {
                        if observed.len() == boundary {
                            return Err(Completion::Cancelled);
                        }
                        observed.push(counter);
                        Ok(())
                    }
                ),
                Err(Completion::Cancelled)
            );
            assert_eq!(observed, expected[..boundary]);
        }
    }

    #[test]
    fn metered_frontier_knowledge_stops_before_each_lookup() {
        let frontier = [hash(1), hash(2), hash(3)];
        let mut looked_up = Vec::new();
        let mut charged = Vec::new();
        let mut visit = |counter| {
            charged.push(counter);
            Ok::<_, Completion>(())
        };
        assert_eq!(
            reasoned_frontier_disposition_metered(
                &frontier,
                |hash, _| {
                    looked_up.push(*hash);
                    Ok(ParentFrontierReference::AcceptedUnderParent)
                },
                &mut visit,
            ),
            Ok(None)
        );
        assert_eq!(looked_up, frontier);
        assert_eq!(charged, vec![WorkCounter::GraphNode; frontier.len()]);

        for boundary in 0..frontier.len() {
            let mut looked_up = Vec::new();
            let mut charged = 0;
            let mut visit = |_| {
                if charged == boundary {
                    return Err(Completion::BudgetExhausted);
                }
                charged += 1;
                Ok(())
            };
            assert_eq!(
                reasoned_frontier_disposition_metered(
                    &frontier,
                    |hash, _| {
                        looked_up.push(*hash);
                        Ok(ParentFrontierReference::AcceptedUnderParent)
                    },
                    &mut visit,
                ),
                Err(Completion::BudgetExhausted)
            );
            assert_eq!(looked_up, frontier[..boundary]);
        }
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

    #[test]
    fn parent_accepted_head_continues_closure_validation() {
        assert_eq!(
            ParentFrontierReference::AcceptedUnderParent.dependent_disposition(),
            None
        );
    }

    #[test]
    fn genuinely_missing_head_remains_pending() {
        assert_eq!(
            ParentFrontierReference::Missing.dependent_disposition(),
            Some(ProtocolDisposition::Pending)
        );
    }

    #[test]
    fn statefully_pending_head_remains_pending() {
        let pending = hash(21);
        assert_eq!(
            reasoned_frontier_disposition([pending], |_| {
                ParentFrontierReference::PendingUnderParent
            }),
            Some(ProtocolDisposition::Pending)
        );
    }

    #[test]
    fn invalid_head_rejects_the_frontier() {
        assert_eq!(
            reasoned_frontier_disposition([hash(22)], |_| {
                ParentFrontierReference::InvalidUnderParent
            }),
            Some(ProtocolDisposition::Invalid)
        );
    }

    #[test]
    fn excluded_head_rejects_the_frontier() {
        assert_eq!(
            reasoned_frontier_disposition([hash(23)], |_| {
                ParentFrontierReference::ExcludedUnderParent
            }),
            Some(ProtocolDisposition::Invalid)
        );
    }

    #[test]
    fn unsupported_head_rejects_the_frontier() {
        assert_eq!(
            reasoned_frontier_disposition([hash(24)], |_| { ParentFrontierReference::Unsupported }),
            Some(ProtocolDisposition::Invalid)
        );
    }

    #[test]
    fn other_control_head_rejects_the_frontier() {
        assert_eq!(
            reasoned_frontier_disposition([hash(25)], |_| {
                ParentFrontierReference::OtherControl
            }),
            Some(ProtocolDisposition::Invalid)
        );
    }
}
