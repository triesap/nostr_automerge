#!/usr/bin/env python3
"""Validate canonical conformance reports and semantic ordering."""

from __future__ import annotations

import base64
import binascii
import copy
import json
import re
import sys
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
from validate_dispositions_digest import encode as encode_dispositions
from validate_history_digest import encode as encode_history


ROOT = Path(__file__).resolve().parents[1]
HEX32 = re.compile(r"^[0-9a-f]{64}$")
COORDINATE = re.compile(r"^31624:[0-9a-f]{64}:[0-9a-f]{64}$")
FIXTURE_ID = re.compile(r"^[a-z0-9][a-z0-9_]{2,127}$")
FLOAT_BITS = re.compile(r"^[0-9a-f]{16}$")
SAFE_INTEGER_MAX = 9_007_199_254_740_991
I64_MIN = -(1 << 63)
I64_MAX = (1 << 63) - 1
U64_MAX = (1 << 64) - 1
REQUIRED_REPORT_FIELDS = (
    "report_schema", "fixture_id", "revision", "coordinate", "canonical_controls",
    "disposition_records", "accepted_changes", "pending_changes", "excluded_changes",
    "invalid_changes", "invalid_events", "unsupported_events", "heads", "history_digest",
    "dispositions_digest", "integrity_alerts", "state_assertions", "completion",
)
REPORT_FIELDS = {*REQUIRED_REPORT_FIELDS, "checkpoints"}
ID_LIST_FIELDS = (
    "accepted_changes", "pending_changes", "excluded_changes", "invalid_changes",
    "invalid_events", "unsupported_events", "heads",
)
EXPECTED_KEYS = {
    "null": {"type"}, "bool": {"type", "value"}, "i64": {"type", "value"},
    "u64": {"type", "value"}, "counter": {"type", "value"},
    "timestamp": {"type", "value"}, "f64_bits": {"type", "value"},
    "bytes32": {"type", "value"}, "change_hash": {"type", "value"},
    "event_id": {"type", "value"}, "string": {"type", "value"},
    "text": {"type", "object_id", "value"}, "bytes_base64": {"type", "value"},
    "map": {"type", "object_id"}, "list": {"type", "object_id"},
    "table": {"type", "object_id"},
    "mark": {"type", "name", "value", "start", "end", "expansion"},
    "conflicts": {"type", "values"},
}
ALERT_KEYS = {
    "controller_equivocation": {"type", "parent_control", "candidate_controls", "selected_control"},
    "canonical_control_reorganization": {"type", "previous_tip", "new_tip", "affected_changes"},
    "device_equivocation": {"type", "actor_id", "first_sequence", "conflicting_changes", "affected_descendants"},
    "potential_cloned_device_key": {"type", "actor_id", "first_sequence", "carrier_event_ids"},
    "checkpoint_mismatch": {"type", "descriptor_event_id", "code"},
}
CHECKPOINT_KEYS = {
    "descriptor_event", "chunk_events", "snapshot_hash", "heads", "change_count",
    "change_set_hash", "historical_carriers", "accepted_at_control", "status",
}
CHECKPOINT_STATUSES = (
    "verified", "pending_control", "unauthorized", "chunk_author_mismatch",
    "chunk_coordinate_mismatch", "chunk_descriptor_mismatch", "chunk_count_mismatch",
    "duplicate_chunk", "missing_chunk", "chunk_size_mismatch", "chunk_assembly_mismatch",
    "merkle_mismatch", "snapshot_size_mismatch", "snapshot_hash_mismatch", "snapshot_load",
    "head_mismatch", "commitment_mismatch", "closure_mismatch",
    "missing_historical_carrier", "not_accepted_at_control", "budget_exhausted", "cancelled",
)
DIAGNOSTIC_CODES = (
    "automerge.canonical", "automerge.checksum", "automerge.chunk_type", "automerge.leb128",
    "automerge.length", "automerge.magic", "automerge.semantics", "base64.noncanonical",
    "budget.exhausted", "cancellation.requested", "carrier.coordinate", "carrier.kind",
    "carrier.revision", "change.actor", "change.hash", "checkpoint.arithmetic",
    "checkpoint.chunk", "checkpoint.descriptor", "checkpoint.heads", "checkpoint.history",
    "checkpoint.merkle", "checkpoint.snapshot", "control.account_changed",
    "control.device_reintroduced", "control.frontier", "control.order", "control.parent",
    "control.retained_writer", "control.role_escalation", "control.structure",
    "control.terminal_child", "graph.actor_sequence", "graph.application", "graph.cycle",
    "graph.epoch_ancestry", "graph.missing_dependency", "graph.operation_counter",
    "jcs.noncanonical", "json.duplicate_member", "json.syntax", "manifest.semantics",
    "manifest.structure", "nip01.event_id", "nip01.identifier", "nip01.shape",
    "nip01.signature", "raw.invalid_utf8", "raw.too_large", "tag.forbidden", "tag.required",
)


