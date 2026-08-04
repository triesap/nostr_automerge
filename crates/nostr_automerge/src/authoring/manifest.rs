use std::collections::BTreeSet;

use serde_json::json;

use crate::{DocumentCoordinate, EventId, ProtocolRevision};

/// Advisory application metadata carried by a manifest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApplicationMetadata {
    /// Printable application identifier.
    pub id: String,
    /// Application schema version.
    pub schema_version: String,
    /// Optional SHA-256 schema identity.
    pub schema_hash: Option<[u8; 32]>,
}

/// Discovery status advertised by a manifest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManifestStatus {
    /// Normal discovery hint.
    Active,
    /// Discovery hint for a frozen document.
    Frozen,
    /// Discovery hint pointing at a successor document.
    Superseded,
}

impl ManifestStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Frozen => "frozen",
            Self::Superseded => "superseded",
        }
    }
}

/// Canonical advisory manifest content awaiting carrier construction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestDraft {
    content: String,
}

/// Why manifest inputs could not form sealed draft-v1 content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManifestDraftError {
    /// A relay, label, or application field exceeded the profile.
    InvalidMetadata,
    /// Status and successor did not describe one valid discovery state.
    InvalidStatus,
    /// Canonical serialization failed.
    Serialization,
}

impl ManifestDraft {
    /// Builds deterministic JCS from advisory discovery inputs.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        control: EventId,
        checkpoint: Option<EventId>,
        relays: BTreeSet<String>,
        name: Option<String>,
        description: Option<String>,
        application: Option<ApplicationMetadata>,
        status: ManifestStatus,
        successor: Option<DocumentCoordinate>,
    ) -> Result<Self, ManifestDraftError> {
        if relays.len() > 16
            || relays
                .iter()
                .any(|relay| crate::wire::scalars::validate_relay_url(relay).is_err())
            || name
                .as_deref()
                .is_some_and(|value| crate::wire::scalars::validate_utf8_bytes(value, 256).is_err())
            || description.as_deref().is_some_and(|value| {
                crate::wire::scalars::validate_utf8_bytes(value, 2_048).is_err()
            })
            || application.as_ref().is_some_and(|value| {
                crate::wire::scalars::validate_printable_ascii(&value.id, 128).is_err()
                    || crate::wire::scalars::validate_utf8_bytes(&value.schema_version, 64).is_err()
            })
        {
            return Err(ManifestDraftError::InvalidMetadata);
        }
        if (status == ManifestStatus::Superseded) != successor.is_some() {
            return Err(ManifestDraftError::InvalidStatus);
        }
        let application = application.map(|value| {
            json!({
                "id": value.id,
                "schema_hash": value.schema_hash.map(|hash| crate::wire::hex::encode_bytes(&hash)),
                "schema_version": value.schema_version,
            })
        });
        let value = json!({
            "application": application,
            "checkpoint": checkpoint.map(EventId::to_hex),
            "control": control.to_hex(),
            "description": description,
            "format": ProtocolRevision::draft_v1().format(),
            "name": name,
            "relays": relays.into_iter().collect::<Vec<_>>(),
            "status": status.as_str(),
            "successor": successor.map(DocumentCoordinate::to_address),
            "text_encoding": ProtocolRevision::draft_v1().text_encoding(),
            "v": 1,
        });
        let bytes = crate::wire::canonical_json::serialize::to_vec(&value)
            .map_err(|_| ManifestDraftError::Serialization)?;
        let content = String::from_utf8(bytes).map_err(|_| ManifestDraftError::Serialization)?;
        Ok(Self { content })
    }

    /// Returns exact JCS manifest content.
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{ApplicationMetadata, ManifestDraft, ManifestStatus};
    use crate::EventId;

    #[test]
    fn build_advisory_manifest_content() {
        let draft = ManifestDraft::new(
            EventId::from_bytes([1; 32]),
            Some(EventId::from_bytes([2; 32])),
            BTreeSet::from(["wss://z.example/".to_owned(), "wss://a.example/".to_owned()]),
            Some("Document".to_owned()),
            None,
            Some(ApplicationMetadata {
                id: "example.app".to_owned(),
                schema_version: "1".to_owned(),
                schema_hash: Some([3; 32]),
            }),
            ManifestStatus::Active,
            None,
        );
        assert!(draft.is_ok());
        let Ok(draft) = draft else { return };
        assert!(crate::carrier::manifest::parse_content(draft.content()).is_ok());
        assert!(
            draft.content().find("wss://a.example/") < draft.content().find("wss://z.example/")
        );
    }
}
