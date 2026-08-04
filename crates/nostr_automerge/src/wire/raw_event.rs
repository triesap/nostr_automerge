use core::fmt;

use crate::{DiagnosticCode, ProtocolRevision};

/// Bounded, valid UTF-8 raw NIP-01 bytes retained before JSON parsing.
#[derive(Clone, PartialEq, Eq)]
pub struct RawEventBytes(Box<str>);

impl RawEventBytes {
    /// Validates the sealed ingress bound and UTF-8 before copying bytes.
    pub fn new(input: &[u8], revision: ProtocolRevision) -> Result<Self, RawEventError> {
        let maximum = revision
            .limits()
            .raw_event
            .try_usize()
            .map_err(|_| RawEventError::TooLarge)?;
        if input.len() > maximum {
            return Err(RawEventError::TooLarge);
        }
        let validated = core::str::from_utf8(input).map_err(|_| RawEventError::InvalidUtf8)?;
        Ok(Self(validated.to_owned().into_boxed_str()))
    }

    /// Returns the retained exact signed bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Returns the already-validated UTF-8 view.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for RawEventBytes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RawEventBytes")
            .field("length", &self.0.len())
            .finish()
    }
}

/// Why raw event ingress failed before JSON parsing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RawEventError {
    /// The sealed raw event byte limit was exceeded.
    TooLarge,
    /// The bytes are not valid UTF-8.
    InvalidUtf8,
}

impl RawEventError {
    /// Returns the stable diagnostic code.
    #[must_use]
    pub fn diagnostic(self) -> DiagnosticCode {
        let code = match self {
            Self::TooLarge => "raw.too_large",
            Self::InvalidUtf8 => "raw.invalid_utf8",
        };
        DiagnosticCode::registered(code)
    }
}

impl fmt::Display for RawEventError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::TooLarge => "raw event exceeds the sealed byte limit",
            Self::InvalidUtf8 => "raw event is not valid UTF-8",
        })
    }
}

impl std::error::Error for RawEventError {}

#[cfg(test)]
mod tests {
    use super::{RawEventBytes, RawEventError};
    use crate::ProtocolRevision;

    #[test]
    fn validates_before_retention() {
        let raw = RawEventBytes::new(b"{}", ProtocolRevision::draft_v1());
        assert_eq!(raw.as_ref().map(RawEventBytes::as_str), Ok("{}"));
        assert_eq!(
            RawEventBytes::new(&[0xff], ProtocolRevision::draft_v1()),
            Err(RawEventError::InvalidUtf8)
        );
        let oversized = vec![b' '; 262_145];
        assert_eq!(
            RawEventBytes::new(&oversized, ProtocolRevision::draft_v1()),
            Err(RawEventError::TooLarge)
        );
    }
}
