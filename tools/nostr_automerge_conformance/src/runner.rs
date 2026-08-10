use core::str::FromStr;
use std::fs;
use std::path::{Path, PathBuf};

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use nostr_automerge::{
    CheckpointVerificationStatus, Completion, ControllerPublicKey, CorpusBuilder, DevicePublicKey,
    DocumentId, EvidenceIdentifier, EvidenceStatus, IntegrityAlert, MaterializedObjectType,
    MaterializedPathElement, MaterializedScalar, MaterializedValue, NeverCancelled,
    ProtocolRevision, ReferenceEvaluator, WorkBudget,
};

use crate::checksum::verify_fixture_files;
use crate::expected::{CheckpointResult, DispositionRecord, ExpectedReport, load_expected};
use crate::fixture::load_fixture;
use crate::report_json::write_canonical_report;
use crate::scenario::ScenarioInput;

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
    let fixture = load_fixture(path).map_err(|_| RunError::Fixture)?;
    let base = path.parent().ok_or(RunError::Fixture)?;
    verify_fixture_files(&fixture, base).map_err(|_| RunError::Checksum)?;
    let expected =
        load_expected(&base.join(&fixture.expected.report_path)).map_err(|_| RunError::Expected)?;
    if fixture.inputs.len() != 1 {
        return Err(RunError::Input);
    }
    let input = fs::read(base.join(&fixture.inputs[0].path)).map_err(|_| RunError::Input)?;
    let actual = if fixture.fixture_id.starts_with("scenario_") {
        generic_report(
            ScenarioInput::parse(&input).map_err(|_| RunError::Input)?,
            expected.clone(),
        )?
    } else if fixture.fixture_id == "actor_derivation_001" {
        let input: ActorDerivationInput =
            serde_json::from_slice(&input).map_err(|_| RunError::Input)?;
        actor_derivation_report(expected.clone(), &input)?
    } else {
        crate::interop::evaluate(&fixture.fixture_id, &input, &expected)?
    };
    compare_expected(&actual, &expected)?;
    write_canonical_report(&actual).map_err(|_| RunError::Expected)
}

fn generic_report(
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
    for assertion in &mut output.state_assertions {
        let path = materialized_path(&assertion.path)?;
        let document = report.document().ok_or(RunError::Input)?;
        let entry = document
            .entries()
            .iter()
            .find(|entry| entry.path() == path)
            .ok_or(RunError::Input)?;
        assertion.expected = materialized_conflicts(entry.conflicts());
    }
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
            } else {
                Err(RunError::Expected)
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
        MaterializedValue::Object { object_type, .. } => serde_json::json!({
            "type": match object_type {
                MaterializedObjectType::Map => "map",
                MaterializedObjectType::List => "list",
                MaterializedObjectType::Table => "table",
                MaterializedObjectType::Text => "text",
            }
        }),
        MaterializedValue::Text { value, .. } => serde_json::json!({"type":"text", "value":value}),
    }
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

pub(crate) fn discover_fixtures(root: &Path) -> Result<Vec<PathBuf>, RunError> {
    let mut pending = vec![root.to_path_buf()];
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
    use std::path::Path;

    use super::{RunError, compare_expected, discover_fixtures, generic_report, run_corpus};
    use crate::expected::load_expected;
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
        let filtered = run_corpus(paths.clone(), Some("actor_derivation"), None);
        assert_eq!(filtered.total, 1);
        assert_eq!(filtered.failed, 0);
        let requirement = run_corpus(paths.clone(), None, Some("NCRDT-ACTOR-001"));
        assert_eq!(requirement.total, 2);
        assert_eq!(requirement.failed, 0);
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
