use std::collections::BTreeSet;

use crate::ChangeHash;

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

pub(crate) fn validate_epoch_ancestry(
    base_heads: &BTreeSet<ChangeHash>,
    dependency_closure: &BTreeSet<ChangeHash>,
    missing_dependencies: &BTreeSet<ChangeHash>,
) -> EpochAncestry {
    if !missing_dependencies.is_empty() {
        return EpochAncestry::from_observation(EpochAncestryObservation::Missing);
    }
    let omits_base_head = base_heads.difference(dependency_closure).next().is_some();
    EpochAncestry::from_observation(EpochAncestryObservation::Complete { omits_base_head })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{EpochAncestry, EpochAncestryObservation, validate_epoch_ancestry};
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
