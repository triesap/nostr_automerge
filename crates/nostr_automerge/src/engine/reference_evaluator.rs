use crate::carrier::VerifiedCarrier;
use crate::checkpoint::authorize::{DescriptorAuthorization, authorize_descriptor};
use crate::checkpoint::join::{JoinError, join_chunks};
use crate::checkpoint::{HistoryVerificationError, historical_carrier_coverage};
use crate::conformance::dispositions_digest::{
    DispositionItem, DispositionNamespace, dispositions_digest,
};
use crate::conformance::history_digest::history_digest;
use crate::control::candidate::{CandidateResult, evaluate_parent_continuity};
use crate::control::validate::ControlEnvelope;
use crate::evidence::event::EventEvidence;
use crate::graph::change_candidate::{CandidateCarrier, ChangeCandidate};
use crate::reference::evaluate::{BatchChange, BatchControl, evaluate_batch};
use crate::types::role::Role;
use crate::{
    CancellationCheck, ChangeHash, CheckpointVerificationResult, CheckpointVerificationStatus,
    Completion, DocumentCoordinate, EvidenceCorpus, ProtocolDisposition, ProtocolRevision,
    WorkBudget, WorkCounter,
};

use super::evaluation_report::{EvaluationFailure, EvaluationReport, EvaluationReportParts};
use crate::automerge_adapter::materialized_view::MaterializedDocumentView;

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

    /// Evaluates one document coordinate from fully retained signed evidence.
    ///
    /// The operation performs no I/O and derives every control and change input
    /// from validated carriers in `corpus`. Local exhaustion or cancellation is
    /// represented by [`crate::Completion`] without changing dispositions.
    #[must_use]
    pub fn evaluate(
        &self,
        corpus: &EvidenceCorpus,
        coordinate: DocumentCoordinate,
        budget: &mut WorkBudget,
        cancellation: &impl CancellationCheck,
    ) -> EvaluationReport {
        let ingress_complete = charge_ingress(corpus, budget);
        let controls = if ingress_complete {
            controls_for_coordinate(corpus, coordinate)
        } else {
            Vec::new()
        };
        let mut batch = evaluate_batch(controls, budget, cancellation);
        if !ingress_complete {
            batch.completion = Completion::BudgetExhausted;
            batch.failure = Some(EvaluationFailure::BudgetExhausted);
            batch.materialized_document = None;
        }
        let canonical_controls = batch.canonical_controls;
        let checkpoints = verify_checkpoints(
            corpus,
            coordinate,
            &canonical_controls,
            &batch.accepted_at_control,
            budget,
            cancellation,
        );
        let dispositions = batch.dispositions.into_iter().collect::<Vec<_>>();
        let accepted_changes = batch.accepted_changes.into_iter().collect::<Vec<_>>();
        let heads = batch.heads.into_iter().collect::<Vec<_>>();
        let pending_changes = disposition_hashes(&dispositions, ProtocolDisposition::Pending);
        let excluded_changes = dispositions
            .iter()
            .filter_map(|(hash, disposition)| {
                (*disposition != ProtocolDisposition::Accepted
                    && *disposition != ProtocolDisposition::Pending)
                    .then_some(*hash)
            })
            .collect::<Vec<_>>();
        let history_digest = history_digest(
            self.revision,
            coordinate,
            &canonical_controls,
            &accepted_changes,
            &heads,
        )
        .unwrap_or_else(|_| unreachable!("engine report collections are canonical"));
        let disposition_items = dispositions
            .iter()
            .map(|(hash, disposition)| DispositionItem {
                namespace: DispositionNamespace::ChangeHash,
                identifier: *hash.as_bytes(),
                disposition: *disposition,
            })
            .collect::<Vec<_>>();
        let dispositions_digest =
            dispositions_digest(self.revision, coordinate, &disposition_items)
                .unwrap_or_else(|_| unreachable!("engine dispositions are canonical"));
        EvaluationReport::from_canonical_parts(EvaluationReportParts {
            coordinate,
            canonical_controls,
            dispositions,
            accepted_changes,
            pending_changes,
            excluded_changes,
            heads,
            evidence: corpus.records().collect(),
            checkpoints,
            history_digest,
            dispositions_digest,
            integrity_alerts: batch.integrity_alerts,
            completion: batch.completion,
            failure: batch.failure,
            document: batch.materialized_document.map(|bytes| {
                MaterializedDocumentView::from_canonical_bytes(bytes)
                    .unwrap_or_else(|_| unreachable!("applied state must project"))
            }),
        })
    }
}

