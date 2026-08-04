use core::cell::Cell;
use core::fmt;

use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value};

use crate::ByteLimit;

use super::serialize::to_vec;

const SAFE_INTEGER_MAX: u64 = 9_007_199_254_740_991;

pub(crate) fn parse_canonical(
    input: &str,
    maximum: ByteLimit,
) -> Result<Value, CanonicalJsonError> {
    if input.len()
        > maximum
            .try_usize()
            .map_err(|_| CanonicalJsonError::TooLarge)?
    {
        return Err(CanonicalJsonError::TooLarge);
    }
    let duplicate = Cell::new(false);
    let number = Cell::new(false);
    let mut deserializer = serde_json::Deserializer::from_str(input);
    let parsed = ValueSeed {
        duplicate: &duplicate,
        number: &number,
    }
    .deserialize(&mut deserializer);
    let value = match parsed {
        Ok(value) => value,
        Err(_) if duplicate.get() => return Err(CanonicalJsonError::DuplicateMember),
        Err(_) if number.get() => return Err(CanonicalJsonError::Number),
        Err(_) => return Err(CanonicalJsonError::Syntax),
    };
    deserializer.end().map_err(|_| CanonicalJsonError::Syntax)?;
    if to_vec(&value).map_err(|_| CanonicalJsonError::Number)? != input.as_bytes() {
        return Err(CanonicalJsonError::NonCanonical);
    }
    Ok(value)
}

#[derive(Clone, Copy)]
struct ValueSeed<'a> {
    duplicate: &'a Cell<bool>,
    number: &'a Cell<bool>,
}

impl<'de> DeserializeSeed<'de> for ValueSeed<'_> {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor { seed: self })
    }
}

struct ValueVisitor<'a> {
    seed: ValueSeed<'a>,
}

impl<'de> Visitor<'de> for ValueVisitor<'_> {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("strict canonical JSON")
    }
    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }
    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }
    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(Value::Bool(value))
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(Value::String(value.to_owned()))
    }
    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(Value::String(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if value.unsigned_abs() > SAFE_INTEGER_MAX {
            self.seed.number.set(true);
            return Err(E::custom("unsafe integer"));
        }
        Ok(Value::Number(value.into()))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if value > SAFE_INTEGER_MAX {
            self.seed.number.set(true);
            return Err(E::custom("unsafe integer"));
        }
        Ok(Value::Number(value.into()))
    }

    fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.seed.number.set(true);
        Err(E::custom("floating-point value"))
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element_seed(self.seed)? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A>(self, mut object: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some(key) = object.next_key::<String>()? {
            if values.contains_key(&key) {
                self.seed.duplicate.set(true);
                return Err(serde::de::Error::custom("duplicate member"));
            }
            let value = object.next_value_seed(self.seed)?;
            values.insert(key, value);
        }
        Ok(Value::Object(values))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CanonicalJsonError {
    TooLarge,
    DuplicateMember,
    Number,
    Syntax,
    NonCanonical,
}

#[cfg(test)]
mod tests {
    use super::{CanonicalJsonError, parse_canonical};
    use crate::ProtocolRevision;

    #[test]
    fn accepts_exact_content_and_rejects_every_normalization() {
        let limit = ProtocolRevision::draft_v1().limits().control_content;
        assert!(parse_canonical(r#"{"a":[1,true],"b":null}"#, limit).is_ok());
        assert_eq!(
            parse_canonical(r#"{"a":{"x":1,"x":2}}"#, limit),
            Err(CanonicalJsonError::DuplicateMember)
        );
        assert_eq!(
            parse_canonical(r#"{"b":null, "a":1}"#, limit),
            Err(CanonicalJsonError::NonCanonical)
        );
        assert_eq!(
            parse_canonical(r#"{"a":1.0}"#, limit),
            Err(CanonicalJsonError::Number)
        );
    }
}
