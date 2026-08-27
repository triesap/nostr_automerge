use core::fmt;

use crate::{ActorId, ChangeHash, DiagnosticCode, EventId};

/// A closed integrity alert emitted without changing canonical selection rules.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IntegrityAlert {
    /// More than one otherwise-valid controller child exists.
    ControllerEquivocation(ControllerEquivocationAlert),
    /// Late evidence changed the canonical control chain.
    CanonicalControlReorganization(CanonicalControlReorganizationAlert),
    /// One actor emitted distinct changes at the same sequence.
    DeviceEquivocation(DeviceEquivocationAlert),
    /// Carrier evidence suggests one device key may be cloned.
    PotentialClonedDeviceKey(PotentialClonedDeviceKeyAlert),
    /// Verified checkpoint commitments disagree.
    CheckpointMismatch(CheckpointMismatchAlert),
}

/// Validated controller-sibling equivocation details.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ControllerEquivocationAlert {
    parent_control: Option<EventId>,
    candidate_controls: Vec<EventId>,
    selected_control: EventId,
}

impl ControllerEquivocationAlert {
    /// Constructs details from a strictly sorted candidate set containing selection.
    pub fn new(
        parent_control: Option<EventId>,
        candidate_controls: Vec<EventId>,
        selected_control: EventId,
    ) -> Result<Self, AlertError> {
        canonical(&candidate_controls, 2)?;
        if candidate_controls.binary_search(&selected_control).is_err() {
            return Err(AlertError);
        }
        Ok(Self {
            parent_control,
            candidate_controls,
            selected_control,
        })
    }

    /// Constructs an alert from an engine-owned ordered candidate traversal.
    pub(crate) fn from_validated_parts(
        parent_control: Option<EventId>,
        candidate_controls: Vec<EventId>,
        selected_control: EventId,
    ) -> Self {
        Self {
            parent_control,
            candidate_controls,
            selected_control,
        }
    }

    /// Returns the canonical candidate controls.
    #[must_use]
    pub fn candidate_controls(&self) -> &[EventId] {
        &self.candidate_controls
    }

    /// Returns the parent, or none for competing genesis controls.
    #[must_use]
    pub const fn parent_control(&self) -> Option<EventId> {
        self.parent_control
    }

    /// Returns the decoded-byte-lowest selected child.
    #[must_use]
    pub const fn selected_control(&self) -> EventId {
        self.selected_control
    }
}

/// Validated canonical-control reorganization details.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonicalControlReorganizationAlert {
    previous_tip: EventId,
    new_tip: EventId,
    affected_changes: Vec<ChangeHash>,
}

impl CanonicalControlReorganizationAlert {
    /// Constructs details with a strictly sorted affected-change set.
    pub fn new(
        previous_tip: EventId,
        new_tip: EventId,
        affected_changes: Vec<ChangeHash>,
    ) -> Result<Self, AlertError> {
        canonical(&affected_changes, 0)?;
        if previous_tip == new_tip {
            return Err(AlertError);
        }
        Ok(Self {
            previous_tip,
            new_tip,
            affected_changes,
        })
    }

    /// Constructs an alert after the engine has validated these exact fields.
    ///
    /// This crate-private boundary deliberately performs no validation traversal:
    /// reevaluation validates each relationship comparison immediately after its
    /// work charge. Public callers must use [`Self::new`].
    pub(crate) fn from_validated_parts(
        previous_tip: EventId,
        new_tip: EventId,
        affected_changes: Vec<ChangeHash>,
    ) -> Self {
        Self {
            previous_tip,
            new_tip,
            affected_changes,
        }
    }

    /// Returns the previously selected tip.
    #[must_use]
    pub const fn previous_tip(&self) -> EventId {
        self.previous_tip
    }

    /// Returns the newly selected tip.
    #[must_use]
    pub const fn new_tip(&self) -> EventId {
        self.new_tip
    }

    /// Returns canonically sorted changes affected by the reorganization.
    #[must_use]
    pub fn affected_changes(&self) -> &[ChangeHash] {
        &self.affected_changes
    }
}

/// Validated device-equivocation and descendant-quarantine details.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeviceEquivocationAlert {
    actor_id: ActorId,
    first_sequence: u64,
    conflicting_changes: Vec<ChangeHash>,
    affected_descendants: Vec<ChangeHash>,
}

impl DeviceEquivocationAlert {
    /// Constructs details from canonical sets at a nonzero actor sequence.
    pub fn new(
        actor_id: ActorId,
        first_sequence: u64,
        conflicting_changes: Vec<ChangeHash>,
        affected_descendants: Vec<ChangeHash>,
    ) -> Result<Self, AlertError> {
        if first_sequence == 0 {
            return Err(AlertError);
        }
        canonical(&conflicting_changes, 2)?;
        canonical(&affected_descendants, 0)?;
        Ok(Self {
            actor_id,
            first_sequence,
            conflicting_changes,
            affected_descendants,
        })
    }

