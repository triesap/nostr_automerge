use std::collections::BTreeSet;

use crate::DevicePublicKey;
use crate::carrier::control::ValidatedControlContent;
#[cfg(test)]
use crate::control::frontier::accepted_frontier_closure;
use crate::control::frontier::accepted_frontier_closure_antichain_metered;
use crate::control::frontier::accepted_frontier_closure_metered;
use crate::control::parent_view::ParentEpochView;
use crate::control::validate::ControlEnvelope;
use crate::types::role::Role;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TransitionError {
    AccountChanged,
    RoleEscalation,
    DeviceReintroduced,
    TerminalChild,
    SuccessorContinuity,
    MissingBaseEvidence,
    BaseFrontierAntichain,
    RetainedWriterFrontier,
}

#[cfg(test)]
pub(crate) fn validate_base_frontier_antichain(
    child: &ValidatedControlContent,
    view: &ParentEpochView,
) -> Result<(), TransitionError> {
    let frontier = child.base_heads.iter().copied().collect::<BTreeSet<_>>();
    let complete = accepted_frontier_closure(
        frontier.iter().copied(),
        view.accepted(),
        view.dependency_index(),
    );
    if !complete.missing.is_empty() {
        return Err(TransitionError::MissingBaseEvidence);
    }
    if !complete.out_of_parent.is_empty() {
        return Err(TransitionError::BaseFrontierAntichain);
    }
    for head in &frontier {
        let closure = accepted_frontier_closure([*head], view.accepted(), view.dependency_index());
        if closure
            .accepted
            .iter()
            .any(|ancestor| ancestor != head && frontier.contains(ancestor))
        {
            return Err(TransitionError::BaseFrontierAntichain);
        }
    }
    Ok(())
}

pub(crate) fn validate_base_frontier_antichain_metered<E>(
    child: &ValidatedControlContent,
    view: &ParentEpochView,
    mut visit: impl FnMut(crate::WorkCounter) -> Result<(), E>,
) -> Result<Result<(), TransitionError>, E> {
    let complete = accepted_frontier_closure_metered(
        &child.base_heads,
        view.accepted(),
        view.dependency_index(),
        &mut visit,
    )?;
    if !complete.missing.is_empty() {
        return Ok(Err(TransitionError::MissingBaseEvidence));
    }
    if !complete.out_of_parent.is_empty() {
        return Ok(Err(TransitionError::BaseFrontierAntichain));
    }
    let mut head_index = 0;
    while head_index < child.base_heads.len() {
        let next_head_index = head_index + 1;
        let head = &child.base_heads[head_index..next_head_index];
        head_index += 1;
        let (_closure, has_ancestor) = accepted_frontier_closure_antichain_metered(
            head,
            &child.base_heads,
            view.accepted(),
            view.dependency_index(),
            &mut visit,
        )?;
        if has_ancestor {
            return Ok(Err(TransitionError::BaseFrontierAntichain));
        }
    }
    Ok(Ok(()))
}

#[cfg(test)]
pub(crate) fn validate_retained_writer_frontier(
    parent: &ValidatedControlContent,
    child: &ValidatedControlContent,
    view: &ParentEpochView,
) -> Result<(), TransitionError> {
    let mut closure = BTreeSet::new();
    let mut stack = child.base_heads.clone();
    while let Some(hash) = stack.pop() {
        if !view.contains(&hash) {
            return Err(TransitionError::MissingBaseEvidence);
        }
        if closure.insert(hash)
            && let Some(dependencies) = view.dependencies(&hash)
        {
            stack.extend(dependencies.iter().copied());
        }
    }
    for grant in &parent.members {
        let retained_writer = grant.roles.contains(&Role::Write)
            && child
                .members
                .iter()
                .any(|child_grant| child_grant.device == grant.device);
        if retained_writer
            && let Some(highest) = view.writer_contribution(&grant.actor)
            && !closure.contains(&highest)
        {
            return Err(TransitionError::RetainedWriterFrontier);
        }
    }
    Ok(())
}