class ReportError(Exception):
    """A stable report validation error."""


def load_json(path: Path) -> dict[str, Any]:
    """Load a JSON object."""

    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ReportError("report_not_object")
    return value


def validate_id_list(value: object) -> None:
    """Require a sorted unique list of 32-byte lowercase identifiers."""

    if not isinstance(value, list) or any(
        not isinstance(item, str) or HEX32.fullmatch(item) is None for item in value
    ):
        raise ReportError("invalid_id_list")
    if value != sorted(set(value)):
        raise ReportError("noncanonical_id_list")


def validate_control_chain(value: object) -> None:
    """Require unique control identifiers while preserving causal chain order."""

    if not isinstance(value, list) or any(
        not isinstance(item, str) or HEX32.fullmatch(item) is None for item in value
    ):
        raise ReportError("invalid_id_list")
    if len(value) != len(set(value)):
        raise ReportError("noncanonical_id_list")


def valid_path_element(value: object) -> bool:
    if isinstance(value, str):
        return True
    if isinstance(value, int) and not isinstance(value, bool):
        return 0 <= value <= SAFE_INTEGER_MAX
    if not isinstance(value, dict) or set(value) != {
        "type", "parent_object_id", "operation_id", "child_object_id"
    }:
        return False
    return value.get("type") == "branch" and all(
        isinstance(value.get(field), str) and bool(value[field])
        for field in ("parent_object_id", "operation_id", "child_object_id")
    )


def canonical_decimal(value: object, minimum: int, maximum: int) -> bool:
    if not isinstance(value, str):
        return False
    try:
        parsed = int(value, 10)
    except ValueError:
        return False
    return minimum <= parsed <= maximum and str(parsed) == value


def canonical_base64(value: object) -> bool:
    if not isinstance(value, str) or not value.isascii() or len(value) % 4:
        return False
    try:
        decoded = base64.b64decode(value, validate=True)
    except (binascii.Error, ValueError):
        return False
    return base64.b64encode(decoded).decode("ascii") == value


def valid_materialized_scalar(value: object) -> bool:
    if not isinstance(value, dict) or "type" not in value:
        return False
    kind = value["type"]
    if kind == "null":
        return set(value) == {"type"}
    if set(value) != {"type", "value"}:
        return False
    raw = value["value"]
    if kind == "bool":
        return isinstance(raw, bool)
    if kind in {"i64", "counter", "timestamp"}:
        return canonical_decimal(raw, I64_MIN, I64_MAX)
    if kind == "u64":
        return canonical_decimal(raw, 0, U64_MAX)
    if kind == "f64_bits":
        return isinstance(raw, str) and FLOAT_BITS.fullmatch(raw) is not None
    if kind == "string":
        return isinstance(raw, str)
    if kind == "bytes_base64":
        return canonical_base64(raw)
    return False


def valid_materialized_value(value: object) -> bool:
    if valid_materialized_scalar(value):
        return True
    if not isinstance(value, dict) or value.get("type") not in {"map", "list", "table", "text"}:
        return False
    if set(value) == {"type", "object_id"}:
        return isinstance(value["object_id"], str) and bool(value["object_id"])
    return (
        value.get("type") == "text"
        and set(value) == {"type", "object_id", "value"}
        and isinstance(value["object_id"], str)
        and bool(value["object_id"])
        and isinstance(value["value"], str)
    )


