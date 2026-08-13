use std::collections::{BTreeMap, BTreeSet};

use crate::carrier::change::ChangeCarrierError;
use crate::carrier::classify::classify;
use crate::carrier::control::{ControlCarrierError, ControlContentError};
use crate::carrier::manifest::ManifestContentError;
use crate::carrier::{CarrierCandidate, VerifiedCarrier};
use crate::evidence::event::{EventEvidence, RawChecksum};
use crate::evidence::source::AcquiredRawEvent;
use crate::wire::tags::TagError;
use crate::{DiagnosticCode, EventId, Nip01VerificationError, RawEventBytes, VerifiedNip01Event};

/// The stable result of ingesting one raw event into an evidence corpus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IngestOutcome {
    /// A supported, verified protocol carrier was retained.
    Accepted {
        /// The verified carrier event identifier.
        event_id: EventId,
    },
    /// The event identifier was already represented in the corpus.
    Duplicate {
        /// The already-retained event identifier.
        event_id: EventId,
    },
    /// The raw event failed strict NIP-01 verification.
    Invalid {
        /// The stable strict-verification failure code.
        diagnostic: DiagnosticCode,
    },
    /// The signed event is valid but is not a draft-v1 carrier kind.
    Irrelevant {
        /// The valid non-carrier event identifier.
        event_id: EventId,
    },
    /// The carrier declares an unsupported protocol revision.
    UnsupportedRevision {
        /// The verified carrier event identifier.
        event_id: EventId,
        /// The stable unsupported-revision code.
        diagnostic: DiagnosticCode,
    },
    /// The signed carrier failed its kind-specific validation.
    InvalidCarrier {
        /// The verified carrier event identifier.
        event_id: EventId,
        /// The stable carrier-validation failure code.
        diagnostic: DiagnosticCode,
    },
}

#[derive(Default)]
/// A deterministic single-use builder for retained raw-event evidence.
pub struct CorpusBuilder {
    events: BTreeMap<EventId, EventEvidence>,
    invalid: BTreeMap<RawChecksum, EventEvidence>,
    duplicates: Vec<EventEvidence>,
}

#[derive(Clone, Debug, PartialEq)]
/// Immutable retained ingress evidence produced by [`CorpusBuilder`].
pub struct EvidenceCorpus {
    pub(crate) events: BTreeMap<EventId, EventEvidence>,
    pub(crate) invalid: BTreeMap<RawChecksum, EventEvidence>,
    pub(crate) duplicates: Vec<EventEvidence>,
    pub(crate) indexes: crate::evidence::indexes::TrustedIndexes,
}

