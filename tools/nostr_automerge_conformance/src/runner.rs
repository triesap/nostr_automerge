use core::str::FromStr;
use std::fs;
use std::path::{Path, PathBuf};

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use nostr_automerge::{
    CheckpointVerificationStatus, Completion, ControllerPublicKey, CorpusBuilder, DevicePublicKey,
    DocumentCoordinate, DocumentId, EvaluationReport, EvidenceIdentifier, EvidenceStatus,
    IngestOutcome, IntegrityAlert, MaterializedMark, MaterializedMarkExpansion,
    MaterializedObjectType, MaterializedPathElement, MaterializedScalar, MaterializedValue,
    NeverCancelled, ProtocolRevision, ReferenceEvaluator, WorkBudget,
    canonical_dispositions_digest, canonical_history_digest,
};

use crate::checksum::verify_fixture_files;
use crate::expected::{
    CheckpointResult, DispositionRecord, ExpectedReport, StateAssertion, load_expected,
};
use crate::fixture::{load_fixture, load_normative_fixture};
use crate::permutations::required_delivery_permutations;
use crate::report_json::write_canonical_report;
use crate::scenario::{ScenarioInput, SignedScenarioInput};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RunError {
    Fixture,
    Checksum,
    Expected,
    Input,
    Evaluation,
    Mismatch,
}

impl RunError {
    pub(crate) const fn exit_code(self) -> u8 {
        match self {
            Self::Mismatch => 1,
            _ => 2,
        }
    }

