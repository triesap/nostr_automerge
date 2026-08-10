use crate::carrier::control::ValidatedControlContent;
use crate::{ControllerPublicKey, DocumentCoordinate, EventId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ControlEnvelope {
    pub(super) event_id: EventId,
    pub(super) author: ControllerPublicKey,
    pub(super) coordinate: DocumentCoordinate,
    pub(super) parent: Option<EventId>,
    pub(super) content: ValidatedControlContent,
}

impl ControlEnvelope {
    pub(crate) fn from_validated(
        carrier: crate::carrier::control::ValidatedControlCarrier,
    ) -> Self {
        let (event_id, author, coordinate, parent, content) = carrier.into_parts();
        Self {
            event_id,
            author,
            coordinate,
            parent,
            content,
        }
    }

    pub(crate) const fn sequence(&self) -> u64 {
        self.content.sequence
    }

    pub(crate) const fn event_id(&self) -> EventId {
        self.event_id
    }

    pub(crate) const fn parent(&self) -> Option<EventId> {
        self.parent
    }

    pub(crate) fn base_heads(&self) -> impl Iterator<Item = crate::ChangeHash> + '_ {
        self.content.base_heads.iter().copied()
    }

    pub(crate) const fn content(&self) -> &ValidatedControlContent {
        &self.content
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ControlValidationError {
    Author,
    Parent,
    Sequence,
    BaseHeads,
    Grants,
    Terminal,
    CanonicalCollections,
}

pub(crate) fn validate_canonical_collections(
    content: &ValidatedControlContent,
) -> Result<(), ControlValidationError> {
    if !content.base_heads.windows(2).all(|pair| pair[0] < pair[1])
        || !content
            .members
            .windows(2)
            .all(|pair| pair[0].device < pair[1].device)
        || content
            .members
            .iter()
            .any(|grant| grant.roles.is_empty() || !grant.roles.windows(2).all(|p| p[0] < p[1]))
    {
        return Err(ControlValidationError::CanonicalCollections);
    }
    Ok(())
}

pub(crate) fn validate_base_frontier(
    content: &ValidatedControlContent,
    genesis: bool,
) -> Result<(), ControlValidationError> {
    let limit = crate::ProtocolRevision::draft_v1()
        .limits()
        .control_heads
        .try_usize()
        .map_err(|_| ControlValidationError::BaseHeads)?;
    if content.base_heads.len() > limit
        || genesis && !content.base_heads.is_empty()
        || !content.base_heads.windows(2).all(|pair| pair[0] < pair[1])
    {
        return Err(ControlValidationError::BaseHeads);
    }
    Ok(())
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
    if control.content.members.is_empty() && !control.content.terminal {
        return Err(ControlValidationError::Grants);
    }
    if control.content.successor.is_some() && !control.content.terminal {
        return Err(ControlValidationError::Terminal);
    }
    validate_canonical_collections(&control.content)?;
    validate_base_frontier(&control.content, true)?;
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{
        ControlEnvelope, ControlValidationError, validate_base_frontier,
        validate_canonical_collections, validate_genesis,
    };
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
        grants.content.terminal = true;
        assert_eq!(validate_genesis(&grants), Ok(()));
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

    #[test]
    fn control_envelope_derives_every_field_from_validated_carrier() {
        let expected = genesis();
        let carrier = crate::carrier::control::ValidatedControlCarrier::for_test(
            expected.event_id,
            expected.author,
            expected.coordinate,
            expected.parent,
            expected.content.clone(),
        );
        assert_eq!(ControlEnvelope::from_validated(carrier), expected);
    }

    #[test]
    fn validate_canonical_ordering_and_uniqueness_fields() {
        let mut control = genesis();
        control.content.base_heads = vec![
            ChangeHash::from_bytes([2; 32]),
            ChangeHash::from_bytes([1; 32]),
        ];
        assert_eq!(
            validate_canonical_collections(&control.content),
            Err(ControlValidationError::CanonicalCollections)
        );
        control.content.base_heads = vec![ChangeHash::from_bytes([1; 32]); 2];
        assert_eq!(
            validate_canonical_collections(&control.content),
            Err(ControlValidationError::CanonicalCollections)
        );
        control.content.base_heads.clear();
        control.content.members = vec![grant(5, vec![Role::Write]), grant(4, vec![Role::Write])];
        assert_eq!(
            validate_canonical_collections(&control.content),
            Err(ControlValidationError::CanonicalCollections)
        );
        control.content.members = vec![grant(4, vec![Role::Write, Role::Checkpoint])];
        assert_eq!(
            validate_canonical_collections(&control.content),
            Err(ControlValidationError::CanonicalCollections)
        );
        control.content.members = vec![grant(4, vec![Role::Checkpoint, Role::Write])];
        assert_eq!(validate_canonical_collections(&control.content), Ok(()));
    }

    #[test]
    fn validate_base_frontier_shape() {
        let mut content = genesis().content;
        content.base_heads = (0_u8..64)
            .map(|byte| ChangeHash::from_bytes([byte; 32]))
            .collect();
        assert_eq!(validate_base_frontier(&content, false), Ok(()));
        assert_eq!(
            validate_base_frontier(&content, true),
            Err(ControlValidationError::BaseHeads)
        );
        content.base_heads.push(ChangeHash::from_bytes([64; 32]));
        assert_eq!(
            validate_base_frontier(&content, false),
            Err(ControlValidationError::BaseHeads)
        );
        content.base_heads = vec![
            ChangeHash::from_bytes([2; 32]),
            ChangeHash::from_bytes([1; 32]),
        ];
        assert_eq!(
            validate_base_frontier(&content, false),
            Err(ControlValidationError::BaseHeads)
        );
        content.base_heads = vec![ChangeHash::from_bytes([1; 32]); 2];
        assert_eq!(
            validate_base_frontier(&content, false),
            Err(ControlValidationError::BaseHeads)
        );
        content.base_heads.clear();
        assert_eq!(validate_base_frontier(&content, false), Ok(()));
    }
}