impl CorpusBuilder {
    /// Creates an empty deterministic evidence-corpus builder.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            events: BTreeMap::new(),
            invalid: BTreeMap::new(),
            duplicates: Vec::new(),
        }
    }

    /// Validates and ingests exact raw signed-event bytes under draft-v1.
    pub fn ingest_bytes(&mut self, bytes: &[u8]) -> IngestOutcome {
        match RawEventBytes::new(bytes, crate::ProtocolRevision::draft_v1()) {
            Ok(raw) => self.ingest(raw),
            Err(error) => IngestOutcome::Invalid {
                diagnostic: error.diagnostic(),
            },
        }
    }

    pub(crate) fn ingest_acquired(&mut self, acquired: AcquiredRawEvent) -> IngestOutcome {
        self.ingest(acquired.into_raw())
    }

    /// Strictly verifies and deterministically retains one bounded raw event.
    pub fn ingest(&mut self, raw: RawEventBytes) -> IngestOutcome {
        let checksum = RawChecksum::of(&raw);
        match VerifiedNip01Event::verify(raw.clone()) {
            Ok(event) => self.ingest_verified(event, checksum),
            Err(error) => {
                let diagnostic = verification_diagnostic(error);
                self.invalid
                    .entry(checksum)
                    .or_insert_with(|| EventEvidence::InvalidEvent {
                        raw,
                        raw_checksum: checksum,
                        diagnostic,
                    });
                IngestOutcome::Invalid { diagnostic }
            }
        }
    }

    fn ingest_verified(
        &mut self,
        event: VerifiedNip01Event,
        checksum: RawChecksum,
    ) -> IngestOutcome {
        let event_id = event.event_id();
        if let Some(existing) = self.events.get(&event_id) {
            let existing_checksum = evidence_checksum(existing);
            self.duplicates.push(EventEvidence::DuplicateEvent {
                event_id,
                raw_checksum: checksum,
            });
            if existing_checksum != checksum {
                self.duplicates.sort_by_key(evidence_checksum);
            }
            return IngestOutcome::Duplicate { event_id };
        }
        let (evidence, outcome) = match classify(event.clone()) {
            Some(CarrierCandidate::UnsupportedRevision {
                event,
                declared_version,
                declared_profile,
            }) => {
                let diagnostic = DiagnosticCode::registered("carrier.revision");
                let carrier = VerifiedCarrier::UnsupportedRevision {
                    event,
                    declared_version,
                    declared_profile,
                };
                (
                    EventEvidence::UnsupportedRevision {
                        carrier,
                        raw_checksum: checksum,
                        diagnostic,
                    },
                    IngestOutcome::UnsupportedRevision {
                        event_id,
                        diagnostic,
                    },
                )
            }
            Some(CarrierCandidate::Manifest(event)) => {
                match crate::carrier::manifest::validate(&event) {
                    Ok(manifest) => (
                        EventEvidence::VerifiedCarrier {
                            carrier: VerifiedCarrier::Manifest(Box::new(manifest)),
                            raw_checksum: checksum,
                        },
                        IngestOutcome::Accepted { event_id },
                    ),
                    Err(error) => {
                        let diagnostic = manifest_diagnostic(error);
                        (
                            EventEvidence::InvalidCarrier {
                                event,
                                raw_checksum: checksum,
                                diagnostic,
                            },
                            IngestOutcome::InvalidCarrier {
                                event_id,
                                diagnostic,
                            },
                        )
                    }
                }
            }
            Some(CarrierCandidate::Control(event)) => {
                match crate::carrier::control::validate(&event) {
                    Ok(control) => accepted(
                        VerifiedCarrier::Control(Box::new(control)),
                        checksum,
                        event_id,
                    ),
                    Err(error) => {
                        invalid_carrier(event, checksum, event_id, control_diagnostic(error))
                    }
                }
            }
            Some(CarrierCandidate::Change(event)) => match crate::carrier::change::validate(&event)
            {
                Ok(change) => accepted(
                    VerifiedCarrier::Change(Box::new(change)),
                    checksum,
                    event_id,
                ),
                Err(error) => invalid_carrier(event, checksum, event_id, change_diagnostic(error)),
            },
            Some(CarrierCandidate::CheckpointDescriptor(event)) => {
                match crate::carrier::checkpoint_descriptor::validate(&event) {
                    Ok(descriptor) => accepted(
                        VerifiedCarrier::CheckpointDescriptor(Box::new(descriptor)),
                        checksum,
                        event_id,
                    ),
                    Err(error) => invalid_carrier(
                        event,
                        checksum,
                        event_id,
                        checkpoint_descriptor_diagnostic(error),
                    ),
                }
            }
            Some(CarrierCandidate::CheckpointChunk(event)) => {
                match crate::carrier::checkpoint_chunk::validate(&event) {
                    Ok(chunk) => accepted(
                        VerifiedCarrier::CheckpointChunk(Box::new(chunk)),
                        checksum,
                        event_id,
                    ),
                    Err(error) => invalid_carrier(
                        event,
                        checksum,
                        event_id,
                        checkpoint_chunk_diagnostic(error),
                    ),
                }
            }
            None => (
                EventEvidence::IrrelevantEvent {
                    event,
                    raw_checksum: checksum,
                },
                IngestOutcome::Irrelevant { event_id },
            ),
        };
        self.events.insert(event_id, evidence);
        outcome
    }

    /// Consumes the builder and returns immutable retained ingress evidence.
    #[must_use]
    pub fn finish(mut self) -> EvidenceCorpus {
        self.duplicates.sort_by_key(evidence_checksum);
        let indexes =
            crate::evidence::indexes::derive_trusted_indexes(&self.events, &self.duplicates);
        EvidenceCorpus {
            events: self.events,
            invalid: self.invalid,
            duplicates: self.duplicates,
            indexes,
        }
    }
}

