use core::str::FromStr;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use nostr_automerge::{
    ChangeHash, DispositionsDigest, DocumentCoordinate, EventId, HistoryDigest, SnapshotHash,
};

const SAFE_INTEGER_MAX: u64 = 9_007_199_254_740_991;
const CHECKPOINT_STATUSES: &[&str] = &[
    "verified",
    "pending_control",
    "unauthorized",
    "chunk_author_mismatch",
    "chunk_coordinate_mismatch",
    "chunk_descriptor_mismatch",
    "chunk_count_mismatch",
    "duplicate_chunk",
    "missing_chunk",
    "chunk_size_mismatch",
    "chunk_assembly_mismatch",
    "merkle_mismatch",
    "snapshot_size_mismatch",
    "snapshot_hash_mismatch",
    "snapshot_load",
    "head_mismatch",
    "commitment_mismatch",
    "closure_mismatch",
    "missing_historical_carrier",
    "not_accepted_at_control",
    "budget_exhausted",
    "cancelled",
];

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

impl ExpectedReport {
    pub(crate) fn empty(
        fixture_id: &str,
        revision: nostr_automerge::ProtocolRevision,
        coordinate: &str,
    ) -> Self {
        Self {
            report_schema: "nostr_automerge.report.v1".to_owned(),
            fixture_id: fixture_id.to_owned(),
            revision: revision.identifier().to_owned(),
            coordinate: coordinate.to_owned(),
            canonical_controls: Vec::new(),
            disposition_records: Vec::new(),
            accepted_changes: Vec::new(),
            pending_changes: Vec::new(),
            excluded_changes: Vec::new(),
            invalid_changes: Vec::new(),
            invalid_events: Vec::new(),
            unsupported_events: Vec::new(),
            heads: Vec::new(),
            history_digest: "00".repeat(32),
            dispositions_digest: "00".repeat(32),
            integrity_alerts: Vec::new(),
            checkpoints: Vec::new(),
            state_assertions: Vec::new(),
            completion: "complete".to_owned(),
        }
    }
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
        || !valid_fixture_id(&report.fixture_id)
        || nostr_automerge::ProtocolRevision::lookup(&report.revision).is_none()
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
        if checkpoint.change_count > SAFE_INTEGER_MAX
            || checkpoint.change_set_hash.len() != 64
            || !checkpoint
                .change_set_hash
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            || !CHECKPOINT_STATUSES.contains(&checkpoint.status.as_str())
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
        if !valid_integrity_alert(alert) {
            return Err(ExpectedError::Schema);
        }
    }
    Ok(())
}

