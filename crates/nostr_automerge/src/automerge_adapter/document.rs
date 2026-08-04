use automerge::{
    Automerge, LoadOptions, OnPartialLoad, StringMigration, TextEncoding, VerificationMode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DocumentLoadError;

pub(crate) struct Document {
    inner: Automerge,
}

impl Document {
    pub(crate) fn new_utf16() -> Self {
        Self {
            inner: Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit),
        }
    }

    pub(crate) fn is_utf16(&self) -> bool {
        self.inner.text_encoding() == TextEncoding::Utf16CodeUnit
    }

    pub(crate) fn load_utf16(bytes: &[u8]) -> Result<Self, DocumentLoadError> {
        let options = LoadOptions::new()
            .text_encoding(TextEncoding::Utf16CodeUnit)
            .migrate_strings(StringMigration::NoMigration)
            .on_partial_load(OnPartialLoad::Error)
            .verification_mode(VerificationMode::Check);
        Automerge::load_with_options(bytes, options)
            .map(|inner| Self { inner })
            .map_err(|_| DocumentLoadError)
    }
}

#[cfg(test)]
mod tests {
    use automerge::{ROOT, ReadDoc, transaction::Transactable};

    use super::Document;

    #[test]
    fn construction_is_explicitly_utf16() {
        assert!(Document::new_utf16().is_utf16());
    }

    #[test]
    fn load_with_utf_16_no_migration_and_no_partial_state() {
        let mut source = Document::new_utf16();
        {
            let mut tx = source.inner.transaction();
            assert!(tx.put(ROOT, "value", "raw string").is_ok());
            tx.commit();
        }
        let expected_heads = source.inner.get_heads();
        let saved = source.inner.save_nocompress();

        let loaded = Document::load_utf16(&saved);
        assert!(loaded.is_ok());
        let loaded = match loaded {
            Ok(document) => document,
            Err(_) => return,
        };
        assert!(loaded.is_utf16());
        assert_eq!(loaded.inner.get_heads(), expected_heads);
        assert!(matches!(
            loaded.inner.get(ROOT, "value"),
            Ok(Some((automerge::Value::Scalar(value), _)))
                if matches!(value.as_ref(), automerge::ScalarValue::Str(text) if text == "raw string")
        ));

        let truncated = &saved[..saved.len().saturating_sub(1)];
        assert!(Document::load_utf16(truncated).is_err());
    }
}