fn checkpoint_chunk_diagnostic(
    error: crate::carrier::checkpoint_chunk::CheckpointChunkCarrierError,
) -> DiagnosticCode {
    use crate::carrier::checkpoint_chunk::CheckpointChunkCarrierError as Error;
    match error {
        Error::Kind => DiagnosticCode::registered("carrier.kind"),
        Error::Tags(crate::wire::tags::TagError::Forbidden) => {
            DiagnosticCode::registered("tag.forbidden")
        }
        Error::Tags(_) => DiagnosticCode::registered("tag.required"),
        Error::Coordinate => DiagnosticCode::registered("carrier.coordinate"),
        Error::Descriptor | Error::Hash | Error::Part | Error::Chunk(_) => {
            DiagnosticCode::registered("checkpoint.chunk")
        }
    }
}

fn checkpoint_descriptor_diagnostic(
    error: crate::carrier::checkpoint_descriptor::CheckpointDescriptorCarrierError,
) -> DiagnosticCode {
    use crate::carrier::checkpoint_descriptor::CheckpointDescriptorCarrierError as Error;
    match error {
        Error::Kind => DiagnosticCode::registered("carrier.kind"),
        Error::Tags(crate::wire::tags::TagError::Forbidden) => {
            DiagnosticCode::registered("tag.forbidden")
        }
        Error::Tags(_) => DiagnosticCode::registered("tag.required"),
        Error::Coordinate => DiagnosticCode::registered("carrier.coordinate"),
        Error::Control | Error::Snapshot | Error::Descriptor(_) => {
            DiagnosticCode::registered("checkpoint.descriptor")
        }
    }
}

fn accepted(
    carrier: VerifiedCarrier,
    raw_checksum: RawChecksum,
    event_id: EventId,
) -> (EventEvidence, IngestOutcome) {
    (
        EventEvidence::VerifiedCarrier {
            carrier,
            raw_checksum,
        },
        IngestOutcome::Accepted { event_id },
    )
}

fn invalid_carrier(
    event: VerifiedNip01Event,
    raw_checksum: RawChecksum,
    event_id: EventId,
    diagnostic: DiagnosticCode,
) -> (EventEvidence, IngestOutcome) {
    (
        EventEvidence::InvalidCarrier {
            event,
            raw_checksum,
            diagnostic,
        },
        IngestOutcome::InvalidCarrier {
            event_id,
            diagnostic,
        },
    )
}

impl EvidenceCorpus {
    pub(crate) fn evaluation_event_count(&self) -> usize {
        self.events
            .len()
            .saturating_add(self.invalid.len())
            .saturating_add(self.duplicates.len())
    }

    pub(crate) fn carrier_evidence_count(&self) -> usize {
        self.events
            .values()
            .filter(|evidence| {
                matches!(
                    evidence,
                    EventEvidence::VerifiedCarrier { .. }
                        | EventEvidence::InvalidCarrier { .. }
                        | EventEvidence::UnsupportedRevision { .. }
                )
            })
            .count()
    }

    pub(crate) fn decode_work_bytes(&self) -> Option<u64> {
        self.indexes
            .changes
            .preferred_carrier
            .values()
            .try_fold(0_u64, |total, event_id| {
                let work = match self.events.get(event_id) {
                    Some(EventEvidence::VerifiedCarrier {
                        carrier: VerifiedCarrier::Change(change),
                        ..
                    }) => change.decode_work_bytes()?,
                    _ => return None,
                };
                total.checked_add(work)
            })
    }

    /// Returns the number of uniquely identified verified signed events.
    #[must_use]
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Returns the number of unique invalid raw-event byte strings.
    #[must_use]
    pub fn invalid_count(&self) -> usize {
        self.invalid.len()
    }

    /// Returns the number of duplicate event observations.
    #[must_use]
    pub fn duplicate_count(&self) -> usize {
        self.duplicates.len()
    }

