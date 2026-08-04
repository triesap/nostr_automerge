use crate::carrier::control::ValidatedControlContent;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TransitionError {
    AccountChanged,
    RoleEscalation,
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
    use super::{TransitionError, validate_account_mapping, validate_monotonic_roles};
    use crate::control::validate::tests::{genesis, grant};
    use crate::types::role::Role;

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
}
