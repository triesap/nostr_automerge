use core::fmt;

#[cfg(test)]
const REGISTRY: &str = include_str!("../../../spec/draft_limits.json");

/// A sealed byte-count validity limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ByteLimit(u64);

impl ByteLimit {
    const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the limit as its canonical unsigned value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Converts to the current platform's allocation index width.
    pub fn try_usize(self) -> Result<usize, LimitConversionError> {
        usize::try_from(self.0).map_err(|_| LimitConversionError)
    }
}

/// A sealed item-count validity limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ItemLimit(u64);

impl ItemLimit {
    const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the limit as its canonical unsigned value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Converts to the current platform's collection index width.
    pub fn try_usize(self) -> Result<usize, LimitConversionError> {
        usize::try_from(self.0).map_err(|_| LimitConversionError)
    }
}

/// A checked integer conversion could not represent a sealed limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LimitConversionError;

impl fmt::Display for LimitConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("protocol limit does not fit the platform index width")
    }
}

impl std::error::Error for LimitConversionError {}

/// Read-only validity limits for the sealed draft-v1 protocol profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProtocolLimits {
    /// Maximum complete raw NIP-01 event bytes accepted at ingress.
    pub raw_event: ByteLimit,
    /// Maximum canonical manifest content bytes.
    pub manifest_content: ByteLimit,
    /// Maximum canonical control content bytes.
    pub control_content: ByteLimit,
    /// Maximum members in one complete control.
    pub control_members: ItemLimit,
    /// Maximum heads in one control frontier.
    pub control_heads: ItemLimit,
    /// Maximum decoded bytes in one Automerge change chunk.
    pub change_bytes: ByteLimit,
    /// Maximum operations in one change.
    pub change_operations: ItemLimit,
    /// Maximum dependencies in one change.
    pub change_dependencies: ItemLimit,
    /// Maximum bytes in one complete raw checkpoint.
    pub checkpoint_bytes: ByteLimit,
    /// Maximum chunks in one checkpoint.
    pub checkpoint_chunks: ItemLimit,
    /// Maximum decoded bytes in one checkpoint chunk.
    pub checkpoint_chunk_bytes: ByteLimit,
    /// Maximum heads in one checkpoint descriptor.
    pub checkpoint_heads: ItemLimit,
    /// Maximum changes embedded in one checkpoint.
    pub checkpoint_changes: ItemLimit,
    /// Maximum operations embedded in one checkpoint.
    pub checkpoint_operations: ItemLimit,
    /// Maximum dependency edges embedded in one checkpoint.
    pub checkpoint_dependency_edges: ItemLimit,
}

impl ProtocolLimits {
    pub(crate) const fn draft_v1() -> Self {
        Self {
            raw_event: ByteLimit::new(262_144),
            manifest_content: ByteLimit::new(16_384),
            control_content: ByteLimit::new(32_768),
            control_members: ItemLimit::new(256),
            control_heads: ItemLimit::new(64),
            change_bytes: ByteLimit::new(32_768),
            change_operations: ItemLimit::new(16_384),
            change_dependencies: ItemLimit::new(256),
            checkpoint_bytes: ByteLimit::new(67_108_864),
            checkpoint_chunks: ItemLimit::new(4_096),
            checkpoint_chunk_bytes: ByteLimit::new(32_768),
            checkpoint_heads: ItemLimit::new(256),
            checkpoint_changes: ItemLimit::new(1_000_000),
            checkpoint_operations: ItemLimit::new(10_000_000),
            checkpoint_dependency_edges: ItemLimit::new(20_000_000),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ProtocolLimits, REGISTRY};

    #[test]
    fn constants_match_checked_in_registry() {
        let limits = ProtocolLimits::draft_v1();
        assert_eq!(limits.raw_event.get(), 262_144);
        assert_eq!(limits.checkpoint_bytes.get(), 67_108_864);
        assert_eq!(limits.checkpoint_dependency_edges.get(), 20_000_000);
        assert!(REGISTRY.contains("\"raw_event_bytes\",\"value\":262144"));
        assert!(REGISTRY.contains("\"checkpoint_dependency_edges\",\"value\":20000000"));
    }

    #[test]
    fn typed_limits_convert_checked() {
        let limits = ProtocolLimits::draft_v1();
        assert_eq!(limits.change_dependencies.try_usize(), Ok(256));
    }
}