    /// Iterates over event IDs of fully validated signed control carriers.
    pub fn control_ids(&self) -> impl Iterator<Item = EventId> + '_ {
        self.indexes.controls.controls_by_id.keys().copied()
    }

    /// Iterates over validated controls awaiting missing parent or frontier evidence.
    pub fn pending_control_ids(&self) -> impl Iterator<Item = EventId> + '_ {
        self.indexes.controls.pending.iter().copied()
    }

    /// Iterates over canonical hashes represented by fully validated change carriers.
    pub fn change_hashes(&self) -> impl Iterator<Item = crate::ChangeHash> + '_ {
        self.indexes.changes.carriers_by_hash.keys().copied()
    }

    /// Iterates over event IDs of fully validated checkpoint descriptors.
    pub fn checkpoint_descriptor_ids(&self) -> impl Iterator<Item = EventId> + '_ {
        self.indexes.checkpoints.descriptors_by_id.keys().copied()
    }

    /// Iterates over event IDs of fully validated checkpoint chunks.
    pub fn checkpoint_chunk_ids(&self) -> impl Iterator<Item = EventId> + '_ {
        self.indexes.checkpoints.chunks_by_id.keys().copied()
    }

    /// Iterates over validated checkpoint evidence awaiting a referenced carrier.
    pub fn pending_checkpoint_ids(&self) -> impl Iterator<Item = EventId> + '_ {
        self.indexes
            .checkpoints
            .pending_descriptors
            .iter()
            .chain(&self.indexes.checkpoints.pending_chunks)
            .copied()
    }

    /// Iterates over every retained evidence record in deterministic order.
    pub fn records(&self) -> impl Iterator<Item = EvidenceRecord> + '_ {
        self.events
            .iter()
            .map(|entry| event_record(entry, &self.indexes))
            .chain(self.invalid.iter().map(invalid_record))
            .chain(self.duplicates.iter().filter_map(duplicate_record))
    }

    /// Iterates over advisory acquisition hints from fully validated manifests.
    ///
    /// This is a static replacement-selection query and does not resolve the
    /// referenced control. These hints never select canonical controls,
    /// changes, or checkpoints. Use [`crate::EvaluationReport::manifest`] for
    /// the replacement-first stateful result.
    pub fn manifest_hints(&self) -> impl Iterator<Item = ManifestHints> + '_ {
        selected_manifests(&self.events)
            .into_values()
            .filter_map(|selection| match selection.state {
                ManifestSelectionState::Available(hints) => Some(hints),
                ManifestSelectionState::Unavailable(_) => None,
            })
    }

    /// Returns static NIP-01 replacement selection and validation for one coordinate.
    ///
    /// An invalid latest event is unavailable; this never falls back to an
    /// older manifest and never grants control or checkpoint authority. This
    /// query does not resolve the selected manifest's control reference; use
    /// [`crate::EvaluationReport::manifest`] for that dynamic result.
    #[must_use]
    pub fn selected_manifest(&self, coordinate: crate::DocumentCoordinate) -> ManifestAvailability {
        let Some(selection) = selected_manifests(&self.events).remove(&coordinate) else {
            return ManifestAvailability::Missing;
        };
        match selection.state {
            ManifestSelectionState::Available(hints) => ManifestAvailability::Available(hints),
            ManifestSelectionState::Unavailable(diagnostic) => ManifestAvailability::Unavailable {
                event_id: selection.event_id,
                diagnostic,
            },
        }
    }

    pub(crate) fn selected_manifest_selection(
        &self,
        coordinate: crate::DocumentCoordinate,
    ) -> Option<ManifestSelection> {
        selected_manifests(&self.events).remove(&coordinate)
    }

    pub(crate) fn selected_manifest_selection_in(
        &self,
        coordinate: crate::DocumentCoordinate,
        event_ids: &std::collections::BTreeSet<EventId>,
    ) -> Option<ManifestSelection> {
        selected_manifest_in(&self.events, event_ids, coordinate)
    }

    pub(crate) fn record_for_event(&self, event_id: &EventId) -> Option<EvidenceRecord> {
        self.events
            .get_key_value(event_id)
            .map(|entry| event_record(entry, &self.indexes))
    }

    pub(crate) fn record_for_duplicate(&self, evidence: &EventEvidence) -> Option<EvidenceRecord> {
        duplicate_record(evidence)
    }

    /// Returns true when no evidence of any class was retained.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty() && self.invalid.is_empty() && self.duplicates.is_empty()
    }
}

/// A safe public identifier for retained evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EvidenceIdentifier {
    /// A strictly verified signed event identifier.
    Event(EventId),
    /// The SHA-256 checksum of invalid raw bytes that had no trusted event ID.
    InvalidRawSha256([u8; 32]),
}

/// The immutable trust disposition of retained ingress evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EvidenceStatus {
    /// A fully validated protocol carrier.
    Valid,
    /// Valid signed evidence awaiting dependency reevaluation.
    Pending,
    /// Invalid raw event or signed carrier evidence.
    Invalid,
    /// A valid signed carrier declaring an unsupported revision.
    Unsupported,
    /// A valid signed event outside the sealed carrier kinds.
    Irrelevant,
    /// A repeated observation of an already retained event.
    Duplicate,
}

