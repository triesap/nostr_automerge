use crate::{ChangeHash, Completion, EventId, SnapshotHash};

/// Stable outcome of verifying one signed checkpoint descriptor and its chunks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CheckpointVerificationStatus {
    /// Every signed, byte, graph, history, and commitment check passed.
    Verified,
    /// The descriptor's referenced control has not been observed.
    PendingControl,
    /// The descriptor signer was not authorized at its referenced control.
    Unauthorized,
    /// A chunk signer differed from the descriptor signer.
    ChunkAuthorMismatch,
    /// A chunk coordinate differed from the descriptor coordinate.
    ChunkCoordinateMismatch,
    /// A chunk referenced a different descriptor.
    ChunkDescriptorMismatch,
    /// A chunk count differed from the descriptor commitment.
    ChunkCountMismatch,
    /// More than one chunk occupied the same index.
    DuplicateChunk,
    /// The descriptor's complete index set was not present.
    MissingChunk,
    /// A chunk length differed from the descriptor commitment.
    ChunkSizeMismatch,
    /// A post-binding chunk shape differed from the descriptor.
    ChunkAssemblyMismatch,
    /// A chunk proof did not reconstruct the descriptor Merkle root.
    MerkleMismatch,
    /// Reconstructed snapshot size differed from the descriptor.
    SnapshotSizeMismatch,
    /// Reconstructed snapshot hash differed from the descriptor.
    SnapshotHashMismatch,
    /// The snapshot could not be loaded safely.
    SnapshotLoad,
    /// Loaded Automerge heads differed from the descriptor.
    HeadMismatch,
    /// Embedded count or change-set commitments differed.
    CommitmentMismatch,
    /// The embedded change graph was not the exact head closure.
    ClosureMismatch,
    /// An embedded change lacked qualifying historical carrier evidence.
    MissingHistoricalCarrier,
    /// An embedded change was not accepted at the referenced control.
    NotAcceptedAtControl,
    /// The caller's deterministic checkpoint work budget was exhausted.
    BudgetExhausted,
    /// The caller cancelled checkpoint verification.
    Cancelled,
}

/// Immutable public evidence binding for one checkpoint verification attempt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckpointVerificationResult {
    descriptor_event: EventId,
    chunk_events: Vec<EventId>,
    snapshot_hash: SnapshotHash,
    heads: Vec<ChangeHash>,
    change_count: u64,
    change_set_hash: [u8; 32],
    historical_carriers: Vec<ChangeHash>,
    accepted_at_control: Vec<ChangeHash>,
    status: CheckpointVerificationStatus,
}

impl CheckpointVerificationResult {
    #[allow(dead_code, clippy::too_many_arguments)]
    pub(crate) fn new(
        descriptor_event: EventId,
        mut chunk_events: Vec<EventId>,
        snapshot_hash: SnapshotHash,
        heads: Vec<ChangeHash>,
        change_count: u64,
        change_set_hash: [u8; 32],
        historical_carriers: Vec<ChangeHash>,
        accepted_at_control: Vec<ChangeHash>,
        status: CheckpointVerificationStatus,
    ) -> Self {
        chunk_events.sort_unstable();
        chunk_events.dedup();
        Self {
            descriptor_event,
            chunk_events,
            snapshot_hash,
            heads,
            change_count,
            change_set_hash,
            historical_carriers,
            accepted_at_control,
            status,
        }
    }

    /// Constructs a result from vectors already proven sorted and unique by
    /// their trusted ordered-index or `BTreeSet` producers.
    ///
    /// This path deliberately performs no repair pass; callers must preserve
    /// those producer invariants before transferring vector ownership.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_trusted_ordered(
        descriptor_event: EventId,
        chunk_events: Vec<EventId>,
        snapshot_hash: SnapshotHash,
        heads: Vec<ChangeHash>,
        change_count: u64,
        change_set_hash: [u8; 32],
        historical_carriers: Vec<ChangeHash>,
        accepted_at_control: Vec<ChangeHash>,
        status: CheckpointVerificationStatus,
    ) -> Self {
        Self {
            descriptor_event,
            chunk_events,
            snapshot_hash,
            heads,
            change_count,
            change_set_hash,
            historical_carriers,
            accepted_at_control,
            status,
        }
    }

    /// Returns the signed descriptor event identity.
    #[must_use]
    pub const fn descriptor_event(&self) -> EventId {
        self.descriptor_event
    }

    /// Returns all bound chunk event identities in canonical order.
    #[must_use]
    pub fn chunk_events(&self) -> &[EventId] {
        &self.chunk_events
    }

    /// Returns the descriptor's exact snapshot hash commitment.
    #[must_use]
    pub const fn snapshot_hash(&self) -> SnapshotHash {
        self.snapshot_hash
    }

    /// Returns the descriptor's exact sorted Automerge heads.
    #[must_use]
    pub fn heads(&self) -> &[ChangeHash] {
        &self.heads
    }

    /// Returns the descriptor's embedded-change count commitment.
    #[must_use]
    pub const fn change_count(&self) -> u64 {
        self.change_count
    }

    /// Returns the descriptor's sorted change-set hash commitment.
    #[must_use]
    pub const fn change_set_hash(&self) -> [u8; 32] {
        self.change_set_hash
    }

    /// Returns qualifying historical carrier coverage through the referenced control.
    #[must_use]
    pub fn historical_carriers(&self) -> &[ChangeHash] {
        &self.historical_carriers
    }

    /// Returns the exact accepted history at the referenced control.
    #[must_use]
    pub fn accepted_at_control(&self) -> &[ChangeHash] {
        &self.accepted_at_control
    }

    /// Returns the stable checkpoint verification outcome.
    #[must_use]
    pub const fn status(&self) -> CheckpointVerificationStatus {
        self.status
    }

    /// Returns local completion for this checkpoint verification attempt.
    #[must_use]
    pub const fn completion(&self) -> Completion {
        match self.status {
            CheckpointVerificationStatus::Verified => Completion::Complete,
            CheckpointVerificationStatus::BudgetExhausted => Completion::BudgetExhausted,
            CheckpointVerificationStatus::Cancelled => Completion::Cancelled,
            _ => Completion::Complete,
        }
    }
}