fn verify_checkpoints(
    corpus: &EvidenceCorpus,
    coordinate: DocumentCoordinate,
    canonical_controls: &[crate::EventId],
    accepted_at_control: &std::collections::BTreeMap<
        crate::EventId,
        std::collections::BTreeSet<ChangeHash>,
    >,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Vec<CheckpointVerificationResult> {
    let canonical_set = canonical_controls.iter().copied().collect();
    let authorizations = checkpoint_authorizations(corpus, coordinate, &canonical_set);
    let chunk_sets = checkpoint_chunk_sets(corpus, coordinate);
    let carrier_coverage = checkpoint_carrier_coverage(corpus, coordinate, canonical_controls);
    let accepted_history = checkpoint_accepted_history(corpus, coordinate, accepted_at_control);
    corpus
        .events
        .values()
        .filter_map(|evidence| match evidence {
            EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::CheckpointDescriptor(descriptor),
                ..
            } if descriptor.coordinate() == coordinate => Some(descriptor.as_ref()),
            _ => None,
        })
        .map(|descriptor| {
            let descriptor_id = descriptor.event_id();
            let chunk_events = checkpoint_chunk_event_ids(corpus, descriptor_id);
            let coverage = carrier_coverage
                .get(&descriptor_id)
                .and_then(|value| value.as_ref().ok())
                .cloned()
                .unwrap_or_default();
            let accepted = accepted_history
                .get(&descriptor_id)
                .and_then(|value| value.as_ref().ok())
                .cloned()
                .unwrap_or_default();
            let status = verify_one_checkpoint(
                descriptor,
                authorizations.get(&descriptor_id).copied(),
                chunk_sets.get(&descriptor_id),
                &coverage,
                &accepted,
                budget,
                cancellation,
            );
            let commitments = descriptor.descriptor();
            CheckpointVerificationResult::new(
                descriptor_id,
                chunk_events,
                descriptor.snapshot_hash(),
                commitments.heads.iter().copied().collect(),
                commitments.change_count,
                commitments.change_set_hash,
                coverage.into_iter().collect(),
                accepted.into_iter().collect(),
                status,
            )
        })
        .collect()
}

fn checkpoint_chunk_event_ids(
    corpus: &EvidenceCorpus,
    descriptor_id: crate::EventId,
) -> Vec<crate::EventId> {
    corpus
        .events
        .values()
        .filter_map(|evidence| match evidence {
            EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::CheckpointChunk(chunk),
                ..
            } if chunk.descriptor_id() == descriptor_id => Some(chunk.event_id()),
            _ => None,
        })
        .collect()
}

