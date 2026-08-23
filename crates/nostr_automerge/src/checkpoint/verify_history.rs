use super::VerifiedSnapshot;
use crate::{ChangeHash, EventId};
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
    /// Caller requested cooperative cancellation.
    Cancelled,
}

/// Derives qualifying validated carrier coverage through one canonical control.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HistoricalCarrierCoverage {
    pub(crate) change_hashes: BTreeSet<ChangeHash>,
    pub(crate) carrier_event_ids: BTreeSet<EventId>,
}

pub(crate) fn historical_carrier_coverage(
    view: &crate::evidence::document_view::DocumentEvidenceView<'_>,
    canonical_controls: &[crate::EventId],
    through: crate::EventId,
    budget: &mut crate::WorkBudget,
    cancellation: &impl crate::CancellationCheck,
    mut qualifies: impl FnMut(EventId, ChangeHash, EventId) -> bool,
) -> Result<HistoricalCarrierCoverage, HistoryVerificationError> {
    let mut coverage = HistoricalCarrierCoverage::default();
    let mut found = false;
    for control in canonical_controls {
        charge_history_item(budget, cancellation)?;
        if let Some(hashes) = view.change_hashes_for_control(*control) {
            for hash in hashes {
                charge_history_item(budget, cancellation)?;
                let mut has_qualifying_carrier = false;
                for event_id in view.change_carrier_event_ids(*hash).into_iter().flatten() {
                    charge_history_item(budget, cancellation)?;
                    if qualifies(*event_id, *hash, *control) {
                        coverage.carrier_event_ids.insert(*event_id);
                        has_qualifying_carrier = true;
                    }
                }
                if has_qualifying_carrier {
                    coverage.change_hashes.insert(*hash);
                }
            }
        }
        if *control == through {
            found = true;
            break;
        }
    }
    if found {
        Ok(coverage)
    } else {
        Err(HistoryVerificationError::UnknownControl)
    }
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
    cancellation: &impl crate::CancellationCheck,
) -> Result<(), HistoryVerificationError> {
    charge_history_item(budget, cancellation)?;
    let changes = snapshot
        .loaded
        .document
        .embedded_changes()
        .map_err(|_| HistoryVerificationError::Snapshot)?;
    let mut embedded = BTreeSet::new();
    for change in changes {
        charge_history_item(budget, cancellation)?;
        embedded.insert(change.hash);
    }
    for hash in &embedded {
        charge_history_item(budget, cancellation)?;
        if !valid_carriers.contains(hash) {
            return Err(HistoryVerificationError::MissingCarrier);
        }
        charge_history_item(budget, cancellation)?;
        if !accepted_at_control.contains(hash) {
            return Err(HistoryVerificationError::NotAccepted);
        }
    }
    Ok(())
}

fn charge_history_item(
    budget: &mut crate::WorkBudget,
    cancellation: &impl crate::CancellationCheck,
) -> Result<(), HistoryVerificationError> {
    if cancellation.is_cancelled() {
        return Err(HistoryVerificationError::Cancelled);
    }
    budget
        .charge_checkpoint_items(1)
        .map_err(|_| HistoryVerificationError::Budget)
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
        let a_event = EventId::from_bytes([10; 32]);
        let b_event = EventId::from_bytes([11; 32]);
        let c_event = EventId::from_bytes([12; 32]);
        let mut indexes = TrustedIndexes::default();
        let coordinate = crate::DocumentCoordinate::new(
            crate::ControllerPublicKey::from_bytes([7; 32]),
            crate::DocumentId::from_bytes([8; 32]),
        );
        indexes.changes.hashes_by_coordinate_control = BTreeMap::from([
            ((coordinate, first), BTreeSet::from([a])),
            ((coordinate, second), BTreeSet::from([b])),
            ((coordinate, third), BTreeSet::from([c])),
        ]);
        indexes.changes.carriers_by_coordinate_hash = BTreeMap::from([
            ((coordinate, a), BTreeSet::from([a_event])),
            ((coordinate, b), BTreeSet::from([b_event])),
            ((coordinate, c), BTreeSet::from([c_event])),
        ]);
        let corpus = EvidenceCorpus {
            events: BTreeMap::new(),
            invalid: BTreeMap::new(),
            duplicates: Vec::new(),
            indexes,
        };
        let view =
            crate::evidence::document_view::DocumentEvidenceView::derive(&corpus, coordinate);
        assert_eq!(
            historical_carrier_coverage(
                &view,
                &[first, second, third],
                second,
                &mut crate::WorkBudget::new(0, 10),
                &crate::NeverCancelled,
                |_, _, _| true,
            ),
            Ok(HistoricalCarrierCoverage {
                change_hashes: BTreeSet::from([a, b]),
                carrier_event_ids: BTreeSet::from([a_event, b_event]),
            })
        );
        assert_eq!(
            historical_carrier_coverage(
                &view,
                &[first, second, third],
                EventId::from_bytes([9; 32]),
                &mut crate::WorkBudget::new(0, 10),
                &crate::NeverCancelled,
                |_, _, _| true,
            ),
            Err(HistoryVerificationError::UnknownControl)
        );
        assert_eq!(
            historical_carrier_coverage(
                &view,
                &[first, second, third],
                second,
                &mut crate::WorkBudget::new(0, 0),
                &crate::NeverCancelled,
                |_, _, _| true,
            ),
            Err(HistoryVerificationError::Budget)
        );
    }
}
