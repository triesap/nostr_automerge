use core::str::FromStr;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use nostr_automerge::{
    ChangeHash, DispositionsDigest, DocumentCoordinate, EventId, HistoryDigest, SnapshotHash,
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExpectedReport {
    pub(crate) report_schema: String,
    pub(crate) fixture_id: String,
    pub(crate) revision: String,
    pub(crate) coordinate: String,
    pub(crate) canonical_controls: Vec<String>,
    pub(crate) disposition_records: Vec<DispositionRecord>,
    pub(crate) accepted_changes: Vec<String>,
    pub(crate) pending_changes: Vec<String>,
    pub(crate) excluded_changes: Vec<String>,
    pub(crate) invalid_changes: Vec<String>,
    pub(crate) invalid_events: Vec<String>,
    pub(crate) unsupported_events: Vec<String>,
    pub(crate) heads: Vec<String>,
    pub(crate) history_digest: String,
    pub(crate) dispositions_digest: String,
    pub(crate) integrity_alerts: Vec<Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) checkpoints: Vec<CheckpointResult>,
    pub(crate) state_assertions: Vec<StateAssertion>,
    pub(crate) completion: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct DispositionRecord {
    pub(crate) namespace: String,
    pub(crate) identifier: String,
    pub(crate) disposition: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) diagnostic: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CheckpointResult {
    pub(crate) descriptor_event: String,
    pub(crate) chunk_events: Vec<String>,
    pub(crate) snapshot_hash: String,
    pub(crate) heads: Vec<String>,
    pub(crate) change_count: u64,
    pub(crate) change_set_hash: String,
    pub(crate) historical_carriers: Vec<String>,
    pub(crate) accepted_at_control: Vec<String>,
    pub(crate) status: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct StateAssertion {
    pub(crate) path: Vec<Value>,
    pub(crate) expected: Value,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExpectedError {
    Io,
    Json,
    Schema,
    Identifier,
    Ordering,
    Assertion,
}

pub(crate) fn load_expected(path: &Path) -> Result<ExpectedReport, ExpectedError> {
    let bytes = fs::read(path).map_err(|_| ExpectedError::Io)?;
    let report: ExpectedReport = serde_json::from_slice(&bytes).map_err(|_| ExpectedError::Json)?;
    validate_expected(&report)?;
    Ok(report)
}

pub(crate) fn validate_expected(report: &ExpectedReport) -> Result<(), ExpectedError> {
    if report.report_schema != "nostr_automerge.report.v1"
        || report.revision != "draft_2026_08"
        || !matches!(
            report.completion.as_str(),
            "complete" | "budget_exhausted" | "cancelled"
        )
    {
        return Err(ExpectedError::Schema);
    }
    DocumentCoordinate::from_str(&report.coordinate).map_err(|_| ExpectedError::Identifier)?;
    HistoryDigest::from_str(&report.history_digest).map_err(|_| ExpectedError::Identifier)?;
    DispositionsDigest::from_str(&report.dispositions_digest)
        .map_err(|_| ExpectedError::Identifier)?;
    unique_ids::<EventId>(&report.canonical_controls)?;
    let mut previous = None;
    for record in &report.disposition_records {
        let namespace = match record.namespace.as_str() {
            "control_event" => 1_u8,
            "change_hash" => 2,
            "event" => 3,
            _ => return Err(ExpectedError::Schema),
        };
        let identifier = match namespace {
            2 => *ChangeHash::from_str(&record.identifier)
                .map_err(|_| ExpectedError::Identifier)?
                .as_bytes(),
            _ => *EventId::from_str(&record.identifier)
                .map_err(|_| ExpectedError::Identifier)?
                .as_bytes(),
        };
        if previous.is_some_and(|value| value >= (namespace, identifier))
            || !matches!(
                record.disposition.as_str(),
                "accepted" | "pending" | "excluded" | "invalid" | "unsupported_revision"
            )
            || record.diagnostic.as_ref().is_some_and(|diagnostic| {
                nostr_automerge::DiagnosticCode::lookup(diagnostic).is_none()
            })
        {
            return Err(ExpectedError::Ordering);
        }
        previous = Some((namespace, identifier));
    }
    for checkpoint in &report.checkpoints {
        EventId::from_str(&checkpoint.descriptor_event).map_err(|_| ExpectedError::Identifier)?;
        SnapshotHash::from_str(&checkpoint.snapshot_hash).map_err(|_| ExpectedError::Identifier)?;
        canonical_ids::<EventId>(&checkpoint.chunk_events)?;
        canonical_ids::<ChangeHash>(&checkpoint.heads)?;
        canonical_ids::<ChangeHash>(&checkpoint.historical_carriers)?;
        canonical_ids::<ChangeHash>(&checkpoint.accepted_at_control)?;
        if checkpoint.change_set_hash.len() != 64
            || !checkpoint
                .change_set_hash
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            || !matches!(
                checkpoint.status.as_str(),
                "verified"
                    | "pending_control"
                    | "unauthorized"
                    | "chunk_author_mismatch"
                    | "chunk_coordinate_mismatch"
                    | "chunk_descriptor_mismatch"
                    | "chunk_count_mismatch"
                    | "duplicate_chunk"
                    | "missing_chunk"
                    | "chunk_size_mismatch"
                    | "chunk_assembly_mismatch"
                    | "merkle_mismatch"
                    | "snapshot_size_mismatch"
                    | "snapshot_hash_mismatch"
                    | "snapshot_load"
                    | "head_mismatch"
                    | "commitment_mismatch"
                    | "closure_mismatch"
                    | "missing_historical_carrier"
                    | "not_accepted_at_control"
                    | "budget_exhausted"
                    | "cancelled"
            )
        {
            return Err(ExpectedError::Schema);
        }
    }
    canonical_ids::<ChangeHash>(&report.accepted_changes)?;
    canonical_ids::<ChangeHash>(&report.pending_changes)?;
    canonical_ids::<ChangeHash>(&report.excluded_changes)?;
    canonical_ids::<ChangeHash>(&report.invalid_changes)?;
    let accepted = report.accepted_changes.iter().collect::<BTreeSet<_>>();
    let pending = report.pending_changes.iter().collect::<BTreeSet<_>>();
    let excluded = report.excluded_changes.iter().collect::<BTreeSet<_>>();
    let invalid = report.invalid_changes.iter().collect::<BTreeSet<_>>();
    if !accepted.is_disjoint(&pending)
        || !accepted.is_disjoint(&excluded)
        || !accepted.is_disjoint(&invalid)
        || !pending.is_disjoint(&excluded)
        || !pending.is_disjoint(&invalid)
        || !excluded.is_disjoint(&invalid)
    {
        return Err(ExpectedError::Ordering);
    }
    canonical_ids::<EventId>(&report.invalid_events)?;
    canonical_ids::<EventId>(&report.unsupported_events)?;
    canonical_ids::<ChangeHash>(&report.heads)?;
    for assertion in &report.state_assertions {
        if assertion.path.iter().any(|part| !valid_path_element(part))
            || !valid_expected_value(&assertion.expected)
        {
            return Err(ExpectedError::Assertion);
        }
    }
    for alert in &report.integrity_alerts {
        let Some(kind) = alert.get("type").and_then(Value::as_str) else {
            return Err(ExpectedError::Schema);
        };
        if !matches!(
            kind,
            "controller_equivocation"
                | "canonical_control_reorganization"
                | "device_equivocation"
                | "potential_cloned_device_key"
                | "checkpoint_mismatch"
        ) {
            return Err(ExpectedError::Schema);
        }
    }
    Ok(())
}

fn valid_path_element(value: &Value) -> bool {
    if value.is_string() || value.as_u64().is_some() {
        return true;
    }
    let Some(object) = value.as_object() else {
        return false;
    };
    object.len() == 4
        && object.get("type").and_then(Value::as_str) == Some("branch")
        && ["parent_object_id", "operation_id", "child_object_id"]
            .iter()
            .all(|field| {
                object
                    .get(*field)
                    .and_then(Value::as_str)
                    .is_some_and(|value| !value.is_empty())
            })
}

fn unique_ids<T: FromStr + Ord>(values: &[String]) -> Result<(), ExpectedError> {
    let parsed = values
        .iter()
        .map(|value| T::from_str(value).map_err(|_| ExpectedError::Identifier))
        .collect::<Result<Vec<_>, _>>()?;
    (parsed.iter().collect::<BTreeSet<_>>().len() == parsed.len())
        .then_some(())
        .ok_or(ExpectedError::Ordering)
}

fn canonical_ids<T: FromStr + Ord>(values: &[String]) -> Result<(), ExpectedError> {
    let parsed = values
        .iter()
        .map(|value| T::from_str(value).map_err(|_| ExpectedError::Identifier))
        .collect::<Result<Vec<_>, _>>()?;
    parsed
        .windows(2)
        .all(|pair| pair[0] < pair[1])
        .then_some(())
        .ok_or(ExpectedError::Ordering)
}

fn valid_expected_value(value: &Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    let Some(kind) = object.get("type").and_then(Value::as_str) else {
        return false;
    };
    match kind {
        "null" => object.len() == 1,
        "map" | "list" | "table" => {
            object.len() == 2
                && object
                    .get("object_id")
                    .and_then(Value::as_str)
                    .is_some_and(|value| !value.is_empty())
        }
        "bool" => object.len() == 2 && object.get("value").is_some_and(Value::is_boolean),
        "i64" | "u64" | "counter" | "timestamp" | "f64_bits" | "bytes32" | "change_hash"
        | "event_id" | "string" | "bytes_base64" => {
            object.len() == 2 && object.get("value").is_some_and(Value::is_string)
        }
        "text" => {
            object.len() == 3
                && object.get("value").is_some_and(Value::is_string)
                && object
                    .get("object_id")
                    .and_then(Value::as_str)
                    .is_some_and(|value| !value.is_empty())
        }
        "mark" => {
            object.len() == 6
                && ["name", "value", "start", "end", "expansion"]
                    .iter()
                    .all(|field| object.contains_key(*field))
                && object
                    .get("expansion")
                    .and_then(Value::as_str)
                    .is_some_and(|value| matches!(value, "none" | "before" | "after" | "both"))
                && object.get("value").is_some_and(valid_expected_value)
                && object.get("start").and_then(Value::as_u64).is_some()
                && object.get("end").and_then(Value::as_u64).is_some()
        }
        "conflicts" => object
            .get("values")
            .and_then(Value::as_array)
            .is_some_and(|values| values.len() >= 2 && values.iter().all(Value::is_object)),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{ExpectedError, load_expected, validate_expected};

    #[test]
    fn parse_expected_canonical_report_schema() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/examples/actor_derivation_001.expected.json");
        let report = load_expected(&path);
        assert!(report.is_ok());
        let Ok(report) = report else { return };

        let mut wrong_schema = report.clone();
        wrong_schema.report_schema = "nostr_automerge.report.v2".to_owned();
        assert_eq!(validate_expected(&wrong_schema), Err(ExpectedError::Schema));

        let mut branch = report.clone();
        branch.state_assertions[0].path.push(serde_json::json!({
            "type":"branch",
            "parent_object_id":"_root",
            "operation_id":"1@actor",
            "child_object_id":"1@actor"
        }));
        assert!(validate_expected(&branch).is_ok());
        branch.state_assertions[0].path[1] = serde_json::json!({"type":"branch"});
        assert_eq!(validate_expected(&branch), Err(ExpectedError::Assertion));

        let mut mark = report.clone();
        mark.state_assertions[0].expected = serde_json::json!({
            "type":"mark",
            "name":"bold",
            "value":{"type":"bool","value":true},
            "start":0,
            "end":1,
            "expansion":"both"
        });
        assert!(validate_expected(&mark).is_ok());
        mark.state_assertions[0].expected["expansion"] = serde_json::json!("invalid");
        assert_eq!(validate_expected(&mark), Err(ExpectedError::Assertion));

        let mut unsorted = report;
        unsorted.accepted_changes = vec!["22".repeat(32), "11".repeat(32)];
        assert_eq!(validate_expected(&unsorted), Err(ExpectedError::Ordering));
    }
}