fn verify_one_checkpoint(
    descriptor: &crate::carrier::checkpoint_descriptor::ValidatedCheckpointDescriptorCarrier,
    authorization: Option<DescriptorAuthorization>,
    chunks: Option<&Result<Vec<crate::checkpoint::CheckpointChunk>, JoinError>>,
    coverage: &std::collections::BTreeSet<ChangeHash>,
    accepted: &std::collections::BTreeSet<ChangeHash>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> CheckpointVerificationStatus {
    use crate::checkpoint::{HistoryVerificationError, VerifyError};
    match authorization {
        Some(DescriptorAuthorization::Authorized) => {}
        Some(DescriptorAuthorization::PendingControl) | None => {
            return CheckpointVerificationStatus::PendingControl;
        }
        Some(DescriptorAuthorization::Invalid) => {
            return CheckpointVerificationStatus::Unauthorized;
        }
    }
    if budget.charge_checkpoint_items(1).is_err() {
        return CheckpointVerificationStatus::BudgetExhausted;
    }
    let mut chunks = match chunks {
        Some(Ok(chunks)) => chunks.clone(),
        Some(Err(error)) => return join_status(*error),
        None => return CheckpointVerificationStatus::MissingChunk,
    };
    let bytes = match crate::checkpoint::assemble_chunks(
        descriptor.descriptor(),
        &mut chunks,
        budget,
        cancellation,
    ) {
        Ok(bytes) => bytes,
        Err(error) => return assembly_status(error),
    };
    let snapshot = match crate::checkpoint::verify_snapshot_heads(
        &bytes,
        descriptor.descriptor(),
        budget,
        cancellation,
    ) {
        Ok(snapshot) => snapshot,
        Err(VerifyError::Budget) => return CheckpointVerificationStatus::BudgetExhausted,
        Err(VerifyError::Cancelled) => return CheckpointVerificationStatus::Cancelled,
        Err(VerifyError::Load) => return CheckpointVerificationStatus::SnapshotLoad,
        Err(VerifyError::Heads) => return CheckpointVerificationStatus::HeadMismatch,
        Err(VerifyError::Commitments) => return CheckpointVerificationStatus::CommitmentMismatch,
        Err(VerifyError::Closure) => return CheckpointVerificationStatus::ClosureMismatch,
    };
    if let Err(error) = snapshot.verify_commitments(descriptor.descriptor(), budget) {
        return match error {
            VerifyError::Budget => CheckpointVerificationStatus::BudgetExhausted,
            VerifyError::Cancelled => CheckpointVerificationStatus::Cancelled,
            VerifyError::Load => CheckpointVerificationStatus::SnapshotLoad,
            VerifyError::Commitments => CheckpointVerificationStatus::CommitmentMismatch,
            VerifyError::Heads => CheckpointVerificationStatus::HeadMismatch,
            VerifyError::Closure => CheckpointVerificationStatus::ClosureMismatch,
        };
    }
    if let Err(error) = snapshot.verify_exact_closure_metered(budget) {
        return match error {
            VerifyError::Budget => CheckpointVerificationStatus::BudgetExhausted,
            VerifyError::Cancelled => CheckpointVerificationStatus::Cancelled,
            _ => CheckpointVerificationStatus::ClosureMismatch,
        };
    }
    match crate::checkpoint::verify_full_history_metered(&snapshot, coverage, accepted, budget) {
        Ok(()) => CheckpointVerificationStatus::Verified,
        Err(HistoryVerificationError::MissingCarrier) => {
            CheckpointVerificationStatus::MissingHistoricalCarrier
        }
        Err(HistoryVerificationError::NotAccepted) => {
            CheckpointVerificationStatus::NotAcceptedAtControl
        }
        Err(HistoryVerificationError::Snapshot) => CheckpointVerificationStatus::SnapshotLoad,
        Err(HistoryVerificationError::UnknownControl) => {
            CheckpointVerificationStatus::PendingControl
        }
        Err(HistoryVerificationError::Budget) => CheckpointVerificationStatus::BudgetExhausted,
    }
}

const fn join_status(error: JoinError) -> CheckpointVerificationStatus {
    match error {
        JoinError::Author => CheckpointVerificationStatus::ChunkAuthorMismatch,
        JoinError::Coordinate => CheckpointVerificationStatus::ChunkCoordinateMismatch,
        JoinError::Descriptor => CheckpointVerificationStatus::ChunkDescriptorMismatch,
        JoinError::Count => CheckpointVerificationStatus::ChunkCountMismatch,
        JoinError::DuplicateIndex => CheckpointVerificationStatus::DuplicateChunk,
        JoinError::MissingIndex => CheckpointVerificationStatus::MissingChunk,
        JoinError::Size => CheckpointVerificationStatus::ChunkSizeMismatch,
    }
}

const fn assembly_status(error: crate::checkpoint::AssemblyError) -> CheckpointVerificationStatus {
    match error {
        crate::checkpoint::AssemblyError::Chunks => {
            CheckpointVerificationStatus::ChunkAssemblyMismatch
        }
        crate::checkpoint::AssemblyError::Proof => CheckpointVerificationStatus::MerkleMismatch,
        crate::checkpoint::AssemblyError::Budget => CheckpointVerificationStatus::BudgetExhausted,
        crate::checkpoint::AssemblyError::Cancelled => CheckpointVerificationStatus::Cancelled,
        crate::checkpoint::AssemblyError::SnapshotSize => {
            CheckpointVerificationStatus::SnapshotSizeMismatch
        }
        crate::checkpoint::AssemblyError::SnapshotHash => {
            CheckpointVerificationStatus::SnapshotHashMismatch
        }
    }
}

fn checkpoint_accepted_history(
    corpus: &EvidenceCorpus,
    coordinate: DocumentCoordinate,
    accepted_at_control: &std::collections::BTreeMap<
        crate::EventId,
        std::collections::BTreeSet<ChangeHash>,
    >,
) -> std::collections::BTreeMap<
    crate::EventId,
    Result<std::collections::BTreeSet<ChangeHash>, HistoryVerificationError>,
> {
    corpus
        .events
        .values()
        .filter_map(|evidence| match evidence {
            EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::CheckpointDescriptor(descriptor),
                ..
            } if descriptor.coordinate() == coordinate => Some((
                descriptor.event_id(),
                accepted_at_control
                    .get(&descriptor.control_id())
                    .cloned()
                    .ok_or(HistoryVerificationError::UnknownControl),
            )),
            _ => None,
        })
        .collect()
}

