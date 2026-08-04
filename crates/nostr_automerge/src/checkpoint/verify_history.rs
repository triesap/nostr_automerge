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
    use super::*;
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
}
