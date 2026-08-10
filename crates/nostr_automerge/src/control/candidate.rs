use crate::DiagnosticCode;
use crate::control::frontier::accepted_frontier_closure;
use crate::control::parent_view::ParentEpochView;
use crate::control::transition::{
    TransitionError, validate_account_mapping, validate_base_frontier_antichain,
    validate_monotonic_roles, validate_no_reintroduction, validate_retained_writer_frontier,
    validate_terminal_child,
};
use crate::control::validate::{
    ControlEnvelope, validate_base_frontier, validate_canonical_collections,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CandidateResult {
    Valid,
    Pending(DiagnosticCode),
    Invalid(DiagnosticCode),
}

pub(crate) fn evaluate_parent_continuity(
    parent: &ControlEnvelope,
    child: &ControlEnvelope,
) -> CandidateResult {
    if child.parent != Some(parent.event_id)
        || child.coordinate != parent.coordinate
        || child.author != child.coordinate.controller()
        || parent
            .content
            .sequence
            .checked_add(1)
            .is_none_or(|sequence| child.content.sequence != sequence)
    {
        CandidateResult::Invalid(DiagnosticCode::registered("control.parent"))
    } else {
        CandidateResult::Valid
    }
}

pub(crate) fn evaluate_account_continuity(
    parent: &ControlEnvelope,
    child: &ControlEnvelope,
) -> CandidateResult {
    if validate_account_mapping(&parent.content, &child.content).is_err() {
        CandidateResult::Invalid(DiagnosticCode::registered("control.account_changed"))
    } else {
        CandidateResult::Valid
    }
}

pub(crate) fn evaluate_child(
    parent: &ControlEnvelope,
    child: &ControlEnvelope,
    ancestry: &[&crate::carrier::control::ValidatedControlContent],
    view: &ParentEpochView,
) -> CandidateResult {
    if let result @ CandidateResult::Invalid(_) = evaluate_parent_continuity(parent, child) {
        return result;
    }
    if validate_canonical_collections(&child.content).is_err()
        || validate_base_frontier(&child.content, false).is_err()
    {
        return CandidateResult::Invalid(DiagnosticCode::registered("control.structure"));
    }
    let base_closure = accepted_frontier_closure(
        child.content.base_heads.iter().copied(),
        view.accepted(),
        view.dependency_index(),
    );
    if !base_closure.missing.is_empty() {
        return CandidateResult::Pending(DiagnosticCode::registered("control.frontier"));
    }
    if !base_closure.out_of_parent.is_empty() {
        return CandidateResult::Invalid(DiagnosticCode::registered("control.frontier"));
    }
    if let result @ CandidateResult::Invalid(_) = evaluate_account_continuity(parent, child) {
        return result;
    }
    if validate_monotonic_roles(&parent.content, &child.content).is_err() {
        return CandidateResult::Invalid(DiagnosticCode::registered("control.role_escalation"));
    }
    if validate_no_reintroduction(ancestry, &child.content).is_err() {
        return CandidateResult::Invalid(DiagnosticCode::registered("control.device_reintroduced"));
    }
    if let Err(error) = validate_terminal_child(&parent.content, &child.content) {
        return CandidateResult::Invalid(DiagnosticCode::registered(match error {
            TransitionError::TerminalChild => "control.terminal_child",
            _ => "control.structure",
        }));
    }
    match validate_base_frontier_antichain(&child.content, view) {
        Ok(()) => {}
        Err(TransitionError::MissingBaseEvidence) => {
            return CandidateResult::Pending(DiagnosticCode::registered("control.frontier"));
        }
        Err(_) => {
            return CandidateResult::Invalid(DiagnosticCode::registered("control.frontier"));
        }
    }
    match validate_retained_writer_frontier(&parent.content, &child.content, view) {
        Ok(()) => CandidateResult::Valid,
        Err(TransitionError::MissingBaseEvidence) => {
            CandidateResult::Pending(DiagnosticCode::registered("control.frontier"))
        }
        Err(_) => CandidateResult::Invalid(DiagnosticCode::registered("control.retained_writer")),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{CandidateResult, evaluate_child};
    use crate::control::parent_view::ParentEpochView;
    use crate::control::validate::tests::genesis;
    use crate::{ChangeHash, DiagnosticCode, EventId};

    #[test]
    fn validate_child_candidates_against_parent_state() {
        let parent = genesis();
        let mut child = parent.clone();
        child.event_id = EventId::from_bytes([5; 32]);
        child.parent = Some(parent.event_id);
        child.content.sequence = 1;
        let view = ParentEpochView::default();
        assert_eq!(
            evaluate_child(&parent, &child, &[&parent.content], &view),
            CandidateResult::Valid
        );

        let mut pending = child.clone();
        pending.content.base_heads = vec![ChangeHash::from_bytes([9; 32])];
        assert_eq!(
            evaluate_child(&parent, &pending, &[&parent.content], &view),
            CandidateResult::Pending(DiagnosticCode::registered("control.frontier"))
        );
        let mut invalid = child;
        invalid.content.members[0].account = None;
        assert_eq!(
            evaluate_child(&parent, &invalid, &[&parent.content], &view),
            CandidateResult::Invalid(DiagnosticCode::registered("control.account_changed"))
        );
    }

    #[test]
    fn pending_frontier_promotes_after_delivery() {
        let parent = genesis();
        let mut child = parent.clone();
        child.event_id = EventId::from_bytes([5; 32]);
        child.parent = Some(parent.event_id);
        child.content.sequence = 1;
        let missing = ChangeHash::from_bytes([9; 32]);
        child.content.base_heads = vec![missing];

        assert_eq!(
            evaluate_child(
                &parent,
                &child,
                &[&parent.content],
                &ParentEpochView::default()
            ),
            CandidateResult::Pending(DiagnosticCode::registered("control.frontier"))
        );

        let delivered = ParentEpochView::from_parts_for_test(
            BTreeSet::from([missing]),
            BTreeSet::from([missing]),
            BTreeMap::from([(missing, BTreeSet::new())]),
            BTreeMap::new(),
            BTreeMap::new(),
        );
        assert_eq!(
            evaluate_child(&parent, &child, &[&parent.content], &delivered),
            CandidateResult::Valid
        );
    }
}
