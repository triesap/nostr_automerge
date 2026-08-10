#!/usr/bin/env python3
"""Validate canonical conformance reports and semantic ordering."""

from __future__ import annotations

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
REPORT_FIELDS = {
    "report_schema", "fixture_id", "revision", "coordinate",
    "canonical_controls", "disposition_records", "accepted_changes", "pending_changes",
    "excluded_changes", "invalid_changes", "invalid_events", "unsupported_events", "heads",
    "history_digest", "dispositions_digest", "integrity_alerts",
    "state_assertions", "completion", "checkpoints",
}
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
    "text": {"type", "value"}, "bytes_base64": {"type", "value"},
    "map": {"type"}, "list": {"type"},
    "mark": {"type", "name", "value", "start", "end"},
    "conflicts": {"type", "values"},
}
ALERT_KEYS = {
    "controller_equivocation": {"type", "parent_control", "candidate_controls", "selected_control"},
    "canonical_control_reorganization": {"type", "previous_tip", "new_tip", "affected_changes"},
    "device_equivocation": {"type", "actor_id", "first_sequence", "conflicting_changes", "affected_descendants"},
    "potential_cloned_device_key": {"type", "actor_id", "first_sequence", "carrier_event_ids"},
    "checkpoint_mismatch": {"type", "descriptor_event_id", "code"},
}


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


def validate(report: dict[str, Any]) -> None:
    """Validate report structure and canonical collection semantics."""

    if set(report) - REPORT_FIELDS or REPORT_FIELDS - {"checkpoints"} - set(report):
        raise ReportError("invalid_report_fields")
    if report["report_schema"] != "nostr_automerge.report.v1":
        raise ReportError("invalid_report_schema")
    if report["revision"] != "draft_2026_08":
        raise ReportError("invalid_revision")
    if not isinstance(report["coordinate"], str) or COORDINATE.fullmatch(
        report["coordinate"]
    ) is None:
        raise ReportError("invalid_coordinate")
    if report["completion"] not in {"complete", "budget_exhausted", "cancelled"}:
        raise ReportError("invalid_completion")
    validate_id_list(report["canonical_controls"])
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
        if set(alert) != ALERT_KEYS[alert["type"]]:
            raise ReportError("invalid_alert_fields")

    assertions = report["state_assertions"]
    if not isinstance(assertions, list):
        raise ReportError("invalid_assertions")
    for assertion in assertions:
        if not isinstance(assertion, dict) or set(assertion) != {"path", "expected"}:
            raise ReportError("invalid_assertion")
        path = assertion["path"]
        if not isinstance(path, list) or any(
            not isinstance(item, (str, int)) or isinstance(item, bool) or
            (isinstance(item, int) and item < 0) for item in path
        ):
            raise ReportError("invalid_assertion_path")
        expected = assertion["expected"]
        if not isinstance(expected, dict) or expected.get("type") not in EXPECTED_KEYS:
            raise ReportError("unknown_assertion_type")
        if set(expected) != EXPECTED_KEYS[expected["type"]]:
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


def main() -> int:
    """Validate schemas, examples, and reviewed negative cases."""

    schema = load_json(ROOT / "fixtures/schema/report.schema.json")
    if schema.get("$schema") != "https://json-schema.org/draft/2020-12/schema":
        raise AssertionError("report schema must use JSON Schema 2020-12")
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

    print("PASS: canonical conformance report")
    print(f"- examples={len(report_paths)}")
    print("- negative_cases=11")
    print(f"- assertion_variants={len(EXPECTED_KEYS)}")
    print(f"- alert_variants={len(ALERT_KEYS)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
