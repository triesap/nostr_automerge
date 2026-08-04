use serde_json::{Map, Value};

use crate::ProtocolRevision;
use crate::wire::canonical_json::parse::{CanonicalJsonError, parse_canonical};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ManifestContentError {
    Canonical(CanonicalJsonError),
    Shape,
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
    use super::{ManifestContentError, parse_content};

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
}