fn checkpoint_carrier_coverage(
    corpus: &EvidenceCorpus,
    coordinate: DocumentCoordinate,
    canonical_controls: &[crate::EventId],
) -> std::collections::BTreeMap<
    crate::EventId,
    Result<std::collections::BTreeSet<ChangeHash>, HistoryVerificationError>,
> {
    corpus
        .events
        .values()
        .filter_map(|evidence| match evidence {
            EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::CheckpointDescriptor(descriptor),
                ..
            } if descriptor.coordinate() == coordinate => Some((
                descriptor.event_id(),
                historical_carrier_coverage(corpus, canonical_controls, descriptor.control_id()),
            )),
            _ => None,
        })
        .collect()
}

fn checkpoint_chunk_sets(
    corpus: &EvidenceCorpus,
    coordinate: DocumentCoordinate,
) -> std::collections::BTreeMap<
    crate::EventId,
    Result<Vec<crate::checkpoint::CheckpointChunk>, JoinError>,
> {
    corpus
        .events
        .values()
        .filter_map(|evidence| match evidence {
            EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::CheckpointDescriptor(descriptor),
                ..
            } if descriptor.coordinate() == coordinate => {
                let chunks = corpus
                    .events
                    .values()
                    .filter_map(|evidence| match evidence {
                        EventEvidence::VerifiedCarrier {
                            carrier: VerifiedCarrier::CheckpointChunk(chunk),
                            ..
                        } if chunk.descriptor_id() == descriptor.event_id() => Some(chunk.as_ref()),
                        _ => None,
                    });
                Some((descriptor.event_id(), join_chunks(descriptor, chunks)))
            }
            _ => None,
        })
        .collect()
}

fn checkpoint_authorizations(
    corpus: &EvidenceCorpus,
    coordinate: DocumentCoordinate,
    canonical_controls: &std::collections::BTreeSet<crate::EventId>,
) -> std::collections::BTreeMap<crate::EventId, DescriptorAuthorization> {
    let controls = corpus
        .events
        .values()
        .filter_map(|evidence| match evidence {
            EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::Control(control),
                ..
            } => Some((control.event_id(), control.as_ref())),
            _ => None,
        })
        .collect();
    corpus
        .events
        .values()
        .filter_map(|evidence| match evidence {
            EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::CheckpointDescriptor(descriptor),
                ..
            } if descriptor.coordinate() == coordinate => Some((
                descriptor.event_id(),
                authorize_descriptor(descriptor, canonical_controls, &controls),
            )),
            _ => None,
        })
        .collect()
}

fn charge_ingress(corpus: &EvidenceCorpus, budget: &mut WorkBudget) -> bool {
    let event_count = u64::try_from(corpus.evaluation_event_count()).unwrap_or(u64::MAX);
    let carrier_count = u64::try_from(corpus.carrier_evidence_count()).unwrap_or(u64::MAX);
    let decode_bytes = corpus.decode_work_bytes().unwrap_or(u64::MAX);
    budget.charge(WorkCounter::Event, event_count).is_ok()
        && budget.charge(WorkCounter::Carrier, carrier_count).is_ok()
        && budget.charge(WorkCounter::DecodeByte, decode_bytes).is_ok()
}

