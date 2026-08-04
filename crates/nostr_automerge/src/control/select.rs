use std::collections::BTreeSet;

use crate::EventId;

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

#[cfg(test)]
mod tests {
    use super::select_child;
    use crate::EventId;

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
}