def valid_expected_value(value: object) -> bool:
    if valid_materialized_value(value):
        return True
    if not isinstance(value, dict) or "type" not in value:
        return False
    kind = value["type"]
    if kind in {"bytes32", "change_hash", "event_id"}:
        return (
            set(value) == {"type", "value"}
            and isinstance(value["value"], str)
            and HEX32.fullmatch(value["value"]) is not None
        )
    if kind == "mark":
        return (
            set(value) == EXPECTED_KEYS["mark"]
            and isinstance(value["name"], str)
            and valid_materialized_scalar(value["value"])
            and isinstance(value["start"], int)
            and not isinstance(value["start"], bool)
            and 0 <= value["start"] <= SAFE_INTEGER_MAX
            and isinstance(value["end"], int)
            and not isinstance(value["end"], bool)
            and 0 <= value["end"] <= SAFE_INTEGER_MAX
            and value["expansion"] in {"none", "before", "after", "both"}
        )
    if kind == "conflicts":
        values = value.get("values")
        return (
            set(value) == EXPECTED_KEYS["conflicts"]
            and isinstance(values, list)
            and len(values) >= 2
            and all(
                isinstance(conflict, dict)
                and set(conflict) == {"operation_id", "value"}
                and isinstance(conflict["operation_id"], str)
                and bool(conflict["operation_id"])
                and valid_materialized_value(conflict["value"])
                for conflict in values
            )
        )
    return False


def valid_alert(alert: object) -> bool:
    if not isinstance(alert, dict) or alert.get("type") not in ALERT_KEYS:
        return False
    kind = alert["type"]
    if set(alert) != ALERT_KEYS[kind]:
        return False
    if kind == "controller_equivocation":
        parent = alert["parent_control"]
        try:
            validate_id_list(alert["candidate_controls"])
        except ReportError:
            return False
        return (
            (parent is None or isinstance(parent, str) and HEX32.fullmatch(parent) is not None)
            and isinstance(alert["selected_control"], str)
            and HEX32.fullmatch(alert["selected_control"]) is not None
        )
    if kind == "canonical_control_reorganization":
        try:
            validate_id_list(alert["affected_changes"])
        except ReportError:
            return False
        return all(
            isinstance(alert[field], str) and HEX32.fullmatch(alert[field]) is not None
            for field in ("previous_tip", "new_tip")
        )
    if kind == "device_equivocation":
        lists = ("conflicting_changes", "affected_descendants")
    elif kind == "potential_cloned_device_key":
        lists = ("carrier_event_ids",)
    else:
        return (
            isinstance(alert["descriptor_event_id"], str)
            and HEX32.fullmatch(alert["descriptor_event_id"]) is not None
            and alert["code"] in DIAGNOSTIC_CODES
        )
    try:
        for field in lists:
            validate_id_list(alert[field])
    except ReportError:
        return False
    return (
        isinstance(alert["actor_id"], str)
        and HEX32.fullmatch(alert["actor_id"]) is not None
        and isinstance(alert["first_sequence"], int)
        and not isinstance(alert["first_sequence"], bool)
        and 1 <= alert["first_sequence"] <= SAFE_INTEGER_MAX
    )


def valid_checkpoint(checkpoint: object) -> bool:
    if not isinstance(checkpoint, dict) or set(checkpoint) != CHECKPOINT_KEYS:
        return False
    if any(
        not isinstance(checkpoint[field], str) or HEX32.fullmatch(checkpoint[field]) is None
        for field in ("descriptor_event", "snapshot_hash", "change_set_hash")
    ):
        return False
    try:
        for field in ("chunk_events", "heads", "historical_carriers", "accepted_at_control"):
            validate_id_list(checkpoint[field])
    except ReportError:
        return False
    return (
        isinstance(checkpoint["change_count"], int)
        and not isinstance(checkpoint["change_count"], bool)
        and 0 <= checkpoint["change_count"] <= SAFE_INTEGER_MAX
        and checkpoint["status"] in CHECKPOINT_STATUSES
    )


