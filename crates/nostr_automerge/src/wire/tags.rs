pub(crate) fn required_tag<'a>(
    tags: &'a [Vec<String>],
    name: &str,
    exact_elements: usize,
) -> Result<&'a [String], TagError> {
    let mut matches = tags
        .iter()
        .filter(|tag| tag.first().is_some_and(|value| value == name));
    let tag = matches.next().ok_or(TagError::Missing)?;
    if matches.next().is_some() {
        return Err(TagError::Repeated);
    }
    if tag.len() != exact_elements {
        return Err(TagError::ElementCount);
    }
    Ok(tag)
}

pub(crate) fn require_absent(tags: &[Vec<String>], name: &str) -> Result<(), TagError> {
    if tags
        .iter()
        .any(|tag| tag.first().is_some_and(|value| value == name))
    {
        Err(TagError::Forbidden)
    } else {
        Ok(())
    }
}

pub(crate) fn require_durable_tags(tags: &[Vec<String>]) -> Result<(), TagError> {
    require_absent(tags, "expiration")?;
    require_absent(tags, "-")
}

/// Validates the complete name-level contract for a carrier.
///
/// Required tags retain exact cardinality and element counts, explicitly
/// forbidden names are rejected, and every other tag name is ignored.
pub(crate) fn require_tag_contract(
    tags: &[Vec<String>],
    required: &[(&str, usize)],
    forbidden: &[&str],
) -> Result<(), TagError> {
    for (name, exact_elements) in required {
        required_tag(tags, name, *exact_elements)?;
    }
    for name in forbidden {
        require_absent(tags, name)?;
    }
    Ok(())
}

pub(crate) fn require_sorted_unique<T: Ord>(values: &[T]) -> Result<(), TagError> {
    if values.windows(2).all(|pair| pair[0] < pair[1]) {
        Ok(())
    } else {
        Err(TagError::NonCanonicalOrder)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TagError {
    Missing,
    Repeated,
    ElementCount,
    Forbidden,
    NonCanonicalOrder,
}

#[cfg(test)]
mod tests {
    use super::{
        TagError, require_absent, require_durable_tags, require_sorted_unique,
        require_tag_contract, required_tag,
    };

    #[test]
    fn exact_cardinality_does_not_depend_on_tag_order() {
        let tags = vec![
            vec!["x".into(), "hash".into()],
            vec!["a".into(), "coordinate".into()],
        ];
        assert_eq!(
            required_tag(&tags, "a", 2).map(|tag| tag[1].as_str()),
            Ok("coordinate")
        );
        assert_eq!(required_tag(&tags, "e", 2), Err(TagError::Missing));
        assert!(require_absent(&tags, "e").is_ok());
    }

    #[test]
    fn rejects_repeated_extra_forbidden_and_unsorted() {
        let repeated = vec![vec!["e".into(), "1".into()], vec!["e".into(), "2".into()]];
        assert_eq!(required_tag(&repeated, "e", 2), Err(TagError::Repeated));
        assert_eq!(
            required_tag(&[vec!["e".into(), "1".into(), "x".into()]], "e", 2),
            Err(TagError::ElementCount)
        );
        assert_eq!(
            require_durable_tags(&[vec!["expiration".into(), "1".into()]]),
            Err(TagError::Forbidden)
        );
        assert_eq!(
            require_sorted_unique(&[2, 1]),
            Err(TagError::NonCanonicalOrder)
        );
        assert_eq!(
            require_sorted_unique(&[1, 1]),
            Err(TagError::NonCanonicalOrder)
        );
    }

    #[test]
    fn contract_ignores_unknown_names_but_preserves_exact_requirements() {
        let tags = vec![
            vec!["a".into(), "coordinate".into()],
            vec!["x-extra".into()],
            vec!["x-extra".into(), "one".into(), "two".into()],
        ];
        assert_eq!(
            require_tag_contract(&tags, &[("a", 2)], &["expiration", "-"]),
            Ok(())
        );
        assert_eq!(
            require_tag_contract(&tags, &[("e", 2)], &[]),
            Err(TagError::Missing)
        );
        assert_eq!(
            require_tag_contract(&[tags[0].clone(), vec!["-".into()]], &[("a", 2)], &["-"]),
            Err(TagError::Forbidden)
        );
    }
}