/// One immutable content-free public evidence summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvidenceRecord {
    identifier: EvidenceIdentifier,
    status: EvidenceStatus,
    diagnostic: Option<DiagnosticCode>,
}

impl EvidenceRecord {
    /// Returns the trusted event ID or invalid-raw checksum identifier.
    #[must_use]
    pub const fn identifier(&self) -> EvidenceIdentifier {
        self.identifier
    }

    /// Returns the retained trust disposition.
    #[must_use]
    pub const fn status(&self) -> EvidenceStatus {
        self.status
    }

    /// Returns the stable failure diagnostic when applicable.
    #[must_use]
    pub const fn diagnostic(&self) -> Option<DiagnosticCode> {
        self.diagnostic
    }
}

fn event_record(
    (event_id, evidence): (&EventId, &EventEvidence),
    indexes: &crate::evidence::indexes::TrustedIndexes,
) -> EvidenceRecord {
    let (status, diagnostic) = match evidence {
        EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::Control(_),
            ..
        } if indexes.controls.pending.contains(event_id) => (EvidenceStatus::Pending, None),
        EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::CheckpointDescriptor(_),
            ..
        } if indexes.checkpoints.pending_descriptors.contains(event_id) => {
            (EvidenceStatus::Pending, None)
        }
        EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::CheckpointChunk(_),
            ..
        } if indexes.checkpoints.pending_chunks.contains(event_id) => {
            (EvidenceStatus::Pending, None)
        }
        EventEvidence::VerifiedCarrier { .. } => (EvidenceStatus::Valid, None),
        EventEvidence::InvalidCarrier { diagnostic, .. } => {
            (EvidenceStatus::Invalid, Some(*diagnostic))
        }
        EventEvidence::UnsupportedRevision { diagnostic, .. } => {
            (EvidenceStatus::Unsupported, Some(*diagnostic))
        }
        EventEvidence::IrrelevantEvent { .. } => (EvidenceStatus::Irrelevant, None),
        EventEvidence::InvalidEvent { diagnostic, .. } => {
            (EvidenceStatus::Invalid, Some(*diagnostic))
        }
        EventEvidence::DuplicateEvent { .. } => (EvidenceStatus::Duplicate, None),
    };
    EvidenceRecord {
        identifier: EvidenceIdentifier::Event(*event_id),
        status,
        diagnostic,
    }
}

fn invalid_record((checksum, evidence): (&RawChecksum, &EventEvidence)) -> EvidenceRecord {
    let diagnostic = match evidence {
        EventEvidence::InvalidEvent { diagnostic, .. } => Some(*diagnostic),
        _ => None,
    };
    EvidenceRecord {
        identifier: EvidenceIdentifier::InvalidRawSha256(*checksum.as_bytes()),
        status: EvidenceStatus::Invalid,
        diagnostic,
    }
}

fn duplicate_record(evidence: &EventEvidence) -> Option<EvidenceRecord> {
    let EventEvidence::DuplicateEvent { event_id, .. } = evidence else {
        return None;
    };
    Some(EvidenceRecord {
        identifier: EvidenceIdentifier::Event(*event_id),
        status: EvidenceStatus::Duplicate,
        diagnostic: None,
    })
}

/// Advisory acquisition hints from one fully validated signed manifest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestHints {
    event_id: EventId,
    coordinate: crate::DocumentCoordinate,
    control: EventId,
    checkpoint: Option<EventId>,
    relays: Vec<String>,
}

/// Advisory manifest availability after NIP-01 replacement selection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ManifestAvailability {
    /// No signed addressable manifest was retained for the coordinate.
    Missing,
    /// The selected signed manifest passed complete profile validation.
    Available(ManifestHints),
    /// The selected signed manifest was invalid; older events are not used.
    Unavailable {
        /// The selected invalid signed event identifier.
        event_id: EventId,
        /// The stable manifest validation diagnostic.
        diagnostic: DiagnosticCode,
    },
}

/// Dynamic status of the selected manifest's advisory control reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ManifestControlStatus {
    /// The referenced control is accepted on the canonical control chain.
    Canonical,
    /// The referenced control is statefully valid but lost canonical selection.
    Noncanonical,
}