fn disposition_hashes(
    dispositions: &[(ChangeHash, ProtocolDisposition)],
    expected: ProtocolDisposition,
) -> Vec<ChangeHash> {
    dispositions
        .iter()
        .filter_map(|(hash, disposition)| (*disposition == expected).then_some(*hash))
        .collect()
}

fn controls_for_coordinate(
    corpus: &EvidenceCorpus,
    coordinate: DocumentCoordinate,
) -> Vec<BatchControl> {
    corpus
        .events
        .values()
        .filter_map(|evidence| match evidence {
            EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::Control(control),
                ..
            } if control.coordinate() == coordinate
                && !corpus
                    .indexes
                    .controls
                    .pending
                    .contains(&control.event_id())
                && genesis_link_is_valid(corpus, control)
                && parent_continuity_is_valid(corpus, control)
                && !has_terminal_parent(corpus, control)
                && !violates_retained_writer_frontier(corpus, control) =>
            {
                Some(BatchControl {
                    event_id: control.event_id(),
                    parent: control.parent(),
                    accepted_base: control.base_heads().collect(),
                    frozen: control.terminal(),
                    changes: changes_for_control(corpus, control),
                })
            }
            _ => None,
        })
        .collect()
}

fn parent_continuity_is_valid(
    corpus: &EvidenceCorpus,
    child: &crate::carrier::control::ValidatedControlCarrier,
) -> bool {
    let Some(parent_id) = child.parent() else {
        return true;
    };
    let Some(EventEvidence::VerifiedCarrier {
        carrier: VerifiedCarrier::Control(parent),
        ..
    }) = corpus.events.get(&parent_id)
    else {
        return false;
    };
    let parent = ControlEnvelope::from_validated(parent.as_ref().clone());
    let child = ControlEnvelope::from_validated(child.clone());
    evaluate_parent_continuity(&parent, &child) == CandidateResult::Valid
}

fn genesis_link_is_valid(
    corpus: &EvidenceCorpus,
    control: &crate::carrier::control::ValidatedControlCarrier,
) -> bool {
    if control.parent().is_some() {
        return control.predecessor().is_none();
    }
    let Some(predecessor) = control.predecessor() else {
        return control.sequence() == 0;
    };
    if control.sequence() != 0 {
        return false;
    }
    matches!(
        corpus.events.get(&predecessor.terminal_control),
        Some(EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::Control(terminal),
            ..
        }) if terminal.coordinate() == predecessor.coordinate
            && terminal.terminal()
            && terminal.successor() == Some(control.coordinate())
    )
}

fn violates_retained_writer_frontier(
    corpus: &EvidenceCorpus,
    child: &crate::carrier::control::ValidatedControlCarrier,
) -> bool {
    let Some(parent_id) = child.parent() else {
        return false;
    };
    let Some(EventEvidence::VerifiedCarrier {
        carrier: VerifiedCarrier::Control(parent),
        ..
    }) = corpus.events.get(&parent_id)
    else {
        return false;
    };
    let mut closure = child
        .base_heads()
        .collect::<std::collections::BTreeSet<_>>();
    let mut stack = closure.iter().copied().collect::<Vec<_>>();
    while let Some(hash) = stack.pop() {
        if let Some(dependencies) = corpus.indexes.changes.dependencies_by_hash.get(&hash) {
            for dependency in dependencies {
                if closure.insert(*dependency) {
                    stack.push(*dependency);
                }
            }
        }
    }
    parent.members().iter().any(|grant| {
        let retained = grant.roles.contains(&Role::Write)
            && child
                .members()
                .iter()
                .any(|child_grant| child_grant.device == grant.device);
        retained
            && highest_writer_contribution(corpus, parent_id, grant.actor)
                .is_some_and(|hash| !closure.contains(&hash))
    })
}

fn highest_writer_contribution(
    corpus: &EvidenceCorpus,
    control_id: crate::EventId,
    actor: crate::ActorId,
) -> Option<ChangeHash> {
    corpus
        .events
        .values()
        .filter_map(|evidence| match evidence {
            EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::Change(change),
                ..
            } if change.control_id() == control_id && change.actor() == actor => {
                Some((change.sequence(), change.change_hash()))
            }
            _ => None,
        })
        .max()
        .map(|(_, hash)| hash)
}

