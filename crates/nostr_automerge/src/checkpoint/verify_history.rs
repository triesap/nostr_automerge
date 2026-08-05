use super::VerifiedSnapshot;
use crate::ChangeHash;
use std::collections::BTreeSet;

/// Why a snapshot was not backed by fully verified historical carriers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoryVerificationError {
    /// Embedded change lacked a valid carrier.
    MissingCarrier,
    /// Embedded change was not accepted by the descriptor control.
    NotAccepted,
    /// Snapshot enumeration failed.
    Snapshot,
    /// Descriptor referenced a control outside the canonical chain.
    UnknownControl,
    /// Caller-selected checkpoint work budget was exhausted.
    Budget,
}

/// Derives qualifying validated carrier coverage through one canonical control.
pub(crate) fn historical_carrier_coverage(
    corpus: &crate::EvidenceCorpus,
    canonical_controls: &[crate::EventId],
    through: crate::EventId,
) -> Result<BTreeSet<ChangeHash>, HistoryVerificationError> {
    let end = canonical_controls
        .iter()
        .position(|control| *control == through)
        .ok_or(HistoryVerificationError::UnknownControl)?;
    Ok(canonical_controls[..=end]
        .iter()
        .filter_map(|control| corpus.indexes.changes.hashes_by_control.get(control))
        .flat_map(BTreeSet::iter)
        .copied()
        .collect())
}

/// Requires every embedded identity to have a valid carrier and accepted status no later than the descriptor control.
pub fn verify_full_history(
    snapshot: &VerifiedSnapshot,
    valid_carriers: &BTreeSet<ChangeHash>,
    accepted_at_control: &BTreeSet<ChangeHash>,
) -> Result<(), HistoryVerificationError> {
    let changes = snapshot
        .loaded
        .document
        .embedded_changes()
        .map_err(|_| HistoryVerificationError::Snapshot)?;
    verify_sets(
        &changes.iter().map(|c| c.hash).collect(),
        valid_carriers,
        accepted_at_control,
    )
}

pub(crate) fn verify_full_history_metered(
    snapshot: &VerifiedSnapshot,
    valid_carriers: &BTreeSet<ChangeHash>,
    accepted_at_control: &BTreeSet<ChangeHash>,
    budget: &mut crate::WorkBudget,
) -> Result<(), HistoryVerificationError> {
    let changes = snapshot
        .loaded
        .document
        .embedded_changes()
        .map_err(|_| HistoryVerificationError::Snapshot)?;
    budget
        .charge_checkpoint_items(u64::try_from(changes.len()).unwrap_or(u64::MAX))
        .map_err(|_| HistoryVerificationError::Budget)?;
    verify_sets(
        &changes.iter().map(|change| change.hash).collect(),
        valid_carriers,
        accepted_at_control,
    )
}
fn verify_sets(
    embedded: &BTreeSet<ChangeHash>,
    carriers: &BTreeSet<ChangeHash>,
    accepted: &BTreeSet<ChangeHash>,
) -> Result<(), HistoryVerificationError> {
    if !embedded.is_subset(carriers) {
        return Err(HistoryVerificationError::MissingCarrier);
    }
    if !embedded.is_subset(accepted) {
        return Err(HistoryVerificationError::NotAccepted);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::EventId;
    use crate::evidence::corpus_builder::EvidenceCorpus;
    use crate::evidence::indexes::TrustedIndexes;
    #[test]
    fn verify_full_historical_carrier_authorization() {
        let a = ChangeHash::from_bytes([1; 32]);
        let b = ChangeHash::from_bytes([2; 32]);
        let embedded = BTreeSet::from([a, b]);
        assert_eq!(
            verify_sets(&embedded, &BTreeSet::from([a, b]), &BTreeSet::from([a, b])),
            Ok(())
        );
        assert_eq!(
            verify_sets(&embedded, &BTreeSet::from([a]), &embedded),
            Err(HistoryVerificationError::MissingCarrier)
        );
        assert_eq!(
            verify_sets(&embedded, &embedded, &BTreeSet::from([a])),
            Err(HistoryVerificationError::NotAccepted)
        );
    }

    #[test]
    fn historical_coverage_stops_at_the_referenced_canonical_control() {
        let first = EventId::from_bytes([1; 32]);
        let second = EventId::from_bytes([2; 32]);
        let third = EventId::from_bytes([3; 32]);
        let a = ChangeHash::from_bytes([4; 32]);
        let b = ChangeHash::from_bytes([5; 32]);
        let c = ChangeHash::from_bytes([6; 32]);
        let mut indexes = TrustedIndexes::default();
        indexes.changes.hashes_by_control = BTreeMap::from([
            (first, BTreeSet::from([a])),
            (second, BTreeSet::from([b])),
            (third, BTreeSet::from([c])),
        ]);
        let corpus = EvidenceCorpus {
            events: BTreeMap::new(),
            invalid: BTreeMap::new(),
            duplicates: Vec::new(),
            indexes,
        };
        assert_eq!(
            historical_carrier_coverage(&corpus, &[first, second, third], second),
            Ok(BTreeSet::from([a, b]))
        );
        assert_eq!(
            historical_carrier_coverage(
                &corpus,
                &[first, second, third],
                EventId::from_bytes([9; 32])
            ),
            Err(HistoryVerificationError::UnknownControl)
        );
    }
}