pub(crate) fn validate_retained_writer_frontier_metered(
    parent: &ValidatedControlContent,
    child: &ValidatedControlContent,
    view: &ParentEpochView,
    visit: &mut impl FnMut() -> Result<(), crate::Completion>,
) -> Result<Result<(), TransitionError>, crate::Completion> {
    let mut closure = BTreeSet::new();
    let mut stack = Vec::new();
    let mut head_index = 0;
    while head_index < child.base_heads.len() {
        visit()?;
        stack.push(child.base_heads[head_index]);
        head_index += 1;
    }
    while !stack.is_empty() {
        visit()?;
        let Some(hash) = stack.pop() else {
            break;
        };
        if !view.contains(&hash) {
            return Ok(Err(TransitionError::MissingBaseEvidence));
        }
        if closure.insert(hash)
            && let Some(dependencies) = view.dependencies(&hash)
        {
            let mut dependencies = dependencies.iter();
            let dependency_count = dependencies.len();
            for _ in 0..dependency_count {
                visit()?;
                let Some(dependency) = dependencies.next() else {
                    break;
                };
                stack.push(*dependency);
            }
        }
    }
    let mut grant_index = 0;
    while grant_index < parent.members.len() {
        visit()?;
        let grant = &parent.members[grant_index];
        grant_index += 1;
        let mut retained_writer = false;
        let mut role_index = 0;
        while role_index < grant.roles.len() {
            visit()?;
            let role = &grant.roles[role_index];
            role_index += 1;
            if *role == Role::Write {
                retained_writer = true;
                break;
            }
        }
        if retained_writer {
            retained_writer = false;
            let mut child_grant_index = 0;
            while child_grant_index < child.members.len() {
                visit()?;
                let child_grant = &child.members[child_grant_index];
                child_grant_index += 1;
                if child_grant.device == grant.device {
                    retained_writer = true;
                    break;
                }
            }
        }
        if retained_writer && let Some(highest) = view.writer_contribution(&grant.actor) {
            visit()?;
            if !closure.contains(&highest) {
                return Ok(Err(TransitionError::RetainedWriterFrontier));
            }
        }
    }
    Ok(Ok(()))
}

pub(crate) fn validate_terminal_child(
    parent: &ValidatedControlContent,
    child: &ValidatedControlContent,
) -> Result<(), TransitionError> {
    if parent.terminal {
        return Err(TransitionError::TerminalChild);
    }
    if child.predecessor.is_some() || child.successor.is_some() && !child.terminal {
        return Err(TransitionError::SuccessorContinuity);
    }
    Ok(())
}

pub(crate) fn validate_successor_continuity(
    terminal: &ControlEnvelope,
    successor_genesis: &ControlEnvelope,
) -> Result<(), TransitionError> {
    let Some(successor_coordinate) = terminal.content.successor else {
        return Err(TransitionError::SuccessorContinuity);
    };
    let Some(predecessor) = &successor_genesis.content.predecessor else {
        return Err(TransitionError::SuccessorContinuity);
    };
    if !terminal.content.terminal
        || successor_genesis.parent.is_some()
        || successor_genesis.content.sequence != 0
        || successor_coordinate != successor_genesis.coordinate
        || predecessor.coordinate != terminal.coordinate
        || predecessor.terminal_control != terminal.event_id
    {
        return Err(TransitionError::SuccessorContinuity);
    }
    Ok(())
}

pub(crate) fn validate_no_reintroduction(
    ancestry: &[&ValidatedControlContent],
    child: &ValidatedControlContent,
) -> Result<(), TransitionError> {
    let mut active = BTreeSet::<DevicePublicKey>::new();
    let mut removed = BTreeSet::<DevicePublicKey>::new();
    for control in ancestry {
        let next: BTreeSet<_> = control.members.iter().map(|grant| grant.device).collect();
        removed.extend(active.difference(&next).copied());
        if next.iter().any(|device| removed.contains(device)) {
            return Err(TransitionError::DeviceReintroduced);
        }
        active = next;
    }
    if child
        .members
        .iter()
        .any(|grant| removed.contains(&grant.device))
    {
        return Err(TransitionError::DeviceReintroduced);
    }
    Ok(())
}

