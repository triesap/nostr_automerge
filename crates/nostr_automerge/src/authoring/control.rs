use std::collections::BTreeSet;

use serde_json::{Value, json};

use crate::{
    AccountPublicKey, ChangeHash, DevicePublicKey, DocumentCoordinate, EventId, ProtocolRevision,
};

/// A role granted to a device by canonical control content.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ControlRole {
    /// The device may publish checkpoints.
    Checkpoint,
    /// The device may author changes.
    Write,
}

impl ControlRole {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Checkpoint => "checkpoint",
            Self::Write => "write",
        }
    }
}

/// One sorted, nonempty device grant in a control draft.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlGrant {
    /// Optional stable account association.
    pub account: Option<AccountPublicKey>,
    /// Device signing key.
    pub device: DevicePublicKey,
    /// Granted roles; input order is ignored.
    pub roles: BTreeSet<ControlRole>,
}

/// Link from the genesis control of a successor document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlPredecessor {
    /// Predecessor document coordinate.
    pub coordinate: DocumentCoordinate,
    /// Its terminal control event.
    pub terminal_control: EventId,
}

/// Canonical control content awaiting carrier construction and signing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlDraft {
    content: String,
}

/// Why semantic control inputs could not form canonical draft-v1 content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlDraftError {
    /// A sealed limit or grant invariant was violated.
    InvalidGrant,
    /// Sequence, predecessor, successor, or terminal continuity was invalid.
    InvalidTransition,
    /// Canonical JSON serialization failed.
    Serialization,
}

impl ControlDraft {
    /// Builds sealed canonical content. The caller supplies the exact accepted frontier.
    pub fn new(
        sequence: u64,
        base_heads: BTreeSet<ChangeHash>,
        grants: Vec<ControlGrant>,
        predecessor: Option<ControlPredecessor>,
        successor: Option<DocumentCoordinate>,
    ) -> Result<Self, ControlDraftError> {
        let limits = ProtocolRevision::draft_v1().limits();
        if grants.is_empty()
            || grants.len()
                > limits
                    .control_members
                    .try_usize()
                    .map_err(|_| ControlDraftError::InvalidGrant)?
            || base_heads.len()
                > limits
                    .control_heads
                    .try_usize()
                    .map_err(|_| ControlDraftError::InvalidGrant)?
            || grants.iter().any(|grant| grant.roles.is_empty())
        {
            return Err(ControlDraftError::InvalidGrant);
        }
        if (sequence == 0) != predecessor.is_none() {
            return Err(ControlDraftError::InvalidTransition);
        }
        let terminal = grants
            .iter()
            .all(|grant| !grant.roles.contains(&ControlRole::Write));
        if successor.is_some() != terminal {
            return Err(ControlDraftError::InvalidTransition);
        }
        let mut grants = grants;
        grants.sort_by_key(|grant| grant.device);
        if grants
            .windows(2)
            .any(|pair| pair[0].device == pair[1].device)
        {
            return Err(ControlDraftError::InvalidGrant);
        }
        let members = grants
            .into_iter()
            .map(|grant| {
                json!({
                    "account": grant.account.map(AccountPublicKey::to_hex),
                    "pubkey": grant.device.to_hex(),
                    "roles": grant.roles.into_iter().map(ControlRole::as_str).collect::<Vec<_>>()
                })
            })
            .collect::<Vec<Value>>();
        let predecessor = predecessor.map(|value| {
            json!({
                "coordinate": value.coordinate.to_address(),
                "terminal_control": value.terminal_control.to_hex(),
            })
        });
        let value = json!({
            "base_heads": base_heads.into_iter().map(ChangeHash::to_hex).collect::<Vec<_>>(),
            "format": ProtocolRevision::draft_v1().format(),
            "members": members,
            "policy": "controller-acl-v1",
            "predecessor": predecessor,
            "seq": sequence,
            "successor": successor.map(DocumentCoordinate::to_address),
            "text_encoding": ProtocolRevision::draft_v1().text_encoding(),
            "v": 1,
        });
        let bytes = crate::wire::canonical_json::serialize::to_vec(&value)
            .map_err(|_| ControlDraftError::Serialization)?;
        let content = String::from_utf8(bytes).map_err(|_| ControlDraftError::Serialization)?;
        Ok(Self { content })
    }

    /// Returns exact JCS content for the NIP-01 event.
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{ControlDraft, ControlDraftError, ControlGrant, ControlRole};
    use crate::{ChangeHash, ControllerPublicKey, DevicePublicKey, DocumentCoordinate, DocumentId};

    #[test]
    fn build_canonical_control_content() {
        let coordinate = DocumentCoordinate::new(
            ControllerPublicKey::from_bytes([1; 32]),
            DocumentId::from_bytes([2; 32]),
        );
        let grant = ControlGrant {
            account: None,
            device: DevicePublicKey::from_bytes([3; 32]),
            roles: BTreeSet::from([ControlRole::Write, ControlRole::Checkpoint]),
        };
        let draft = ControlDraft::new(
            0,
            BTreeSet::from([ChangeHash::from_bytes([4; 32])]),
            vec![grant.clone()],
            None,
            None,
        );
        assert!(draft.is_ok());
        let Ok(draft) = draft else { return };
        assert!(crate::carrier::control::validate_content(draft.content(), coordinate).is_ok());
        assert_eq!(
            ControlDraft::new(1, BTreeSet::new(), vec![grant], None, None),
            Err(ControlDraftError::InvalidTransition)
        );
    }
}
