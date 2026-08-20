use std::collections::{BTreeMap, BTreeSet};

use crate::carrier::VerifiedCarrier;
use crate::carrier::control::ValidatedControlCarrier;
use crate::evidence::corpus_builder::EvidenceCorpus;
use crate::evidence::event::EventEvidence;
use crate::{DiagnosticCode, DocumentCoordinate, EventId, ProtocolDisposition};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReferencedControlState<'a> {
    Canonical(&'a ValidatedControlCarrier),
    NoncanonicalValid(&'a ValidatedControlCarrier),
    Pending(&'a ValidatedControlCarrier),
    Missing,
    WrongKind,
    WrongCoordinate,
    StaticInvalid,
    DynamicInvalid(&'a ValidatedControlCarrier),
    UnsupportedRevision,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ControlParentState<'a> {
    Canonical(&'a ValidatedControlCarrier),
    NoncanonicalValid(&'a ValidatedControlCarrier),
    Pending(&'a ValidatedControlCarrier),
    Missing,
    WrongKind,
    WrongCoordinate,
    StaticInvalid,
    DynamicInvalid(&'a ValidatedControlCarrier),
    UnsupportedRevision,
}

impl<'a> From<ReferencedControlState<'a>> for ControlParentState<'a> {
    fn from(state: ReferencedControlState<'a>) -> Self {
        match state {
            ReferencedControlState::Canonical(control) => Self::Canonical(control),
            ReferencedControlState::NoncanonicalValid(control) => Self::NoncanonicalValid(control),
            ReferencedControlState::Pending(control) => Self::Pending(control),
            ReferencedControlState::Missing => Self::Missing,
            ReferencedControlState::WrongKind => Self::WrongKind,
            ReferencedControlState::WrongCoordinate => Self::WrongCoordinate,
            ReferencedControlState::StaticInvalid => Self::StaticInvalid,
            ReferencedControlState::DynamicInvalid(control) => Self::DynamicInvalid(control),
            ReferencedControlState::UnsupportedRevision => Self::UnsupportedRevision,
        }
    }
}

impl ControlParentState<'_> {
    pub(crate) const fn dependent_disposition(self) -> Option<ProtocolDisposition> {
        match self {
            Self::Canonical(_) | Self::NoncanonicalValid(_) => None,
            Self::Pending(_) | Self::Missing => Some(ProtocolDisposition::Pending),
            Self::WrongKind
            | Self::WrongCoordinate
            | Self::StaticInvalid
            | Self::DynamicInvalid(_)
            | Self::UnsupportedRevision => Some(ProtocolDisposition::Invalid),
        }
    }
}

impl ReferencedControlState<'_> {
    pub(crate) const fn diagnostic(self) -> DiagnosticCode {
        let code = match self {
            Self::Canonical(_) | Self::NoncanonicalValid(_) => "control.parent",
            Self::Pending(_) => "control.frontier",
            Self::Missing => "control.parent",
            Self::WrongKind => "carrier.kind",
            Self::WrongCoordinate => "carrier.coordinate",
            Self::StaticInvalid => "control.structure",
            Self::DynamicInvalid(_) => "control.parent",
            Self::UnsupportedRevision => "carrier.revision",
        };
        DiagnosticCode::registered(code)
    }
}

pub(crate) fn resolve_referenced_control<'a>(
    corpus: &'a EvidenceCorpus,
    event_id: EventId,
    coordinate: DocumentCoordinate,
    dispositions: &BTreeMap<EventId, ProtocolDisposition>,
    statefully_valid: &BTreeSet<EventId>,
) -> ReferencedControlState<'a> {
    let Some(evidence) = corpus.events.get(&event_id) else {
        return ReferencedControlState::Missing;
    };
    let control = match evidence {
        EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::Control(control),
            ..
        } => control.as_ref(),
        EventEvidence::UnsupportedRevision { .. }
        | EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::UnsupportedRevision { .. },
            ..
        } => return ReferencedControlState::UnsupportedRevision,
        EventEvidence::InvalidCarrier { event, .. } if event.kind() == 1_625 => {
            return ReferencedControlState::StaticInvalid;
        }
        EventEvidence::InvalidCarrier { .. }
        | EventEvidence::VerifiedCarrier { .. }
        | EventEvidence::IrrelevantEvent { .. }
        | EventEvidence::InvalidEvent { .. }
        | EventEvidence::DuplicateEvent { .. } => return ReferencedControlState::WrongKind,
    };
    if control.coordinate() != coordinate {
        return ReferencedControlState::WrongCoordinate;
    }
    match dispositions.get(&event_id).copied() {
        Some(ProtocolDisposition::Accepted) => ReferencedControlState::Canonical(control),
        Some(ProtocolDisposition::Excluded) if statefully_valid.contains(&event_id) => {
            ReferencedControlState::NoncanonicalValid(control)
        }
        Some(ProtocolDisposition::Pending) | None => ReferencedControlState::Pending(control),
        Some(
            ProtocolDisposition::Invalid
            | ProtocolDisposition::Excluded
            | ProtocolDisposition::UnsupportedRevision,
        ) => ReferencedControlState::DynamicInvalid(control),
    }
}