    pub(crate) const fn message(self) -> &'static str {
        match self {
            Self::Fixture => "fixture metadata is invalid",
            Self::Checksum => "fixture checksum verification failed",
            Self::Expected => "expected report is invalid",
            Self::Input => "fixture input is invalid or unsupported",
            Self::Evaluation => "reference evaluation failed",
            Self::Mismatch => "fixture result does not match expected report",
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ActorDerivationInput {
    controller: String,
    device: String,
    document_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StateAssertionPolicy {
    None,
    CompleteMaterializedState,
}

pub(crate) fn state_assertion_policy(requirements: &[impl AsRef<str>]) -> StateAssertionPolicy {
    if requirements
        .iter()
        .any(|requirement| requirement.as_ref() == "NCRDT-STATE-002")
    {
        StateAssertionPolicy::CompleteMaterializedState
    } else {
        StateAssertionPolicy::None
    }
}

pub(crate) fn run_fixture(path: &Path) -> Result<Vec<u8>, RunError> {
    let fixture = if is_normative_signed_fixture(path) {
        load_normative_fixture(path)
    } else {
        load_fixture(path)
    }
    .map_err(|_| RunError::Fixture)?;
    let base = path.parent().ok_or(RunError::Fixture)?;
    verify_fixture_files(&fixture, base).map_err(|_| RunError::Checksum)?;
    let expected =
        load_expected(&base.join(&fixture.expected.report_path)).map_err(|_| RunError::Expected)?;
    let assertion_policy = state_assertion_policy(&fixture.requirements);
    if fixture.inputs.len() != 1 {
        return Err(RunError::Input);
    }
    let input = fs::read(base.join(&fixture.inputs[0].path)).map_err(|_| RunError::Input)?;
    let signed = SignedScenarioInput::parse(&input);
    if is_normative_signed_fixture(path) && signed.is_err() {
        return Err(RunError::Input);
    }
    let actual = if let Ok(signed) = signed {
        if signed.fixture_id != fixture.fixture_id
            || signed.revision != fixture.revision
            || signed.requirements != fixture.requirements
            || signed.expected_report
                != serde_json::to_value(&expected).map_err(|_| RunError::Expected)?
        {
            return Err(RunError::Expected);
        }
        signed_permutation_report(signed, assertion_policy)?
    } else if fixture.fixture_id.starts_with("scenario_") {
        generic_report(
            &fixture.fixture_id,
            ScenarioInput::parse(&input).map_err(|_| RunError::Input)?,
            assertion_policy,
        )?
    } else if fixture.fixture_id == "actor_derivation_001" {
        let input: ActorDerivationInput =
            serde_json::from_slice(&input).map_err(|_| RunError::Input)?;
        actor_derivation_report(&fixture.fixture_id, &input)?
    } else {
        return Err(RunError::Input);
    };
    compare_expected(&actual, &expected)?;
    write_canonical_report(&actual).map_err(|_| RunError::Expected)
}

fn signed_permutation_report(
    signed: SignedScenarioInput,
    assertion_policy: StateAssertionPolicy,
) -> Result<ExpectedReport, RunError> {
    let fixture_id = signed.fixture_id.clone();
    let permutations = required_delivery_permutations(
        &signed.raw_events,
        |event| event_kind(event) == Some(1624),
        |event| event_kind(event) == Some(1625),
        raw_event_is_invalid,
    );
    let mut baseline = None;
    for permutation in permutations {
        let report = generic_report(
            &fixture_id,
            signed
                .clone()
                .with_raw_events(permutation.events)
                .into_scenario(),
            assertion_policy,
        )?;
        if let Some(canonical) = &baseline {
            if canonical != &report {
                return Err(RunError::Mismatch);
            }
        } else {
            baseline = Some(report);
        }
    }
    baseline.ok_or(RunError::Input)
}

fn event_value(event: &crate::scenario::EncodedRawEventV2) -> Option<Value> {
    serde_json::from_slice(&event.decoded().ok()?).ok()
}

fn event_kind(event: &crate::scenario::EncodedRawEventV2) -> Option<u64> {
    event_value(event)?.get("kind")?.as_u64()
}

fn raw_event_is_invalid(event: &crate::scenario::EncodedRawEventV2) -> bool {
    let Ok(raw) = event.decoded() else {
        return true;
    };
    matches!(
        CorpusBuilder::new().ingest_bytes(&raw),
        IngestOutcome::Invalid { .. }
            | IngestOutcome::InvalidCarrier { .. }
            | IngestOutcome::UnsupportedRevision { .. }
    )
}

fn is_normative_signed_fixture(path: &Path) -> bool {
    path.ancestors()
        .any(|ancestor| ancestor.file_name().is_some_and(|name| name == "scenarios"))
}

pub(crate) fn generic_report(
    fixture_id: &str,
    scenario: ScenarioInput,
    assertion_policy: StateAssertionPolicy,
) -> Result<ExpectedReport, RunError> {
    let report = evaluate_scenario(scenario)?;
    let mut output = ExpectedReport::empty(
        fixture_id,
        report.revision(),
        &report.coordinate().to_address(),
    );
    output.coordinate = report.coordinate().to_address();
    output.revision = report.revision().identifier().to_owned();
    output.canonical_controls = report
        .canonical_controls()
        .iter()
        .map(|event_id| (*event_id).to_hex())
        .collect();
    output.disposition_records = report
        .disposition_records()
        .iter()
        .map(|record| {
            let namespace = match record.identifier() {
                nostr_automerge::ProtocolItemIdentifier::ControlEvent(_) => "control_event",
                nostr_automerge::ProtocolItemIdentifier::ChangeHash(_) => "change_hash",
                nostr_automerge::ProtocolItemIdentifier::Event(_) => "event",
                _ => return Err(RunError::Input),
            };
            Ok(DispositionRecord {
                namespace: namespace.to_owned(),
                identifier: record.identifier().as_bytes().iter().fold(
                    String::with_capacity(64),
                    |mut output, byte| {
                        use core::fmt::Write as _;
                        let _ = write!(&mut output, "{byte:02x}");
                        output
                    },
                ),
                disposition: record.disposition().as_str().to_owned(),
                diagnostic: record
                    .diagnostic()
                    .map(|diagnostic| diagnostic.as_str().to_owned()),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    output.accepted_changes = report
        .accepted_changes()
        .iter()
        .map(|change_hash| (*change_hash).to_hex())
        .collect();
    output.pending_changes = report
        .pending_changes()
        .iter()
        .map(|change_hash| (*change_hash).to_hex())
        .collect();
    output.excluded_changes = report
        .excluded_changes()
        .iter()
        .map(|change_hash| (*change_hash).to_hex())
        .collect();
    output.invalid_changes = report
        .invalid_changes()
        .iter()
        .map(|change_hash| (*change_hash).to_hex())
        .collect();
    output.heads = report
        .heads()
        .iter()
        .map(|change_hash| (*change_hash).to_hex())
        .collect();
    output.history_digest = report.history_digest().to_hex();
    output.dispositions_digest = report.dispositions_digest().to_hex();
    output.invalid_events = evidence_ids(&report, EvidenceStatus::Invalid);
    output.unsupported_events = evidence_ids(&report, EvidenceStatus::Unsupported);
    output.completion = match report.completion() {
        Completion::Complete => "complete",
        Completion::BudgetExhausted => "budget_exhausted",
        Completion::Cancelled => "cancelled",
    }
    .to_owned();
    output.integrity_alerts = report
        .integrity_alerts()
        .iter()
        .map(integrity_alert)
        .collect();
    let state_assertions = materialized_state_assertions(&report, assertion_policy)?;
    validate_materialized_state_assertions(&report, assertion_policy, &state_assertions)?;
    output.state_assertions = state_assertions;
    output.checkpoints = report
        .checkpoints()
        .iter()
        .map(|checkpoint| CheckpointResult {
            descriptor_event: checkpoint.descriptor_event().to_hex(),
            chunk_events: checkpoint
                .chunk_events()
                .iter()
                .map(|event_id| (*event_id).to_hex())
                .collect(),
            snapshot_hash: checkpoint.snapshot_hash().to_hex(),
            heads: checkpoint
                .heads()
                .iter()
                .map(|hash| (*hash).to_hex())
                .collect(),
            change_count: checkpoint.change_count(),
            change_set_hash: hex_bytes(checkpoint.change_set_hash()),
            historical_carriers: checkpoint
                .historical_carriers()
                .iter()
                .map(|hash| (*hash).to_hex())
                .collect(),
            accepted_at_control: checkpoint
                .accepted_at_control()
                .iter()
                .map(|hash| (*hash).to_hex())
                .collect(),
            status: checkpoint_status(checkpoint.status()).to_owned(),
        })
        .collect();
    Ok(output)
}

pub(crate) fn evaluate_scenario(scenario: ScenarioInput) -> Result<EvaluationReport, RunError> {
    let coordinate = scenario.coordinate.parse().map_err(|_| RunError::Input)?;
    let mut builder = CorpusBuilder::new();
    for raw in scenario.raw_events {
        let raw = raw.decode().map_err(|_| RunError::Input)?;
        let _ = builder.ingest_bytes(&raw);
    }
    let corpus = builder.finish();
    let mut budget = WorkBudget::new(scenario.budget.max_bytes, scenario.budget.max_items);
    if let Some(cancel_after) = scenario.cancel_after {
        let calls = std::cell::Cell::new(0_u64);
        ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate(
            &corpus,
            coordinate,
            &mut budget,
            &|| {
                let current = calls.get();
                calls.set(current.saturating_add(1));
                current >= cancel_after
            },
        )
    } else {
        ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate(
            &corpus,
            coordinate,
            &mut budget,
            &NeverCancelled,
        )
    }
    .map_err(|_| RunError::Evaluation)
}

fn materialized_state_assertions(
    report: &nostr_automerge::EvaluationReport,
    policy: StateAssertionPolicy,
) -> Result<Vec<StateAssertion>, RunError> {
    if report.completion() != Completion::Complete || policy == StateAssertionPolicy::None {
        return Ok(Vec::new());
    }
    let document = report.document().ok_or(RunError::Input)?;
    let mut assertions = document
        .entries()
        .iter()
        .filter(|entry| {
            !entry
                .conflicts()
                .iter()
                .all(|conflict| matches!(conflict.value(), MaterializedValue::Object { .. }))
                && !document
                    .marks()
                    .iter()
                    .any(|mark| mark.path() == entry.path())
        })
        .map(|entry| {
            let operation = entry
                .conflicts()
                .first()
                .ok_or(RunError::Evaluation)?
                .operation_id();
            let (counter, actor) = operation.split_once('@').ok_or(RunError::Evaluation)?;
            let counter = counter.parse::<u64>().map_err(|_| RunError::Evaluation)?;
            Ok((
                (counter, actor.to_owned()),
                StateAssertion {
                    path: materialized_path_json(entry.path()),
                    expected: materialized_conflicts(entry.conflicts()),
                },
            ))
        })
        .collect::<Result<Vec<_>, RunError>>()?;
    assertions.sort_by(|left, right| left.0.cmp(&right.0));
    let mut assertions = assertions
        .into_iter()
        .map(|(_, assertion)| assertion)
        .collect::<Vec<_>>();
    assertions.extend(document.marks().iter().map(|mark| StateAssertion {
        path: materialized_path_json(mark.path()),
        expected: materialized_mark(mark),
    }));
    Ok(assertions)
}

fn validate_materialized_state_assertions(
    report: &nostr_automerge::EvaluationReport,
    policy: StateAssertionPolicy,
    assertions: &[StateAssertion],
) -> Result<(), RunError> {
    let expected = materialized_state_assertions(report, policy)?;
    (assertions == expected)
        .then_some(())
        .ok_or(RunError::Evaluation)
}

fn checkpoint_status(status: CheckpointVerificationStatus) -> &'static str {
    match status {
        CheckpointVerificationStatus::Verified => "verified",
        CheckpointVerificationStatus::PendingControl => "pending_control",
        CheckpointVerificationStatus::Unauthorized => "unauthorized",
        CheckpointVerificationStatus::ChunkAuthorMismatch => "chunk_author_mismatch",
        CheckpointVerificationStatus::ChunkCoordinateMismatch => "chunk_coordinate_mismatch",
        CheckpointVerificationStatus::ChunkDescriptorMismatch => "chunk_descriptor_mismatch",
        CheckpointVerificationStatus::ChunkCountMismatch => "chunk_count_mismatch",
        CheckpointVerificationStatus::DuplicateChunk => "duplicate_chunk",
        CheckpointVerificationStatus::MissingChunk => "missing_chunk",
        CheckpointVerificationStatus::ChunkSizeMismatch => "chunk_size_mismatch",
        CheckpointVerificationStatus::ChunkAssemblyMismatch => "chunk_assembly_mismatch",
        CheckpointVerificationStatus::MerkleMismatch => "merkle_mismatch",
        CheckpointVerificationStatus::SnapshotSizeMismatch => "snapshot_size_mismatch",
        CheckpointVerificationStatus::SnapshotHashMismatch => "snapshot_hash_mismatch",
        CheckpointVerificationStatus::SnapshotLoad => "snapshot_load",
        CheckpointVerificationStatus::HeadMismatch => "head_mismatch",
        CheckpointVerificationStatus::CommitmentMismatch => "commitment_mismatch",
        CheckpointVerificationStatus::ClosureMismatch => "closure_mismatch",
        CheckpointVerificationStatus::MissingHistoricalCarrier => "missing_historical_carrier",
        CheckpointVerificationStatus::NotAcceptedAtControl => "not_accepted_at_control",
        CheckpointVerificationStatus::BudgetExhausted => "budget_exhausted",
        CheckpointVerificationStatus::Cancelled => "cancelled",
        _ => "unsupported_status",
    }
}

fn hex_bytes(bytes: [u8; 32]) -> String {
    let mut value = String::with_capacity(64);
    for byte in bytes {
        use core::fmt::Write as _;
        let _ = write!(value, "{byte:02x}");
    }
    value
}

fn materialized_path_json(path: &[MaterializedPathElement]) -> Vec<Value> {
    path.iter()
        .map(|element| match element {
            MaterializedPathElement::Key(key) => Value::String(key.clone()),
            MaterializedPathElement::Index(index) => Value::from(*index),
            branch => {
                let (parent, operation, child) = branch.branch_identity().unwrap_or(("", "", ""));
                serde_json::json!({
                    "type":"branch",
                    "parent_object_id":parent,
                    "operation_id":operation,
                    "child_object_id":child,
                })
            }
        })
        .collect()
}

fn materialized_conflicts(conflicts: &[nostr_automerge::MaterializedConflict]) -> Value {
    if let [conflict] = conflicts {
        return materialized_value(conflict.value());
    }
    serde_json::json!({
        "type": "conflicts",
        "values": conflicts.iter().map(|conflict| serde_json::json!({
            "operation_id": conflict.operation_id(),
            "value": materialized_value(conflict.value()),
        })).collect::<Vec<_>>()
    })
}

fn materialized_value(value: &MaterializedValue) -> Value {
    match value {
        MaterializedValue::Scalar(value) => materialized_scalar(value),
        MaterializedValue::Object {
            object_type,
            object_id,
        } => serde_json::json!({
            "type": match object_type {
                MaterializedObjectType::Map => "map",
                MaterializedObjectType::List => "list",
                MaterializedObjectType::Table => "table",
                MaterializedObjectType::Text => "text",
            },
            "object_id": object_id,
        }),
        MaterializedValue::Text { object_id, value } => {
            serde_json::json!({"type":"text", "object_id":object_id, "value":value})
        }
    }
}

fn materialized_mark(mark: &MaterializedMark) -> Value {
    serde_json::json!({
        "type":"mark",
        "name":mark.name(),
        "value":materialized_scalar(mark.value()),
        "start":mark.start(),
        "end":mark.end(),
        "expansion":match mark.expansion() {
            MaterializedMarkExpansion::None => "none",
            MaterializedMarkExpansion::Before => "before",
            MaterializedMarkExpansion::After => "after",
            MaterializedMarkExpansion::Both => "both",
        }
    })
}

fn materialized_scalar(value: &MaterializedScalar) -> Value {
    match value {
        MaterializedScalar::Null => serde_json::json!({"type":"null"}),
        MaterializedScalar::Bool(value) => serde_json::json!({"type":"bool", "value":value}),
        MaterializedScalar::I64(value) => {
            serde_json::json!({"type":"i64", "value":value.to_string()})
        }
        MaterializedScalar::U64(value) => {
            serde_json::json!({"type":"u64", "value":value.to_string()})
        }
        MaterializedScalar::F64Bits(value) => {
            serde_json::json!({"type":"f64_bits", "value":format!("{value:016x}")})
        }
        MaterializedScalar::String(value) => serde_json::json!({"type":"string", "value":value}),
        MaterializedScalar::Bytes(value) => serde_json::json!({
            "type":"bytes_base64",
            "value": base64::engine::general_purpose::STANDARD.encode(value)
        }),
        MaterializedScalar::Timestamp(value) => {
            serde_json::json!({"type":"timestamp", "value":value.to_string()})
        }
        MaterializedScalar::Counter(value) => {
            serde_json::json!({"type":"counter", "value":value.to_string()})
        }
    }
}

fn integrity_alert(alert: &IntegrityAlert) -> Value {
    match alert {
        IntegrityAlert::ControllerEquivocation(details) => serde_json::json!({
            "type":"controller_equivocation",
            "parent_control":details.parent_control().map(nostr_automerge::EventId::to_hex),
            "candidate_controls":details.candidate_controls().iter().map(|id| (*id).to_hex()).collect::<Vec<_>>(),
            "selected_control":details.selected_control().to_hex(),
        }),
        IntegrityAlert::CanonicalControlReorganization(details) => serde_json::json!({
            "type":"canonical_control_reorganization",
            "previous_tip":details.previous_tip().to_hex(),
            "new_tip":details.new_tip().to_hex(),
            "affected_changes":details.affected_changes().iter().map(|hash| (*hash).to_hex()).collect::<Vec<_>>(),
        }),
        IntegrityAlert::DeviceEquivocation(details) => serde_json::json!({
            "type":"device_equivocation",
            "actor_id":details.actor_id().to_hex(),
            "first_sequence":details.first_sequence(),
            "conflicting_changes":details.conflicting_changes().iter().map(|hash| (*hash).to_hex()).collect::<Vec<_>>(),
            "affected_descendants":details.affected_descendants().iter().map(|hash| (*hash).to_hex()).collect::<Vec<_>>(),
        }),
        IntegrityAlert::PotentialClonedDeviceKey(details) => serde_json::json!({
            "type":"potential_cloned_device_key",
            "actor_id":details.actor_id().to_hex(),
            "first_sequence":details.first_sequence(),
            "carrier_event_ids":details.carrier_event_ids().iter().map(|id| (*id).to_hex()).collect::<Vec<_>>(),
        }),
        IntegrityAlert::CheckpointMismatch(details) => serde_json::json!({
            "type":"checkpoint_mismatch",
            "descriptor_event_id":details.descriptor_event_id().to_hex(),
            "code":details.code().as_str(),
        }),
    }
}

fn evidence_ids(report: &nostr_automerge::EvaluationReport, status: EvidenceStatus) -> Vec<String> {
    let mut values = report
        .evidence()
        .iter()
        .filter_map(|record| {
            if record.status() != status {
                return None;
            }
            match record.identifier() {
                EvidenceIdentifier::Event(event_id) => Some(event_id.to_hex()),
                EvidenceIdentifier::InvalidRawSha256(_) => None,
            }
        })
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn actor_derivation_report(
    fixture_id: &str,
    input: &ActorDerivationInput,
) -> Result<ExpectedReport, RunError> {
    let controller =
        ControllerPublicKey::from_str(&input.controller).map_err(|_| RunError::Input)?;
    let document = DocumentId::from_str(&input.document_id).map_err(|_| RunError::Input)?;
    let device = DevicePublicKey::from_str(&input.device).map_err(|_| RunError::Input)?;
    let mut hasher = Sha256::new();
    hasher.update(b"nostr-crdt/automerge/actor/v1\0");
    hasher.update(controller.as_bytes());
    hasher.update(document.as_bytes());
    hasher.update(device.as_bytes());
    let actor: [u8; 32] = hasher.finalize().into();
    let mut encoded = String::with_capacity(64);
    for byte in actor {
        use core::fmt::Write;
        write!(&mut encoded, "{byte:02x}").map_err(|_| RunError::Input)?;
    }
    let mut value = Map::new();
    value.insert("type".to_owned(), Value::String("bytes32".to_owned()));
    value.insert("value".to_owned(), Value::String(encoded));
    let coordinate = format!("31624:{}:{}", input.controller, input.document_id)
        .parse::<DocumentCoordinate>()
        .map_err(|_| RunError::Input)?;
    let revision = ProtocolRevision::draft_v1();
    let mut report = ExpectedReport::empty(fixture_id, revision, &coordinate.to_address());
    report.history_digest = canonical_history_digest(revision, coordinate, &[], &[], &[])
        .map_err(|_| RunError::Evaluation)?
        .to_hex();
    report.dispositions_digest = canonical_dispositions_digest(revision, coordinate, &[])
        .map_err(|_| RunError::Evaluation)?
        .to_hex();
    report.state_assertions.push(StateAssertion {
        path: vec![Value::String("derived_actor_id".to_owned())],
        expected: Value::Object(value),
    });
    Ok(report)
}

pub(crate) fn compare_expected(
    actual: &ExpectedReport,
    expected: &ExpectedReport,
) -> Result<(), RunError> {
    (actual == expected).then_some(()).ok_or(RunError::Mismatch)
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct CorpusSummary {
    pub(crate) failed: u64,
    pub(crate) fixtures: Vec<FixtureSummary>,
    pub(crate) passed: u64,
    pub(crate) schema: &'static str,
    pub(crate) total: u64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct FixtureSummary {
    pub(crate) fixture_id: String,
    pub(crate) status: &'static str,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct DistributionRun {
    pub(crate) canonical_output_sha256: String,
    pub(crate) delivery_permutations: u64,
    pub(crate) fixture_count: u64,
    pub(crate) reports: Vec<DistributionReport>,
    pub(crate) schema: &'static str,
    pub(crate) status: &'static str,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct DistributionReport {
    pub(crate) fixture_id: String,
    pub(crate) report_sha256: String,
}

#[derive(Deserialize)]
struct DistributionManifest {
    complete: bool,
    distribution_schema: String,
    fixture_count: u64,
    fixtures: Vec<DistributionFixture>,
    status: String,
    target_fixture_count: u64,
    #[serde(default)]
    transition_stage: Option<String>,
}

#[derive(Deserialize)]
struct DistributionFixture {
    fixture_id: String,
    metadata_path: String,
}

fn validate_distribution_authority(manifest: &DistributionManifest) -> Result<(), RunError> {
    let expected_count = match manifest.distribution_schema.as_str() {
        "nostr_automerge.fixture_distribution.v9" if manifest.transition_stage.is_none() => 180,
        "nostr_automerge.fixture_distribution.v10"
            if manifest.transition_stage.as_deref() == Some("distribution_complete") =>
        {
            192
        }
        "nostr_automerge.fixture_distribution.v11" if manifest.transition_stage.is_none() => 193,
        "nostr_automerge.fixture_distribution.v12"
            if manifest.transition_stage.as_deref() == Some("distribution_complete") =>
        {
            198
        }
        "nostr_automerge.fixture_distribution.v13"
            if manifest.transition_stage.as_deref() == Some("distribution_complete") =>
        {
            204
        }
        "nostr_automerge.fixture_distribution.v14"
            if manifest.transition_stage.as_deref() == Some("distribution_complete") =>
        {
            204
        }
        _ => return Err(RunError::Fixture),
    };
    let actual_count = u64::try_from(manifest.fixtures.len()).map_err(|_| RunError::Fixture)?;
    if !manifest.complete
        || manifest.status != "canonical_signed_neutral_corpus"
        || manifest.fixture_count != expected_count
        || manifest.target_fixture_count != expected_count
        || actual_count != expected_count
    {
        return Err(RunError::Fixture);
    }
    Ok(())
}

pub(crate) fn run_distribution(path: &Path) -> Result<DistributionRun, RunError> {
    let bytes = fs::read(path).map_err(|_| RunError::Fixture)?;
    let manifest: DistributionManifest =
        serde_json::from_slice(&bytes).map_err(|_| RunError::Fixture)?;
    validate_distribution_authority(&manifest)?;
    let root = path
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .ok_or(RunError::Fixture)?;
    let mut fixtures = manifest.fixtures;
    fixtures.sort_by(|left, right| left.fixture_id.cmp(&right.fixture_id));
    if fixtures
        .windows(2)
        .any(|pair| pair[0].fixture_id == pair[1].fixture_id)
    {
        return Err(RunError::Fixture);
    }
    let mut aggregate = Sha256::new();
    let mut reports = Vec::with_capacity(fixtures.len());
    for fixture in fixtures {
        let canonical = run_fixture(&root.join(&fixture.metadata_path))?;
        update_length_prefixed(&mut aggregate, fixture.fixture_id.as_bytes())?;
        update_length_prefixed(&mut aggregate, &canonical)?;
        reports.push(DistributionReport {
            fixture_id: fixture.fixture_id,
            report_sha256: sha256_hex(&canonical),
        });
    }
    Ok(DistributionRun {
        canonical_output_sha256: digest_hex(aggregate.finalize()),
        delivery_permutations: 8,
        fixture_count: reports.len() as u64,
        reports,
        schema: "nostr_automerge.distribution_run.v1",
        status: "pass",
    })
}

pub(crate) fn write_distribution_run(run: &DistributionRun) -> Result<Vec<u8>, RunError> {
    let value = serde_json::to_value(run).map_err(|_| RunError::Input)?;
    let mut bytes = serde_json::to_vec(&value).map_err(|_| RunError::Input)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn update_length_prefixed(digest: &mut Sha256, bytes: &[u8]) -> Result<(), RunError> {
    let length = u64::try_from(bytes.len()).map_err(|_| RunError::Input)?;
    digest.update(length.to_be_bytes());
    digest.update(bytes);
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    digest_hex(Sha256::digest(bytes))
}

fn digest_hex(digest: impl AsRef<[u8]>) -> String {
    let mut encoded = String::with_capacity(64);
    for byte in digest.as_ref() {
        use core::fmt::Write as _;
        let _ = write!(&mut encoded, "{byte:02x}");
    }
    encoded
}

pub(crate) fn discover_fixtures(root: &Path) -> Result<Vec<PathBuf>, RunError> {
    let signed_scenarios = root.join("v1_draft/scenarios");
    let discovery_root = if signed_scenarios.is_dir() {
        signed_scenarios
    } else {
        root.to_path_buf()
    };
    let mut pending = vec![discovery_root];
    let mut fixtures = Vec::new();
    while let Some(path) = pending.pop() {
        let entries = fs::read_dir(path).map_err(|_| RunError::Fixture)?;
        for entry in entries {
            let entry = entry.map_err(|_| RunError::Fixture)?;
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".fixture.json"))
            {
                fixtures.push(path);
            }
        }
    }
    fixtures.sort();
    Ok(fixtures)
}

pub(crate) fn run_corpus(
    paths: impl IntoIterator<Item = PathBuf>,
    family: Option<&str>,
    requirement: Option<&str>,
) -> CorpusSummary {
    let mut paths = paths.into_iter().collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    let mut fixtures = Vec::new();
    for path in paths {
        let metadata = load_fixture(&path);
        let (fixture_id, included) = match &metadata {
            Ok(metadata) => (
                metadata.fixture_id.clone(),
                family.is_none_or(|family| metadata.fixture_id.starts_with(family))
                    && requirement
                        .is_none_or(|id| metadata.requirements.iter().any(|item| item == id)),
            ),
            Err(_) => (
                path.display().to_string(),
                family.is_none() && requirement.is_none(),
            ),
        };
        if !included {
            continue;
        }
        fixtures.push(FixtureSummary {
            fixture_id,
            status: if run_fixture(&path).is_ok() {
                "passed"
            } else {
                "failed"
            },
        });
    }
    fixtures.sort_by(|left, right| left.fixture_id.cmp(&right.fixture_id));
    let passed = fixtures
        .iter()
        .filter(|fixture| fixture.status == "passed")
        .count() as u64;
    let total = fixtures.len() as u64;
    CorpusSummary {
        failed: total - passed,
        fixtures,
        passed,
        schema: "nostr_automerge.corpus_summary.v1",
        total,
    }
}

pub(crate) fn write_corpus_summary(summary: &CorpusSummary) -> Result<Vec<u8>, RunError> {
    let value = serde_json::to_value(summary).map_err(|_| RunError::Input)?;
    let mut bytes = serde_json::to_vec(&value).map_err(|_| RunError::Input)?;
    bytes.push(b'\n');
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use nostr_automerge::ProtocolRevision;

    use super::{
        RunError, StateAssertionPolicy, compare_expected, discover_fixtures, evaluate_scenario,
        generic_report, materialized_state_assertions, run_corpus, run_fixture,
        signed_permutation_report, state_assertion_policy, validate_materialized_state_assertions,
    };
    use crate::expected::load_expected;
    use crate::fixture::load_fixture;
    use crate::report_json::write_canonical_report;
    use crate::scenario::SignedScenarioInput;

    #[test]
    fn normative_fixtures_use_public_engine() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/v1_draft/scenarios");
        let fixtures = discover_fixtures(&root);
        assert!(fixtures.is_ok());
        let Ok(fixtures) = fixtures else { return };
        assert!(!fixtures.is_empty());
        for path in fixtures {
            let fixture = load_fixture(&path);
            assert!(fixture.is_ok(), "{}", path.display());
            let Ok(fixture) = fixture else { continue };
            assert_eq!(fixture.inputs.len(), 1);
            let base = path.parent();
            assert!(base.is_some(), "fixture has no parent: {}", path.display());
            let Some(base) = base else { continue };
            let input = fs::read(base.join(&fixture.inputs[0].path));
            assert!(input.is_ok(), "{}", path.display());
            let Ok(input) = input else { continue };
            assert!(
                SignedScenarioInput::parse(&input).is_ok(),
                "{}",
                path.display()
            );
        }
        let source = include_str!("runner.rs");
        let signed_branch = source
            .split("let actual = if let Ok(signed) = signed")
            .nth(1)
            .and_then(|tail| tail.split("} else if").next());
        assert!(signed_branch.is_some_and(|branch| {
            branch.contains("signed_permutation_report") && !branch.contains("interop::evaluate")
        }));
    }

    #[test]
    fn signed_fixtures_execute_all_delivery_permutations() {
        let source = include_str!("runner.rs");
        assert!(source.contains("required_delivery_permutations"));
        assert!(source.contains("signed_permutation_report"));
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/v1_draft/scenarios/dependencies/dependencies_late_recovery.fixture.json");
        assert!(run_fixture(&fixture).is_ok());
    }

    #[test]
    fn actor_counter_frontier_reports_match_predecessor_bytes() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/v1_draft/scenarios/actor_counters");
        for fixture_id in [
            "actor_counter_sequence_start",
            "actor_counter_exact_predecessor",
            "actor_counter_missing_predecessor",
            "actor_counter_sequence_gap",
            "actor_counter_sequence_rollback",
            "actor_counter_start_op",
            "actor_counter_empty_preservation",
            "actor_counter_empty_frontier",
        ] {
            let fixture = root.join(format!("{fixture_id}.fixture.json"));
            let expected = fs::read(root.join(format!("{fixture_id}.expected.json")));
            assert!(expected.is_ok(), "{fixture_id}");
            assert_eq!(
                run_fixture(&fixture),
                expected.map_err(|_| RunError::Expected)
            );
        }
    }

    #[test]
    fn report_contract_compatibility_consumers_are_exact() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let fixture_path =
            root.join("fixtures/v1_draft/scenarios/projection/projection_all_scalars.fixture.json");
        let fixture = load_fixture(&fixture_path);
        assert!(fixture.is_ok());
        let Ok(fixture) = fixture else { return };
        assert_eq!(fixture.revision, ProtocolRevision::draft_v1().identifier());
        let Some(base) = fixture_path.parent() else {
            return;
        };
        let expected_path = base.join(&fixture.expected.report_path);
        let expected = load_expected(&expected_path);
        let expected_bytes = fs::read(&expected_path);
        let input = fs::read(base.join(&fixture.inputs[0].path));
        assert!(expected.is_ok() && expected_bytes.is_ok() && input.is_ok());
        let (Ok(expected), Ok(expected_bytes), Ok(input)) = (expected, expected_bytes, input)
        else {
            return;
        };
        let signed = SignedScenarioInput::parse(&input);
        assert!(signed.is_ok());
        let Ok(signed) = signed else { return };
        assert_eq!(signed.revision, ProtocolRevision::draft_v1().identifier());
        assert_eq!(signed.requirements, fixture.requirements);

        let actual =
            signed_permutation_report(signed, state_assertion_policy(&fixture.requirements));
        assert_eq!(actual, Ok(expected.clone()));
        assert_eq!(
            actual.as_ref().map(|report| report.revision.as_str()),
            Ok(ProtocolRevision::draft_v1().identifier())
        );
        assert_eq!(
            actual
                .and_then(|report| write_canonical_report(&report).map_err(|_| RunError::Expected)),
            Ok(expected_bytes)
        );
    }
    use crate::expected::StateAssertion;
    use crate::scenario::ScenarioInput;

    #[test]
    fn expected_mismatch_has_stable_exit_code() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/examples/actor_derivation_001.expected.json");
        let expected = load_expected(&path);
        assert!(expected.is_ok());
        let Ok(expected) = expected else { return };
        let mut actual = expected.clone();
        actual.completion = "cancelled".to_owned();
        assert_eq!(
            compare_expected(&actual, &expected),
            Err(RunError::Mismatch)
        );
        assert_eq!(RunError::Mismatch.exit_code(), 1);
    }

    #[test]
    fn report_parity_rejects_malformed_and_structurally_valid_mismatch() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let expected_path = root
            .join("fixtures/v1_draft/scenarios/checkpoints/checkpoints_single_chunk.expected.json");
        let expected = load_expected(&expected_path);
        assert!(expected.is_ok());
        let Ok(expected) = expected else { return };
        assert_eq!(
            expected.checkpoints[0].historical_carriers,
            ["9a0701b37736afc4c28c82bfdc94ddf53a3b054fecfa191f18ed94c14982ac7f"]
        );
        assert_eq!(
            expected.checkpoints[0].accepted_at_control,
            ["66be06a76d30b453372abdd246e6ea8aecf8e2dd9c134264b3cce7d57bbda43f"]
        );
        assert_ne!(
            expected.checkpoints[0].historical_carriers,
            expected.checkpoints[0].accepted_at_control
        );

        let malformed = b"{\"report_schema\":}\n";
        let temporary = std::env::temp_dir().join(format!(
            "nostr-automerge-report-parity-{}-{}.json",
            std::process::id(),
            expected.fixture_id
        ));
        assert!(fs::write(&temporary, malformed).is_ok());
        assert!(load_expected(&temporary).is_err());

        let canonical = write_canonical_report(&expected);
        assert!(canonical.is_ok());
        let Ok(canonical) = canonical else { return };
        let mut noncanonical = b" \n".to_vec();
        noncanonical.extend_from_slice(&canonical);
        assert!(fs::write(&temporary, noncanonical).is_ok());
        assert_eq!(
            load_expected(&temporary),
            Err(crate::expected::ExpectedError::Canonical)
        );
        let _ = fs::remove_file(&temporary);

        let mut mismatch = expected.clone();
        mismatch.checkpoints[0].historical_carriers = vec!["aa".repeat(32)];
        let mismatch_bytes = write_canonical_report(&mismatch);
        assert!(mismatch_bytes.is_ok());
        let Ok(mismatch_bytes) = mismatch_bytes else {
            return;
        };
        assert!(fs::write(&temporary, mismatch_bytes).is_ok());
        let parsed_mismatch = load_expected(&temporary);
        let _ = fs::remove_file(&temporary);
        assert!(parsed_mismatch.is_ok());
        let Ok(parsed_mismatch) = parsed_mismatch else {
            return;
        };
        assert_eq!(
            compare_expected(&parsed_mismatch, &expected),
            Err(RunError::Mismatch)
        );
    }

    #[test]
    fn add_corpus_cli_command() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
        let paths = discover_fixtures(&root);
        assert!(paths.is_ok());
        let Ok(paths) = paths else { return };
        let mut reversed = paths.clone();
        reversed.reverse();
        let baseline = run_corpus(paths.clone(), None, None);
        assert_eq!(baseline, run_corpus(reversed, None, None));
        let actor = root.join("examples/actor_derivation_001.fixture.json");
        let filtered = run_corpus([actor.clone()], Some("actor_derivation"), None);
        assert_eq!(filtered.total, 1);
        assert_eq!(filtered.failed, 0);
        let requirement = run_corpus(
            paths.iter().cloned().chain([actor]),
            None,
            Some("NCRDT-ACTOR-001"),
        );
        assert!(requirement.total >= 1);
        assert_eq!(requirement.failed, 0);
        assert_eq!(requirement.passed, requirement.total);
        let failures = run_corpus(
            paths
                .into_iter()
                .chain([Path::new("missing.fixture.json").to_path_buf()]),
            None,
            None,
        );
        assert_eq!(failures.passed, baseline.passed);
        assert_eq!(failures.total.checked_sub(baseline.total), Some(1));
        assert_eq!(failures.failed.checked_sub(baseline.failed), Some(1));
        assert_eq!(
            failures.passed.checked_add(failures.failed),
            Some(failures.total)
        );
    }

    #[test]
    fn execute_generic_raw_event_through_public_engine() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let raw_event =
            std::fs::read_to_string(root.join("fixtures/v1_draft/nip01/valid_event.json"));
        assert!(raw_event.is_ok());
        let Ok(raw_event) = raw_event else { return };
        let scenario = serde_json::json!({
            "budget": {"max_bytes": 100_000, "max_items": 10_000},
            "cancel_after": null,
            "coordinate": format!("31624:{}:{}", "11".repeat(32), "22".repeat(32)),
            "raw_events": [raw_event],
            "scenario_schema": "nostr_automerge.scenario.v1"
        });
        let parsed = ScenarioInput::parse(&serde_json::to_vec(&scenario).unwrap_or_default());
        assert!(parsed.is_ok());
        let Ok(parsed) = parsed else { return };
        let actual = generic_report(
            "scenario_generic_raw_event",
            parsed,
            StateAssertionPolicy::None,
        );
        assert!(actual.is_ok());
        let Ok(actual) = actual else { return };
        assert_eq!(actual.completion, "complete");
        assert_eq!(actual.revision, ProtocolRevision::draft_v1().identifier());
        assert!(actual.canonical_controls.is_empty());
        assert!(actual.accepted_changes.is_empty());
    }

    #[test]
    fn incomplete_engine_report_projects_exact_empty_neutral_state() {
        let coordinate = format!("31624:{}:{}", "31".repeat(32), "32".repeat(32));
        let scenario = serde_json::json!({
            "budget": {"max_bytes": 0, "max_items": 0},
            "cancel_after": null,
            "coordinate": coordinate,
            "raw_events": ["{}"],
            "scenario_schema": "nostr_automerge.scenario.v1"
        });
        let parsed = ScenarioInput::parse(&serde_json::to_vec(&scenario).unwrap_or_default());
        assert!(parsed.is_ok());
        let Ok(parsed) = parsed else { return };
        let actual = generic_report(
            "scenario_no_progress",
            parsed,
            StateAssertionPolicy::CompleteMaterializedState,
        );
        assert!(actual.is_ok());
        let Ok(actual) = actual else { return };
        assert_eq!(actual.completion, "budget_exhausted");
        assert_eq!(actual.revision, ProtocolRevision::draft_v1().identifier());
        assert_eq!(actual.coordinate, coordinate);
        assert!(actual.canonical_controls.is_empty());
        assert!(actual.disposition_records.is_empty());
        assert!(actual.accepted_changes.is_empty());
        assert!(actual.pending_changes.is_empty());
        assert!(actual.excluded_changes.is_empty());
        assert!(actual.invalid_changes.is_empty());
        assert!(actual.invalid_events.is_empty());
        assert!(actual.unsupported_events.is_empty());
        assert!(actual.heads.is_empty());
        assert!(actual.integrity_alerts.is_empty());
        assert!(actual.checkpoints.is_empty());
        assert!(actual.state_assertions.is_empty());
        let coordinate = actual
            .coordinate
            .parse::<nostr_automerge::DocumentCoordinate>();
        assert!(coordinate.is_ok());
        let Ok(coordinate) = coordinate else { return };
        assert_eq!(
            actual.history_digest,
            nostr_automerge::canonical_history_digest(
                ProtocolRevision::draft_v1(),
                coordinate,
                &[],
                &[],
                &[],
            )
            .map(|digest| digest.to_hex())
            .unwrap_or_default()
        );
        assert_eq!(
            actual.dispositions_digest,
            nostr_automerge::canonical_dispositions_digest(
                ProtocolRevision::draft_v1(),
                coordinate,
                &[],
            )
            .map(|digest| digest.to_hex())
            .unwrap_or_default()
        );
    }

    #[test]
    fn expected_report_values_never_drive_engine_output() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let fixture_path =
            root.join("fixtures/v1_draft/scenarios/projection/projection_all_scalars.fixture.json");
        let fixture = load_fixture(&fixture_path);
        assert!(fixture.is_ok());
        let Ok(fixture) = fixture else { return };
        let base = fixture_path.parent();
        assert!(base.is_some());
        let Some(base) = base else { return };
        let expected = load_expected(&base.join(&fixture.expected.report_path));
        assert!(expected.is_ok());
        let Ok(expected) = expected else { return };
        let input = fs::read(base.join(&fixture.inputs[0].path));
        assert!(input.is_ok());
        let Ok(input) = input else { return };
        let signed = SignedScenarioInput::parse(&input);
        assert!(signed.is_ok());
        let Ok(signed) = signed else { return };

        let policy = super::state_assertion_policy(&fixture.requirements);
        assert_eq!(policy, StateAssertionPolicy::CompleteMaterializedState);
        let baseline = super::signed_permutation_report(signed.clone(), policy);
        assert_eq!(baseline, Ok(expected.clone()));
        let Ok(baseline_report) = baseline.as_ref() else {
            return;
        };
        let baseline_bytes = write_canonical_report(baseline_report);
        assert!(baseline_bytes.is_ok());

        let mut poisoned_expected = expected.clone();
        poisoned_expected.revision = "draft_2026_09".to_owned();
        poisoned_expected.completion = "cancelled".to_owned();
        poisoned_expected.canonical_controls = vec!["ff".repeat(32)];
        poisoned_expected.history_digest = "ee".repeat(32);
        poisoned_expected.dispositions_digest = "dd".repeat(32);
        poisoned_expected.state_assertions.reverse();
        poisoned_expected.state_assertions.pop();
        poisoned_expected.state_assertions.push(StateAssertion {
            path: vec![serde_json::json!("poison-selector")],
            expected: serde_json::json!({"type":"mark","name":"poison"}),
        });
        assert_ne!(poisoned_expected, expected);

        let mut poisoned_signed = signed;
        let original_declaration = poisoned_signed.expected_report.clone();
        let poisoned_declaration = serde_json::to_value(&poisoned_expected);
        assert!(poisoned_declaration.is_ok());
        let Ok(poisoned_declaration) = poisoned_declaration else {
            return;
        };
        assert_ne!(poisoned_declaration, original_declaration);
        poisoned_signed.expected_report = poisoned_declaration.clone();
        assert_eq!(poisoned_signed.expected_report, poisoned_declaration);

        let poisoned_actual = super::signed_permutation_report(poisoned_signed, policy);
        assert_eq!(poisoned_actual, baseline);
        let Ok(poisoned_report) = poisoned_actual.as_ref() else {
            return;
        };
        assert_eq!(write_canonical_report(poisoned_report), baseline_bytes);
        assert_eq!(
            compare_expected(baseline_report, &poisoned_expected),
            Err(RunError::Mismatch)
        );
    }