fn valid_path_element(value: &Value) -> bool {
    if value.is_string() || value.as_u64().is_some() {
        return value.as_u64().is_none_or(|index| index <= SAFE_INTEGER_MAX);
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

fn valid_fixture_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    (3..=128).contains(&bytes.len())
        && (bytes[0].is_ascii_lowercase() || bytes[0].is_ascii_digit())
        && bytes[1..]
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_')
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

fn exact_fields(object: &serde_json::Map<String, Value>, fields: &[&str]) -> bool {
    object.len() == fields.len() && fields.iter().all(|field| object.contains_key(*field))
}

fn canonical_value_ids<T: FromStr + Ord>(value: &Value) -> bool {
    let Some(values) = value.as_array() else {
        return false;
    };
    let parsed = values
        .iter()
        .map(|value| value.as_str().and_then(|value| T::from_str(value).ok()))
        .collect::<Option<Vec<_>>>();
    parsed.is_some_and(|values| values.windows(2).all(|pair| pair[0] < pair[1]))
}

fn valid_integrity_alert(value: &Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    match object.get("type").and_then(Value::as_str) {
        Some("controller_equivocation") => {
            exact_fields(
                object,
                &[
                    "type",
                    "parent_control",
                    "candidate_controls",
                    "selected_control",
                ],
            ) && object.get("parent_control").is_some_and(|value| {
                value.is_null()
                    || value
                        .as_str()
                        .is_some_and(|value| EventId::from_str(value).is_ok())
            }) && object
                .get("candidate_controls")
                .is_some_and(canonical_value_ids::<EventId>)
                && object
                    .get("selected_control")
                    .and_then(Value::as_str)
                    .is_some_and(|value| EventId::from_str(value).is_ok())
        }
        Some("canonical_control_reorganization") => {
            exact_fields(
                object,
                &["type", "previous_tip", "new_tip", "affected_changes"],
            ) && ["previous_tip", "new_tip"].iter().all(|field| {
                object
                    .get(*field)
                    .and_then(Value::as_str)
                    .is_some_and(|value| EventId::from_str(value).is_ok())
            }) && object
                .get("affected_changes")
                .is_some_and(canonical_value_ids::<ChangeHash>)
        }
        Some("device_equivocation") => {
            exact_fields(
                object,
                &[
                    "type",
                    "actor_id",
                    "first_sequence",
                    "conflicting_changes",
                    "affected_descendants",
                ],
            ) && object
                .get("actor_id")
                .and_then(Value::as_str)
                .is_some_and(|value| EventId::from_str(value).is_ok())
                && object
                    .get("first_sequence")
                    .and_then(Value::as_u64)
                    .is_some_and(|value| (1..=SAFE_INTEGER_MAX).contains(&value))
                && object
                    .get("conflicting_changes")
                    .is_some_and(canonical_value_ids::<ChangeHash>)
                && object
                    .get("affected_descendants")
                    .is_some_and(canonical_value_ids::<ChangeHash>)
        }
        Some("potential_cloned_device_key") => {
            exact_fields(
                object,
                &["type", "actor_id", "first_sequence", "carrier_event_ids"],
            ) && object
                .get("actor_id")
                .and_then(Value::as_str)
                .is_some_and(|value| EventId::from_str(value).is_ok())
                && object
                    .get("first_sequence")
                    .and_then(Value::as_u64)
                    .is_some_and(|value| (1..=SAFE_INTEGER_MAX).contains(&value))
                && object
                    .get("carrier_event_ids")
                    .is_some_and(canonical_value_ids::<EventId>)
        }
        Some("checkpoint_mismatch") => {
            exact_fields(object, &["type", "descriptor_event_id", "code"])
                && object
                    .get("descriptor_event_id")
                    .and_then(Value::as_str)
                    .is_some_and(|value| EventId::from_str(value).is_ok())
                && object
                    .get("code")
                    .and_then(Value::as_str)
                    .is_some_and(|code| nostr_automerge::DiagnosticCode::lookup(code).is_some())
        }
        _ => false,
    }
}

fn valid_expected_value(value: &Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    let Some(kind) = object.get("type").and_then(Value::as_str) else {
        return false;
    };
    match kind {
        "null" | "bool" | "i64" | "u64" | "counter" | "timestamp" | "f64_bits" | "string"
        | "bytes_base64" => valid_materialized_scalar(value),
        "bytes32" => {
            exact_fields(object, &["type", "value"])
                && object
                    .get("value")
                    .and_then(Value::as_str)
                    .is_some_and(|value| SnapshotHash::from_str(value).is_ok())
        }
        "change_hash" => {
            exact_fields(object, &["type", "value"])
                && object
                    .get("value")
                    .and_then(Value::as_str)
                    .is_some_and(|value| ChangeHash::from_str(value).is_ok())
        }
        "event_id" => {
            exact_fields(object, &["type", "value"])
                && object
                    .get("value")
                    .and_then(Value::as_str)
                    .is_some_and(|value| EventId::from_str(value).is_ok())
        }
        "map" | "list" | "table" => valid_materialized_value(value),
        "text" => valid_materialized_value(value),
        "mark" => {
            exact_fields(
                object,
                &["type", "name", "value", "start", "end", "expansion"],
            ) && object.get("name").is_some_and(Value::is_string)
                && object
                    .get("expansion")
                    .and_then(Value::as_str)
                    .is_some_and(|value| matches!(value, "none" | "before" | "after" | "both"))
                && object.get("value").is_some_and(valid_materialized_scalar)
                && object
                    .get("start")
                    .and_then(Value::as_u64)
                    .is_some_and(|value| value <= SAFE_INTEGER_MAX)
                && object
                    .get("end")
                    .and_then(Value::as_u64)
                    .is_some_and(|value| value <= SAFE_INTEGER_MAX)
        }
        "conflicts" => {
            exact_fields(object, &["type", "values"])
                && object
                    .get("values")
                    .and_then(Value::as_array)
                    .is_some_and(|values| {
                        values.len() >= 2
                            && values.iter().all(|value| {
                                value.as_object().is_some_and(|conflict| {
                                    exact_fields(conflict, &["operation_id", "value"])
                                        && conflict
                                            .get("operation_id")
                                            .and_then(Value::as_str)
                                            .is_some_and(|value| !value.is_empty())
                                        && conflict
                                            .get("value")
                                            .is_some_and(valid_materialized_value)
                                })
                            })
                    })
        }
        _ => false,
    }
}

fn valid_materialized_scalar(value: &Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    let Some(kind) = object.get("type").and_then(Value::as_str) else {
        return false;
    };
    match kind {
        "null" => exact_fields(object, &["type"]),
        "bool" => {
            exact_fields(object, &["type", "value"])
                && object.get("value").is_some_and(Value::is_boolean)
        }
        "i64" | "counter" | "timestamp" => {
            exact_fields(object, &["type", "value"])
                && object
                    .get("value")
                    .and_then(Value::as_str)
                    .is_some_and(|value| {
                        value
                            .parse::<i64>()
                            .is_ok_and(|parsed| parsed.to_string() == value)
                    })
        }
        "u64" => {
            exact_fields(object, &["type", "value"])
                && object
                    .get("value")
                    .and_then(Value::as_str)
                    .is_some_and(|value| {
                        value
                            .parse::<u64>()
                            .is_ok_and(|parsed| parsed.to_string() == value)
                    })
        }
        "f64_bits" => {
            exact_fields(object, &["type", "value"])
                && object
                    .get("value")
                    .and_then(Value::as_str)
                    .is_some_and(|value| {
                        value.len() == 16
                            && value
                                .bytes()
                                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                    })
        }
        "string" => {
            exact_fields(object, &["type", "value"])
                && object.get("value").is_some_and(Value::is_string)
        }
        "bytes_base64" => {
            exact_fields(object, &["type", "value"])
                && object
                    .get("value")
                    .and_then(Value::as_str)
                    .is_some_and(|value| {
                        base64::engine::general_purpose::STANDARD
                            .decode(value)
                            .is_ok_and(|bytes| {
                                base64::engine::general_purpose::STANDARD.encode(bytes) == value
                            })
                    })
        }
        _ => false,
    }
}

fn valid_materialized_value(value: &Value) -> bool {
    if valid_materialized_scalar(value) {
        return true;
    }
    let Some(object) = value.as_object() else {
        return false;
    };
    match object.get("type").and_then(Value::as_str) {
        Some("map" | "list" | "table" | "text") => {
            let base = exact_fields(object, &["type", "object_id"])
                && object
                    .get("object_id")
                    .and_then(Value::as_str)
                    .is_some_and(|value| !value.is_empty());
            base || (object.get("type").and_then(Value::as_str) == Some("text")
                && exact_fields(object, &["type", "object_id", "value"])
                && object
                    .get("object_id")
                    .and_then(Value::as_str)
                    .is_some_and(|value| !value.is_empty())
                && object.get("value").is_some_and(Value::is_string))
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        CHECKPOINT_STATUSES, CheckpointResult, ExpectedError, SAFE_INTEGER_MAX, load_expected,
        valid_expected_value, valid_integrity_alert, validate_expected,
    };

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

        let mut wrong_revision = report.clone();
        wrong_revision.revision = "draft_2026_09".to_owned();
        assert_eq!(
            validate_expected(&wrong_revision),
            Err(ExpectedError::Schema)
        );

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

    #[test]
    fn expected_report_values_and_vocabularies_are_closed() {
        for value in [
            serde_json::json!({"type":"i64","value":i64::MIN.to_string()}),
            serde_json::json!({"type":"u64","value":u64::MAX.to_string()}),
            serde_json::json!({"type":"counter","value":"-7"}),
            serde_json::json!({"type":"timestamp","value":i64::MAX.to_string()}),
            serde_json::json!({"type":"bytes_base64","value":"AAE="}),
            serde_json::json!({
                "type":"mark",
                "name":"mode",
                "value":{"type":"bool","value":true},
                "start":0,
                "end":SAFE_INTEGER_MAX,
                "expansion":"both"
            }),
            serde_json::json!({
                "type":"conflicts",
                "values":[
                    {"operation_id":"1@a","value":{"type":"string","value":"left"}},
                    {"operation_id":"1@b","value":{"type":"text","object_id":"1@b"}}
                ]
            }),
        ] {
            assert!(valid_expected_value(&value), "rejected {value}");
        }

        for value in [
            serde_json::json!({"type":"i64","value":"9223372036854775808"}),
            serde_json::json!({"type":"u64","value":"18446744073709551616"}),
            serde_json::json!({"type":"counter","value":"-0"}),
            serde_json::json!({"type":"timestamp","value":"0001"}),
            serde_json::json!({"type":"bytes_base64","value":"AB=="}),
            serde_json::json!({
                "type":"mark",
                "name":"mode",
                "value":{"type":"null","extra":true},
                "start":0,
                "end":1,
                "expansion":"both"
            }),
            serde_json::json!({
                "type":"conflicts",
                "values":[
                    {"operation_id":"1@a","value":{"type":"string","value":"left"}},
                    {"operation_id":"","value":{"type":"string","value":"right"}}
                ]
            }),
            serde_json::json!({
                "type":"conflicts",
                "values":[
                    {"operation_id":"1@a","value":{"type":"string","value":"left"}},
                    {"operation_id":"1@b","value":{"type":"map","object_id":"1@b","extra":true}}
                ]
            }),
        ] {
            assert!(!valid_expected_value(&value), "accepted {value}");
        }

        assert!(valid_integrity_alert(&serde_json::json!({
            "type":"checkpoint_mismatch",
            "descriptor_event_id":"00".repeat(32),
            "code":"checkpoint.history"
        })));
        assert!(!valid_integrity_alert(&serde_json::json!({
            "type":"checkpoint_mismatch",
            "descriptor_event_id":"00".repeat(32),
            "code":"future.code"
        })));
    }

    #[test]
    fn expected_checkpoint_statuses_and_safe_numbers_are_closed() -> Result<(), ExpectedError> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/examples/actor_derivation_001.expected.json");
        let report = load_expected(&path)?;
        let checkpoint = |status: &str, change_count| CheckpointResult {
            descriptor_event: "00".repeat(32),
            chunk_events: Vec::new(),
            snapshot_hash: "11".repeat(32),
            heads: Vec::new(),
            change_count,
            change_set_hash: "22".repeat(32),
            historical_carriers: Vec::new(),
            accepted_at_control: Vec::new(),
            status: status.to_owned(),
        };
        for status in CHECKPOINT_STATUSES {
            let mut candidate = report.clone();
            candidate.checkpoints = vec![checkpoint(status, 0)];
            assert_eq!(validate_expected(&candidate), Ok(()), "rejected {status}");
        }
        for candidate_checkpoint in [
            checkpoint("future_status", 0),
            checkpoint("verified", SAFE_INTEGER_MAX + 1),
        ] {
            let mut candidate = report.clone();
            candidate.checkpoints = vec![candidate_checkpoint];
            assert_eq!(validate_expected(&candidate), Err(ExpectedError::Schema));
        }

        let mut malformed_fixture = report;
        malformed_fixture.fixture_id.push('\n');
        assert_eq!(
            validate_expected(&malformed_fixture),
            Err(ExpectedError::Schema)
        );
        Ok(())
    }
}
