use serde_json::Value;
use url::Url;

const SAFE_INTEGER_MAX: u64 = 9_007_199_254_740_991;

pub(crate) fn validate_relay_url(value: &str) -> Result<(), ScalarError> {
    let url = Url::parse(value).map_err(|_| ScalarError::Url)?;
    if !matches!(url.scheme(), "ws" | "wss") || url.host_str().is_none() {
        return Err(ScalarError::Url);
    }
    Ok(())
}

pub(crate) fn validate_printable_ascii(
    value: &str,
    maximum_bytes: usize,
) -> Result<(), ScalarError> {
    if value.len() > maximum_bytes || !value.bytes().all(|byte| (0x20..=0x7e).contains(&byte)) {
        return Err(ScalarError::Text);
    }
    Ok(())
}

pub(crate) fn validate_utf8_bytes(value: &str, maximum_bytes: usize) -> Result<(), ScalarError> {
    if value.len() > maximum_bytes {
        Err(ScalarError::Text)
    } else {
        Ok(())
    }
}

pub(crate) fn validate_sorted_unique_strings(values: &[String]) -> Result<(), ScalarError> {
    if values
        .windows(2)
        .all(|pair| pair[0].as_bytes() < pair[1].as_bytes())
    {
        Ok(())
    } else {
        Err(ScalarError::Order)
    }
}

pub(crate) fn safe_u64(value: &Value) -> Result<u64, ScalarError> {
    value
        .as_u64()
        .filter(|number| *number <= SAFE_INTEGER_MAX)
        .ok_or(ScalarError::Integer)
}

pub(crate) fn nullable_string(
    value: &Value,
    maximum_bytes: usize,
) -> Result<Option<&str>, ScalarError> {
    if value.is_null() {
        return Ok(None);
    }
    let string = value.as_str().ok_or(ScalarError::Text)?;
    validate_utf8_bytes(string, maximum_bytes)?;
    Ok(Some(string))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScalarError {
    Url,
    Text,
    Integer,
    Order,
}

#[cfg(test)]
mod tests {
    use super::{
        ScalarError, nullable_string, safe_u64, validate_printable_ascii, validate_relay_url,
        validate_sorted_unique_strings,
    };

    #[test]
    fn validates_absolute_websocket_urls() {
        assert!(validate_relay_url("wss://relay.example/path").is_ok());
        assert_eq!(
            validate_relay_url("https://relay.example"),
            Err(ScalarError::Url)
        );
        assert_eq!(validate_relay_url("/relative"), Err(ScalarError::Url));
    }

    #[test]
    fn validates_text_numbers_nullability_and_byte_order() {
        assert!(validate_printable_ascii("app-1", 5).is_ok());
        assert_eq!(
            validate_printable_ascii("line\n", 10),
            Err(ScalarError::Text)
        );
        assert_eq!(
            safe_u64(&serde_json::json!(9_007_199_254_740_992_u64)),
            Err(ScalarError::Integer)
        );
        assert_eq!(nullable_string(&Value::Null, 1), Ok(None));
        assert!(validate_sorted_unique_strings(&["a".into(), "b".into()]).is_ok());
        assert_eq!(
            validate_sorted_unique_strings(&["b".into(), "a".into()]),
            Err(ScalarError::Order)
        );
    }

    use serde_json::Value;
}
