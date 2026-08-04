use automerge::{Automerge, TextEncoding};

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
}

#[cfg(test)]
mod tests {
    use super::Document;

    #[test]
    fn construction_is_explicitly_utf16() {
        assert!(Document::new_utf16().is_utf16());
    }
}
