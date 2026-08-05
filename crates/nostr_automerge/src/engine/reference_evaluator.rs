use crate::ProtocolRevision;

/// Stateless deterministic batch evaluator for immutable signed evidence.
///
/// Evaluation performs no networking, storage, clock access, signing, or key
/// custody. Callers supply an immutable evidence corpus plus explicit local
/// work and cancellation policy to the evaluation operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferenceEvaluator {
    revision: ProtocolRevision,
}

impl ReferenceEvaluator {
    /// Creates an evaluator for the sealed protocol revision.
    #[must_use]
    pub const fn new(revision: ProtocolRevision) -> Self {
        Self { revision }
    }

    /// Returns the sealed revision interpreted by this evaluator.
    #[must_use]
    pub const fn revision(&self) -> ProtocolRevision {
        self.revision
    }
}