fn has_terminal_parent(
    corpus: &EvidenceCorpus,
    control: &crate::carrier::control::ValidatedControlCarrier,
) -> bool {
    control.parent().is_some_and(|parent_id| {
        matches!(
            corpus.events.get(&parent_id),
            Some(EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::Control(parent),
                ..
            }) if parent.coordinate() == control.coordinate() && parent.terminal()
        )
    })
}

fn changes_for_control(
    corpus: &EvidenceCorpus,
    control: &crate::carrier::control::ValidatedControlCarrier,
) -> Vec<BatchChange> {
    let hashes = corpus
        .indexes
        .changes
        .hashes_by_control
        .get(&control.event_id())
        .cloned()
        .unwrap_or_default();
    hashes
        .into_iter()
        .filter_map(|hash| change_for_hash(corpus, control, hash))
        .collect()
}

fn change_for_hash(
    corpus: &EvidenceCorpus,
    control: &crate::carrier::control::ValidatedControlCarrier,
    hash: ChangeHash,
) -> Option<BatchChange> {
    let event_ids = corpus.indexes.changes.carriers_by_hash.get(&hash)?;
    let mut raw = None;
    let carriers = event_ids
        .iter()
        .filter_map(|event_id| match corpus.events.get(event_id) {
            Some(EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::Change(change),
                ..
            }) if change.control_id() == control.event_id()
                && change.coordinate() == control.coordinate() =>
            {
                raw.get_or_insert_with(|| change.canonical_raw_bytes().to_vec());
                Some(CandidateCarrier {
                    event_id: change.event_id(),
                    change_hash: change.change_hash(),
                    actor: change.actor(),
                    sequence: change.sequence(),
                    start_op: change.start_op(),
                    operation_count: change.operation_count(),
                    dependencies: change.dependencies().collect(),
                    control_id: change.control_id(),
                    author: change.author_device(),
                })
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let candidate = ChangeCandidate::from_carriers(carriers).ok()?;
    let authorized = control.members().iter().any(|member| {
        member.actor == candidate.actor
            && member.device == candidate.author
            && member.roles.contains(&Role::Write)
    });
    Some(BatchChange {
        candidate,
        semantically_valid: authorized && !control.terminal(),
        raw_change: raw,
    })
}

#[cfg(test)]
mod tests {
    use super::{assembly_status, join_status};
    use crate::CheckpointVerificationStatus as Status;
    use crate::checkpoint::AssemblyError;
    use crate::checkpoint::join::JoinError;

    #[test]
    fn every_checkpoint_refusal_has_a_stable_public_status() {
        assert_eq!(join_status(JoinError::Author), Status::ChunkAuthorMismatch);
        assert_eq!(
            join_status(JoinError::Coordinate),
            Status::ChunkCoordinateMismatch
        );
        assert_eq!(
            join_status(JoinError::Descriptor),
            Status::ChunkDescriptorMismatch
        );
        assert_eq!(join_status(JoinError::Count), Status::ChunkCountMismatch);
        assert_eq!(
            join_status(JoinError::DuplicateIndex),
            Status::DuplicateChunk
        );
        assert_eq!(join_status(JoinError::MissingIndex), Status::MissingChunk);
        assert_eq!(join_status(JoinError::Size), Status::ChunkSizeMismatch);
        assert_eq!(
            assembly_status(AssemblyError::Chunks),
            Status::ChunkAssemblyMismatch
        );
        assert_eq!(
            assembly_status(AssemblyError::Proof),
            Status::MerkleMismatch
        );
        assert_eq!(
            assembly_status(AssemblyError::SnapshotSize),
            Status::SnapshotSizeMismatch
        );
        assert_eq!(
            assembly_status(AssemblyError::SnapshotHash),
            Status::SnapshotHashMismatch
        );
        assert_eq!(
            assembly_status(AssemblyError::Budget),
            Status::BudgetExhausted
        );
        assert_eq!(assembly_status(AssemblyError::Cancelled), Status::Cancelled);
    }
}
