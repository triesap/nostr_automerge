use core::str::FromStr;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use nostr_automerge::{
    Completion, ControllerPublicKey, CorpusBuilder, DevicePublicKey, DocumentId,
    EvidenceIdentifier, EvidenceStatus, NeverCancelled, ProtocolRevision, ReferenceEvaluator,
    WorkBudget,
};

use crate::checksum::verify_fixture_files;
use crate::expected::{ExpectedReport, load_expected};
use crate::fixture::load_fixture;
use crate::report_json::write_canonical_report;
use crate::scenario::ScenarioInput;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RunError {
    Fixture,
    Checksum,
    Expected,
    Input,
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
    };
    output.coordinate = report.coordinate().to_address();
    output.canonical_controls = report
        .canonical_controls()
        .iter()
        .map(|event_id| (*event_id).to_hex())
        .collect();
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
        Completion::Failed => return Err(RunError::Input),
    }
    .to_owned();
    if !report.integrity_alerts().is_empty() || !output.state_assertions.is_empty() {
        return Err(RunError::Input);
    }
    output.integrity_alerts.clear();
    Ok(output)
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
