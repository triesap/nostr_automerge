use serde_json::{Map, Value};

use crate::ProtocolRevision;
use crate::wire::canonical_json::parse::{CanonicalJsonError, parse_canonical};

const CONTROL_FIELDS: &[&str] = &[
    "base_heads",
    "format",
    "members",
    "policy",
    "predecessor",
    "seq",
    "successor",
    "text_encoding",
    "v",
];
const MEMBER_FIELDS: &[&str] = &["account", "pubkey", "roles"];
const PREDECESSOR_FIELDS: &[&str] = &["coordinate", "terminal_control"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ControlContent {
    pub(crate) base_heads: Vec<String>,
    pub(crate) format: String,
    pub(crate) members: Vec<MemberContent>,
    pub(crate) policy: String,
    pub(crate) predecessor: Option<PredecessorContent>,
    pub(crate) sequence: u64,
    pub(crate) successor: Option<String>,
    pub(crate) text_encoding: String,
    pub(crate) version: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MemberContent {
    pub(crate) account: Option<String>,
    pub(crate) pubkey: String,
    pub(crate) roles: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PredecessorContent {
    pub(crate) coordinate: String,
    pub(crate) terminal_control: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ControlContentError {
    Canonical(CanonicalJsonError),
    Shape,
}

pub(crate) fn parse_content(content: &str) -> Result<ControlContent, ControlContentError> {
    let value = parse_canonical(
        content,
        ProtocolRevision::draft_v1().limits().control_content,
    )
    .map_err(ControlContentError::Canonical)?;
    let object = value.as_object().ok_or(ControlContentError::Shape)?;
    exact_fields(object, CONTROL_FIELDS)?;
    Ok(ControlContent {
        base_heads: string_array(member(object, "base_heads")?)?,
        format: string(member(object, "format")?)?,
        members: member(object, "members")?
            .as_array()
            .ok_or(ControlContentError::Shape)?
            .iter()
            .map(parse_member)
            .collect::<Result<Vec<_>, _>>()?,
        policy: string(member(object, "policy")?)?,
        predecessor: predecessor(member(object, "predecessor")?)?,
        sequence: member(object, "seq")?
            .as_u64()
            .ok_or(ControlContentError::Shape)?,
        successor: nullable_string(member(object, "successor")?)?,
        text_encoding: string(member(object, "text_encoding")?)?,
        version: member(object, "v")?
            .as_u64()
            .ok_or(ControlContentError::Shape)?,
    })
}

fn parse_member(value: &Value) -> Result<MemberContent, ControlContentError> {
    let object = value.as_object().ok_or(ControlContentError::Shape)?;
    exact_fields(object, MEMBER_FIELDS)?;
    Ok(MemberContent {
        account: nullable_string(member(object, "account")?)?,
        pubkey: string(member(object, "pubkey")?)?,
        roles: string_array(member(object, "roles")?)?,
    })
}

fn predecessor(value: &Value) -> Result<Option<PredecessorContent>, ControlContentError> {
    if value.is_null() {
        return Ok(None);
    }
    let object = value.as_object().ok_or(ControlContentError::Shape)?;
    exact_fields(object, PREDECESSOR_FIELDS)?;
    Ok(Some(PredecessorContent {
        coordinate: string(member(object, "coordinate")?)?,
        terminal_control: string(member(object, "terminal_control")?)?,
    }))
}

fn exact_fields(object: &Map<String, Value>, expected: &[&str]) -> Result<(), ControlContentError> {
    if object.len() == expected.len() && expected.iter().all(|name| object.contains_key(*name)) {
        Ok(())
    } else {
        Err(ControlContentError::Shape)
    }
}

fn member<'a>(
    object: &'a Map<String, Value>,
    name: &str,
) -> Result<&'a Value, ControlContentError> {
    object.get(name).ok_or(ControlContentError::Shape)
}

fn string(value: &Value) -> Result<String, ControlContentError> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or(ControlContentError::Shape)
}

fn nullable_string(value: &Value) -> Result<Option<String>, ControlContentError> {
    if value.is_null() {
        Ok(None)
    } else {
        string(value).map(Some)
    }
}

fn string_array(value: &Value) -> Result<Vec<String>, ControlContentError> {
    value
        .as_array()
        .ok_or(ControlContentError::Shape)?
        .iter()
        .map(string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{ControlContentError, parse_content};

    const CONTENT: &str = r#"{"base_heads":[],"format":"automerge-change-v1","members":[{"account":null,"pubkey":"1111111111111111111111111111111111111111111111111111111111111111","roles":["checkpoint","write"]}],"policy":"controller-acl-v1","predecessor":null,"seq":0,"successor":null,"text_encoding":"utf16","v":1}"#;

    #[test]
    fn add_control_content_model() {
        let parsed = parse_content(CONTENT);
        assert!(parsed.is_ok());
        let parsed = match parsed {
            Ok(value) => value,
            Err(_) => return,
        };
        assert_eq!(parsed.sequence, 0);
        assert!(parsed.base_heads.is_empty());
        assert_eq!(parsed.members.len(), 1);
        assert_eq!(parsed.predecessor, None);
        assert_eq!(parsed.successor, None);

        let extra = CONTENT.replace("\"v\":1", "\"unknown\":null,\"v\":1");
        assert_eq!(parse_content(&extra), Err(ControlContentError::Shape));
        let bad_member = CONTENT.replace("\"roles\":[\"checkpoint\",\"write\"]", "\"roles\":null");
        assert_eq!(parse_content(&bad_member), Err(ControlContentError::Shape));
    }
}
