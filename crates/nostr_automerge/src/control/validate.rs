use crate::carrier::control::ValidatedControlContent;
use crate::{ControllerPublicKey, DocumentCoordinate, EventId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ControlEnvelope {
    pub(crate) event_id: EventId,
    pub(crate) author: ControllerPublicKey,
    pub(crate) coordinate: DocumentCoordinate,
    pub(crate) parent: Option<EventId>,
    pub(crate) content: ValidatedControlContent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ControlValidationError {
    Author,
    Parent,
    Sequence,
    BaseHeads,
    Grants,
    Terminal,
}

pub(crate) fn validate_genesis(control: &ControlEnvelope) -> Result<(), ControlValidationError> {
    if control.author != control.coordinate.controller() {
        return Err(ControlValidationError::Author);
    }
    if control.parent.is_some() {
        return Err(ControlValidationError::Parent);
    }
    if control.content.sequence != 0 {
        return Err(ControlValidationError::Sequence);
    }
    if !control.content.base_heads.is_empty() {
        return Err(ControlValidationError::BaseHeads);
    }
    if control.content.members.is_empty() {
        return Err(ControlValidationError::Grants);
    }
    if control.content.successor.is_some() && !control.content.terminal {
        return Err(ControlValidationError::Terminal);
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{ControlEnvelope, ControlValidationError, validate_genesis};
    use crate::carrier::control::{DeviceGrant, ValidatedControlContent};
    use crate::types::role::Role;
    use crate::{
        AccountPublicKey, ActorId, ChangeHash, ControllerPublicKey, DevicePublicKey,
        DocumentCoordinate, DocumentId, EventId,
    };

    pub(crate) fn grant(byte: u8, roles: Vec<Role>) -> DeviceGrant {
        DeviceGrant {
            account: Some(AccountPublicKey::from_bytes([byte; 32])),
            actor: ActorId::from_bytes([byte; 32]),
            device: DevicePublicKey::from_bytes([byte; 32]),
            roles,
        }
    }

    pub(crate) fn genesis() -> ControlEnvelope {
        let controller = ControllerPublicKey::from_bytes([1; 32]);
        ControlEnvelope {
            event_id: EventId::from_bytes([2; 32]),
            author: controller,
            coordinate: DocumentCoordinate::new(controller, DocumentId::from_bytes([3; 32])),
            parent: None,
            content: ValidatedControlContent {
                base_heads: Vec::new(),
                members: vec![grant(4, vec![Role::Write])],
                predecessor: None,
                sequence: 0,
                successor: None,
                terminal: false,
            },
        }
    }

    #[test]
    fn implement_genesis_control_structural_validation() {
        let valid = genesis();
        assert_eq!(validate_genesis(&valid), Ok(()));

        let mut wrong_author = valid.clone();
        wrong_author.author = ControllerPublicKey::from_bytes([9; 32]);
        assert_eq!(
            validate_genesis(&wrong_author),
            Err(ControlValidationError::Author)
        );
        let mut parent = valid.clone();
        parent.parent = Some(EventId::from_bytes([8; 32]));
        assert_eq!(
            validate_genesis(&parent),
            Err(ControlValidationError::Parent)
        );
        let mut sequence = valid.clone();
        sequence.content.sequence = 1;
        assert_eq!(
            validate_genesis(&sequence),
            Err(ControlValidationError::Sequence)
        );
        let mut heads = valid.clone();
        heads.content.base_heads = vec![ChangeHash::from_bytes([7; 32])];
        assert_eq!(
            validate_genesis(&heads),
            Err(ControlValidationError::BaseHeads)
        );
        let mut grants = valid.clone();
        grants.content.members.clear();
        assert_eq!(
            validate_genesis(&grants),
            Err(ControlValidationError::Grants)
        );
        let mut successor = valid;
        successor.content.successor = Some(DocumentCoordinate::new(
            ControllerPublicKey::from_bytes([5; 32]),
            DocumentId::from_bytes([6; 32]),
        ));
        assert_eq!(
            validate_genesis(&successor),
            Err(ControlValidationError::Terminal)
        );
    }
}
