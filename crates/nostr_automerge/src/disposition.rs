/// A canonical protocol outcome for evidence under the sealed revision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ProtocolDisposition {
    /// Fully validated evidence that contributes to canonical state.
    Accepted = 1,
    /// Evidence waiting for a control, dependency, or other required evidence.
    Pending = 2,
    /// Valid evidence outside canonical state or below quarantine.
    Excluded = 3,
    /// Evidence that failed a known-revision validity rule.
    Invalid = 4,
    /// Evidence declaring an unknown revision or Automerge profile.
    UnsupportedRevision = 5,
}

impl ProtocolDisposition {
    /// Returns the stable dispositions-digest numeric code.
    #[must_use]
    pub const fn code(self) -> u8 {
        self as u8
    }

    /// Returns the canonical report spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Pending => "pending",
            Self::Excluded => "excluded",
            Self::Invalid => "invalid",
            Self::UnsupportedRevision => "unsupported_revision",
        }
    }

    pub(crate) const fn for_revision(supported: bool, valid: bool) -> Self {
        if !supported {
            Self::UnsupportedRevision
        } else if valid {
            Self::Accepted
        } else {
            Self::Invalid
        }
    }
}

/// Local execution completion, which never changes protocol dispositions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Completion {
    /// The requested deterministic evaluation finished.
    Complete,
    /// The caller-selected deterministic work budget was exhausted.
    BudgetExhausted,
    /// The caller requested cooperative cancellation.
    Cancelled,
}

impl Completion {
    /// Returns the canonical report spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::BudgetExhausted => "budget_exhausted",
            Self::Cancelled => "cancelled",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Completion, ProtocolDisposition};

    #[test]
    fn digest_codes_are_exact_and_completion_has_none() {
        assert_eq!(ProtocolDisposition::Accepted.code(), 1);
        assert_eq!(ProtocolDisposition::Pending.code(), 2);
        assert_eq!(ProtocolDisposition::Excluded.code(), 3);
        assert_eq!(ProtocolDisposition::Invalid.code(), 4);
        assert_eq!(ProtocolDisposition::UnsupportedRevision.code(), 5);
        assert_eq!(Completion::BudgetExhausted.as_str(), "budget_exhausted");
        assert_eq!(
            ProtocolDisposition::for_revision(false, false),
            ProtocolDisposition::UnsupportedRevision
        );
    }
}