/// Reason that a selected manifest's control reference is not yet resolvable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ManifestPendingReason {
    /// The referenced control event is not present in retained evidence.
    MissingControl,
    /// The referenced control cannot yet receive a conclusive stateful outcome.
    ControlPending,
}

/// Replacement-first selected-manifest result after stateful control evaluation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedManifestAvailability {
    /// No signed addressable manifest was retained for the evaluated coordinate.
    Missing,
    /// The selected manifest is usable as advisory discovery data.
    Available {
        /// The selected manifest's non-authoritative acquisition hints.
        hints: ManifestHints,
        /// Whether the hinted control is canonical or valid but noncanonical.
        control_status: ManifestControlStatus,
    },
    /// The selected manifest awaits conclusive control evidence; no older event is used.
    Pending {
        /// The selected manifest's non-authoritative acquisition hints.
        hints: ManifestHints,
        /// Why the control reference cannot yet be resolved.
        reason: ManifestPendingReason,
    },
    /// The selected manifest or its referenced control is invalid or unavailable.
    Unavailable {
        /// The selected signed manifest event identifier.
        event_id: EventId,
        /// The referenced control when static manifest validation exposed it.
        control: Option<EventId>,
        /// The stable validation or dynamic-resolution diagnostic.
        diagnostic: DiagnosticCode,
    },
}

impl ManifestHints {
    pub(crate) fn new(
        event_id: EventId,
        coordinate: crate::DocumentCoordinate,
        control: EventId,
        checkpoint: Option<EventId>,
        relays: Vec<String>,
    ) -> Self {
        Self {
            event_id,
            coordinate,
            control,
            checkpoint,
            relays,
        }
    }

    /// Returns the signed manifest event identifier.
    #[must_use]
    pub const fn event_id(&self) -> EventId {
        self.event_id
    }

    /// Returns the document coordinate named by the manifest.
    #[must_use]
    pub const fn coordinate(&self) -> crate::DocumentCoordinate {
        self.coordinate
    }

    /// Returns the manifest's non-authoritative control hint.
    #[must_use]
    pub const fn control(&self) -> EventId {
        self.control
    }

    /// Returns the manifest's non-authoritative checkpoint hint.
    #[must_use]
    pub const fn checkpoint(&self) -> Option<EventId> {
        self.checkpoint
    }

    /// Returns sorted relay acquisition hints.
    #[must_use]
    pub fn relays(&self) -> &[String] {
        &self.relays
    }
}

#[derive(Clone)]
pub(crate) struct ManifestSelection {
    pub(crate) created_at: u64,
    pub(crate) event_id: EventId,
    pub(crate) state: ManifestSelectionState,
}

#[derive(Clone)]
pub(crate) enum ManifestSelectionState {
    Available(ManifestHints),
    Unavailable(DiagnosticCode),
}

fn selected_manifests(
    events: &BTreeMap<EventId, EventEvidence>,
) -> BTreeMap<crate::DocumentCoordinate, ManifestSelection> {
    let mut selected = BTreeMap::new();
    for evidence in events.values() {
        let candidate = manifest_selection_candidate(evidence);
        let Some((coordinate, candidate)) = candidate else {
            continue;
        };
        select_manifest_candidate(&mut selected, coordinate, candidate);
    }
    selected
}

fn manifest_selection_candidate(
    evidence: &EventEvidence,
) -> Option<(crate::DocumentCoordinate, ManifestSelection)> {
    match evidence {
        EventEvidence::VerifiedCarrier {
            carrier: VerifiedCarrier::Manifest(manifest),
            ..
        } => Some((
            manifest.coordinate(),
            ManifestSelection {
                created_at: manifest.created_at(),
                event_id: manifest.event_id,
                state: ManifestSelectionState::Available(manifest.acquisition_hints()),
            },
        )),
        EventEvidence::InvalidCarrier {
            event, diagnostic, ..
        } => manifest_coordinate(event).map(|coordinate| {
            (
                coordinate,
                ManifestSelection {
                    created_at: event.created_at(),
                    event_id: event.event_id(),
                    state: ManifestSelectionState::Unavailable(*diagnostic),
                },
            )
        }),
        EventEvidence::UnsupportedRevision {
            carrier: VerifiedCarrier::UnsupportedRevision { event, .. },
            diagnostic,
            ..
        } => manifest_coordinate(event).map(|coordinate| {
            (
                coordinate,
                ManifestSelection {
                    created_at: event.created_at(),
                    event_id: event.event_id(),
                    state: ManifestSelectionState::Unavailable(*diagnostic),
                },
            )
        }),
        _ => None,
    }
}