def validate(report: dict[str, Any]) -> None:
    """Validate report structure and canonical collection semantics."""

    if set(report) - REPORT_FIELDS or REPORT_FIELDS - {"checkpoints"} - set(report):
        raise ReportError("invalid_report_fields")
    if report["report_schema"] != "nostr_automerge.report.v1":
        raise ReportError("invalid_report_schema")
    if not isinstance(report["fixture_id"], str) or FIXTURE_ID.fullmatch(report["fixture_id"]) is None:
        raise ReportError("invalid_fixture_id")
    if report["revision"] != "draft_2026_08":
        raise ReportError("invalid_revision")
    if not isinstance(report["coordinate"], str) or COORDINATE.fullmatch(
        report["coordinate"]
    ) is None:
        raise ReportError("invalid_coordinate")
    if report["completion"] not in {"complete", "budget_exhausted", "cancelled"}:
        raise ReportError("invalid_completion")
    validate_control_chain(report["canonical_controls"])
    for field in ID_LIST_FIELDS:
        validate_id_list(report[field])
    records = report["disposition_records"]
    if not isinstance(records, list):
        raise ReportError("invalid_disposition_records")
    keys = []
    namespaces = {"control_event": 1, "change_hash": 2, "event": 3}
    dispositions = {"accepted", "pending", "excluded", "invalid", "unsupported_revision"}
    for record in records:
        if not isinstance(record, dict) or set(record) not in (
            {"namespace", "identifier", "disposition"},
            {"namespace", "identifier", "disposition", "diagnostic"},
        ):
            raise ReportError("invalid_disposition_record")
        if record["namespace"] not in namespaces or record["disposition"] not in dispositions:
            raise ReportError("invalid_disposition_record")
        if not isinstance(record["identifier"], str) or HEX32.fullmatch(record["identifier"]) is None:
            raise ReportError("invalid_disposition_record")
        if "diagnostic" in record and record["diagnostic"] not in DIAGNOSTIC_CODES:
            raise ReportError("invalid_disposition_record")
        keys.append((namespaces[record["namespace"]], record["identifier"]))
    if keys != sorted(set(keys)):
        raise ReportError("noncanonical_disposition_records")
    change_sets = [set(report[field]) for field in (
        "accepted_changes", "pending_changes", "excluded_changes", "invalid_changes"
    )]
    if any(left & right for index, left in enumerate(change_sets) for right in change_sets[index + 1:]):
        raise ReportError("conflated_change_collections")
    for field in ("history_digest", "dispositions_digest"):
        if not isinstance(report[field], str) or HEX32.fullmatch(report[field]) is None:
            raise ReportError("invalid_digest")
    history_vector = {
        "revision": report["revision"],
        "coordinate": report["coordinate"],
        "canonical_controls": report["canonical_controls"],
        "accepted_changes": report["accepted_changes"],
        "heads": report["heads"],
    }
    disposition_vector = {
        "revision": report["revision"],
        "coordinate": report["coordinate"],
        "items": report["disposition_records"],
    }
    import hashlib
    if hashlib.sha256(encode_history(history_vector)).hexdigest() != report["history_digest"]:
        raise ReportError("history_digest_mismatch")
    if hashlib.sha256(encode_dispositions(disposition_vector)).hexdigest() != report["dispositions_digest"]:
        raise ReportError("dispositions_digest_mismatch")

    alerts = report["integrity_alerts"]
    if not isinstance(alerts, list):
        raise ReportError("invalid_alerts")
    for alert in alerts:
        if not isinstance(alert, dict) or alert.get("type") not in ALERT_KEYS:
            raise ReportError("unknown_alert")
        if not valid_alert(alert):
            raise ReportError("invalid_alert")

    checkpoints = report.get("checkpoints", [])
    if not isinstance(checkpoints, list) or any(not valid_checkpoint(item) for item in checkpoints):
        raise ReportError("invalid_checkpoint")

    assertions = report["state_assertions"]
    if not isinstance(assertions, list):
        raise ReportError("invalid_assertions")
    for assertion in assertions:
        if not isinstance(assertion, dict) or set(assertion) != {"path", "expected"}:
            raise ReportError("invalid_assertion")
        path = assertion["path"]
        if not isinstance(path, list) or any(not valid_path_element(item) for item in path):
            raise ReportError("invalid_assertion_path")
        expected = assertion["expected"]
        if not isinstance(expected, dict) or expected.get("type") not in EXPECTED_KEYS:
            raise ReportError("unknown_assertion_type")
        if not valid_expected_value(expected):
            raise ReportError("invalid_assertion_value")


