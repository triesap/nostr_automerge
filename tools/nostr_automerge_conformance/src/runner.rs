use core::str::FromStr;
use std::fs;
use std::path::{Path, PathBuf};

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use nostr_automerge::{
    CheckpointVerificationStatus, Completion, ControllerPublicKey, CorpusBuilder, DevicePublicKey,
    DocumentId, EvidenceIdentifier, EvidenceStatus, IntegrityAlert, MaterializedMark,
    MaterializedMarkExpansion, MaterializedObjectType, MaterializedPathElement, MaterializedScalar,
    MaterializedValue, NeverCancelled, ProtocolRevision, ReferenceEvaluator, WorkBudget,
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
        signed_permutation_report(signed, expected.clone())?
    } else if fixture.fixture_id.starts_with("scenario_") {
        generic_report(
            ScenarioInput::parse(&input).map_err(|_| RunError::Input)?,
            expected.clone(),
        )?
    } else if fixture.fixture_id == "actor_derivation_001" {
        let input: ActorDerivationInput =
            serde_json::from_slice(&input).map_err(|_| RunError::Input)?;
        actor_derivation_report(expected.clone(), &input)?
    } else {
        return Err(RunError::Input);
    };
    compare_expected(&actual, &expected)?;
    write_canonical_report(&actual).map_err(|_| RunError::Expected)
}

