use serde_json::{Map, Value};

use crate::wire::canonical_json::parse::{CanonicalJsonError, parse_canonical};
use crate::wire::{hex, scalars, tags};
use crate::{
    ControllerPublicKey, DocumentCoordinate, DocumentId, EventId, ProtocolRevision,
    VerifiedNip01Event,
};

const MANIFEST_FIELDS: &[&str] = &[
    "application",
    "checkpoint",
    "control",
    "description",
    "format",
    "name",
    "relays",
    "status",
    "successor",
    "text_encoding",
    "v",
];
const APPLICATION_FIELDS: &[&str] = &["id", "schema_hash", "schema_version"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManifestContent {
    pub(crate) application: Option<Application>,
    pub(crate) checkpoint: Option<String>,
    pub(crate) control: String,
    pub(crate) description: Option<String>,
    pub(crate) format: String,
    pub(crate) name: Option<String>,
    pub(crate) relays: Vec<String>,
    pub(crate) status: String,
    pub(crate) successor: Option<String>,
    pub(crate) text_encoding: String,
    pub(crate) version: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Application {
    pub(crate) id: String,
    pub(crate) schema_hash: Option<String>,
    pub(crate) schema_version: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ValidatedManifest {
    pub(crate) event_id: EventId,
    pub(crate) created_at: u64,
    pub(crate) coordinate: DocumentCoordinate,
    pub(crate) application: Option<ValidatedApplication>,
    pub(crate) checkpoint: Option<EventId>,
    pub(crate) control: EventId,
    pub(crate) description: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) relays: Vec<String>,
    pub(crate) status: ManifestStatus,
    pub(crate) successor: Option<DocumentCoordinate>,
}

impl ValidatedManifest {
    pub(crate) const fn created_at(&self) -> u64 {
        self.created_at
    }

    pub(crate) const fn coordinate(&self) -> DocumentCoordinate {
        self.coordinate
    }

    pub(crate) fn acquisition_hints(&self) -> crate::ManifestHints {
        crate::ManifestHints::new(
            self.event_id,
            self.coordinate,
            self.control,
            self.checkpoint,
            self.relays.clone(),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ValidatedApplication {
    pub(crate) id: String,
    pub(crate) schema_hash: Option<[u8; 32]>,
    pub(crate) schema_version: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ManifestStatus {
    Active,
    Frozen,
    Superseded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ManifestContentError {
    Canonical(CanonicalJsonError),
    Shape,
    Tags,
    Semantics,
}

pub(crate) fn validate(
    event: &VerifiedNip01Event,
) -> Result<ValidatedManifest, ManifestContentError> {
    if event.kind() != 31_624 {
        return Err(ManifestContentError::Semantics);
    }
    validate_parts(
        event.event_id(),
        event.created_at(),
        *event.author_bytes(),
        event.tags(),
        event.content(),
    )
}

fn validate_parts(
    event_id: EventId,
    created_at: u64,
    author: [u8; 32],
    event_tags: &[Vec<String>],
    content: &str,
) -> Result<ValidatedManifest, ManifestContentError> {
    tags::require_tag_contract(event_tags, &[("d", 2)], &["expiration", "-"])
        .map_err(|_| ManifestContentError::Tags)?;
    let d = tags::required_tag(event_tags, "d", 2).map_err(|_| ManifestContentError::Tags)?;
    let document_id: DocumentId = d[1].parse().map_err(|_| ManifestContentError::Tags)?;
    let coordinate = DocumentCoordinate::new(ControllerPublicKey::from_bytes(author), document_id);

    let parsed = parse_content(content)?;
    if parsed.version != 1
        || parsed.format != ProtocolRevision::draft_v1().format()
        || parsed.text_encoding != ProtocolRevision::draft_v1().text_encoding()
        || parsed.relays.len() > 16
        || scalars::validate_sorted_unique_strings(&parsed.relays).is_err()
        || parsed
            .relays
            .iter()
            .any(|value| scalars::validate_relay_url(value).is_err())
        || parsed
            .name
            .as_deref()
            .is_some_and(|value| scalars::validate_utf8_bytes(value, 256).is_err())
        || parsed
            .description
            .as_deref()
            .is_some_and(|value| scalars::validate_utf8_bytes(value, 2_048).is_err())
    {
        return Err(ManifestContentError::Semantics);
    }
    let status = match parsed.status.as_str() {
        "active" => ManifestStatus::Active,
        "frozen" => ManifestStatus::Frozen,
        "superseded" => ManifestStatus::Superseded,
        _ => return Err(ManifestContentError::Semantics),
    };
    Ok(ValidatedManifest {
        event_id,
        created_at,
        coordinate,
        application: parsed.application.map(validate_application).transpose()?,
        checkpoint: parse_optional(&parsed.checkpoint)?,
        control: parsed
            .control
            .parse()
            .map_err(|_| ManifestContentError::Semantics)?,
        description: parsed.description,
        name: parsed.name,
        relays: parsed.relays,
        status,
        successor: parsed
            .successor
            .map(|value| value.parse())
            .transpose()
            .map_err(|_| ManifestContentError::Semantics)?,
    })
}

fn validate_application(value: Application) -> Result<ValidatedApplication, ManifestContentError> {
    if scalars::validate_printable_ascii(&value.id, 128).is_err()
        || scalars::validate_utf8_bytes(&value.schema_version, 64).is_err()
    {
        return Err(ManifestContentError::Semantics);
    }
    Ok(ValidatedApplication {
        id: value.id,
        schema_hash: value
            .schema_hash
            .map(|value| hex::decode_bytes(&value))
            .transpose()
            .map_err(|_| ManifestContentError::Semantics)?,
        schema_version: value.schema_version,
    })
}

fn parse_optional<T: core::str::FromStr>(
    value: &Option<String>,
) -> Result<Option<T>, ManifestContentError> {
    value
        .as_deref()
        .map(str::parse)
        .transpose()
        .map_err(|_| ManifestContentError::Semantics)
}

pub(crate) fn parse_content(content: &str) -> Result<ManifestContent, ManifestContentError> {
    let value = parse_canonical(
        content,
        ProtocolRevision::draft_v1().limits().manifest_content,
    )
    .map_err(ManifestContentError::Canonical)?;
    let object = value.as_object().ok_or(ManifestContentError::Shape)?;
    exact_fields(object, MANIFEST_FIELDS)?;
    Ok(ManifestContent {
        application: application(member(object, "application")?)?,
        checkpoint: nullable_owned_string(member(object, "checkpoint")?)?,
        control: owned_string(member(object, "control")?)?,
        description: nullable_owned_string(member(object, "description")?)?,
        format: owned_string(member(object, "format")?)?,
        name: nullable_owned_string(member(object, "name")?)?,
        relays: string_array(member(object, "relays")?)?,
        status: owned_string(member(object, "status")?)?,
        successor: nullable_owned_string(member(object, "successor")?)?,
        text_encoding: owned_string(member(object, "text_encoding")?)?,
        version: member(object, "v")?
            .as_u64()
            .ok_or(ManifestContentError::Shape)?,
    })
}

fn application(value: &Value) -> Result<Option<Application>, ManifestContentError> {
    if value.is_null() {
        return Ok(None);
    }
    let object = value.as_object().ok_or(ManifestContentError::Shape)?;
    exact_fields(object, APPLICATION_FIELDS)?;
    Ok(Some(Application {
        id: owned_string(member(object, "id")?)?,
        schema_hash: nullable_owned_string(member(object, "schema_hash")?)?,
        schema_version: owned_string(member(object, "schema_version")?)?,
    }))
}

fn exact_fields(
    object: &Map<String, Value>,
    expected: &[&str],
) -> Result<(), ManifestContentError> {
    if object.len() == expected.len() && expected.iter().all(|name| object.contains_key(*name)) {
        Ok(())
    } else {
        Err(ManifestContentError::Shape)
    }
}

fn member<'a>(
    object: &'a Map<String, Value>,
    name: &str,
) -> Result<&'a Value, ManifestContentError> {
    object.get(name).ok_or(ManifestContentError::Shape)
}

fn owned_string(value: &Value) -> Result<String, ManifestContentError> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or(ManifestContentError::Shape)
}

fn nullable_owned_string(value: &Value) -> Result<Option<String>, ManifestContentError> {
    if value.is_null() {
        Ok(None)
    } else {
        owned_string(value).map(Some)
    }
}

fn string_array(value: &Value) -> Result<Vec<String>, ManifestContentError> {
    value
        .as_array()
        .ok_or(ManifestContentError::Shape)?
        .iter()
        .map(owned_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{ManifestContentError, ManifestStatus, parse_content, validate_parts};
    use crate::EventId;

    const CONTENT: &str = r#"{"application":{"id":"org.example.editor","schema_hash":null,"schema_version":"1"},"checkpoint":null,"control":"1111111111111111111111111111111111111111111111111111111111111111","description":null,"format":"automerge-change-v1","name":null,"relays":["wss://relay.example"],"status":"active","successor":null,"text_encoding":"utf16","v":1}"#;

    #[test]
    fn add_manifest_content_model() {
        let parsed = parse_content(CONTENT);
        assert!(parsed.is_ok());
        let parsed = match parsed {
            Ok(value) => value,
            Err(_) => return,
        };
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.checkpoint, None);
        assert_eq!(parsed.name, None);
        assert_eq!(parsed.description, None);
        assert_eq!(parsed.successor, None);
        assert_eq!(
            parsed.application.map(|value| value.id),
            Some("org.example.editor".to_owned())
        );

        let unknown = CONTENT.replace("\"v\":1", "\"unknown\":null,\"v\":1");
        assert_eq!(parse_content(&unknown), Err(ManifestContentError::Shape));
        let wrong_null = CONTENT.replace("\"checkpoint\":null", "\"checkpoint\":false");
        assert_eq!(parse_content(&wrong_null), Err(ManifestContentError::Shape));
    }

    #[test]
    fn validate_manifests_and_addressable_selection_input() {
        let author = [0x22; 32];
        let tags = vec![vec!["d".to_owned(), "33".repeat(32)]];
        let validated = validate_parts(EventId::from_bytes([0x44; 32]), 0, author, &tags, CONTENT);
        assert!(validated.is_ok());
        let validated = match validated {
            Ok(value) => value,
            Err(_) => return,
        };
        assert_eq!(validated.coordinate.controller().as_bytes(), &author);
        assert_eq!(validated.control, EventId::from_bytes([0x11; 32]));
        assert_eq!(validated.status, ManifestStatus::Active);

        let unsorted = CONTENT.replace(
            "[\"wss://relay.example\"]",
            "[\"wss://z.example\",\"wss://a.example\"]",
        );
        assert_eq!(
            validate_parts(EventId::from_bytes([0; 32]), 0, author, &tags, &unsorted),
            Err(ManifestContentError::Semantics)
        );
        let bad_status = CONTENT.replace("\"active\"", "\"paused\"");
        assert_eq!(
            validate_parts(EventId::from_bytes([0; 32]), 0, author, &tags, &bad_status),
            Err(ManifestContentError::Semantics)
        );
        let extended = vec![
            tags[0].clone(),
            vec!["a".to_owned(), "ignored".to_owned()],
            vec!["e".to_owned(), "ignored".to_owned()],
            vec!["x".to_owned()],
            vec!["future".to_owned(), "one".to_owned(), "two".to_owned()],
        ];
        assert!(
            validate_parts(EventId::from_bytes([0; 32]), 0, author, &extended, CONTENT).is_ok()
        );
        let forbidden = vec![tags[0].clone(), vec!["-".to_owned()]];
        assert_eq!(
            validate_parts(EventId::from_bytes([0; 32]), 0, author, &forbidden, CONTENT),
            Err(ManifestContentError::Tags)
        );
    }
}
