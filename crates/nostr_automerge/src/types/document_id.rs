semantic_id!(
    DocumentId,
    "The immutable random 32-byte identifier of a document."
);

#[cfg(test)]
mod tests {
    use super::DocumentId;
    use core::str::FromStr;

    #[test]
    fn document_id_roundtrips() {
        let text = "ab".repeat(32);
        assert_eq!(
            DocumentId::from_str(&text).map(DocumentId::to_hex),
            Ok(text)
        );
    }
}
