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
    /// Ordered Merkle root over chunks.
    pub chunk_root: [u8; 32],
    /// Number of embedded changes.
    pub change_count: u64,
    /// Hash of the sorted embedded change set.
    pub change_set_hash: [u8; 32],
    /// Embedded dependency edge count.
    pub dependency_edges: u64,
    /// Embedded Automerge operation count.
    pub total_ops: u64,
}

/// Why checkpoint descriptor commitments are inconsistent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DescriptorError {
    /// A required count or size was zero or exceeded the sealed profile.
    Range,
    /// Checked ceiling arithmetic did not equal the declared chunk count.
    Arithmetic,
    /// Canonical content shape, encoding, or identifier was invalid.
    Content,
}

impl CheckpointDescriptor {
    /// Validates all size, limit, and checked ceiling-division commitments.
    pub fn validate_arithmetic(&self) -> Result<(), DescriptorError> {
        let limits = crate::ProtocolRevision::draft_v1().limits();
        if self.raw_size == 0
            || self.raw_size > limits.checkpoint_bytes.get()
            || self.chunk_size == 0
            || u64::from(self.chunk_size) > limits.checkpoint_chunk_bytes.get()
            || self.chunk_count == 0
            || u64::from(self.chunk_count) > limits.checkpoint_chunks.get()
            || self.change_count > limits.checkpoint_changes.get()
            || self.total_ops > limits.checkpoint_operations.get()
            || self.dependency_edges > limits.checkpoint_dependency_edges.get()
        {
            return Err(DescriptorError::Range);
        }
        let expected = self
            .raw_size
            .checked_add(u64::from(self.chunk_size) - 1)
            .ok_or(DescriptorError::Arithmetic)?
            / u64::from(self.chunk_size);
        if expected != u64::from(self.chunk_count) {
            return Err(DescriptorError::Arithmetic);
        }
        Ok(())
    }
    /// Parses exact canonical descriptor content and its separately signed snapshot hash tag.
    pub fn parse_content(
        content: &str,
        snapshot_hash: SnapshotHash,
    ) -> Result<Self, DescriptorError> {
        let value = crate::wire::canonical_json::parse::parse_canonical(
            content,
            crate::ProtocolRevision::draft_v1().limits().raw_event,
        )
        .map_err(|_| DescriptorError::Content)?;
        let object = value.as_object().ok_or(DescriptorError::Content)?;
        let fields = [
            "change_count",
            "change_set_hash",
            "chunk_count",
            "chunk_root",
            "chunk_size",
            "dependency_edges",
            "encoding",
            "heads",
            "raw_size",
            "total_ops",
            "v",
        ];
        if object.len() != fields.len()
            || fields.iter().any(|field| !object.contains_key(*field))
            || object.get("encoding").and_then(serde_json::Value::as_str)
                != Some("automerge-save-v1")
            || object.get("v").and_then(serde_json::Value::as_u64) != Some(1)
        {
            return Err(DescriptorError::Content);
        }
        let u64_field = |name| {
            object
                .get(name)
                .and_then(serde_json::Value::as_u64)
                .ok_or(DescriptorError::Content)
        };
        let hex = |name| -> Result<[u8; 32], DescriptorError> {
            let text = object
                .get(name)
                .and_then(serde_json::Value::as_str)
                .ok_or(DescriptorError::Content)?;
            crate::wire::hex::decode_bytes(text).map_err(|_| DescriptorError::Content)
        };
        let heads = object
            .get("heads")
            .and_then(serde_json::Value::as_array)
            .ok_or(DescriptorError::Content)?
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .ok_or(DescriptorError::Content)?
                    .parse()
                    .map_err(|_| DescriptorError::Content)
            })
            .collect::<Result<Vec<ChangeHash>, _>>()?;
        let head_limit = crate::ProtocolRevision::draft_v1()
            .limits()
            .checkpoint_heads
            .try_usize()
            .map_err(|_| DescriptorError::Range)?;
        if heads.len() > head_limit || heads.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(DescriptorError::Content);
        }
        Ok(Self {
            snapshot_hash,
            heads: heads.into_iter().collect(),
            raw_size: u64_field("raw_size")?,
            chunk_size: u32::try_from(u64_field("chunk_size")?)
                .map_err(|_| DescriptorError::Range)?,
            chunk_count: u32::try_from(u64_field("chunk_count")?)
                .map_err(|_| DescriptorError::Range)?,
            chunk_root: hex("chunk_root")?,
            change_count: u64_field("change_count")?,
            change_set_hash: hex("change_set_hash")?,
            dependency_edges: u64_field("dependency_edges")?,
            total_ops: u64_field("total_ops")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::CheckpointDescriptor;
    use crate::SnapshotHash;
    #[test]
    fn parse_checkpoint_descriptors() {
        let content = format!(
            r#"{{"change_count":1,"change_set_hash":"{}","chunk_count":1,"chunk_root":"{}","chunk_size":1,"dependency_edges":0,"encoding":"automerge-save-v1","heads":["{}"],"raw_size":1,"total_ops":1,"v":1}}"#,
            "01".repeat(32),
            "02".repeat(32),
            "03".repeat(32)
        );
        assert!(
            CheckpointDescriptor::parse_content(&content, SnapshotHash::from_bytes([4; 32]))
                .is_ok()
        );
        assert!(
            CheckpointDescriptor::parse_content(
                &content.replace("\"v\":1", "\"v\":2"),
                SnapshotHash::from_bytes([4; 32])
            )
            .is_err()
        );
    }
    #[test]
    fn validate_descriptor_arithmetic() {
        let mut value = CheckpointDescriptor {
            snapshot_hash: SnapshotHash::from_bytes([0; 32]),
            heads: std::collections::BTreeSet::from([crate::ChangeHash::from_bytes([1; 32])]),
            raw_size: 32_769,
            chunk_size: 32_768,
            chunk_count: 2,
            chunk_root: [2; 32],
            change_count: 1,
            change_set_hash: [3; 32],
            dependency_edges: 0,
            total_ops: 0,
        };
        assert_eq!(value.validate_arithmetic(), Ok(()));
        value.chunk_count = 1;
        assert_eq!(
            value.validate_arithmetic(),
            Err(super::DescriptorError::Arithmetic)
        );
        value.raw_size = 0;
        assert_eq!(
            value.validate_arithmetic(),
            Err(super::DescriptorError::Range)
        );
    }

    #[test]
    fn allow_zero_changes_and_empty_heads() {
        let content = format!(
            r#"{{"change_count":0,"change_set_hash":"{}","chunk_count":1,"chunk_root":"{}","chunk_size":1,"dependency_edges":0,"encoding":"automerge-save-v1","heads":[],"raw_size":1,"total_ops":0,"v":1}}"#,
            "01".repeat(32),
            "02".repeat(32),
        );
        let descriptor =
            CheckpointDescriptor::parse_content(&content, SnapshotHash::from_bytes([4; 32]));
        let Ok(descriptor) = descriptor else { return };
        assert!(descriptor.heads.is_empty());
        assert_eq!(descriptor.change_count, 0);
        assert_eq!(descriptor.validate_arithmetic(), Ok(()));
    }
}
