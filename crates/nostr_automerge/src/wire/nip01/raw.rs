use serde_json::{Map, Value};

use crate::types::public_key::VerifiedPublicKey;
use crate::wire::nip01::tags::{Nip01Tags, TagShapeError};
use crate::wire::strict_json::{StrictJsonError, scan_top_level_members};
use crate::{EventId, HexError, Nip01Signature, RawEventBytes};

const SAFE_INTEGER_MAX: u64 = 9_007_199_254_740_991;
const REQUIRED_MEMBERS: &[&str] = &[
    "content",
    "created_at",
    "id",
    "kind",
    "pubkey",
    "sig",
    "tags",
];

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub(crate) struct RawNip01Event {
    pub(crate) id: EventId,
    pub(crate) pubkey: VerifiedPublicKey,
    pub(crate) created_at: u64,
    pub(crate) kind: u16,
    pub(crate) tags: Nip01Tags,
    pub(crate) content: String,
    pub(crate) signature: Nip01Signature,
}

pub(crate) fn parse(raw: &RawEventBytes) -> Result<RawNip01Event, RawNip01Error> {
    scan_top_level_members(raw).map_err(RawNip01Error::Json)?;
    let value: Value = serde_json::from_str(raw.as_str()).map_err(|_| RawNip01Error::Shape)?;
    let object = value.as_object().ok_or(RawNip01Error::Shape)?;
    validate_members(object)?;
    let created_at = member(object, "created_at")?
        .as_u64()
        .filter(|value| *value <= SAFE_INTEGER_MAX)
        .ok_or(RawNip01Error::Shape)?;
    let kind_u64 = member(object, "kind")?
        .as_u64()
        .filter(|value| *value <= u64::from(u16::MAX))
        .ok_or(RawNip01Error::Shape)?;
    let kind = u16::try_from(kind_u64).map_err(|_| RawNip01Error::Shape)?;
    Ok(RawNip01Event {
        id: string(object, "id")?
            .parse()
            .map_err(RawNip01Error::Identifier)?,
        pubkey: VerifiedPublicKey::parse(string(object, "pubkey")?)
            .map_err(RawNip01Error::Identifier)?,
        created_at,
        kind,
        tags: Nip01Tags::parse(member(object, "tags")?).map_err(RawNip01Error::Tags)?,
        content: string(object, "content")?.to_owned(),
        signature: string(object, "sig")?
            .parse()
            .map_err(RawNip01Error::Identifier)?,
    })
}

fn validate_members(object: &Map<String, Value>) -> Result<(), RawNip01Error> {
    if object.len() != REQUIRED_MEMBERS.len()
        || REQUIRED_MEMBERS
            .iter()
            .any(|name| !object.contains_key(*name))
    {
        return Err(RawNip01Error::Shape);
    }
    Ok(())
}

fn string<'a>(object: &'a Map<String, Value>, name: &str) -> Result<&'a str, RawNip01Error> {
    member(object, name)?.as_str().ok_or(RawNip01Error::Shape)
}

fn member<'a>(object: &'a Map<String, Value>, name: &str) -> Result<&'a Value, RawNip01Error> {
    object.get(name).ok_or(RawNip01Error::Shape)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RawNip01Error {
    Json(StrictJsonError),
    Identifier(HexError),
    Tags(TagShapeError),
    Shape,
}

#[cfg(test)]
mod tests {
    use super::{RawNip01Error, parse};
    use crate::{ProtocolRevision, RawEventBytes};

    #[allow(clippy::expect_used)]
    fn raw(value: &str) -> RawEventBytes {
        RawEventBytes::new(value.as_bytes(), ProtocolRevision::draft_v1())
            .expect("trusted UTF-8 fixture")
    }

    #[test]
    fn parses_exact_shape() {
        let event = raw(&format!(
            r#"{{"id":"{}","pubkey":"{}","created_at":0,"kind":1,"tags":[],"content":"","sig":"{}"}}"#,
            "00".repeat(32),
            "11".repeat(32),
            "22".repeat(64)
        ));
        assert!(parse(&event).is_ok());
    }

    #[test]
    fn rejects_missing_extra_and_out_of_range_scalars() {
        let missing = raw(r#"{"id":"00"}"#);
        assert_eq!(parse(&missing), Err(RawNip01Error::Shape));
        let extra = raw(
            r#"{"id":"00","pubkey":"00","created_at":0,"kind":1,"tags":[],"content":"","sig":"00","extra":0}"#,
        );
        assert_eq!(parse(&extra), Err(RawNip01Error::Shape));
        let range = raw(
            r#"{"id":"00","pubkey":"00","created_at":9007199254740992,"kind":1,"tags":[],"content":"","sig":"00"}"#,
        );
        assert_eq!(parse(&range), Err(RawNip01Error::Shape));
    }
}
