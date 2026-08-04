use core::fmt;
use core::str::FromStr;

use crate::{ControllerPublicKey, DocumentId, HexError};

const MANIFEST_KIND: &str = "31624";

/// The immutable NIP-01 address of one draft-v1 document manifest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocumentCoordinate {
    controller: ControllerPublicKey,
    document_id: DocumentId,
}

impl DocumentCoordinate {
    /// Constructs a coordinate from its semantically typed components.
    #[must_use]
    pub const fn new(controller: ControllerPublicKey, document_id: DocumentId) -> Self {
        Self {
            controller,
            document_id,
        }
    }

    /// Returns the controller fixed by this coordinate.
    #[must_use]
    pub const fn controller(&self) -> ControllerPublicKey {
        self.controller
    }

    /// Returns the immutable document identifier.
    #[must_use]
    pub const fn document_id(&self) -> DocumentId {
        self.document_id
    }

    /// Renders the exact draft NIP-01 address without relay hints.
    #[must_use]
    pub fn to_address(self) -> String {
        format!(
            "{MANIFEST_KIND}:{}:{}",
            self.controller.to_hex(),
            self.document_id.to_hex()
        )
    }
}

impl FromStr for DocumentCoordinate {
    type Err = CoordinateError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut parts = value.split(':');
        let kind = parts.next().ok_or(CoordinateError::Shape)?;
        let controller = parts.next().ok_or(CoordinateError::Shape)?;
        let document = parts.next().ok_or(CoordinateError::Shape)?;
        if kind != MANIFEST_KIND || parts.next().is_some() {
            return Err(CoordinateError::Shape);
        }
        Ok(Self::new(controller.parse()?, document.parse()?))
    }
}

/// Why a printable document coordinate was rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoordinateError {
    /// The kind, separator count, or component shape was invalid.
    Shape,
    /// A coordinate identifier was not canonical lowercase hexadecimal.
    Identifier(HexError),
}

impl From<HexError> for CoordinateError {
    fn from(error: HexError) -> Self {
        Self::Identifier(error)
    }
}

impl fmt::Display for CoordinateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Shape => "document coordinate has an invalid shape",
            Self::Identifier(_) => "document coordinate contains an invalid identifier",
        })
    }
}

impl std::error::Error for CoordinateError {}

#[cfg(test)]
mod tests {
    use super::{CoordinateError, DocumentCoordinate};
    use core::str::FromStr;

    #[test]
    fn exact_address_roundtrips_without_relay_data() {
        let address = format!("31624:{}:{}", "11".repeat(32), "22".repeat(32));
        assert_eq!(
            DocumentCoordinate::from_str(&address).map(DocumentCoordinate::to_address),
            Ok(address)
        );
    }

    #[test]
    fn rejects_other_kinds_and_extra_parts() {
        assert_eq!(
            DocumentCoordinate::from_str("1:a:b"),
            Err(CoordinateError::Shape)
        );
        assert_eq!(
            DocumentCoordinate::from_str("31624:a:b:relay"),
            Err(CoordinateError::Shape)
        );
    }
}
