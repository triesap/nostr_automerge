use std::collections::BTreeSet;

use crate::EventId;
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

#[cfg(test)]
mod tests {
    use super::{select_child, select_with_alert};
    use crate::{EventId, IntegrityAlert};

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
}
