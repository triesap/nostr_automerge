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
}
