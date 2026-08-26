use crate::DiagnosticCode;
use crate::control::ancestry::ControlAncestry;
#[cfg(test)]
use crate::control::frontier::reasoned_frontier_disposition;
use crate::control::frontier::reasoned_frontier_disposition_metered;
use crate::control::parent_view::ParentEpochView;
use crate::control::transition::{
    TransitionError, validate_account_mapping, validate_account_mapping_metered,
    validate_base_frontier_antichain_metered, validate_monotonic_roles,
    validate_monotonic_roles_metered, validate_no_reintroduction,
    validate_no_reintroduction_metered, validate_retained_writer_frontier_metered,
    validate_successor_continuity, validate_terminal_child,
};
#[cfg(test)]
use crate::control::transition::{
    validate_base_frontier_antichain, validate_retained_writer_frontier,
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

pub(crate) fn evaluate_account_continuity_metered(
    parent: &ControlEnvelope,
    child: &ControlEnvelope,
    visit: &mut impl FnMut() -> Result<(), crate::Completion>,
) -> Result<CandidateResult, crate::Completion> {
    Ok(
        if validate_account_mapping_metered(&parent.content, &child.content, visit)?.is_err() {
            CandidateResult::Invalid(DiagnosticCode::registered("control.account_changed"))
        } else {
            CandidateResult::Valid
        },
    )
}

pub(crate) fn evaluate_role_continuity(
    parent: &ControlEnvelope,
    child: &ControlEnvelope,
) -> CandidateResult {
    if validate_monotonic_roles(&parent.content, &child.content).is_err() {
        CandidateResult::Invalid(DiagnosticCode::registered("control.role_escalation"))
    } else {
        CandidateResult::Valid
    }
}

pub(crate) fn evaluate_role_continuity_metered(
    parent: &ControlEnvelope,
    child: &ControlEnvelope,
    visit: &mut impl FnMut() -> Result<(), crate::Completion>,
) -> Result<CandidateResult, crate::Completion> {
    Ok(
        if validate_monotonic_roles_metered(&parent.content, &child.content, visit)?.is_err() {
            CandidateResult::Invalid(DiagnosticCode::registered("control.role_escalation"))
        } else {
            CandidateResult::Valid
        },
    )
}

pub(crate) fn evaluate_device_ancestry(
    ancestry: &[ControlEnvelope],
    child: &ControlEnvelope,
) -> CandidateResult {
    let contents = ancestry
        .iter()
        .map(|control| &control.content)
        .collect::<Vec<_>>();
    if validate_no_reintroduction(&contents, &child.content).is_err() {
        CandidateResult::Invalid(DiagnosticCode::registered("control.device_reintroduced"))
    } else {
        CandidateResult::Valid
    }
}

pub(crate) fn evaluate_device_ancestry_metered(
    ancestry: &[ControlEnvelope],
    child: &ControlEnvelope,
    visit: &mut impl FnMut() -> Result<(), crate::Completion>,
) -> Result<CandidateResult, crate::Completion> {
    let mut contents = Vec::new();
    if !ancestry.is_empty() {
        visit()?;
        contents = Vec::with_capacity(ancestry.len());
        contents.push(&ancestry[0].content);
    }
    let mut index = 1;
    while index < ancestry.len() {
        visit()?;
        let control = &ancestry[index];
        index += 1;
        contents.push(&control.content);
    }
    Ok(
        if validate_no_reintroduction_metered(&contents, &child.content, visit)?.is_err() {
            CandidateResult::Invalid(DiagnosticCode::registered("control.device_reintroduced"))
        } else {
            CandidateResult::Valid
        },
    )
}

pub(crate) fn evaluate_terminal_continuity(
    parent: &ControlEnvelope,
    child: &ControlEnvelope,
) -> CandidateResult {
    match validate_terminal_child(&parent.content, &child.content) {
        Ok(()) => CandidateResult::Valid,
        Err(TransitionError::TerminalChild) => {
            CandidateResult::Invalid(DiagnosticCode::registered("control.terminal_child"))
        }
        Err(_) => CandidateResult::Invalid(DiagnosticCode::registered("control.structure")),
    }
}

pub(crate) fn evaluate_successor_genesis(
    terminal: &ControlEnvelope,
    successor_genesis: &ControlEnvelope,
) -> CandidateResult {
    if validate_successor_continuity(terminal, successor_genesis).is_err() {
        CandidateResult::Invalid(DiagnosticCode::registered("control.structure"))
    } else {
        CandidateResult::Valid
    }
}

#[cfg(test)]
pub(crate) fn evaluate_retained_writer_continuity(
    parent: &ControlEnvelope,
    child: &ControlEnvelope,
    view: &ParentEpochView,
) -> CandidateResult {
    match validate_retained_writer_frontier(&parent.content, &child.content, view) {
        Ok(()) => CandidateResult::Valid,
        Err(TransitionError::MissingBaseEvidence) => {
            CandidateResult::Pending(DiagnosticCode::registered("control.frontier"))
        }
        Err(_) => CandidateResult::Invalid(DiagnosticCode::registered("control.retained_writer")),
    }
}

#[cfg(test)]
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
    match reasoned_frontier_disposition(child.content.base_heads.iter().copied(), |hash| {
        view.frontier_knowledge(hash)
    }) {
        Some(crate::ProtocolDisposition::Pending) => {
            return CandidateResult::Pending(DiagnosticCode::registered("control.frontier"));
        }
        Some(crate::ProtocolDisposition::Invalid) => {
            return CandidateResult::Invalid(DiagnosticCode::registered("control.frontier"));
        }
        Some(
            crate::ProtocolDisposition::Accepted
            | crate::ProtocolDisposition::Excluded
            | crate::ProtocolDisposition::UnsupportedRevision,
        )
        | None => {}
    }
    if let result @ CandidateResult::Invalid(_) = evaluate_account_continuity(parent, child) {
        return result;
    }
    if let result @ CandidateResult::Invalid(_) = evaluate_role_continuity(parent, child) {
        return result;
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
    evaluate_retained_writer_continuity(parent, child, view)
}

pub(crate) fn evaluate_child_metered(
    parent: &ControlEnvelope,
    child: &ControlEnvelope,
    ancestry: &ControlAncestry,
    view: &ParentEpochView,
    visit: &mut impl FnMut(crate::WorkCounter) -> Result<(), crate::Completion>,
) -> Result<CandidateResult, crate::Completion> {
    visit(crate::WorkCounter::Control)?;
    if let result @ CandidateResult::Invalid(_) = evaluate_parent_continuity(parent, child) {
        return Ok(result);
    }
    if validate_canonical_collections(&child.content).is_err()
        || validate_base_frontier(&child.content, false).is_err()
    {
        return Ok(CandidateResult::Invalid(DiagnosticCode::registered(
            "control.structure",
        )));
    }
    match reasoned_frontier_disposition_metered(
        &child.content.base_heads,
        |hash, metered| {
            view.frontier_knowledge_metered(hash, || metered(crate::WorkCounter::GraphNode))
        },
        visit,
    )? {
        Some(crate::ProtocolDisposition::Pending) => {
            return Ok(CandidateResult::Pending(DiagnosticCode::registered(
                "control.frontier",
            )));
        }
        Some(crate::ProtocolDisposition::Invalid) => {
            return Ok(CandidateResult::Invalid(DiagnosticCode::registered(
                "control.frontier",
            )));
        }
        Some(
            crate::ProtocolDisposition::Accepted
            | crate::ProtocolDisposition::Excluded
            | crate::ProtocolDisposition::UnsupportedRevision,
        )
        | None => {}
    }
    let account = evaluate_account_continuity_metered(parent, child, &mut || {
        visit(crate::WorkCounter::Control)
    })?;
    if let result @ CandidateResult::Invalid(_) = account {
        return Ok(result);
    }
    let roles = evaluate_role_continuity_metered(parent, child, &mut || {
        visit(crate::WorkCounter::Control)
    })?;
    if let result @ CandidateResult::Invalid(_) = roles {
        return Ok(result);
    }
    if ancestry
        .no_reintroduction_metered(&child.content, &mut || visit(crate::WorkCounter::Control))?
        .is_err()
    {
        return Ok(CandidateResult::Invalid(DiagnosticCode::registered(
            "control.device_reintroduced",
        )));
    }
    visit(crate::WorkCounter::Control)?;
    if let Err(error) = validate_terminal_child(&parent.content, &child.content) {
        return Ok(CandidateResult::Invalid(DiagnosticCode::registered(
            match error {
                TransitionError::TerminalChild => "control.terminal_child",
                _ => "control.structure",
            },
        )));
    }
    match validate_base_frontier_antichain_metered(&child.content, view, &mut *visit)? {
        Ok(()) => {}
        Err(TransitionError::MissingBaseEvidence) => {
            return Ok(CandidateResult::Pending(DiagnosticCode::registered(
                "control.frontier",
            )));
        }
        Err(_) => {
            return Ok(CandidateResult::Invalid(DiagnosticCode::registered(
                "control.frontier",
            )));
        }
    }
    Ok(
        match validate_retained_writer_frontier_metered(
            &parent.content,
            &child.content,
            view,
            &mut || visit(crate::WorkCounter::Control),
        )? {
            Ok(()) => CandidateResult::Valid,
            Err(TransitionError::MissingBaseEvidence) => {
                CandidateResult::Pending(DiagnosticCode::registered("control.frontier"))
            }
            Err(_) => {
                CandidateResult::Invalid(DiagnosticCode::registered("control.retained_writer"))
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{CandidateResult, evaluate_child, evaluate_child_metered};
    use crate::control::ancestry::ControlAncestry;
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
    fn metered_child_continuity_stops_before_every_member_visit() {
        let parent = genesis();
        let mut child = parent.clone();
        child.event_id = EventId::from_bytes([6; 32]);
        child.parent = Some(parent.event_id);
        child.content.sequence = 1;
        let view = ParentEpochView::default();
        let ancestry = ControlAncestry::from_ordered([parent.clone()]);
        assert!(ancestry.is_ok());
        let Ok(ancestry) = ancestry else {
            return;
        };
        let visits = std::cell::Cell::new(0_u64);
        let mut count = |_counter| {
            visits.set(visits.get() + 1);
            Ok(())
        };
        assert_eq!(
            evaluate_child_metered(&parent, &child, &ancestry, &view, &mut count),
            Ok(CandidateResult::Valid)
        );
        let exact = visits.get();
        assert!(exact > 0);

        for boundary in 0..exact {
            let observed = std::cell::Cell::new(0_u64);
            let mut stop = |_counter| {
                let current = observed.get();
                if current == boundary {
                    return Err(crate::Completion::Cancelled);
                }
                observed.set(current + 1);
                Ok(())
            };
            assert_eq!(
                evaluate_child_metered(&parent, &child, &ancestry, &view, &mut stop),
                Err(crate::Completion::Cancelled)
            );
            assert_eq!(observed.get(), boundary);
        }
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
