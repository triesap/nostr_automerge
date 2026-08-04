use std::collections::BTreeSet;

use crate::DevicePublicKey;
use crate::carrier::control::ValidatedControlContent;
use crate::control::validate::ControlEnvelope;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TransitionError {
    AccountChanged,
    RoleEscalation,
    DeviceReintroduced,
    TerminalChild,
    SuccessorContinuity,
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

#[cfg(test)]
mod tests {
    use super::{
        TransitionError, validate_account_mapping, validate_monotonic_roles,
        validate_no_reintroduction, validate_successor_continuity, validate_terminal_child,
    };
    use crate::carrier::control::Predecessor;
    use crate::control::validate::tests::{genesis, grant};
    use crate::types::role::Role;
    use crate::{ControllerPublicKey, DocumentCoordinate, DocumentId};

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
}