def expect_failure(report: dict[str, Any], code: str) -> None:
    """Require a report to fail with *code*."""

    try:
        validate(report)
    except ReportError as error:
        if str(error) != code:
            raise AssertionError(f"expected {code}, received {error}") from error
    else:
        raise AssertionError(f"invalid report unexpectedly passed: {code}")


def validate_schema_contract(schema: dict[str, Any]) -> None:
    if (
        schema.get("$schema") != "https://json-schema.org/draft/2020-12/schema"
        or schema.get("additionalProperties") is not False
        or schema.get("required") != list(REQUIRED_REPORT_FIELDS)
        or set(schema.get("properties", {})) != REPORT_FIELDS
    ):
        raise AssertionError("report schema top-level contract is open or reordered")
    definitions = schema.get("$defs")
    if not isinstance(definitions, dict):
        raise AssertionError("report schema definitions are missing")
    if definitions.get("diagnostic_code", {}).get("enum") != list(DIAGNOSTIC_CODES):
        raise AssertionError("report schema diagnostic registry drift")
    checkpoint_statuses = (
        definitions.get("checkpoint_result", {})
        .get("properties", {})
        .get("status", {})
        .get("enum")
    )
    if checkpoint_statuses != list(CHECKPOINT_STATUSES):
        raise AssertionError("report schema checkpoint status registry drift")
    if (
        definitions.get("expected_value", {}).get("oneOf", [])[-1]
        .get("properties", {}).get("values", {}).get("items", {}).get("$ref")
        != "#/$defs/conflict_value"
        or definitions.get("expected_value", {}).get("oneOf", [])[-2]
        .get("properties", {}).get("value", {}).get("$ref")
        != "#/$defs/materialized_scalar"
        or definitions.get("integrity_alert", {}).get("oneOf", [])[-1]
        .get("properties", {}).get("code", {}).get("$ref")
        != "#/$defs/diagnostic_code"
    ):
        raise AssertionError("report schema recursive value or diagnostic binding drift")

    i64_schemas = definitions.get("canonical_i64", {}).get("oneOf", [])
    if len(i64_schemas) != 2:
        raise AssertionError("report schema i64 domain is not closed")
    i64_patterns = [re.compile(item["pattern"]) for item in i64_schemas]
    u64_pattern = re.compile(definitions["canonical_u64"]["pattern"])
    base64_pattern = re.compile(definitions["canonical_base64"]["pattern"])
    for value in (str(I64_MIN), "-1", "0", "1", str(I64_MAX)):
        if sum(pattern.fullmatch(value) is not None for pattern in i64_patterns) != 1:
            raise AssertionError(f"report schema rejects canonical i64 {value}")
    for value in (str(I64_MIN - 1), str(I64_MAX + 1), "-0", "00", "+1"):
        if any(pattern.fullmatch(value) is not None for pattern in i64_patterns):
            raise AssertionError(f"report schema accepts noncanonical i64 {value}")
    for value in ("0", "1", str(U64_MAX)):
        if u64_pattern.fullmatch(value) is None:
            raise AssertionError(f"report schema rejects canonical u64 {value}")
    for value in ("-1", "00", "+1", str(U64_MAX + 1)):
        if u64_pattern.fullmatch(value) is not None:
            raise AssertionError(f"report schema accepts noncanonical u64 {value}")
    for value in ("", "AA==", "AAE=", "AAEC"):
        if base64_pattern.fullmatch(value) is None:
            raise AssertionError(f"report schema rejects canonical base64 {value!r}")
    for value in ("A", "AB==", "AAE", "AAE_", "AAE=\n"):
        if base64_pattern.fullmatch(value) is not None:
            raise AssertionError(f"report schema accepts noncanonical base64 {value!r}")


