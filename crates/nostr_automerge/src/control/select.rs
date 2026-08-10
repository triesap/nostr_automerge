use std::collections::BTreeSet;

use crate::EventId;
use crate::ProtocolDisposition;
use crate::control::candidate_outcome::ControlCandidateOutcome;
use crate::{ControllerEquivocationAlert, IntegrityAlert};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ControlSelection {
    pub(crate) selected: Option<EventId>,
    pub(crate) excluded_siblings: BTreeSet<EventId>,
}

pub(crate) fn select_child(candidates: impl IntoIterator<Item = EventId>) -> ControlSelection {
    let candidates: BTreeSet<_> = candidates.into_iter().collect();
    let selected = candidates.first().copied();
    let excluded_siblings = candidates
        .iter()
        .copied()
        .filter(|candidate| Some(*candidate) != selected)
        .collect();
    ControlSelection {
        selected,
        excluded_siblings,
    }
}

pub(crate) fn select_with_alert(
    parent: Option<EventId>,
    candidates: impl IntoIterator<Item = EventId>,
) -> (ControlSelection, Option<IntegrityAlert>) {
    let selection = select_child(candidates);
    let mut all = selection
        .excluded_siblings
        .iter()
        .copied()
        .collect::<Vec<_>>();
    if let Some(selected) = selection.selected {
        all.push(selected);
    }
    all.sort_unstable();
    let alert = selection.selected.and_then(|selected| {
        ControllerEquivocationAlert::new(parent, all, selected)
            .ok()
            .map(IntegrityAlert::ControllerEquivocation)
    });
    (selection, alert)
}

pub(crate) fn select_valid_outcomes_with_alert(
    parent: Option<EventId>,
    outcomes: impl IntoIterator<Item = ControlCandidateOutcome>,
) -> (ControlSelection, Option<IntegrityAlert>) {
    select_with_alert(
        parent,
        outcomes.into_iter().filter_map(|outcome| {
            (outcome.disposition() == ProtocolDisposition::Accepted).then_some(outcome.event_id())
        }),
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{select_child, select_valid_outcomes_with_alert, select_with_alert};
    use crate::control::candidate_outcome::ControlCandidateOutcome;
    use crate::{ChangeHash, DiagnosticCode, EventId, IntegrityAlert};

    #[test]
    fn select_canonical_child_by_lowest_eventid() {
        let low = EventId::from_bytes([0x01; 32]);
        let middle = EventId::from_bytes([0x7f; 32]);
        let high = EventId::from_bytes([0xff; 32]);
        let orders = [
            vec![high, low, middle],
            vec![middle, high, low],
            vec![low, high],
        ];
        assert_eq!(select_child(orders[0].clone()).selected, Some(low));
        assert_eq!(select_child(orders[1].clone()).selected, Some(low));
        assert_eq!(select_child(orders[2].clone()).selected, Some(low));
        assert_eq!(
            select_child(orders[0].clone()).excluded_siblings,
            [middle, high].into()
        );
        assert_eq!(select_child([]).selected, None);
    }

    #[test]
    fn emit_controller_equivocation_alerts() {
        let parent = EventId::from_bytes([1; 32]);
        let low = EventId::from_bytes([2; 32]);
        let high = EventId::from_bytes([3; 32]);
        let (selection, alert) = select_with_alert(Some(parent), [high, low]);
        assert_eq!(selection.selected, Some(low));
        assert!(matches!(
            alert,
            Some(IntegrityAlert::ControllerEquivocation(details))
                if details.parent_control() == Some(parent)
                    && details.candidate_controls() == [low, high]
                    && details.selected_control() == low
        ));
        assert_eq!(select_with_alert(Some(parent), [low]).1, None);
    }

    #[test]
    fn select_only_valid_control_candidates() {
        let parent = EventId::from_bytes([9; 32]);
        let low_pending = ControlCandidateOutcome::pending(
            EventId::from_bytes([1; 32]),
            Some(parent),
            1,
            DiagnosticCode::registered("control.frontier"),
            None,
        );
        let low_invalid = ControlCandidateOutcome::invalid(
            EventId::from_bytes([2; 32]),
            Some(parent),
            1,
            DiagnosticCode::registered("control.parent"),
            None,
        );
        let first_valid = ControlCandidateOutcome::valid(
            EventId::from_bytes([3; 32]),
            Some(parent),
            1,
            BTreeSet::<ChangeHash>::new(),
        );
        let second_valid = ControlCandidateOutcome::valid(
            EventId::from_bytes([4; 32]),
            Some(parent),
            1,
            BTreeSet::new(),
        );
        for outcomes in [
            vec![
                second_valid.clone(),
                low_invalid.clone(),
                first_valid.clone(),
                low_pending.clone(),
            ],
            vec![
                low_pending.clone(),
                first_valid.clone(),
                second_valid.clone(),
            ],
            vec![low_invalid, second_valid, first_valid],
        ] {
            let (selection, _) = select_valid_outcomes_with_alert(Some(parent), outcomes);
            assert_eq!(selection.selected, Some(EventId::from_bytes([3; 32])));
            assert_eq!(
                selection.excluded_siblings,
                BTreeSet::from([EventId::from_bytes([4; 32])])
            );
        }
    }
}
