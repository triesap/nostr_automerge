//! Optional verified-history checkpoints that can only reproduce full replay.

mod chunk;
mod descriptor;
mod merkle;

pub use chunk::{CheckpointChunk, ChunkError};
pub use descriptor::{CheckpointDescriptor, DescriptorError};
pub use merkle::{MerkleError, ProofStep, Side, leaf_hash, merkle_root, verify_proof};

/// Provisional regular checkpoint descriptor event kind.
pub const DESCRIPTOR_KIND: u16 = 1_626;
/// Provisional regular checkpoint chunk event kind.
pub const CHUNK_KIND: u16 = 1_627;
/// Maximum raw bytes in one checkpoint chunk.
pub const MAX_CHUNK_SIZE: u32 = 32_768;
/// Maximum chunks in one checkpoint.
pub const MAX_CHUNK_COUNT: u32 = 4_096;
pub(crate) const LEAF_DOMAIN: &[u8] = b"nostr-crdt/checkpoint/leaf/v1";
pub(crate) const NODE_DOMAIN: &[u8] = b"nostr-crdt/checkpoint/node/v1";

#[cfg(test)]
mod tests {
    #[test]
    fn activate_checkpoint_module_and_sealed_constants() {
        assert_eq!((super::DESCRIPTOR_KIND, super::CHUNK_KIND), (1_626, 1_627));
        assert_eq!(
            (super::MAX_CHUNK_SIZE, super::MAX_CHUNK_COUNT),
            (32_768, 4_096)
        );
        assert!(super::LEAF_DOMAIN.starts_with(b"nostr-crdt/checkpoint/"));
    }
}