fn select_manifest_candidate(
    selected: &mut BTreeMap<crate::DocumentCoordinate, ManifestSelection>,
    coordinate: crate::DocumentCoordinate,
    candidate: ManifestSelection,
) {
    selected
        .entry(coordinate)
        .and_modify(|current| {
            if candidate.created_at > current.created_at
                || candidate.created_at == current.created_at
                    && candidate.event_id < current.event_id
            {
                *current = candidate.clone();
            }
        })
        .or_insert(candidate);
}

fn selected_manifest_in(
    events: &BTreeMap<EventId, EventEvidence>,
    event_ids: &std::collections::BTreeSet<EventId>,
    coordinate: crate::DocumentCoordinate,
) -> Option<ManifestSelection> {
    let mut selected = None;
    for event_id in event_ids {
        let Some((candidate_coordinate, candidate)) =
            events.get(event_id).and_then(manifest_selection_candidate)
        else {
            continue;
        };
        if candidate_coordinate != coordinate {
            continue;
        }
        let replace = selected.as_ref().is_none_or(|current: &ManifestSelection| {
            candidate.created_at > current.created_at
                || candidate.created_at == current.created_at
                    && candidate.event_id < current.event_id
        });
        if replace {
            selected = Some(candidate);
        }
    }
    selected
}

pub(crate) fn manifest_coordinate(event: &VerifiedNip01Event) -> Option<crate::DocumentCoordinate> {
    if event.kind() != 31_624 {
        return None;
    }
    let document_ids = event
        .tags()
        .iter()
        .filter(|tag| tag.first().is_some_and(|value| value == "d"))
        .filter_map(|tag| tag.get(1)?.parse().ok())
        .collect::<BTreeSet<_>>();
    if document_ids.len() != 1 {
        return None;
    }
    let document_id = document_ids.into_iter().next()?;
    Some(crate::DocumentCoordinate::new(
        crate::ControllerPublicKey::from_bytes(*event.author_bytes()),
        document_id,
    ))
}

fn evidence_checksum(evidence: &EventEvidence) -> RawChecksum {
    match evidence {
        EventEvidence::VerifiedCarrier { raw_checksum, .. }
        | EventEvidence::InvalidEvent { raw_checksum, .. }
        | EventEvidence::InvalidCarrier { raw_checksum, .. }
        | EventEvidence::UnsupportedRevision { raw_checksum, .. }
        | EventEvidence::IrrelevantEvent { raw_checksum, .. }
        | EventEvidence::DuplicateEvent { raw_checksum, .. } => *raw_checksum,
    }
}

const fn manifest_diagnostic(error: ManifestContentError) -> DiagnosticCode {
    match error {
        ManifestContentError::Canonical(_) => DiagnosticCode::registered("jcs.noncanonical"),
        ManifestContentError::Tags => DiagnosticCode::registered("tag.required"),
        ManifestContentError::Shape => DiagnosticCode::registered("manifest.structure"),
        ManifestContentError::Semantics => DiagnosticCode::registered("manifest.semantics"),
    }
}

const fn control_diagnostic(error: ControlCarrierError) -> DiagnosticCode {
    match error {
        ControlCarrierError::Kind => DiagnosticCode::registered("carrier.kind"),
        ControlCarrierError::Tags(
            TagError::Missing | TagError::Repeated | TagError::ElementCount,
        ) => DiagnosticCode::registered("tag.required"),
        ControlCarrierError::Tags(TagError::Forbidden | TagError::NonCanonicalOrder) => {
            DiagnosticCode::registered("tag.forbidden")
        }
        ControlCarrierError::Coordinate => DiagnosticCode::registered("carrier.coordinate"),
        ControlCarrierError::Content(ControlContentError::Canonical(_)) => {
            DiagnosticCode::registered("jcs.noncanonical")
        }
        ControlCarrierError::Content(ControlContentError::Shape) => {
            DiagnosticCode::registered("control.structure")
        }
        ControlCarrierError::Content(ControlContentError::Semantics) => {
            DiagnosticCode::registered("control.structure")
        }
    }
}

