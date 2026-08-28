#!/usr/bin/env python3
"""Validate and optionally execute the closed Rust distribution-v14 evidence."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
import tempfile
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
REPORT_PATH = "reports/rust_conformance_v14.json"
SCHEMA_PATH = "tools/validation/rust_conformance_v14.schema.json"
MANIFEST_PATH = "fixtures/distribution/manifest_v14.json"
FIELDS = (
    "schema", "status", "base_candidate", "manifest_sha256", "manifest_lock_sha256",
    "distribution_schema_sha256", "lock_schema_sha256", "generator_sha256",
    "fixture_generator_sha256", "runner_sha256", "cargo_lock_sha256",
    "rust_toolchain_sha256", "scenario_count", "fixture_rebinding_count",
    "process_count", "delivery_order_count", "canonical_process_bytes",
    "canonical_output_sha256", "serialized_run_sha256",
    "deliberate_expectation_mismatch", "result_identity_sha256",
)


class EvidenceError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise EvidenceError(code)


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def digest(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def identity(value: dict[str, Any]) -> str:
    return hashlib.sha256(canonical({key: value[key] for key in FIELDS[:-1]})).hexdigest()


def validate_report(value: object) -> None:
    require(type(value) is dict and tuple(value) == FIELDS, "report:shape")
    assert isinstance(value, dict)
    require(value["schema"] == "nostr_automerge.rust_conformance.v14.v1", "report:schema")
    require(value["status"] == "pass", "report:status")
    require(value["base_candidate"] == "54537099a48f79150e46a7d6ebbdab55044a4e42", "report:candidate")
    expected_sources = {
        "manifest_sha256": "fixtures/distribution/manifest_v14.json",
        "manifest_lock_sha256": "fixtures/distribution/manifest_v14.lock.json",
        "distribution_schema_sha256": "tools/validation/distribution_v14.schema.json",
        "lock_schema_sha256": "tools/validation/distribution_v14_lock.schema.json",
        "generator_sha256": "scripts/generate_distribution_v14.py",
        "fixture_generator_sha256": "tools/nostr_automerge_conformance/src/fixture_generation.rs",
        "runner_sha256": "tools/nostr_automerge_conformance/src/runner.rs",
        "cargo_lock_sha256": "Cargo.lock",
        "rust_toolchain_sha256": "rust-toolchain.toml",
    }
    for field, relative in expected_sources.items():
        require(value[field] == digest(relative), "report:source:" + field)
    require(value["scenario_count"] == 204 and value["fixture_rebinding_count"] == 9, "report:inventory")
    require(value["process_count"] == 2 and value["delivery_order_count"] == 8, "report:execution")
    require(value["canonical_process_bytes"] == "identical", "report:process_bytes")
    require(value["canonical_output_sha256"] == "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415", "report:canonical")
    require(value["serialized_run_sha256"] == "000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344", "report:serialized")
    require(value["deliberate_expectation_mismatch"] == "rejected", "report:mismatch")
    require(value["result_identity_sha256"] == identity(value), "report:identity")
    candidate = subprocess.run(("git", "rev-parse", value["base_candidate"] + "^{commit}"), cwd=ROOT, capture_output=True, text=True, check=False)
    require(candidate.returncode == 0 and candidate.stdout.strip() == value["base_candidate"], "report:candidate_exists")


def validate_schema(value: object) -> None:
    require(type(value) is dict, "schema:object")
    assert isinstance(value, dict)
    require(value.get("type") == "object" and value.get("additionalProperties") is False, "schema:closed")
    require(value.get("required") == list(FIELDS), "schema:required")
    require(tuple(value.get("properties", {})) == FIELDS, "schema:properties")
    require(value["properties"]["scenario_count"] == {"const": 204}, "schema:count")
    require(value["properties"]["fixture_rebinding_count"] == {"const": 9}, "schema:rebindings")


def self_test(report: dict[str, Any], schema: dict[str, Any]) -> int:
    count = 0
    for field, replacement in (
        ("base_candidate", "0" * 40), ("manifest_sha256", "0" * 64),
        ("manifest_lock_sha256", "0" * 64), ("runner_sha256", "0" * 64),
        ("scenario_count", 203), ("fixture_rebinding_count", 8),
        ("process_count", 1), ("delivery_order_count", 7),
        ("canonical_process_bytes", "different"), ("canonical_output_sha256", "0" * 64),
        ("serialized_run_sha256", "0" * 64), ("deliberate_expectation_mismatch", "accepted"),
        ("result_identity_sha256", "0" * 64),
    ):
        changed = copy.deepcopy(report)
        changed[field] = replacement
        try:
            validate_report(changed)
        except EvidenceError:
            count += 1
            continue
        raise EvidenceError("mutation:report:" + field)
    for mutate in (
        lambda value: value.pop("status"),
        lambda value: value.update(extra=False),
        lambda value: value.update(canonical_output_sha256="1" * 64, result_identity_sha256="2" * 64),
    ):
        changed = copy.deepcopy(report)
        mutate(changed)
        try:
            validate_report(changed)
        except EvidenceError:
            count += 1
            continue
        raise EvidenceError("mutation:report:shape")
    for mutate in (
        lambda value: value.update(additionalProperties=True),
        lambda value: value["required"].pop(),
        lambda value: value["properties"].pop("fixture_rebinding_count"),
        lambda value: value["properties"]["scenario_count"].update(const=203),
    ):
        changed = copy.deepcopy(schema)
        mutate(changed)
        try:
            validate_schema(changed)
        except EvidenceError:
            count += 1
            continue
        raise EvidenceError("mutation:schema")
    return count


def command() -> tuple[str, ...]:
    return ("cargo", "run", "--quiet", "-p", "nostr_automerge_conformance", "--locked", "--", "run_distribution", MANIFEST_PATH)


def run_twice(report: dict[str, Any]) -> None:
    outputs = [subprocess.run(command(), cwd=ROOT, check=True, capture_output=True).stdout for _ in range(2)]
    require(outputs[0] == outputs[1], "run:identity")
    require(hashlib.sha256(outputs[0]).hexdigest() == report["serialized_run_sha256"], "run:serialized")
    value = json.loads(outputs[0])
    require(value["status"] == "pass" and value["fixture_count"] == 204 and value["delivery_permutations"] == 8, "run:coverage")
    require(value["canonical_output_sha256"] == report["canonical_output_sha256"] and len(value["reports"]) == 204, "run:canonical")


def run_mismatch() -> None:
    root = ROOT / "fixtures/v14/rebindings/causal_projection/canonical_derivation_exact_budget"
    fixture = json.loads(root.with_suffix(".fixture.json").read_text())
    scenario = json.loads(root.with_suffix(".input.json").read_text())
    expected = json.loads(root.with_suffix(".expected.json").read_text())
    expected["history_digest"] = "0" * 64
    scenario["expected_report"] = copy.deepcopy(expected)
    input_bytes = canonical(scenario) + b"\n"
    expected_bytes = canonical(expected) + b"\n"
    fixture["inputs"][0]["sha256"] = hashlib.sha256(input_bytes).hexdigest()
    fixture["expected"]["sha256"] = hashlib.sha256(expected_bytes).hexdigest()
    with tempfile.TemporaryDirectory() as temporary:
        directory = Path(temporary)
        (directory / "canonical_derivation_exact_budget.input.json").write_bytes(input_bytes)
        (directory / "canonical_derivation_exact_budget.expected.json").write_bytes(expected_bytes)
        path = directory / "canonical_derivation_exact_budget.fixture.json"
        path.write_bytes(canonical(fixture) + b"\n")
        completed = subprocess.run(("cargo", "run", "--quiet", "-p", "nostr_automerge_conformance", "--locked", "--", "run_fixture", str(path)), cwd=ROOT, capture_output=True, check=False)
    require(completed.returncode == 1 and completed.stdout == b"" and completed.stderr == b"fixture result does not match expected report\n", "run:mismatch")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run", action="store_true")
    args = parser.parse_args()
    report = json.loads((ROOT / REPORT_PATH).read_text())
    schema = json.loads((ROOT / SCHEMA_PATH).read_text())
    validate_report(report)
    validate_schema(schema)
    mutations = self_test(report, schema)
    if args.run:
        run_twice(report)
        run_mismatch()
    print("PASS: Rust distribution-v14 conformance evidence")
    print(f"- scenarios=204 rebindings=9 mutations={mutations}")
    print(f"- executed={int(args.run) * 2} deliberate_mismatch={int(args.run)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
