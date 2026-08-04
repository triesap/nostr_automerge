use crate::carrier::control::ValidatedControlContent;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TransitionError {
    AccountChanged,
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

#[cfg(test)]
mod tests {
    use super::{TransitionError, validate_account_mapping};
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
}
