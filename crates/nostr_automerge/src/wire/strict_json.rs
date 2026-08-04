use core::cell::Cell;
use core::fmt;
use std::collections::BTreeSet;

use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, Visitor};

use super::raw_event::RawEventBytes;

pub(crate) fn scan_top_level_members(raw: &RawEventBytes) -> Result<(), StrictJsonError> {
    let duplicate = Cell::new(false);
    let mut deserializer = serde_json::Deserializer::from_str(raw.as_str());
    let result = TopLevelSeed {
        duplicate: &duplicate,
    }
    .deserialize(&mut deserializer);
    if duplicate.get() {
        return Err(StrictJsonError::DuplicateMember);
    }
    result.map_err(|_| StrictJsonError::Syntax)?;
    deserializer.end().map_err(|_| StrictJsonError::Syntax)
}

struct TopLevelSeed<'a> {
    duplicate: &'a Cell<bool>,
}

impl<'de> DeserializeSeed<'de> for TopLevelSeed<'_> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(TopLevelVisitor {
            duplicate: self.duplicate,
        })
    }
}

struct TopLevelVisitor<'a> {
    duplicate: &'a Cell<bool>,
}

impl<'de> Visitor<'de> for TopLevelVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("one top-level JSON object")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut members = BTreeSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !members.insert(key) {
                self.duplicate.set(true);
                return Err(serde::de::Error::custom("duplicate top-level member"));
            }
            map.next_value::<IgnoredAny>()?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StrictJsonError {
    Syntax,
    DuplicateMember,
}

#[cfg(test)]
mod tests {
    use super::{StrictJsonError, scan_top_level_members};
    use crate::{ProtocolRevision, RawEventBytes};

    #[allow(clippy::expect_used)]
    fn raw(value: &str) -> RawEventBytes {
        RawEventBytes::new(value.as_bytes(), ProtocolRevision::draft_v1())
            .expect("trusted test input must fit and be UTF-8")
    }

    #[test]
    fn rejects_duplicates_after_escape_decoding() {
        assert_eq!(
            scan_top_level_members(&raw(r#"{"id":1,"\u0069d":2}"#)),
            Err(StrictJsonError::DuplicateMember)
        );
    }

    #[test]
    fn requires_one_complete_object() {
        assert!(scan_top_level_members(&raw(r#"{"id":1}"#)).is_ok());
        assert_eq!(
            scan_top_level_members(&raw(r#"{"id":1} []"#)),
            Err(StrictJsonError::Syntax)
        );
        assert_eq!(
            scan_top_level_members(&raw("[]")),
            Err(StrictJsonError::Syntax)
        );
    }
}
