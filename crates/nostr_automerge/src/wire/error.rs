use core::fmt;

use crate::{DiagnosticCode, Nip01VerificationError, RawEventError};

use super::base64::Base64Error;
use super::canonical_json::parse::CanonicalJsonError;
use super::scalars::ScalarError;
use super::tags::TagError;

/// A privacy-safe stable wire diagnostic independent of human error text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WireDiagnostic {
    code: DiagnosticCode,
}

impl WireDiagnostic {
    /// Maps a raw-ingress failure to its stable code.
    #[must_use]
    pub fn from_raw(error: RawEventError) -> Self {
        Self {
            code: error.diagnostic(),
        }
    }

    /// Maps a complete strict NIP-01 failure to its stable code.
    #[must_use]
    pub fn from_nip01(error: Nip01VerificationError) -> Self {
        let code = match error {
            Nip01VerificationError::JsonSyntax | Nip01VerificationError::Serialization => {
                "json.syntax"
            }
            Nip01VerificationError::DuplicateMember => "json.duplicate_member",
            Nip01VerificationError::Shape => "nip01.shape",
            Nip01VerificationError::Identifier => "nip01.identifier",
            Nip01VerificationError::EventIdMismatch => "nip01.event_id",
            Nip01VerificationError::InvalidPublicKey | Nip01VerificationError::InvalidSignature => {
                "nip01.signature"
            }
        };
        Self::registered(code)
    }

    /// Returns the stable code suitable for canonical diagnostic output.
    #[must_use]
    pub const fn code(self) -> DiagnosticCode {
        self.code
    }

    pub(crate) const fn registered(code: &'static str) -> Self {
        Self {
            code: DiagnosticCode::registered(code),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_base64(error: Base64Error) -> Self {
        let _ = error;
        Self::registered("base64.noncanonical")
    }

    #[allow(dead_code)]
    pub(crate) fn from_canonical_json(error: CanonicalJsonError) -> Self {
        match error {
            CanonicalJsonError::DuplicateMember => Self::registered("json.duplicate_member"),
            CanonicalJsonError::Syntax => Self::registered("json.syntax"),
            CanonicalJsonError::TooLarge
            | CanonicalJsonError::Number
            | CanonicalJsonError::NonCanonical => Self::registered("jcs.noncanonical"),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_tag(error: TagError) -> Self {
        match error {
            TagError::Forbidden => Self::registered("tag.forbidden"),
            TagError::Missing
            | TagError::Repeated
            | TagError::ElementCount
            | TagError::NonCanonicalOrder => Self::registered("tag.required"),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_scalar(error: ScalarError) -> Self {
        let _ = error;
        Self::registered("carrier.coordinate")
    }
}

impl fmt::Display for WireDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::WireDiagnostic;
    use crate::wire::base64::Base64Error;
    use crate::wire::canonical_json::parse::CanonicalJsonError;
    use crate::wire::scalars::ScalarError;
    use crate::wire::tags::TagError;
    use crate::{Nip01VerificationError, RawEventError};

    #[test]
    fn every_wire_family_maps_without_human_string_matching() {
        assert_eq!(
            WireDiagnostic::from_raw(RawEventError::TooLarge)
                .code()
                .as_str(),
            "raw.too_large"
        );
        assert_eq!(
            WireDiagnostic::from_nip01(Nip01VerificationError::EventIdMismatch)
                .code()
                .as_str(),
            "nip01.event_id"
        );
        assert_eq!(
            WireDiagnostic::from_base64(Base64Error::NonCanonical)
                .code()
                .as_str(),
            "base64.noncanonical"
        );
        assert_eq!(
            WireDiagnostic::from_canonical_json(CanonicalJsonError::DuplicateMember)
                .code()
                .as_str(),
            "json.duplicate_member"
        );
        assert_eq!(
            WireDiagnostic::from_tag(TagError::Forbidden)
                .code()
                .as_str(),
            "tag.forbidden"
        );
        assert_eq!(
            WireDiagnostic::from_scalar(ScalarError::Url)
                .code()
                .as_str(),
            "carrier.coordinate"
        );
    }
}