    #[test]
    fn signed_requirements_and_materialized_state_reject_assertion_mutations() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let fixture_path =
            root.join("fixtures/v1_draft/scenarios/projection/projection_all_scalars.fixture.json");
        let fixture = load_fixture(&fixture_path);
        assert!(fixture.is_ok());
        let Ok(fixture) = fixture else { return };
        let Some(base) = fixture_path.parent() else {
            return;
        };
        let input = fs::read(base.join(&fixture.inputs[0].path));
        assert!(input.is_ok());
        let Ok(input) = input else { return };
        let signed = SignedScenarioInput::parse(&input);
        assert!(signed.is_ok());
        let Ok(signed) = signed else { return };
        assert_eq!(signed.requirements, fixture.requirements);
        let policy = state_assertion_policy(&signed.requirements);
        assert_eq!(policy, StateAssertionPolicy::CompleteMaterializedState);
        let report = evaluate_scenario(signed.into_scenario());
        assert!(report.is_ok());
        let Ok(report) = report else { return };
        assert!(
            report
                .document()
                .is_some_and(|document| !document.entries().is_empty())
        );
        let assertions = materialized_state_assertions(&report, policy);
        assert!(assertions.is_ok());
        let Ok(assertions) = assertions else { return };
        assert!(!assertions.is_empty());
        assert_eq!(
            validate_materialized_state_assertions(&report, policy, &assertions),
            Ok(())
        );

