use serde_json::{Map, Value};

use crate::ProtocolRevision;
use crate::types::role::Role;
use crate::wire::canonical_json::parse::{CanonicalJsonError, parse_canonical};
use crate::wire::tags::{self, TagError};
use crate::{
    AccountPublicKey, ActorId, ChangeHash, ControllerPublicKey, DevicePublicKey,
    DocumentCoordinate, EventId, VerifiedNip01Event,
};

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
    Semantics,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ControlCarrierError {
    Kind,
    Tags(TagError),
    Coordinate,
    Content(ControlContentError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ValidatedControlCarrier {
    event_id: EventId,
    author: ControllerPublicKey,
    coordinate: DocumentCoordinate,
    parent: Option<EventId>,
    content: ValidatedControlContent,
}

impl ValidatedControlCarrier {
    pub(crate) const fn event_id(&self) -> EventId {
        self.event_id
    }

    pub(crate) const fn parent(&self) -> Option<EventId> {
        self.parent
    }

    pub(crate) fn base_heads(&self) -> impl Iterator<Item = ChangeHash> + '_ {
        self.content.base_heads.iter().copied()
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        EventId,
        ControllerPublicKey,
        DocumentCoordinate,
        Option<EventId>,
        ValidatedControlContent,
    ) {
        (
            self.event_id,
            self.author,
            self.coordinate,
            self.parent,
            self.content,
        )
    }

    #[cfg(test)]
    pub(crate) const fn synthetic(
        event_id: EventId,
        author: ControllerPublicKey,
        coordinate: DocumentCoordinate,
        parent: Option<EventId>,
        content: ValidatedControlContent,
    ) -> Self {
        Self {
            event_id,
            author,
            coordinate,
            parent,
            content,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ValidatedControlContent {
    pub(crate) base_heads: Vec<ChangeHash>,
    pub(crate) members: Vec<DeviceGrant>,
    pub(crate) predecessor: Option<Predecessor>,
    pub(crate) sequence: u64,
    pub(crate) successor: Option<DocumentCoordinate>,
    pub(crate) terminal: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DeviceGrant {
    pub(crate) account: Option<AccountPublicKey>,
    pub(crate) actor: ActorId,
    pub(crate) device: DevicePublicKey,
    pub(crate) roles: Vec<Role>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Predecessor {
    pub(crate) coordinate: DocumentCoordinate,
    pub(crate) terminal_control: EventId,
}

pub(crate) fn validate(
    event: &VerifiedNip01Event,
) -> Result<ValidatedControlCarrier, ControlCarrierError> {
    if event.kind() != 1_625 {
        return Err(ControlCarrierError::Kind);
    }
    let coordinate_tag =
        tags::required_tag(event.tags(), "a", 2).map_err(ControlCarrierError::Tags)?;
    let coordinate: DocumentCoordinate = coordinate_tag[1]
        .parse()
        .map_err(|_| ControlCarrierError::Coordinate)?;
    let author = ControllerPublicKey::from_bytes(*event.author_bytes());
    if author != coordinate.controller() {
        return Err(ControlCarrierError::Coordinate);
    }
    for forbidden in ["d", "expiration", "-"] {
        tags::require_absent(event.tags(), forbidden).map_err(ControlCarrierError::Tags)?;
    }
    if event
        .tags()
        .iter()
        .any(|tag| tag.first().is_none_or(|name| name != "a" && name != "e"))
    {
        return Err(ControlCarrierError::Tags(TagError::Forbidden));
    }
    let content =
        validate_content(event.content(), coordinate).map_err(ControlCarrierError::Content)?;
    let parent = if content.sequence == 0 {
        tags::require_absent(event.tags(), "e").map_err(ControlCarrierError::Tags)?;
        None
    } else {
        let parent = tags::required_tag(event.tags(), "e", 2).map_err(ControlCarrierError::Tags)?;
        Some(
            parent[1]
                .parse()
                .map_err(|_| ControlCarrierError::Tags(TagError::ElementCount))?,
        )
    };
    if event.tags().len() != usize::from(parent.is_some()) + 1 {
        return Err(ControlCarrierError::Tags(TagError::Repeated));
    }
    Ok(ValidatedControlCarrier {
        event_id: event.event_id(),
        author,
        coordinate,
        parent,
        content,
    })
}

pub(crate) fn validate_content(
    content: &str,
    coordinate: DocumentCoordinate,
) -> Result<ValidatedControlContent, ControlContentError> {
    let parsed = parse_content(content)?;
    if parsed.version != 1
        || parsed.format != ProtocolRevision::draft_v1().format()
        || parsed.text_encoding != ProtocolRevision::draft_v1().text_encoding()
        || parsed.policy != "controller-acl-v1"
        || parsed.members.len()
            > ProtocolRevision::draft_v1()
                .limits()
                .control_members
                .try_usize()
                .map_err(|_| ControlContentError::Semantics)?
        || parsed.base_heads.len()
            > ProtocolRevision::draft_v1()
                .limits()
                .control_heads
                .try_usize()
                .map_err(|_| ControlContentError::Semantics)?
    {
        return Err(ControlContentError::Semantics);
    }
    let base_heads = parsed
        .base_heads
        .iter()
        .map(|value| value.parse())
        .collect::<Result<Vec<ChangeHash>, _>>()
        .map_err(|_| ControlContentError::Semantics)?;
    tags::require_sorted_unique(&base_heads).map_err(|_| ControlContentError::Semantics)?;
    let members = parsed
        .members
        .into_iter()
        .map(|member| validate_member(member, coordinate))
        .collect::<Result<Vec<_>, _>>()?;
    let devices: Vec<_> = members.iter().map(|member| member.device).collect();
    tags::require_sorted_unique(&devices).map_err(|_| ControlContentError::Semantics)?;
    let terminal = members
        .iter()
        .all(|member| !member.roles.contains(&Role::Write));
    Ok(ValidatedControlContent {
        base_heads,
        members,
        predecessor: parsed
            .predecessor
            .map(|value| {
                Ok(Predecessor {
                    coordinate: value
                        .coordinate
                        .parse()
                        .map_err(|_| ControlContentError::Semantics)?,
                    terminal_control: value
                        .terminal_control
                        .parse()
                        .map_err(|_| ControlContentError::Semantics)?,
                })
            })
            .transpose()?,
        sequence: parsed.sequence,
        successor: parsed
            .successor
            .map(|value| value.parse())
            .transpose()
            .map_err(|_| ControlContentError::Semantics)?,
        terminal,
    })
}

fn validate_member(
    member: MemberContent,
    coordinate: DocumentCoordinate,
) -> Result<DeviceGrant, ControlContentError> {
    let device: DevicePublicKey = member
        .pubkey
        .parse()
        .map_err(|_| ControlContentError::Semantics)?;
    if member.roles.is_empty() {
        return Err(ControlContentError::Semantics);
    }
    let roles = member
        .roles
        .iter()
        .map(|value| Role::parse(value).ok_or(ControlContentError::Semantics))
        .collect::<Result<Vec<_>, _>>()?;
    tags::require_sorted_unique(&roles).map_err(|_| ControlContentError::Semantics)?;
    Ok(DeviceGrant {
        account: member
            .account
            .map(|value| value.parse())
            .transpose()
            .map_err(|_| ControlContentError::Semantics)?,
        actor: ActorId::derive(coordinate, device),
        device,
        roles,
    })
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
    use super::{ControlContentError, parse_content, validate_content};
    use crate::DocumentCoordinate;

    const CONTENT: &str = r#"{"base_heads":[],"format":"automerge-change-v1","members":[{"account":null,"pubkey":"3333333333333333333333333333333333333333333333333333333333333333","roles":["checkpoint","write"]}],"policy":"controller-acl-v1","predecessor":null,"seq":0,"successor":null,"text_encoding":"utf16","v":1}"#;

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

    #[test]
    fn validate_device_grants_and_role_syntax() {
        let coordinate: Result<DocumentCoordinate, _> =
            format!("31624:{}:{}", "11".repeat(32), "22".repeat(32)).parse();
        assert!(coordinate.is_ok());
        let coordinate = match coordinate {
            Ok(value) => value,
            Err(_) => return,
        };
        let validated = validate_content(CONTENT, coordinate);
        assert!(validated.is_ok());
        let validated = match validated {
            Ok(value) => value,
            Err(_) => return,
        };
        assert_eq!(
            validated.members[0].actor.to_hex(),
            "020b17c6252b4193d5c5c88620f8e7b29709bb010348108881b99e954352dfeb"
        );
        assert!(!validated.terminal);

        for roles in [
            "[]",
            "[\"write\",\"checkpoint\"]",
            "[\"write\",\"write\"]",
            "[\"admin\"]",
        ] {
            let invalid = CONTENT.replace("[\"checkpoint\",\"write\"]", roles);
            assert_eq!(
                validate_content(&invalid, coordinate),
                Err(ControlContentError::Semantics)
            );
        }
        let duplicate = CONTENT.replace("]}],\"policy\"", "]},{\"account\":null,\"pubkey\":\"3333333333333333333333333333333333333333333333333333333333333333\",\"roles\":[\"write\"]}],\"policy\"");
        assert_eq!(
            validate_content(&duplicate, coordinate),
            Err(ControlContentError::Semantics)
        );
    }
}
