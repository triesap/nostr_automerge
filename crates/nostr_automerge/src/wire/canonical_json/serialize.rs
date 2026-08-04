use serde_json::Value;

const SAFE_INTEGER_MAX: u64 = 9_007_199_254_740_991;

pub(crate) fn to_vec(value: &Value) -> Result<Vec<u8>, JcsError> {
    let mut output = Vec::new();
    write_value(value, &mut output)?;
    Ok(output)
}

fn write_value(value: &Value, output: &mut Vec<u8>) -> Result<(), JcsError> {
    match value {
        Value::Null => output.extend_from_slice(b"null"),
        Value::Bool(false) => output.extend_from_slice(b"false"),
        Value::Bool(true) => output.extend_from_slice(b"true"),
        Value::String(value) => write_string(value, output)?,
        Value::Number(number) => {
            let encoded = if let Some(value) = number.as_i64() {
                if value.unsigned_abs() > SAFE_INTEGER_MAX {
                    return Err(JcsError::Number);
                }
                value.to_string()
            } else if let Some(value) = number.as_u64() {
                if value > SAFE_INTEGER_MAX {
                    return Err(JcsError::Number);
                }
                value.to_string()
            } else {
                return Err(JcsError::Number);
            };
            output.extend_from_slice(encoded.as_bytes());
        }
        Value::Array(values) => {
            output.push(b'[');
            for (index, item) in values.iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                write_value(item, output)?;
            }
            output.push(b']');
        }
        Value::Object(object) => {
            let mut keys: Vec<_> = object.keys().collect();
            keys.sort_by(|left, right| left.encode_utf16().cmp(right.encode_utf16()));
            output.push(b'{');
            for (index, key) in keys.into_iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                write_string(key, output)?;
                output.push(b':');
                let member = object.get(key).ok_or(JcsError::Serialization)?;
                write_value(member, output)?;
            }
            output.push(b'}');
        }
    }
    Ok(())
}

fn write_string(value: &str, output: &mut Vec<u8>) -> Result<(), JcsError> {
    let encoded = serde_json::to_string(value).map_err(|_| JcsError::Serialization)?;
    output.extend_from_slice(encoded.as_bytes());
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum JcsError {
    Number,
    Serialization,
}

#[cfg(test)]
mod tests {
    use super::{JcsError, to_vec};

    #[test]
    fn orders_keys_by_utf16_and_emits_minimal_json() {
        let value = serde_json::json!({"\u{e000}": 2, "\u{10000}": 1, "a": [true, null, "x\n"]});
        assert_eq!(
            to_vec(&value),
            Ok("{\"a\":[true,null,\"x\\n\"],\"𐀀\":1,\"\":2}"
                .as_bytes()
                .to_vec())
        );
    }

    #[test]
    fn rejects_floats_and_unsafe_integers() {
        assert_eq!(to_vec(&serde_json::json!(1.5)), Err(JcsError::Number));
        assert_eq!(
            to_vec(&serde_json::json!(9_007_199_254_740_992_u64)),
            Err(JcsError::Number)
        );
    }
}
