use crate::carrier::checkpoint_descriptor::ValidatedCheckpointDescriptorCarrier;
use crate::control::reference_state::ReferencedControlState;
use crate::types::role::Role;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DescriptorControlOutcome {
    CanonicalAuthorized,
    Missing,
    Pending,
    Noncanonical,
    WrongKind,
    WrongCoordinate,
    StaticInvalid,
    DynamicInvalid,
    UnsupportedRevision,
    RoleDenied,
}

#[cfg(test)]
pub(crate) fn authorize_descriptor(
    descriptor: &ValidatedCheckpointDescriptorCarrier,
    state: ReferencedControlState<'_>,
) -> DescriptorControlOutcome {
    match authorize_descriptor_with(descriptor, state, &mut || {
        Ok::<(), core::convert::Infallible>(())
    }) {
        Ok(outcome) => outcome,
        Err(error) => match error {},
    }
}

pub(crate) fn authorize_descriptor_with<E>(
    descriptor: &ValidatedCheckpointDescriptorCarrier,
    state: ReferencedControlState<'_>,
    visit_member: &mut impl FnMut() -> Result<(), E>,
) -> Result<DescriptorControlOutcome, E> {
    match state {
        ReferencedControlState::Canonical(control) => {
            for grant in control.members() {
                visit_member()?;
                if grant.device == descriptor.author() && grant.roles.contains(&Role::Checkpoint) {
                    return Ok(DescriptorControlOutcome::CanonicalAuthorized);
                }
            }
            Ok(DescriptorControlOutcome::RoleDenied)
        }
        ReferencedControlState::Missing => Ok(DescriptorControlOutcome::Missing),
        ReferencedControlState::Pending(_) => Ok(DescriptorControlOutcome::Pending),
        ReferencedControlState::NoncanonicalValid(_) => Ok(DescriptorControlOutcome::Noncanonical),
        ReferencedControlState::WrongKind => Ok(DescriptorControlOutcome::WrongKind),
        ReferencedControlState::WrongCoordinate => Ok(DescriptorControlOutcome::WrongCoordinate),
        ReferencedControlState::StaticInvalid => Ok(DescriptorControlOutcome::StaticInvalid),
        ReferencedControlState::DynamicInvalid(_) => Ok(DescriptorControlOutcome::DynamicInvalid),
        ReferencedControlState::UnsupportedRevision => {
            Ok(DescriptorControlOutcome::UnsupportedRevision)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{DescriptorControlOutcome, authorize_descriptor};
    use crate::carrier::checkpoint_descriptor::ValidatedCheckpointDescriptorCarrier;
    use crate::carrier::control::{DeviceGrant, ValidatedControlCarrier, ValidatedControlContent};
    use crate::checkpoint::CheckpointDescriptor;
    use crate::control::reference_state::ReferencedControlState;
    use crate::types::role::Role;
    use crate::{
        ActorId, ChangeHash, ControllerPublicKey, DevicePublicKey, DocumentCoordinate, DocumentId,
        EventId, SnapshotHash,
    };

    #[test]
    fn descriptor_control_outcome_preserves_every_reference_family_and_role_denial() {
        let controller = ControllerPublicKey::from_bytes([1; 32]);
        let coordinate = DocumentCoordinate::new(controller, DocumentId::from_bytes([2; 32]));
        let control_id = EventId::from_bytes([3; 32]);
        let author = DevicePublicKey::from_bytes([4; 32]);
        let control = ValidatedControlCarrier::for_test(
            control_id,
            controller,
            coordinate,
            None,
            ValidatedControlContent {
                base_heads: Vec::new(),
                members: vec![DeviceGrant {
                    account: None,
                    actor: ActorId::derive(coordinate, author),
                    device: author,
                    roles: vec![Role::Checkpoint, Role::Write],
                }],
                predecessor: None,
                sequence: 0,
                successor: None,
                terminal: false,
            },
        );
        let descriptor = ValidatedCheckpointDescriptorCarrier::for_test(
            EventId::from_bytes([5; 32]),
            author,
            coordinate,
            control_id,
            CheckpointDescriptor {
                snapshot_hash: SnapshotHash::from_bytes([6; 32]),
                heads: BTreeSet::from([ChangeHash::from_bytes([7; 32])]),
                raw_size: 1,
                chunk_size: 1,
                chunk_count: 1,
                chunk_root: [8; 32],
                change_count: 1,
                change_set_hash: [9; 32],
                dependency_edges: 0,
                total_ops: 1,
            },
        );
        for (state, expected) in [
            (
                ReferencedControlState::Canonical(&control),
                DescriptorControlOutcome::CanonicalAuthorized,
            ),
            (
                ReferencedControlState::Missing,
                DescriptorControlOutcome::Missing,
            ),
            (
                ReferencedControlState::Pending(&control),
                DescriptorControlOutcome::Pending,
            ),
            (
                ReferencedControlState::NoncanonicalValid(&control),
                DescriptorControlOutcome::Noncanonical,
            ),
            (
                ReferencedControlState::WrongKind,
                DescriptorControlOutcome::WrongKind,
            ),
            (
                ReferencedControlState::WrongCoordinate,
                DescriptorControlOutcome::WrongCoordinate,
            ),
            (
                ReferencedControlState::StaticInvalid,
                DescriptorControlOutcome::StaticInvalid,
            ),
            (
                ReferencedControlState::DynamicInvalid(&control),
                DescriptorControlOutcome::DynamicInvalid,
            ),
            (
                ReferencedControlState::UnsupportedRevision,
                DescriptorControlOutcome::UnsupportedRevision,
            ),
        ] {
            assert_eq!(authorize_descriptor(&descriptor, state), expected);
        }

        let mut write_only = control.clone();
        write_only.set_test_roles(vec![Role::Write]);
        assert_eq!(
            authorize_descriptor(&descriptor, ReferencedControlState::Canonical(&write_only)),
            DescriptorControlOutcome::RoleDenied
        );
    }
}
