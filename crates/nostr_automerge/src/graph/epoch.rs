use std::collections::BTreeSet;
use std::convert::Infallible;

use crate::{ChangeHash, WorkCounter};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EpochAncestry {
    Valid,
    PendingMissing,
    InvalidOmission,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EpochAncestryObservation {
    Missing,
    Complete { omits_base_head: bool },
}

impl EpochAncestry {
    const fn from_observation(observation: EpochAncestryObservation) -> Self {
        match observation {
            EpochAncestryObservation::Missing => Self::PendingMissing,
            EpochAncestryObservation::Complete {
                omits_base_head: false,
            } => Self::Valid,
            EpochAncestryObservation::Complete {
                omits_base_head: true,
            } => Self::InvalidOmission,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EpochAncestryOperation {
    MissingDependencyPull,
    BaseHeadPull,
    AcceptedClosureLookup,
    InclusionComparison,
    StateTransition,
}

pub(crate) fn classify_epoch_ancestry_metered<E>(
    base_heads: &BTreeSet<ChangeHash>,
    dependency_closure: &BTreeSet<ChangeHash>,
    missing_dependencies: &BTreeSet<ChangeHash>,
    charge: impl FnMut(WorkCounter) -> Result<(), E>,
) -> Result<EpochAncestry, E> {
    classify_epoch_ancestry_metered_observed(
        base_heads,
        dependency_closure,
        missing_dependencies,
        charge,
        |_| {},
    )
}

fn classify_epoch_ancestry_metered_observed<E>(
    base_heads: &BTreeSet<ChangeHash>,
    dependency_closure: &BTreeSet<ChangeHash>,
    missing_dependencies: &BTreeSet<ChangeHash>,
    mut charge: impl FnMut(WorkCounter) -> Result<(), E>,
    mut observed: impl FnMut(EpochAncestryOperation),
) -> Result<EpochAncestry, E> {
    let mut missing = missing_dependencies.iter();
    let first_missing = ancestry_operation(
        &mut charge,
        &mut observed,
        WorkCounter::GraphEdge,
        EpochAncestryOperation::MissingDependencyPull,
        || missing.next().copied(),
    )?;
    if first_missing.is_some() {
        return ancestry_operation(
            &mut charge,
            &mut observed,
            WorkCounter::GraphNode,
            EpochAncestryOperation::StateTransition,
            || EpochAncestry::from_observation(EpochAncestryObservation::Missing),
        );
    }

    let mut base = base_heads.iter();
    loop {
        let next = ancestry_operation(
            &mut charge,
            &mut observed,
            WorkCounter::GraphNode,
            EpochAncestryOperation::BaseHeadPull,
            || base.next().copied(),
        )?;
        let Some(head) = next else {
            return ancestry_operation(
                &mut charge,
                &mut observed,
                WorkCounter::GraphNode,
                EpochAncestryOperation::StateTransition,
                || {
                    EpochAncestry::from_observation(EpochAncestryObservation::Complete {
                        omits_base_head: false,
                    })
                },
            );
        };
        let accepted = ancestry_operation(
            &mut charge,
            &mut observed,
            WorkCounter::GraphNode,
            EpochAncestryOperation::AcceptedClosureLookup,
            || dependency_closure.contains(&head),
        )?;
        let omits_base_head = ancestry_operation(
            &mut charge,
            &mut observed,
            WorkCounter::GraphEdge,
            EpochAncestryOperation::InclusionComparison,
            || !accepted,
        )?;
        if omits_base_head {
            return ancestry_operation(
                &mut charge,
                &mut observed,
                WorkCounter::GraphNode,
                EpochAncestryOperation::StateTransition,
                || {
                    EpochAncestry::from_observation(EpochAncestryObservation::Complete {
                        omits_base_head: true,
                    })
                },
            );
        }
    }
}

fn ancestry_operation<E, T>(
    charge: &mut impl FnMut(WorkCounter) -> Result<(), E>,
    observed: &mut impl FnMut(EpochAncestryOperation),
    counter: WorkCounter,
    operation: EpochAncestryOperation,
    target: impl FnOnce() -> T,
) -> Result<T, E> {
    charge(counter)?;
    let result = target();
    observed(operation);
    Ok(result)
}

pub(crate) fn validate_epoch_ancestry(
    base_heads: &BTreeSet<ChangeHash>,
    dependency_closure: &BTreeSet<ChangeHash>,
    missing_dependencies: &BTreeSet<ChangeHash>,
) -> EpochAncestry {
    match classify_epoch_ancestry_metered(
        base_heads,
        dependency_closure,
        missing_dependencies,
        |_| Ok::<(), Infallible>(()),
    ) {
        Ok(outcome) => outcome,
        Err(never) => match never {},
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{
        EpochAncestry, EpochAncestryObservation, EpochAncestryOperation,
        classify_epoch_ancestry_metered, classify_epoch_ancestry_metered_observed,
        validate_epoch_ancestry,
    };
    use crate::ChangeHash;

    fn hash(byte: u8) -> ChangeHash {
        ChangeHash::from_bytes([byte; 32])
    }

    #[test]
    fn enforce_epoch_base_ancestry() {
        let base = BTreeSet::from([hash(1), hash(2)]);
        assert_eq!(
            validate_epoch_ancestry(
                &base,
                &BTreeSet::from([hash(1), hash(2), hash(3)]),
                &BTreeSet::new()
            ),
            EpochAncestry::Valid
        );
        assert_eq!(
            validate_epoch_ancestry(&base, &BTreeSet::from([hash(1)]), &BTreeSet::new()),
            EpochAncestry::InvalidOmission
        );
        assert_eq!(
            validate_epoch_ancestry(&base, &BTreeSet::new(), &BTreeSet::new()),
            EpochAncestry::InvalidOmission
        );
        assert_eq!(
            validate_epoch_ancestry(&base, &BTreeSet::new(), &BTreeSet::from([hash(9)])),
            EpochAncestry::PendingMissing
        );
    }

    #[test]
    fn compact_epoch_ancestry_outcomes_are_closed_and_unambiguous() {
        assert_eq!(std::mem::size_of::<EpochAncestry>(), 1);
        assert_eq!(
            EpochAncestry::from_observation(EpochAncestryObservation::Missing),
            EpochAncestry::PendingMissing
        );
        assert_eq!(
            EpochAncestry::from_observation(EpochAncestryObservation::Complete {
                omits_base_head: false,
            }),
            EpochAncestry::Valid
        );
        assert_eq!(
            EpochAncestry::from_observation(EpochAncestryObservation::Complete {
                omits_base_head: true,
            }),
            EpochAncestry::InvalidOmission
        );
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum TestStop {
        BudgetExhausted,
        Cancelled,
    }

    #[test]
    fn ancestry_classification_is_nonallocating_streaming_and_exactly_metered() {
        let cases = [
            (
                BTreeSet::new(),
                BTreeSet::new(),
                BTreeSet::new(),
                EpochAncestry::Valid,
            ),
            (
                BTreeSet::from([hash(1), hash(2)]),
                BTreeSet::from([hash(1), hash(2), hash(9)]),
                BTreeSet::new(),
                EpochAncestry::Valid,
            ),
            (
                BTreeSet::from([hash(1), hash(2)]),
                BTreeSet::from([hash(1)]),
                BTreeSet::new(),
                EpochAncestry::InvalidOmission,
            ),
            (
                BTreeSet::from([hash(1), hash(2)]),
                BTreeSet::new(),
                BTreeSet::from([hash(9), hash(10)]),
                EpochAncestry::PendingMissing,
            ),
            (
                (1..=64).map(hash).collect(),
                (1..=65).map(hash).collect(),
                BTreeSet::new(),
                EpochAncestry::Valid,
            ),
        ];

        for (base, closure, missing, expected) in cases {
            let mut expected_trace = Vec::new();
            assert_eq!(
                classify_epoch_ancestry_metered_observed(
                    &base,
                    &closure,
                    &missing,
                    |_| Ok::<(), TestStop>(()),
                    |operation| expected_trace.push(operation),
                ),
                Ok(expected)
            );
            let exact = expected_trace.len();
            for stop in [TestStop::BudgetExhausted, TestStop::Cancelled] {
                for limit in 0..exact {
                    let mut calls = 0;
                    let mut observed = Vec::new();
                    let result = classify_epoch_ancestry_metered_observed(
                        &base,
                        &closure,
                        &missing,
                        |_| {
                            if calls == limit {
                                return Err(stop);
                            }
                            calls += 1;
                            Ok(())
                        },
                        |operation| observed.push(operation),
                    );
                    assert_eq!(result, Err(stop));
                    assert_eq!(calls, limit);
                    assert_eq!(observed, expected_trace[..limit]);
                }
            }
            let mut exact_calls = 0;
            assert_eq!(
                classify_epoch_ancestry_metered(&base, &closure, &missing, |_| {
                    if exact_calls == exact {
                        return Err(TestStop::BudgetExhausted);
                    }
                    exact_calls += 1;
                    Ok(())
                }),
                Ok(expected)
            );
            assert_eq!(exact_calls, exact);
        }

        let mut trace = Vec::new();
        let outcome = classify_epoch_ancestry_metered_observed(
            &BTreeSet::from([hash(1), hash(2)]),
            &BTreeSet::from([hash(1), hash(2), hash(9)]),
            &BTreeSet::new(),
            |_| Ok::<(), TestStop>(()),
            |operation| trace.push(operation),
        );
        assert_eq!(outcome, Ok(EpochAncestry::Valid));
        assert_eq!(
            trace,
            vec![
                EpochAncestryOperation::MissingDependencyPull,
                EpochAncestryOperation::BaseHeadPull,
                EpochAncestryOperation::AcceptedClosureLookup,
                EpochAncestryOperation::InclusionComparison,
                EpochAncestryOperation::BaseHeadPull,
                EpochAncestryOperation::AcceptedClosureLookup,
                EpochAncestryOperation::InclusionComparison,
                EpochAncestryOperation::BaseHeadPull,
                EpochAncestryOperation::StateTransition,
            ]
        );
    }

    #[test]
    #[ignore = "remediation v12 expected failure: unmetered ancestry materialization"]
    fn finding_100_epoch_ancestry_work_reproduction() {
        let base = (1..=64).map(hash).collect::<BTreeSet<_>>();
        assert_eq!(
            validate_epoch_ancestry(&base, &base, &BTreeSet::new()),
            EpochAncestry::Valid
        );

        let closure = (1..64).map(hash).collect::<BTreeSet<_>>();
        assert_eq!(
            validate_epoch_ancestry(&base, &closure, &BTreeSet::new()),
            EpochAncestry::InvalidOmission
        );
        assert_eq!(
            validate_epoch_ancestry(&base, &closure, &BTreeSet::from([hash(65)])),
            EpochAncestry::PendingMissing
        );

        let source = include_str!("epoch.rs");
        assert!(
            !source.contains("pub(crate) fn validate_epoch_ancestry("),
            "unmetered epoch ancestry materialization remains"
        );
    }
}