def main() -> int:
    """Validate schemas, examples, and reviewed negative cases."""

    schema = load_json(ROOT / "fixtures/schema/report.schema.json")
    validate_schema_contract(schema)
    schema_mutations = []
    open_schema = copy.deepcopy(schema)
    open_schema["additionalProperties"] = True
    schema_mutations.append(open_schema)
    missing_status = copy.deepcopy(schema)
    missing_status["$defs"]["checkpoint_result"]["properties"]["status"]["enum"].pop()
    schema_mutations.append(missing_status)
    extra_diagnostic = copy.deepcopy(schema)
    extra_diagnostic["$defs"]["diagnostic_code"]["enum"].append("future.code")
    schema_mutations.append(extra_diagnostic)
    open_conflicts = copy.deepcopy(schema)
    open_conflicts["$defs"]["expected_value"]["oneOf"][-1]["properties"]["values"]["items"] = {"type": "object"}
    schema_mutations.append(open_conflicts)
    for mutation in schema_mutations:
        try:
            validate_schema_contract(mutation)
        except AssertionError:
            pass
        else:
            raise AssertionError("report schema mutation unexpectedly passed")

    report_paths = sorted((ROOT / "fixtures").rglob("*.expected.json"))
    for path in report_paths:
        validate(load_json(path))
    report = load_json(ROOT / "fixtures/examples/actor_derivation_001.expected.json")

    unknown_field = copy.deepcopy(report)
    unknown_field["extra"] = None
    expect_failure(unknown_field, "invalid_report_fields")
    unknown_outcome = copy.deepcopy(report)
    unknown_outcome["completion"] = "resource_refused"
    expect_failure(unknown_outcome, "invalid_completion")
    unordered = copy.deepcopy(report)
    unordered["heads"] = ["f" * 64, "0" * 64]
    expect_failure(unordered, "noncanonical_id_list")
    unknown_assertion = copy.deepcopy(report)
    unknown_assertion["state_assertions"][0]["expected"]["type"] = "number"
    expect_failure(unknown_assertion, "unknown_assertion_type")
    unknown_alert = copy.deepcopy(report)
    unknown_alert["integrity_alerts"] = [{"type": "warning"}]
    expect_failure(unknown_alert, "unknown_alert")
    failed = copy.deepcopy(report)
    failed["completion"] = "failed"
    expect_failure(failed, "invalid_completion")
    duplicate_record = copy.deepcopy(report)
    synthetic_record = {
        "namespace": "change_hash",
        "identifier": "0" * 64,
        "disposition": "invalid",
    }
    duplicate_record["disposition_records"] = [synthetic_record, synthetic_record]
    expect_failure(duplicate_record, "noncanonical_disposition_records")
    wrong_order = copy.deepcopy(report)
    wrong_order["disposition_records"] = [
        {"namespace": "event", "identifier": "0" * 64, "disposition": "invalid"},
        {"namespace": "control_event", "identifier": "1" * 64, "disposition": "accepted"},
    ]
    expect_failure(wrong_order, "noncanonical_disposition_records")
    conflated = copy.deepcopy(report)
    conflated["excluded_changes"] = ["0" * 64]
    conflated["invalid_changes"] = ["0" * 64]
    expect_failure(conflated, "conflated_change_collections")
    wrong_history = copy.deepcopy(report)
    wrong_history["history_digest"] = "0" * 64
    expect_failure(wrong_history, "history_digest_mismatch")
    wrong_dispositions = copy.deepcopy(report)
    wrong_dispositions["dispositions_digest"] = "0" * 64
    expect_failure(wrong_dispositions, "dispositions_digest_mismatch")

    malformed_fixture = copy.deepcopy(report)
    malformed_fixture["fixture_id"] = "actor_derivation_001\n"
    expect_failure(malformed_fixture, "invalid_fixture_id")
    unknown_diagnostic = copy.deepcopy(report)
    unknown_diagnostic["disposition_records"] = [{
        "namespace": "event",
        "identifier": "0" * 64,
        "disposition": "invalid",
        "diagnostic": "future.code",
    }]
    expect_failure(unknown_diagnostic, "invalid_disposition_record")

    assertion_values = [
        {"type": "i64", "value": str(I64_MIN)},
        {"type": "u64", "value": str(U64_MAX)},
        {"type": "counter", "value": "-7"},
        {"type": "timestamp", "value": str(I64_MAX)},
        {"type": "bytes_base64", "value": "AAE="},
        {
            "type": "mark", "name": "mode", "value": {"type": "bool", "value": True},
            "start": 0, "end": SAFE_INTEGER_MAX, "expansion": "both",
        },
        {
            "type": "conflicts",
            "values": [
                {"operation_id": "1@a", "value": {"type": "string", "value": "left"}},
                {"operation_id": "1@b", "value": {"type": "text", "object_id": "1@b"}},
            ],
        },
    ]
    for value in assertion_values:
        positive = copy.deepcopy(report)
        positive["state_assertions"] = [{"path": ["value", SAFE_INTEGER_MAX], "expected": value}]
        validate(positive)

    invalid_assertions = [
        {"type": "i64", "value": str(I64_MIN - 1)},
        {"type": "u64", "value": str(U64_MAX + 1)},
        {"type": "counter", "value": "-0"},
        {"type": "timestamp", "value": "0001"},
        {"type": "bytes_base64", "value": "AB=="},
        {
            "type": "mark", "name": "mode", "value": {"type": "null", "extra": True},
            "start": 0, "end": 1, "expansion": "both",
        },
        {
            "type": "conflicts",
            "values": [
                {"operation_id": "1@a", "value": {"type": "string", "value": "left"}},
                {"operation_id": "", "value": {"type": "string", "value": "right"}},
            ],
        },
        {
            "type": "conflicts",
            "values": [
                {"operation_id": "1@a", "value": {"type": "string", "value": "left"}},
                {"operation_id": "1@b", "value": {"type": "map", "object_id": "1@b", "extra": True}},
            ],
        },
    ]
    for value in invalid_assertions:
        mutation = copy.deepcopy(report)
        mutation["state_assertions"] = [{"path": [], "expected": value}]
        expect_failure(mutation, "invalid_assertion_value")

    invalid_path = copy.deepcopy(report)
    invalid_path["state_assertions"] = [
        {"path": [SAFE_INTEGER_MAX + 1], "expected": {"type": "null"}}
    ]
    expect_failure(invalid_path, "invalid_assertion_path")

    checkpoint = {
        "descriptor_event": "0" * 64,
        "chunk_events": [],
        "snapshot_hash": "1" * 64,
        "heads": [],
        "change_count": 0,
        "change_set_hash": "2" * 64,
        "historical_carriers": [],
        "accepted_at_control": [],
        "status": "verified",
    }
    for status in CHECKPOINT_STATUSES:
        positive = copy.deepcopy(report)
        positive["checkpoints"] = [{**checkpoint, "status": status}]
        validate(positive)
    for field, value in (
        ("status", "future_status"),
        ("change_count", SAFE_INTEGER_MAX + 1),
        ("chunk_events", ["f" * 64, "0" * 64]),
    ):
        mutation = copy.deepcopy(report)
        mutation["checkpoints"] = [{**checkpoint, field: value}]
        expect_failure(mutation, "invalid_checkpoint")
    extra_checkpoint = copy.deepcopy(report)
    extra_checkpoint["checkpoints"] = [{**checkpoint, "extra": None}]
    expect_failure(extra_checkpoint, "invalid_checkpoint")

    checkpoint_alert = copy.deepcopy(report)
    checkpoint_alert["integrity_alerts"] = [{
        "type": "checkpoint_mismatch",
        "descriptor_event_id": "0" * 64,
        "code": "checkpoint.history",
    }]
    validate(checkpoint_alert)
    checkpoint_alert["integrity_alerts"][0]["code"] = "future.code"
    expect_failure(checkpoint_alert, "invalid_alert")

    print("PASS: canonical conformance report")
    print(f"- examples={len(report_paths)}")
    print("- negative_cases=27")
    print(f"- schema_mutations={len(schema_mutations)}")
    print(f"- checkpoint_statuses={len(CHECKPOINT_STATUSES)}")
    print(f"- diagnostic_codes={len(DIAGNOSTIC_CODES)}")
    print(f"- assertion_variants={len(EXPECTED_KEYS)}")
    print(f"- alert_variants={len(ALERT_KEYS)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