fn signed_permutation_report(
    signed: SignedScenarioInput,
    expected: ExpectedReport,
) -> Result<ExpectedReport, RunError> {
    let invalid_ids = expected
        .invalid_events
        .iter()
        .chain(expected.unsupported_events.iter())
        .map(ToString::to_string)
        .collect::<std::collections::BTreeSet<_>>();
    let permutations = required_delivery_permutations(
        &signed.raw_events,
        |event| event_kind(event) == Some(1624),
        |event| event_kind(event) == Some(1625),
        |event| {
            event_id_text(event)
                .map(|identifier| invalid_ids.contains(&identifier))
                .unwrap_or(true)
        },
    );
    let mut baseline = None;
    for permutation in permutations {
        let report = generic_report(
            signed
                .clone()
                .with_raw_events(permutation.events)
                .into_scenario(),
            expected.clone(),
        )?;
        compare_expected(&report, &expected)?;
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

fn event_id_text(event: &crate::scenario::EncodedRawEventV2) -> Option<String> {
    event_value(event)?.get("id")?.as_str().map(str::to_owned)
}

fn is_normative_signed_fixture(path: &Path) -> bool {
    path.ancestors()
        .any(|ancestor| ancestor.file_name().is_some_and(|name| name == "scenarios"))
}

pub(crate) fn generic_report(
    scenario: ScenarioInput,
    mut output: ExpectedReport,
) -> Result<ExpectedReport, RunError> {
    let coordinate = scenario.coordinate.parse().map_err(|_| RunError::Input)?;
    let mut builder = CorpusBuilder::new();
    for raw in scenario.raw_events {
        let raw = raw.decode().map_err(|_| RunError::Input)?;
        let _ = builder.ingest_bytes(&raw);
    }
    let corpus = builder.finish();
    let mut budget = WorkBudget::new(scenario.budget.max_bytes, scenario.budget.max_items);
    let report = if let Some(cancel_after) = scenario.cancel_after {
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
    .map_err(|_| RunError::Evaluation)?;
    output.coordinate = report.coordinate().to_address();
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
    let document = report.document();
    let assertions = core::mem::take(&mut output.state_assertions);
    let mut resolved_assertions = Vec::new();
    for mut assertion in assertions {
        if assertion.expected.get("type").and_then(Value::as_str) == Some("all_branch_descendants")
        {
            let document = document.ok_or(RunError::Input)?;
            for entry in document.entries().iter().filter(|entry| {
                entry
                    .path()
                    .iter()
                    .any(|element| element.branch_identity().is_some())
            }) {
                resolved_assertions.push(StateAssertion {
                    path: materialized_path_json(entry.path()),
                    expected: materialized_conflicts(entry.conflicts()),
                });
            }
            continue;
        }
        let path = materialized_path(&assertion.path)?;
        let document = document.ok_or(RunError::Input)?;
        if assertion.expected.get("type").and_then(Value::as_str) == Some("mark") {
            let mark = exactly_one(document.marks(), |mark| mark.path() == path)?;
            assertion.expected = materialized_mark(mark);
        } else {
            let entry = exactly_one(document.entries(), |entry| entry.path() == path)?;
            assertion.expected = materialized_conflicts(entry.conflicts());
        }
        resolved_assertions.push(assertion);
    }
    output.state_assertions = resolved_assertions;
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

fn exactly_one<T>(values: &[T], mut matches: impl FnMut(&T) -> bool) -> Result<&T, RunError> {
    let mut found = values.iter().filter(|value| matches(value));
    let value = found.next().ok_or(RunError::Input)?;
    if found.next().is_some() {
        return Err(RunError::Input);
    }
    Ok(value)
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

fn materialized_path(path: &[Value]) -> Result<Vec<MaterializedPathElement>, RunError> {
    path.iter()
        .map(|part| {
            if let Some(key) = part.as_str() {
                Ok(MaterializedPathElement::Key(key.to_owned()))
            } else if let Some(index) = part.as_u64() {
                Ok(MaterializedPathElement::Index(index))
            } else if let Some(branch) = part.as_object() {
                if branch.len() != 4 || branch.get("type").and_then(Value::as_str) != Some("branch")
                {
                    return Err(RunError::Expected);
                }
                Ok(MaterializedPathElement::branch(
                    branch
                        .get("parent_object_id")
                        .and_then(Value::as_str)
                        .ok_or(RunError::Expected)?,
                    branch
                        .get("operation_id")
                        .and_then(Value::as_str)
                        .ok_or(RunError::Expected)?,
                    branch
                        .get("child_object_id")
                        .and_then(Value::as_str)
                        .ok_or(RunError::Expected)?,
                ))
            } else {
                Err(RunError::Expected)
            }
        })
        .collect()
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
    mut report: ExpectedReport,
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
    let Some(assertion) = report.state_assertions.first_mut() else {
        return Err(RunError::Expected);
    };
    assertion.expected = Value::Object(value);
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
    fixtures: Vec<DistributionFixture>,
}

#[derive(Deserialize)]
struct DistributionFixture {
    fixture_id: String,
    metadata_path: String,
}

pub(crate) fn run_distribution(path: &Path) -> Result<DistributionRun, RunError> {
    let bytes = fs::read(path).map_err(|_| RunError::Fixture)?;
    let manifest: DistributionManifest =
        serde_json::from_slice(&bytes).map_err(|_| RunError::Fixture)?;
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

    use super::{
        RunError, compare_expected, discover_fixtures, exactly_one, generic_report,
        materialized_path, run_corpus, run_fixture,
    };
    use crate::fixture::load_fixture;
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
    use crate::expected::load_expected;
    use crate::scenario::ScenarioInput;

    #[test]
    fn ambiguous_assertion_selection_is_rejected() {
        let values = [1_u8, 1, 2];
        assert_eq!(exactly_one(&values, |value| *value == 2), Ok(&2));
        assert_eq!(
            exactly_one(&values, |value| *value == 3),
            Err(RunError::Input)
        );
        assert_eq!(
            exactly_one(&values, |value| *value == 1),
            Err(RunError::Input)
        );
    }

    #[test]
    fn neutral_branch_path_maps_to_the_public_projection_type() {
        let path = materialized_path(&[
            serde_json::json!("root"),
            serde_json::json!({
                "type":"branch",
                "parent_object_id":"_root",
                "operation_id":"1@actor",
                "child_object_id":"1@actor"
            }),
        ]);
        let Ok(path) = path else { return };
        assert_eq!(path.len(), 2);
        assert_eq!(
            path[1].branch_identity(),
            Some(("_root", "1@actor", "1@actor"))
        );
    }

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
    fn add_corpus_cli_command() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
        let paths = discover_fixtures(&root);
        assert!(paths.is_ok());
        let Ok(paths) = paths else { return };
        let mut reversed = paths.clone();
        reversed.reverse();
        assert_eq!(
            run_corpus(paths.clone(), None, None),
            run_corpus(reversed, None, None)
        );
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
        assert_eq!(failures.failed, 1);
        assert_eq!(failures.passed + failures.failed, failures.total);
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
        let expected =
            load_expected(&root.join("fixtures/examples/actor_derivation_001.expected.json"));
        assert!(expected.is_ok());
        let Ok(mut expected) = expected else { return };
        expected.state_assertions.clear();
        let actual = generic_report(parsed, expected);
        assert!(actual.is_ok());
        let Ok(actual) = actual else { return };
        assert_eq!(actual.completion, "complete");
        assert!(actual.canonical_controls.is_empty());
        assert!(actual.accepted_changes.is_empty());
    }
}
