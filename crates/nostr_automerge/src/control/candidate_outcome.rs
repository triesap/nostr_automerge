use std::collections::BTreeSet;

use crate::{ChangeHash, DiagnosticCode, EventId, ProtocolDisposition};

/// Stateful result for one control candidate before canonical-child selection.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ControlCandidateOutcome {
    event_id: EventId,
    parent: Option<EventId>,
    sequence: u64,
    disposition: ProtocolDisposition,
    diagnostic: Option<DiagnosticCode>,
    validated_base_closure: Option<BTreeSet<ChangeHash>>,
}

impl ControlCandidateOutcome {
    pub(crate) fn valid(
        event_id: EventId,
        parent: Option<EventId>,
        sequence: u64,
        validated_base_closure: BTreeSet<ChangeHash>,
    ) -> Self {
        Self {
            event_id,
            parent,
            sequence,
            disposition: ProtocolDisposition::Accepted,
            diagnostic: None,
            validated_base_closure: Some(validated_base_closure),
        }
    }

    pub(crate) fn pending(
        event_id: EventId,
        parent: Option<EventId>,
        sequence: u64,
        diagnostic: DiagnosticCode,
        validated_base_closure: Option<BTreeSet<ChangeHash>>,
    ) -> Self {
        Self {
            event_id,
            parent,
            sequence,
            disposition: ProtocolDisposition::Pending,
            diagnostic: Some(diagnostic),
            validated_base_closure,
        }
    }

    pub(crate) fn invalid(
        event_id: EventId,
        parent: Option<EventId>,
        sequence: u64,
        diagnostic: DiagnosticCode,
        validated_base_closure: Option<BTreeSet<ChangeHash>>,
    ) -> Self {
        Self {
            event_id,
            parent,
            sequence,
            disposition: ProtocolDisposition::Invalid,
            diagnostic: Some(diagnostic),
            validated_base_closure,
        }
    }

    pub(crate) const fn event_id(&self) -> EventId {
        self.event_id
    }

    pub(crate) const fn parent(&self) -> Option<EventId> {
        self.parent
    }

    pub(crate) const fn sequence(&self) -> u64 {
        self.sequence
    }

    pub(crate) const fn disposition(&self) -> ProtocolDisposition {
        self.disposition
    }

    pub(crate) const fn diagnostic(&self) -> Option<DiagnosticCode> {
        self.diagnostic
    }

    pub(crate) fn validated_base_closure(&self) -> Option<&BTreeSet<ChangeHash>> {
        self.validated_base_closure.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::ControlCandidateOutcome;
    use crate::{ChangeHash, DiagnosticCode, EventId, ProtocolDisposition};

    #[test]
    fn orders_and_diagnoses_every_stateful_outcome() {
        let parent = EventId::from_bytes([9; 32]);
        let closure = BTreeSet::from([ChangeHash::from_bytes([8; 32])]);
        let valid = ControlCandidateOutcome::valid(
            EventId::from_bytes([3; 32]),
            Some(parent),
            2,
            closure.clone(),
        );
        let pending = ControlCandidateOutcome::pending(
            EventId::from_bytes([1; 32]),
            Some(parent),
            2,
            DiagnosticCode::registered("control.frontier"),
            None,
        );
        let invalid = ControlCandidateOutcome::invalid(
            EventId::from_bytes([2; 32]),
            Some(parent),
            2,
            DiagnosticCode::registered("control.account_changed"),
            Some(closure.clone()),
        );

        assert_eq!(valid.disposition(), ProtocolDisposition::Accepted);
        assert_eq!(valid.diagnostic(), None);
        assert_eq!(valid.validated_base_closure(), Some(&closure));
        assert_eq!(pending.disposition(), ProtocolDisposition::Pending);
        assert_eq!(
            pending.diagnostic(),
            Some(DiagnosticCode::registered("control.frontier"))
        );
        assert_eq!(invalid.disposition(), ProtocolDisposition::Invalid);
        assert_eq!(
            invalid.diagnostic(),
            Some(DiagnosticCode::registered("control.account_changed"))
        );

        let mut ordered = [valid, invalid, pending];
        ordered.sort();
        assert_eq!(
            ordered
                .iter()
                .map(ControlCandidateOutcome::event_id)
                .collect::<Vec<_>>(),
            vec![
                EventId::from_bytes([1; 32]),
                EventId::from_bytes([2; 32]),
                EventId::from_bytes([3; 32]),
            ]
        );
        assert!(
            ordered
                .iter()
                .all(|outcome| outcome.parent() == Some(parent))
        );
        assert!(ordered.iter().all(|outcome| outcome.sequence() == 2));
    }
}