#[cfg(test)]
mod tests {
    use super::{ControlParentState, ReferencedControlState};
    use crate::carrier::control::{ValidatedControlCarrier, ValidatedControlContent};
    use crate::{
        ControllerPublicKey, DocumentCoordinate, DocumentId, EventId, ProtocolDisposition,
    };

    fn control() -> ValidatedControlCarrier {
        let controller = ControllerPublicKey::from_bytes([1; 32]);
        ValidatedControlCarrier::for_test(
            EventId::from_bytes([3; 32]),
            controller,
            DocumentCoordinate::new(controller, DocumentId::from_bytes([2; 32])),
            None,
            ValidatedControlContent {
                base_heads: Vec::new(),
                members: Vec::new(),
                predecessor: None,
                sequence: 0,
                successor: None,
                terminal: true,
            },
        )
    }

    #[test]
    fn every_state_has_a_stable_diagnostic() {
        assert_eq!(
            ReferencedControlState::Missing.diagnostic().as_str(),
            "control.parent"
        );
        assert_eq!(
            ReferencedControlState::WrongKind.diagnostic().as_str(),
            "carrier.kind"
        );
        assert_eq!(
            ReferencedControlState::WrongCoordinate
                .diagnostic()
                .as_str(),
            "carrier.coordinate"
        );
        assert_eq!(
            ReferencedControlState::StaticInvalid.diagnostic().as_str(),
            "control.structure"
        );
        assert_eq!(
            ReferencedControlState::UnsupportedRevision
                .diagnostic()
                .as_str(),
            "carrier.revision"
        );
        assert_eq!(
            ReferencedControlState::DynamicInvalid(&control())
                .diagnostic()
                .as_str(),
            "control.parent"
        );
    }

    #[test]
    fn every_parent_state_has_an_exhaustive_dependent_outcome() {
        let control = control();
        let cases = [
            (ControlParentState::Canonical(&control), None),
            (ControlParentState::NoncanonicalValid(&control), None),
            (
                ControlParentState::Pending(&control),
                Some(ProtocolDisposition::Pending),
            ),
            (
                ControlParentState::Missing,
                Some(ProtocolDisposition::Pending),
            ),
            (
                ControlParentState::WrongKind,
                Some(ProtocolDisposition::Invalid),
            ),
            (
                ControlParentState::WrongCoordinate,
                Some(ProtocolDisposition::Invalid),
            ),
            (
                ControlParentState::StaticInvalid,
                Some(ProtocolDisposition::Invalid),
            ),
            (
                ControlParentState::DynamicInvalid(&control),
                Some(ProtocolDisposition::Invalid),
            ),
            (
                ControlParentState::UnsupportedRevision,
                Some(ProtocolDisposition::Invalid),
            ),
        ];
        for (state, expected) in cases {
            assert_eq!(state.dependent_disposition(), expected);
        }
    }

    #[test]
    fn present_wrong_kind_parent_is_invalid() {
        assert_eq!(
            ControlParentState::WrongKind.dependent_disposition(),
            Some(ProtocolDisposition::Invalid)
        );
    }

    #[test]
    fn present_wrong_coordinate_parent_is_invalid() {
        assert_eq!(
            ControlParentState::WrongCoordinate.dependent_disposition(),
            Some(ProtocolDisposition::Invalid)
        );
    }

    #[test]
    fn present_statically_invalid_parent_is_invalid() {
        assert_eq!(
            ControlParentState::StaticInvalid.dependent_disposition(),
            Some(ProtocolDisposition::Invalid)
        );
    }

    #[test]
    fn present_unsupported_parent_is_invalid() {
        assert_eq!(
            ControlParentState::UnsupportedRevision.dependent_disposition(),
            Some(ProtocolDisposition::Invalid)
        );
    }
}
