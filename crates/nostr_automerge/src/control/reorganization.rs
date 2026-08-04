use std::collections::{BTreeMap, BTreeSet};

use crate::{CanonicalControlReorganizationAlert, ChangeHash, EventId, IntegrityAlert};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ControlChainSummary {
    pub(crate) controls: Vec<EventId>,
    pub(crate) changes_by_control: BTreeMap<EventId, BTreeSet<ChangeHash>>,
}

pub(crate) fn detect_reorganization(
    previous: &ControlChainSummary,
    current: &ControlChainSummary,
) -> Option<IntegrityAlert> {
    if previous.controls == current.controls
        || current.controls.starts_with(&previous.controls)
        || previous.controls.is_empty()
        || current.controls.is_empty()
    {
        return None;
    }
    let common = previous
        .controls
        .iter()
        .zip(&current.controls)
        .take_while(|(left, right)| left == right)
        .count();
    let affected_changes = previous.controls[common..]
        .iter()
        .chain(&current.controls[common..])
        .filter_map(|control| {
            previous
                .changes_by_control
                .get(control)
                .or_else(|| current.changes_by_control.get(control))
        })
        .flatten()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let previous_tip = previous.controls.last().copied()?;
    let new_tip = current.controls.last().copied()?;
    CanonicalControlReorganizationAlert::new(previous_tip, new_tip, affected_changes)
        .ok()
        .map(IntegrityAlert::CanonicalControlReorganization)
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{ControlChainSummary, detect_reorganization};
    use crate::{ChangeHash, EventId, IntegrityAlert};

    #[test]
    fn detect_and_report_canonical_reorganization() {
        let root = EventId::from_bytes([1; 32]);
        let old = EventId::from_bytes([3; 32]);
        let new = EventId::from_bytes([2; 32]);
        let old_change = ChangeHash::from_bytes([4; 32]);
        let new_change = ChangeHash::from_bytes([5; 32]);
        let previous = ControlChainSummary {
            controls: vec![root, old],
            changes_by_control: BTreeMap::from([(old, BTreeSet::from([old_change]))]),
        };
        let current = ControlChainSummary {
            controls: vec![root, new],
            changes_by_control: BTreeMap::from([(new, BTreeSet::from([new_change]))]),
        };
        assert!(matches!(
            detect_reorganization(&previous, &current),
            Some(IntegrityAlert::CanonicalControlReorganization(details))
                if details.previous_tip() == old
                    && details.new_tip() == new
                    && details.affected_changes() == [old_change, new_change]
        ));
        assert_eq!(detect_reorganization(&previous, &previous), None);
        let extension = ControlChainSummary {
            controls: vec![root, old, EventId::from_bytes([6; 32])],
            changes_by_control: previous.changes_by_control.clone(),
        };
        assert_eq!(detect_reorganization(&previous, &extension), None);
    }
}