const fn change_diagnostic(error: ChangeCarrierError) -> DiagnosticCode {
    use crate::automerge_adapter::encode::ReencodeError;
    use crate::automerge_adapter::framing::FramingError;

    match error {
        ChangeCarrierError::Kind => DiagnosticCode::registered("carrier.kind"),
        ChangeCarrierError::Tags => DiagnosticCode::registered("tag.required"),
        ChangeCarrierError::Base64 => DiagnosticCode::registered("base64.noncanonical"),
        ChangeCarrierError::Hash => DiagnosticCode::registered("change.hash"),
        ChangeCarrierError::Actor => DiagnosticCode::registered("change.actor"),
        ChangeCarrierError::Automerge(ReencodeError::Framing(FramingError::Magic)) => {
            DiagnosticCode::registered("automerge.magic")
        }
        ChangeCarrierError::Automerge(ReencodeError::Framing(FramingError::ForbiddenChunk(_))) => {
            DiagnosticCode::registered("automerge.chunk_type")
        }
        ChangeCarrierError::Automerge(ReencodeError::Framing(FramingError::Leb128(_))) => {
            DiagnosticCode::registered("automerge.leb128")
        }
        ChangeCarrierError::Automerge(ReencodeError::Framing(FramingError::Checksum)) => {
            DiagnosticCode::registered("automerge.checksum")
        }
        ChangeCarrierError::Automerge(ReencodeError::Framing(
            FramingError::Truncated | FramingError::TooLarge | FramingError::Length,
        )) => DiagnosticCode::registered("automerge.length"),
        ChangeCarrierError::Automerge(ReencodeError::NonCanonical) => {
            DiagnosticCode::registered("automerge.canonical")
        }
        ChangeCarrierError::Automerge(_) => DiagnosticCode::registered("automerge.semantics"),
    }
}

const fn verification_diagnostic(error: Nip01VerificationError) -> DiagnosticCode {
    match error {
        Nip01VerificationError::JsonSyntax => DiagnosticCode::registered("json.syntax"),
        Nip01VerificationError::DuplicateMember => {
            DiagnosticCode::registered("json.duplicate_member")
        }
        Nip01VerificationError::Shape => DiagnosticCode::registered("nip01.shape"),
        Nip01VerificationError::Identifier => DiagnosticCode::registered("nip01.identifier"),
        Nip01VerificationError::Serialization | Nip01VerificationError::EventIdMismatch => {
            DiagnosticCode::registered("nip01.event_id")
        }
        Nip01VerificationError::InvalidPublicKey | Nip01VerificationError::InvalidSignature => {
            DiagnosticCode::registered("nip01.signature")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CorpusBuilder, IngestOutcome};
    use crate::{ProtocolRevision, RawEventBytes};

    fn raw(value: &[u8]) -> Option<RawEventBytes> {
        RawEventBytes::new(value, ProtocolRevision::draft_v1()).ok()
    }

    #[test]
    fn implement_corpusbuilder_idempotent_ingestion() {
        let valid = include_bytes!("../../../../fixtures/v1_draft/nip01/valid_event.json");
        let valid = raw(valid);
        let first_invalid = raw(b"{}");
        let second_invalid = raw(b"[]");
        assert!(valid.is_some() && first_invalid.is_some() && second_invalid.is_some());
        let (valid, first_invalid, second_invalid) = match (valid, first_invalid, second_invalid) {
            (Some(valid), Some(first), Some(second)) => (valid, first, second),
            _ => return,
        };

        let mut first = CorpusBuilder::default();
        assert!(matches!(
            first.ingest(second_invalid.clone()),
            IngestOutcome::Invalid { .. }
        ));
        assert!(matches!(
            first.ingest(valid.clone()),
            IngestOutcome::Irrelevant { .. }
        ));
        assert!(matches!(
            first.ingest(first_invalid.clone()),
            IngestOutcome::Invalid { .. }
        ));
        assert!(matches!(
            first.ingest(valid.clone()),
            IngestOutcome::Duplicate { .. }
        ));
        let first = first.finish();
        assert_eq!(first.events.len(), 1);
        assert_eq!(first.invalid.len(), 2);
        assert_eq!(first.duplicates.len(), 1);

        let mut second = CorpusBuilder::default();
        second.ingest(first_invalid);
        second.ingest(valid.clone());
        second.ingest(second_invalid);
        second.ingest(valid);
        assert_eq!(first, second.finish());
    }
}