pub(crate) fn validate_no_reintroduction_metered(
    ancestry: &[&ValidatedControlContent],
    child: &ValidatedControlContent,
    visit: &mut impl FnMut() -> Result<(), crate::Completion>,
) -> Result<Result<(), TransitionError>, crate::Completion> {
    let mut active = BTreeSet::<DevicePublicKey>::new();
    let mut removed = BTreeSet::<DevicePublicKey>::new();
    let mut control_index = 0;
    while control_index < ancestry.len() {
        visit()?;
        let control = ancestry[control_index];
        control_index += 1;
        let mut next = BTreeSet::new();
        let mut grant_index = 0;
        while grant_index < control.members.len() {
            visit()?;
            let grant = &control.members[grant_index];
            grant_index += 1;
            next.insert(grant.device);
        }
        let mut active_iter = active.iter();
        let active_count = active_iter.len();
        for _ in 0..active_count {
            visit()?;
            let Some(device) = active_iter.next() else {
                break;
            };
            if !next.contains(device) {
                removed.insert(*device);
            }
        }
        let mut next_iter = next.iter();
        let next_count = next_iter.len();
        for _ in 0..next_count {
            visit()?;
            let Some(device) = next_iter.next() else {
                break;
            };
            if removed.contains(device) {
                return Ok(Err(TransitionError::DeviceReintroduced));
            }
        }
        active = next;
    }
    let mut grant_index = 0;
    while grant_index < child.members.len() {
        visit()?;
        let grant = &child.members[grant_index];
        grant_index += 1;
        if removed.contains(&grant.device) {
            return Ok(Err(TransitionError::DeviceReintroduced));
        }
    }
    Ok(Ok(()))
}

pub(crate) fn validate_account_mapping(
    parent: &ValidatedControlContent,
    child: &ValidatedControlContent,
) -> Result<(), TransitionError> {
    for child_grant in &child.members {
        if let Some(parent_grant) = parent
            .members
            .iter()
            .find(|grant| grant.device == child_grant.device)
            && parent_grant.account != child_grant.account
        {
            return Err(TransitionError::AccountChanged);
        }
    }
    Ok(())
}

pub(crate) fn validate_account_mapping_metered(
    parent: &ValidatedControlContent,
    child: &ValidatedControlContent,
    visit: &mut impl FnMut() -> Result<(), crate::Completion>,
) -> Result<Result<(), TransitionError>, crate::Completion> {
    let mut child_index = 0;
    while child_index < child.members.len() {
        visit()?;
        let child_grant = &child.members[child_index];
        child_index += 1;
        let mut parent_index = 0;
        while parent_index < parent.members.len() {
            visit()?;
            let parent_grant = &parent.members[parent_index];
            parent_index += 1;
            if parent_grant.device == child_grant.device {
                visit()?;
                if parent_grant.account != child_grant.account {
                    return Ok(Err(TransitionError::AccountChanged));
                }
                break;
            }
        }
    }
    Ok(Ok(()))
}

pub(crate) fn validate_monotonic_roles(
    parent: &ValidatedControlContent,
    child: &ValidatedControlContent,
) -> Result<(), TransitionError> {
    for child_grant in &child.members {
        if let Some(parent_grant) = parent
            .members
            .iter()
            .find(|grant| grant.device == child_grant.device)
            && !child_grant
                .roles
                .iter()
                .all(|role| parent_grant.roles.contains(role))
        {
            return Err(TransitionError::RoleEscalation);
        }
    }
    Ok(())
}

pub(crate) fn validate_monotonic_roles_metered(
    parent: &ValidatedControlContent,
    child: &ValidatedControlContent,
    visit: &mut impl FnMut() -> Result<(), crate::Completion>,
) -> Result<Result<(), TransitionError>, crate::Completion> {
    let mut child_index = 0;
    while child_index < child.members.len() {
        visit()?;
        let child_grant = &child.members[child_index];
        child_index += 1;
        let mut parent_index = 0;
        while parent_index < parent.members.len() {
            visit()?;
            let parent_grant = &parent.members[parent_index];
            parent_index += 1;
            if parent_grant.device != child_grant.device {
                continue;
            }
            let mut child_role_index = 0;
            while child_role_index < child_grant.roles.len() {
                visit()?;
                let child_role = &child_grant.roles[child_role_index];
                child_role_index += 1;
                let mut found = false;
                let mut parent_role_index = 0;
                while parent_role_index < parent_grant.roles.len() {
                    visit()?;
                    let parent_role = &parent_grant.roles[parent_role_index];
                    parent_role_index += 1;
                    if parent_role == child_role {
                        found = true;
                        break;
                    }
                }
                if !found {
                    return Ok(Err(TransitionError::RoleEscalation));
                }
            }
            break;
        }
    }
    Ok(Ok(()))
}

