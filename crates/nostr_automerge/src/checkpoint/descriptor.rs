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
        if heads.is_empty() || heads.windows(2).any(|pair| pair[0] >= pair[1]) {
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
}
