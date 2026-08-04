/// The one sealed protocol revision implemented by this crate.
///
/// Callers can look up the approved revision but cannot construct alternate
/// behavior or substitute profile strings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProtocolRevision(SealedRevision);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum SealedRevision {
    Draft2026_08,
}

impl ProtocolRevision {
    /// Returns the approved draft-v1 revision.
    #[must_use]
    pub const fn draft_v1() -> Self {
        Self(SealedRevision::Draft2026_08)
    }

    /// Looks up a supported exact revision identifier.
    #[must_use]
    pub fn lookup(identifier: &str) -> Option<Self> {
        (identifier == "draft_2026_08").then(Self::draft_v1)
    }

    /// Returns the stable revision identifier used by signed content and reports.
    #[must_use]
    pub const fn identifier(self) -> &'static str {
        match self.0 {
            SealedRevision::Draft2026_08 => "draft_2026_08",
        }
    }

    /// Returns the exact Automerge format identifier.
    #[must_use]
    pub const fn format(self) -> &'static str {
        match self.0 {
            SealedRevision::Draft2026_08 => "automerge-change-v1",
        }
    }

    /// Returns the exact text-index encoding identifier.
    #[must_use]
    pub const fn text_encoding(self) -> &'static str {
        match self.0 {
            SealedRevision::Draft2026_08 => "utf16",
        }
    }

    #[allow(dead_code)]
    pub(crate) const fn classify_kind(self, kind: u16) -> Option<kinds::CarrierKind> {
        match self.0 {
            SealedRevision::Draft2026_08 => kinds::classify(kind),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProtocolRevision;

    #[test]
    fn only_exact_draft_revision_is_available() {
        let revision = ProtocolRevision::lookup("draft_2026_08");
        assert_eq!(
            revision.map(ProtocolRevision::identifier),
            Some("draft_2026_08")
        );
        assert_eq!(ProtocolRevision::lookup("draft_2026_09"), None);
        assert_eq!(ProtocolRevision::draft_v1().format(), "automerge-change-v1");
        assert_eq!(ProtocolRevision::draft_v1().text_encoding(), "utf16");
        assert!(ProtocolRevision::draft_v1().classify_kind(1624).is_some());
        assert!(ProtocolRevision::draft_v1().classify_kind(1).is_none());
    }
}
#[allow(dead_code)]
mod kinds;