#[cfg(test)]
mod tests {
    use super::{
        TransitionError, validate_account_mapping, validate_base_frontier_antichain,
        validate_monotonic_roles, validate_no_reintroduction, validate_retained_writer_frontier,
        validate_successor_continuity, validate_terminal_child,
    };
    use crate::carrier::control::Predecessor;
    use crate::control::parent_view::ParentEpochView;
    use crate::control::validate::tests::{genesis, grant};
    use crate::types::role::Role;
    use crate::{ActorId, ChangeHash, ControllerPublicKey, DocumentCoordinate, DocumentId};
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn enforce_immutable_account_mapping() {
        let parent = genesis().content;
        let mut child = parent.clone();
        assert_eq!(validate_account_mapping(&parent, &child), Ok(()));

        child.members[0].account = None;
        assert_eq!(
            validate_account_mapping(&parent, &child),
            Err(TransitionError::AccountChanged)
        );
        let mut null_parent = parent.clone();
        null_parent.members[0].account = None;
        assert_eq!(
            validate_account_mapping(&null_parent, &parent),
            Err(TransitionError::AccountChanged)
        );
        let mut fresh = parent.clone();
        fresh.members.push(grant(9, vec![Role::Write]));
        assert_eq!(validate_account_mapping(&parent, &fresh), Ok(()));
    }

    #[test]
    fn enforce_monotonic_role_reduction() {
        let mut parent = genesis().content;
        parent.members[0].roles = vec![Role::Checkpoint, Role::Write];
        for roles in [
            vec![Role::Checkpoint, Role::Write],
            vec![Role::Checkpoint],
            vec![Role::Write],
        ] {
            let mut child = parent.clone();
            child.members[0].roles = roles;
            assert_eq!(validate_monotonic_roles(&parent, &child), Ok(()));
        }

        let mut checkpoint_parent = parent.clone();
        checkpoint_parent.members[0].roles = vec![Role::Checkpoint];
        let mut escalated = checkpoint_parent.clone();
        escalated.members[0].roles.push(Role::Write);
        assert_eq!(
            validate_monotonic_roles(&checkpoint_parent, &escalated),
            Err(TransitionError::RoleEscalation)
        );
        let mut fresh = checkpoint_parent.clone();
        fresh.members.push(grant(9, vec![Role::Write]));
        assert_eq!(validate_monotonic_roles(&checkpoint_parent, &fresh), Ok(()));
    }

    #[test]
    fn forbid_removed_device_reintroduction() {
        let first = genesis().content;
        let mut removed = first.clone();
        removed.members.clear();
        let mut later = removed.clone();
        later.members.push(grant(9, vec![Role::Write]));
        let mut reintroduced = later.clone();
        reintroduced.members.insert(0, first.members[0].clone());
        assert_eq!(
            validate_no_reintroduction(&[&first, &removed, &later], &reintroduced),
            Err(TransitionError::DeviceReintroduced)
        );
        assert_eq!(
            validate_no_reintroduction(&[&first, &removed], &later),
            Ok(())
        );
    }

