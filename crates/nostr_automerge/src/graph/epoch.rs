use std::collections::BTreeSet;

use crate::ChangeHash;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum EpochAncestry {
    Valid,
    PendingMissing(Vec<ChangeHash>),
    InvalidOmission(Vec<ChangeHash>),
}

pub(crate) fn validate_epoch_ancestry(
    base_heads: &BTreeSet<ChangeHash>,
    dependency_closure: &BTreeSet<ChangeHash>,
    missing_dependencies: &BTreeSet<ChangeHash>,
) -> EpochAncestry {
    if !missing_dependencies.is_empty() {
        return EpochAncestry::PendingMissing(missing_dependencies.iter().copied().collect());
    }
    let omitted = base_heads
        .difference(dependency_closure)
        .copied()
        .collect::<Vec<_>>();
    if omitted.is_empty() {
        EpochAncestry::Valid
    } else {
        EpochAncestry::InvalidOmission(omitted)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{EpochAncestry, validate_epoch_ancestry};
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
            EpochAncestry::InvalidOmission(vec![hash(2)])
        );
        assert_eq!(
            validate_epoch_ancestry(&base, &BTreeSet::new(), &BTreeSet::new()),
            EpochAncestry::InvalidOmission(vec![hash(1), hash(2)])
        );
        assert_eq!(
            validate_epoch_ancestry(&base, &BTreeSet::new(), &BTreeSet::from([hash(9)])),
            EpochAncestry::PendingMissing(vec![hash(9)])
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
            EpochAncestry::InvalidOmission(vec![hash(64)])
        );
        assert_eq!(
            validate_epoch_ancestry(&base, &closure, &BTreeSet::from([hash(65)])),
            EpochAncestry::PendingMissing(vec![hash(65)])
        );

        let source = include_str!("epoch.rs");
        assert!(
            !source.contains("PendingMissing(Vec<ChangeHash>)")
                && !source.contains("InvalidOmission(Vec<ChangeHash>)")
                && !source.contains(".collect::<Vec<_>>()"),
            "unmetered epoch ancestry materialization remains"
        );
    }
}
