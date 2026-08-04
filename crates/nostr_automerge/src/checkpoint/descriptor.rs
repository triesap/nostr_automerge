use std::collections::BTreeSet;

use crate::{ChangeHash, SnapshotHash};

/// Semantic commitments declared by one verified checkpoint descriptor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckpointDescriptor {
    /// Exact snapshot hash.
    pub snapshot_hash: SnapshotHash,
    /// Exact sorted heads.
    pub heads: BTreeSet<ChangeHash>,
    /// Raw byte count.
    pub raw_size: u64,
    /// Nominal chunk size.
    pub chunk_size: u32,
    /// Exact chunk count.
    pub chunk_count: u32,
}

/// Why checkpoint descriptor commitments are inconsistent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DescriptorError {
    /// A required count or size was zero or exceeded the sealed profile.
    Range,
    /// Checked ceiling arithmetic did not equal the declared chunk count.
    Arithmetic,
}
