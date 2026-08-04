use std::collections::BTreeSet;

use super::{AuthoredChange, AuthoringDocument};
use crate::ChangeHash;
use crate::automerge_adapter::document::AdapterAuthoringError;

/// A deterministic bounded dependency set requiring one empty merge change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FanInPlan {
    dependencies: Vec<ChangeHash>,
}

/// Why an accepted frontier cannot be consolidated by one v1 fan-in change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FanInError {
    /// The frontier already fits the 64-head control limit.
    Unnecessary,
    /// The frontier exceeds the 256-dependency change limit.
    DependencyLimit,
    /// Automerge could not create the empty merge change.
    Authoring,
}

impl FanInPlan {
    /// Plans one sorted empty merge for a frontier above 64 and at most 256 heads.
    pub fn new(heads: &BTreeSet<ChangeHash>) -> Result<Self, FanInError> {
        let count = u64::try_from(heads.len()).map_err(|_| FanInError::DependencyLimit)?;
        if count <= 64 {
            return Err(FanInError::Unnecessary);
        }
        if count > 256 {
            return Err(FanInError::DependencyLimit);
        }
        Ok(Self {
            dependencies: heads.iter().copied().collect(),
        })
    }

    /// Returns the strictly sorted dependency frontier.
    #[must_use]
    pub fn dependencies(&self) -> &[ChangeHash] {
        &self.dependencies
    }

    /// Authors the planned empty merge when the document remains on that frontier.
    pub fn author(self, document: &mut AuthoringDocument) -> Result<AuthoredChange, FanInError> {
        if document
            .actor_state()
            .accepted_heads()
            .iter()
            .copied()
            .collect::<Vec<_>>()
            != self.dependencies
        {
            return Err(FanInError::Authoring);
        }
        let previous_state = document.actor_state().clone();
        let mut staged = document.document.clone();
        let authored = staged.author_empty_change().map_err(|error| match error {
            AdapterAuthoringError::Limit => FanInError::DependencyLimit,
            _ => FanInError::Authoring,
        })?;
        let new_state = previous_state
            .transition(authored.hash, 0)
            .map_err(|_| FanInError::Authoring)?;
        document.document = staged;
        document.actor_state = new_state.clone();
        Ok(AuthoredChange::from_adapter(
            authored,
            previous_state,
            new_state,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{FanInError, FanInPlan};
    use crate::ChangeHash;
    use std::collections::BTreeSet;

    fn heads(count: u16) -> BTreeSet<ChangeHash> {
        (0..count)
            .map(|value| {
                let mut bytes = [0_u8; 32];
                bytes[30..].copy_from_slice(&value.to_be_bytes());
                ChangeHash::from_bytes(bytes)
            })
            .collect()
    }

    #[test]
    fn create_empty_fan_in_merge_changes() {
        assert_eq!(FanInPlan::new(&heads(64)), Err(FanInError::Unnecessary));
        let sixty_five = FanInPlan::new(&heads(65));
        assert!(sixty_five.is_ok());
        assert_eq!(
            sixty_five.as_ref().map(|plan| plan.dependencies().len()),
            Ok(65)
        );
        let boundary = FanInPlan::new(&heads(256));
        assert!(boundary.is_ok());
        assert!(
            boundary.is_ok_and(|plan| plan.dependencies().windows(2).all(|pair| pair[0] < pair[1]))
        );
        assert_eq!(
            FanInPlan::new(&heads(257)),
            Err(FanInError::DependencyLimit)
        );
    }
}