    /// Constructs an alert from engine-owned canonical set projections.
    ///
    /// This crate-private boundary performs no collection traversal. The
    /// quarantine engine validates the nonzero sequence and derives both
    /// vectors from ordered sets under immediately preceding work charges.
    pub(crate) fn from_validated_parts(
        actor_id: ActorId,
        first_sequence: u64,
        conflicting_changes: Vec<ChangeHash>,
        affected_descendants: Vec<ChangeHash>,
    ) -> Self {
        Self {
            actor_id,
            first_sequence,
            conflicting_changes,
            affected_descendants,
        }
    }

    /// Returns the equivocated actor.
    #[must_use]
    pub const fn actor_id(&self) -> ActorId {
        self.actor_id
    }

    /// Returns the first conflicting nonzero sequence.
    #[must_use]
    pub const fn first_sequence(&self) -> u64 {
        self.first_sequence
    }

    /// Returns the canonical conflicting change set.
    #[must_use]
    pub fn conflicting_changes(&self) -> &[ChangeHash] {
        &self.conflicting_changes
    }

    /// Returns the canonical quarantined descendant set.
    #[must_use]
    pub fn affected_descendants(&self) -> &[ChangeHash] {
        &self.affected_descendants
    }
}

/// Validated carrier evidence for a potentially cloned device key.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PotentialClonedDeviceKeyAlert {
    actor_id: ActorId,
    first_sequence: u64,
    carrier_event_ids: Vec<EventId>,
}

impl PotentialClonedDeviceKeyAlert {
    /// Constructs details from a nonzero sequence and canonical carrier set.
    pub fn new(
        actor_id: ActorId,
        first_sequence: u64,
        carrier_event_ids: Vec<EventId>,
    ) -> Result<Self, AlertError> {
        if first_sequence == 0 {
            return Err(AlertError);
        }
        canonical(&carrier_event_ids, 2)?;
        Ok(Self {
            actor_id,
            first_sequence,
            carrier_event_ids,
        })
    }

    /// Returns the actor associated with the device key.
    #[must_use]
    pub const fn actor_id(&self) -> ActorId {
        self.actor_id
    }

    /// Returns the first suspicious sequence.
    #[must_use]
    pub const fn first_sequence(&self) -> u64 {
        self.first_sequence
    }

    /// Returns canonical supporting carrier event identifiers.
    #[must_use]
    pub fn carrier_event_ids(&self) -> &[EventId] {
        &self.carrier_event_ids
    }
}

/// Validated checkpoint mismatch details.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CheckpointMismatchAlert {
    descriptor_event_id: EventId,
    code: DiagnosticCode,
}

impl CheckpointMismatchAlert {
    /// Constructs mismatch details using a stable checkpoint diagnostic.
    pub fn new(descriptor_event_id: EventId, code: DiagnosticCode) -> Result<Self, AlertError> {
        if !code.as_str().starts_with("checkpoint.") {
            return Err(AlertError);
        }
        Ok(Self {
            descriptor_event_id,
            code,
        })
    }

    /// Returns the checkpoint descriptor event.
    #[must_use]
    pub const fn descriptor_event_id(&self) -> EventId {
        self.descriptor_event_id
    }

    /// Returns the stable checkpoint diagnostic.
    #[must_use]
    pub const fn code(&self) -> DiagnosticCode {
        self.code
    }
}

/// Integrity alert fields were not canonical or internally consistent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AlertError;

impl fmt::Display for AlertError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("integrity alert is not canonical")
    }
}

impl std::error::Error for AlertError {}

fn canonical<T: Ord>(values: &[T], minimum: usize) -> Result<(), AlertError> {
    if values.len() < minimum || !values.windows(2).all(|pair| pair[0] < pair[1]) {
        return Err(AlertError);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ControllerEquivocationAlert, DeviceEquivocationAlert};
    use crate::{ActorId, ChangeHash, EventId};

    #[test]
    fn alerts_reject_noncanonical_sets_and_zero_sequence() {
        let id = EventId::from_bytes([1; 32]);
        assert!(ControllerEquivocationAlert::new(None, vec![id, id], id).is_err());
        assert!(
            DeviceEquivocationAlert::new(
                ActorId::from_bytes([2; 32]),
                0,
                vec![
                    ChangeHash::from_bytes([1; 32]),
                    ChangeHash::from_bytes([2; 32])
                ],
                vec![]
            )
            .is_err()
        );
    }
}
