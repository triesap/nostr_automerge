use serde_json::Value;

/// Exact ordered NIP-01 tag arrays after structural validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Nip01Tags(Vec<Vec<String>>);

impl Nip01Tags {
    pub(crate) fn parse(value: &Value) -> Result<Self, TagShapeError> {
        let tags = value.as_array().ok_or(TagShapeError)?;
        let mut parsed = Vec::with_capacity(tags.len());
        for tag in tags {
            let elements = tag.as_array().ok_or(TagShapeError)?;
            if elements.is_empty() {
                return Err(TagShapeError);
            }
            let mut parsed_tag = Vec::with_capacity(elements.len());
            for element in elements {
                parsed_tag.push(element.as_str().ok_or(TagShapeError)?.to_owned());
            }
            parsed.push(parsed_tag);
        }
        Ok(Self(parsed))
    }

    pub(crate) fn as_slice(&self) -> &[Vec<String>] {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TagShapeError;

#[cfg(test)]
mod tests {
    use super::{Nip01Tags, TagShapeError};

    #[test]
    fn preserves_exact_order_and_elements() {
        let value = serde_json::json!([["e", "two", "three"], ["a", "one"]]);
        let tags = Nip01Tags::parse(&value);
        assert_eq!(
            tags.as_ref().map(Nip01Tags::as_slice),
            Ok(&[
                vec!["e".into(), "two".into(), "three".into()],
                vec!["a".into(), "one".into()]
            ][..])
        );
    }

    #[test]
    fn rejects_nonarrays_empty_tags_and_nonstrings() {
        assert_eq!(Nip01Tags::parse(&serde_json::json!({})), Err(TagShapeError));
        assert_eq!(
            Nip01Tags::parse(&serde_json::json!([[]])),
            Err(TagShapeError)
        );
        assert_eq!(
            Nip01Tags::parse(&serde_json::json!([["e", 1]])),
            Err(TagShapeError)
        );
    }
}
