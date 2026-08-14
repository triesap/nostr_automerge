use crate::ProtocolDisposition;
use crate::carrier::checkpoint_descriptor::ValidatedCheckpointDescriptorCarrier;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReferencedDescriptorState<'a> {
    VerifiedTarget(&'a ValidatedCheckpointDescriptorCarrier),
    Pending(&'a ValidatedCheckpointDescriptorCarrier),
    Missing,
    WrongKind,
    WrongCoordinate,
    StaticInvalid,
    DynamicInvalid,
    UnsupportedRevision,
}

impl ReferencedDescriptorState<'_> {
    pub(crate) const fn dependent_disposition(self) -> Option<ProtocolDisposition> {
        match self {
            Self::VerifiedTarget(_) => None,
            Self::Pending(_) | Self::Missing => Some(ProtocolDisposition::Pending),
            Self::WrongKind
            | Self::WrongCoordinate
            | Self::StaticInvalid
            | Self::DynamicInvalid
            | Self::UnsupportedRevision => Some(ProtocolDisposition::Invalid),
        }
    }
}
