#!/usr/bin/env python3
"""Fail-closed validation for executed requirement evidence v3."""

from __future__ import annotations

import argparse
import copy
import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCHEMA = ROOT / "tools/validation/requirement_coverage_v3.schema.json"
SHA256 = re.compile(r"^[0-9a-f]{64}$")
COMMIT = re.compile(r"^[0-9a-f]{40}$")


class EvidenceError(Exception):
    """One v3 evidence invariant failed."""


def validate_shape(report: dict[str, object]) -> None:
    required = {
        "schema", "requirements_sha256", "applicability_sha256",
        "fixture_distribution_sha256", "requirement_count", "rows",
    }
    if set(report) != required:
        raise EvidenceError("unknown_or_missing_report_field")
    if report["schema"] != "nostr_automerge.requirement_coverage.v3":
        raise EvidenceError("unknown_schema")
    for field in ("requirements_sha256", "applicability_sha256", "fixture_distribution_sha256"):
        if not isinstance(report[field], str) or not SHA256.fullmatch(report[field]):
            raise EvidenceError(f"invalid_digest:{field}")
    if report["requirement_count"] != 87 or not isinstance(report["rows"], list) or len(report["rows"]) != 87:
        raise EvidenceError("requirement_count")


def schema_self_test() -> None:
    schema = json.loads(SCHEMA.read_text())
    if schema.get("$schema") != "https://json-schema.org/draft/2020-12/schema":
        raise AssertionError("v3 schema does not declare JSON Schema 2020-12")
    required_proof = set(schema["$defs"]["proof"]["required"])
    expected = {
        "language", "implementation_identity", "implementation_commit",
        "implementation_path", "evidence_kind", "evidence_id",
        "execution_command", "runner_job", "result_artifact",
        "result_sha256", "result",
    }
    if required_proof != expected:
        raise AssertionError("v3 proof fields are incomplete")
    baseline = {
        "schema": "nostr_automerge.requirement_coverage.v3",
        "requirements_sha256": "00" * 32,
        "applicability_sha256": "11" * 32,
        "fixture_distribution_sha256": "22" * 32,
        "requirement_count": 87,
        "rows": [{} for _ in range(87)],
    }
    validate_shape(baseline)
    for field in ("applicability_sha256", "fixture_distribution_sha256"):
        mutated = copy.deepcopy(baseline)
        del mutated[field]
        try:
            validate_shape(mutated)
        except EvidenceError:
            continue
        raise AssertionError(f"missing {field} unexpectedly passed")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--report", type=Path, default=ROOT / "reports/requirements_coverage_v3.json")
    parser.add_argument("--schema-self-test", action="store_true")
    args = parser.parse_args()
    if args.schema_self_test:
        schema_self_test()
        print("PASS: requirement evidence schema v3 is complete and fail-closed")
        return 0
    validate_shape(json.loads(args.report.read_text()))
    print("PASS: requirement evidence v3 has a valid top-level shape")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
