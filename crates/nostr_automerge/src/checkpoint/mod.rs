//! Optional verified-history checkpoints that can only reproduce full replay.

mod assemble;
pub(crate) mod authorize;
mod chunk;
mod descriptor;
pub(crate) mod join;
mod merkle;
pub(crate) mod reference_state;
mod verify;
mod verify_history;

pub(crate) use assemble::assemble_ordered_chunks;
pub use assemble::{AssemblyError, assemble_chunks};
pub use chunk::{CheckpointChunk, ChunkError};
pub use descriptor::{CheckpointDescriptor, DescriptorError};
pub use merkle::{MerkleError, ProofStep, Side, leaf_hash, merkle_root, verify_proof};
pub use verify::{VerifiedSnapshot, VerifyError, verify_snapshot_heads};
pub(crate) use verify_history::verify_full_history_metered;
pub(crate) use verify_history::{HistoricalCarrierCoverage, historical_carrier_coverage};
pub use verify_history::{HistoryVerificationError, verify_full_history};

/// Provisional regular checkpoint descriptor event kind.
pub const DESCRIPTOR_KIND: u16 = 1_626;
/// Provisional regular checkpoint chunk event kind.
pub const CHUNK_KIND: u16 = 1_627;
/// Maximum raw bytes in one checkpoint chunk.
pub const MAX_CHUNK_SIZE: u32 = 32_768;
/// Maximum chunks in one checkpoint.
pub const MAX_CHUNK_COUNT: u32 = 4_096;
pub(crate) const MERKLE_DOMAIN: &[u8] = b"nostr-crdt/checkpoint-merkle/v1";

#[cfg(test)]
mod tests {
    #[test]
    fn activate_checkpoint_module_and_sealed_constants() {
        assert_eq!((super::DESCRIPTOR_KIND, super::CHUNK_KIND), (1_626, 1_627));
        assert_eq!(
            (super::MAX_CHUNK_SIZE, super::MAX_CHUNK_COUNT),
            (32_768, 4_096)
        );
        assert_eq!(super::MERKLE_DOMAIN, b"nostr-crdt/checkpoint-merkle/v1");
    }
}