        let mut missing = assertions.clone();
        missing.pop();
        assert_eq!(
            validate_materialized_state_assertions(&report, policy, &missing),
            Err(RunError::Evaluation)
        );
        let mut extra = assertions.clone();
        extra.push(StateAssertion {
            path: vec![serde_json::json!("forged")],
            expected: serde_json::json!(true),
        });
        assert_eq!(
            validate_materialized_state_assertions(&report, policy, &extra),
            Err(RunError::Evaluation)
        );
        let mut reordered = assertions.clone();
        reordered.reverse();
        assert_eq!(
            validate_materialized_state_assertions(&report, policy, &reordered),
            Err(RunError::Evaluation)
        );
        let mut rewritten = assertions.clone();
        rewritten[0].expected = serde_json::json!({"coordinated": "rewrite"});
        assert_eq!(
            validate_materialized_state_assertions(&report, policy, &rewritten),
            Err(RunError::Evaluation)
        );
        assert_eq!(
            validate_materialized_state_assertions(
                &report,
                StateAssertionPolicy::None,
                &assertions,
            ),
            Err(RunError::Evaluation)
        );
    }

    #[test]
    fn signed_complete_report_field_families_pass_from_independent_inputs() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/v1_draft/scenarios");
        for relative in [
            "projection/projection_all_scalars.fixture.json",
            "checkpoints/checkpoints_multichunk.fixture.json",
            "manifest/manifest_dynamic_canonical_control.fixture.json",
            "equivocation/equivocation_first_conflict.fixture.json",
        ] {
            let path = root.join(relative);
            assert!(run_fixture(&path).is_ok(), "{relative}");
        }
    }

    #[test]
    fn distribution_authority_rejects_missing_or_incomplete_fixture_inventory() {
        let fixtures = (0..192)
            .map(|index| super::DistributionFixture {
                fixture_id: format!("fixture_{index:03}"),
                metadata_path: format!("fixture_{index:03}.fixture.json"),
            })
            .collect::<Vec<_>>();
        let manifest = super::DistributionManifest {
            complete: true,
            distribution_schema: "nostr_automerge.fixture_distribution.v10".to_owned(),
            fixture_count: 192,
            fixtures,
            status: "canonical_signed_neutral_corpus".to_owned(),
            target_fixture_count: 192,
            transition_stage: Some("distribution_complete".to_owned()),
        };
        assert!(super::validate_distribution_authority(&manifest).is_ok());

        let mut missing = manifest;
        missing.fixtures.pop();
        assert_eq!(
            super::validate_distribution_authority(&missing),
            Err(super::RunError::Fixture)
        );
        missing.fixture_count = 191;
        missing.target_fixture_count = 191;
        assert_eq!(
            super::validate_distribution_authority(&missing),
            Err(super::RunError::Fixture)
        );
        missing.complete = false;
        assert_eq!(
            super::validate_distribution_authority(&missing),
            Err(super::RunError::Fixture)
        );
    }

    #[test]
    fn appended_distribution_authority_requires_exact_v11_inventory() {
        let fixtures = (0..193)
            .map(|index| super::DistributionFixture {
                fixture_id: format!("fixture_{index:03}"),
                metadata_path: format!("fixture_{index:03}.fixture.json"),
            })
            .collect::<Vec<_>>();
        let manifest = super::DistributionManifest {
            complete: true,
            distribution_schema: "nostr_automerge.fixture_distribution.v11".to_owned(),
            fixture_count: 193,
            fixtures,
            status: "canonical_signed_neutral_corpus".to_owned(),
            target_fixture_count: 193,
            transition_stage: None,
        };
        assert!(super::validate_distribution_authority(&manifest).is_ok());
        let mut missing = manifest;
        missing.fixtures.pop();
        assert_eq!(
            super::validate_distribution_authority(&missing),
            Err(super::RunError::Fixture)
        );
    }

    #[test]
    fn appended_distribution_authority_requires_exact_v12_inventory() {
        let fixtures = (0..198)
            .map(|index| super::DistributionFixture {
                fixture_id: format!("fixture_{index:03}"),
                metadata_path: format!("fixture_{index:03}.fixture.json"),
            })
            .collect::<Vec<_>>();
        let manifest = super::DistributionManifest {
            complete: true,
            distribution_schema: "nostr_automerge.fixture_distribution.v12".to_owned(),
            fixture_count: 198,
            fixtures,
            status: "canonical_signed_neutral_corpus".to_owned(),
            target_fixture_count: 198,
            transition_stage: Some("distribution_complete".to_owned()),
        };
        assert!(super::validate_distribution_authority(&manifest).is_ok());
        let mut missing = manifest;
        missing.fixtures.pop();
        assert_eq!(
            super::validate_distribution_authority(&missing),
            Err(super::RunError::Fixture)
        );
    }

    #[test]
    fn budget_rebound_distribution_authority_requires_exact_v14_inventory() {
        let fixtures = (0..204)
            .map(|index| super::DistributionFixture {
                fixture_id: format!("fixture_{index:03}"),
                metadata_path: format!("fixture_{index:03}.fixture.json"),
            })
            .collect::<Vec<_>>();
        let manifest = super::DistributionManifest {
            complete: true,
            distribution_schema: "nostr_automerge.fixture_distribution.v14".to_owned(),
            fixture_count: 204,
            fixtures,
            status: "canonical_signed_neutral_corpus".to_owned(),
            target_fixture_count: 204,
            transition_stage: Some("distribution_complete".to_owned()),
        };
        assert!(super::validate_distribution_authority(&manifest).is_ok());
        let mut missing = manifest;
        missing.fixtures.pop();
        assert_eq!(
            super::validate_distribution_authority(&missing),
            Err(super::RunError::Fixture)
        );
    }
}
