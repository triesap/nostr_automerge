use std::collections::BTreeSet;

use crate::DevicePublicKey;
use crate::carrier::control::ValidatedControlContent;
use crate::types::role::Role;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ControlState {
    pub(crate) writers: BTreeSet<DevicePublicKey>,
    pub(crate) checkpointers: BTreeSet<DevicePublicKey>,
    pub(crate) frozen: bool,
}

pub(crate) fn derive_state(content: &ValidatedControlContent) -> ControlState {
    let writers = content
        .members
        .iter()
        .filter(|grant| grant.roles.contains(&Role::Write))
        .map(|grant| grant.device)
        .collect::<BTreeSet<_>>();
    let checkpointers = content
        .members
        .iter()
        .filter(|grant| grant.roles.contains(&Role::Checkpoint))
        .map(|grant| grant.device)
        .collect::<BTreeSet<_>>();
    ControlState {
        frozen: writers.is_empty(),
        writers,
        checkpointers,
    }
}

#[cfg(test)]
mod tests {
    use super::derive_state;
    use crate::DevicePublicKey;
    use crate::control::validate::tests::{genesis, grant};
    use crate::types::role::Role;

    #[test]
    fn validate_writer_and_frozen_state() {
        let mut content = genesis().content;
        content.members = vec![
            grant(4, vec![Role::Write]),
            grant(5, vec![Role::Checkpoint]),
        ];
        let state = derive_state(&content);
        assert_eq!(state.writers, [DevicePublicKey::from_bytes([4; 32])].into());
        assert_eq!(
            state.checkpointers,
            [DevicePublicKey::from_bytes([5; 32])].into()
        );
        assert!(!state.frozen);
        assert!(
            !state
                .writers
                .contains(&DevicePublicKey::from_bytes([1; 32]))
        );

        content.members[0].roles = vec![Role::Checkpoint];
        let frozen = derive_state(&content);
        assert!(frozen.frozen);
        assert!(frozen.writers.is_empty());
    }
}