    #[test]
    fn validate_terminal_and_successor_continuity() {
        let mut terminal = genesis();
        terminal.content.members[0].roles = vec![Role::Checkpoint];
        terminal.content.terminal = true;
        let mut ordinary_child = genesis();
        ordinary_child.parent = Some(terminal.event_id);
        ordinary_child.content.sequence = 1;
        assert_eq!(
            validate_terminal_child(&terminal.content, &ordinary_child.content),
            Err(TransitionError::TerminalChild)
        );

        let successor_coordinate = DocumentCoordinate::new(
            ControllerPublicKey::from_bytes([8; 32]),
            DocumentId::from_bytes([9; 32]),
        );
        terminal.content.successor = Some(successor_coordinate);
        let mut successor = genesis();
        successor.coordinate = successor_coordinate;
        successor.author = successor_coordinate.controller();
        successor.content.predecessor = Some(Predecessor {
            coordinate: terminal.coordinate,
            terminal_control: terminal.event_id,
        });
        assert_eq!(validate_successor_continuity(&terminal, &successor), Ok(()));

        successor.content.predecessor = None;
        assert_eq!(
            validate_successor_continuity(&terminal, &successor),
            Err(TransitionError::SuccessorContinuity)
        );
        let mut nonterminal = genesis().content;
        nonterminal.successor = Some(successor_coordinate);
        assert_eq!(
            validate_terminal_child(&genesis().content, &nonterminal),
            Err(TransitionError::SuccessorContinuity)
        );
    }

    #[test]
    fn enforce_retained_writer_frontier_rule() {
        let mut parent = genesis().content;
        parent.members.push(grant(5, vec![Role::Write]));
        let first = ChangeHash::from_bytes([1; 32]);
        let second = ChangeHash::from_bytes([2; 32]);
        let head = ChangeHash::from_bytes([3; 32]);
        let view = ParentEpochView::from_parts_for_test(
            BTreeSet::from([first, second, head]),
            BTreeSet::from([head]),
            BTreeMap::from([(head, BTreeSet::from([first, second]))]),
            BTreeMap::new(),
            BTreeMap::from([
                (parent.members[0].actor, first),
                (parent.members[1].actor, second),
            ]),
        );
        let mut child = parent.clone();
        child.base_heads = vec![head];
        assert_eq!(
            validate_retained_writer_frontier(&parent, &child, &view),
            Ok(())
        );

        child.base_heads = vec![first];
        assert_eq!(
            validate_retained_writer_frontier(&parent, &child, &view),
            Err(TransitionError::RetainedWriterFrontier)
        );
        child.members.remove(1);
        assert_eq!(
            validate_retained_writer_frontier(&parent, &child, &view),
            Ok(())
        );
        child.base_heads = vec![ChangeHash::from_bytes([9; 32])];
        assert_eq!(
            validate_retained_writer_frontier(&parent, &child, &view),
            Err(TransitionError::MissingBaseEvidence)
        );
        assert_eq!(parent.members[0].actor, ActorId::from_bytes([4; 32]));
    }

    #[test]
    fn require_an_exact_accepted_base_frontier_antichain() {
        let parent = genesis().content;
        let first = ChangeHash::from_bytes([1; 32]);
        let second = ChangeHash::from_bytes([2; 32]);
        let actor = parent.members[0].actor;
        let view = ParentEpochView::from_parts_for_test(
            BTreeSet::from([first, second]),
            BTreeSet::from([second]),
            BTreeMap::from([(first, BTreeSet::new()), (second, BTreeSet::from([first]))]),
            BTreeMap::new(),
            BTreeMap::from([(actor, second)]),
        );

        let mut child = parent.clone();
        child.base_heads = vec![second];
        assert_eq!(validate_base_frontier_antichain(&child, &view), Ok(()));

        child.base_heads = vec![first, second];
        assert_eq!(
            validate_base_frontier_antichain(&child, &view),
            Err(TransitionError::BaseFrontierAntichain)
        );

        child.base_heads = vec![ChangeHash::from_bytes([9; 32])];
        assert_eq!(
            validate_base_frontier_antichain(&child, &view),
            Err(TransitionError::MissingBaseEvidence)
        );

        let third = ChangeHash::from_bytes([3; 32]);
        let fan_in = ParentEpochView::from_parts_for_test(
            BTreeSet::from([first, second, third]),
            BTreeSet::from([third]),
            BTreeMap::from([
                (first, BTreeSet::new()),
                (second, BTreeSet::new()),
                (third, BTreeSet::from([first, second])),
            ]),
            BTreeMap::new(),
            BTreeMap::new(),
        );
        child.base_heads = vec![first, second];
        assert_eq!(validate_base_frontier_antichain(&child, &fan_in), Ok(()));
    }
}
