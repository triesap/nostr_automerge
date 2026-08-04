use super::{ProofStep, Side};
use sha2::{Digest, Sha256};

/// One validated raw checkpoint chunk and its ordered proof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckpointChunk {
    /// Zero-based position.
    pub index: u32,
    /// Declared total count.
    pub count: u32,
    /// Exact raw bytes.
    pub data: Vec<u8>,
    /// Ordered proof from leaf to root.
    pub proof: Vec<ProofStep>,
}
/// Why checkpoint chunk content was rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChunkError {
    /// Shape, index, count, or proof was invalid.
    Shape,
    /// Base64 was noncanonical or over limit.
    Data,
    /// Declared chunk hash did not match bytes.
    Hash,
}

impl CheckpointChunk {
    /// Parses canonical content with exact separately validated `part` and `x` tag values.
    pub fn parse_content(
        content: &str,
        index: u32,
        count: u32,
        declared_hash: [u8; 32],
    ) -> Result<Self, ChunkError> {
        if count == 0 || count > super::MAX_CHUNK_COUNT || index >= count {
            return Err(ChunkError::Shape);
        }
        let value = crate::wire::canonical_json::parse::parse_canonical(
            content,
            crate::ProtocolRevision::draft_v1().limits().raw_event,
        )
        .map_err(|_| ChunkError::Shape)?;
        let object = value.as_object().ok_or(ChunkError::Shape)?;
        if object.len() != 3 || object.get("v").and_then(serde_json::Value::as_u64) != Some(1) {
            return Err(ChunkError::Shape);
        }
        let data = object
            .get("data")
            .and_then(serde_json::Value::as_str)
            .ok_or(ChunkError::Shape)?;
        let data = crate::wire::base64::decode_padded(
            data,
            crate::ProtocolRevision::draft_v1()
                .limits()
                .checkpoint_chunk_bytes,
        )
        .map_err(|_| ChunkError::Data)?;
        if <[u8; 32]>::from(Sha256::digest(&data)) != declared_hash {
            return Err(ChunkError::Hash);
        }
        let proof = object
            .get("proof")
            .and_then(serde_json::Value::as_array)
            .ok_or(ChunkError::Shape)?
            .iter()
            .map(|entry| {
                let entry = entry.as_object().ok_or(ChunkError::Shape)?;
                if entry.len() != 2 {
                    return Err(ChunkError::Shape);
                }
                let hash = entry
                    .get("hash")
                    .and_then(serde_json::Value::as_str)
                    .ok_or(ChunkError::Shape)?;
                let hash = crate::wire::hex::decode_bytes(hash).map_err(|_| ChunkError::Shape)?;
                let side = match entry.get("side").and_then(serde_json::Value::as_str) {
                    Some("left") => Side::Left,
                    Some("right") => Side::Right,
                    _ => return Err(ChunkError::Shape),
                };
                Ok(ProofStep { hash, side })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            index,
            count,
            data,
            proof,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::CheckpointChunk;
    use sha2::{Digest, Sha256};
    #[test]
    fn parse_checkpoint_chunks() {
        let bytes = b"chunk";
        let hash: [u8; 32] = Sha256::digest(bytes).into();
        let content = r#"{"data":"Y2h1bms=","proof":[],"v":1}"#.to_owned();
        assert!(CheckpointChunk::parse_content(&content, 0, 1, hash).is_ok());
        assert!(CheckpointChunk::parse_content(&content, 1, 1, hash).is_err());
        assert!(CheckpointChunk::parse_content(&content, 0, 1, [0; 32]).is_err());
    }
}
