use crate::DiagnosticCode;
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

pub(crate) fn evaluate_child(
    parent: &ControlEnvelope,
    child: &ControlEnvelope,
    ancestry: &[&crate::carrier::control::ValidatedControlContent],
    view: &ParentEpochView,
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
        return CandidateResult::Invalid(DiagnosticCode::registered("control.parent"));
    }
    if validate_canonical_collections(&child.content).is_err()
        || validate_base_frontier(&child.content, false).is_err()
    {
        return CandidateResult::Invalid(DiagnosticCode::registered("control.structure"));
    }
    if validate_account_mapping(&parent.content, &child.content).is_err() {
        return CandidateResult::Invalid(DiagnosticCode::registered("control.account_changed"));
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
}
