pub(crate) mod change;
pub(crate) mod classify;
pub(crate) mod control;
pub(crate) mod manifest;
pub(crate) mod version;

use crate::VerifiedNip01Event;

use self::change::ChangeCarrier;
use self::control::ValidatedControlCarrier;
use self::manifest::ValidatedManifest;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CarrierCandidate {
    Manifest(VerifiedNip01Event),
    Control(VerifiedNip01Event),
    Change(VerifiedNip01Event),
    CheckpointDescriptor(VerifiedNip01Event),
    CheckpointChunk(VerifiedNip01Event),
    UnsupportedRevision {
        event: VerifiedNip01Event,
        declared_version: Option<u64>,
        declared_profile: Option<String>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum VerifiedCarrier {
    Manifest(Box<ValidatedManifest>),
    Control(Box<ValidatedControlCarrier>),
    Change(Box<ChangeCarrier>),
    CheckpointDescriptor(VerifiedNip01Event),
    CheckpointChunk(VerifiedNip01Event),
    UnsupportedRevision {
        event: VerifiedNip01Event,
        declared_version: Option<u64>,
        declared_profile: Option<String>,
    },
}
