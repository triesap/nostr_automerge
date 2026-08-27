#!/usr/bin/env python3
"""Validate and optionally execute the closed Rust distribution-v12 evidence."""

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
REPORT_PATH = "reports/rust_conformance_v12.json"
SCHEMA_PATH = "tools/validation/rust_conformance_v12.schema.json"
MANIFEST_PATH = "fixtures/distribution/manifest_v12.json"
RUNNER_PATH = "tools/nostr_automerge_conformance/src/runner.rs"
SOURCE_CANDIDATE = "69a9e10050c8674a462a712f0c8215351f4657a7"
SCHEMA_SHA256 = "c5465dbecdc3e5450ff73b23b331a9e9702e0ba88261c92f601d330870fdf08e"
FIELDS = (
    "schema", "status", "source_candidate", "manifest_sha256",
    "distribution_schema_sha256", "generator_sha256", "runner_sha256",
    "cargo_lock_sha256", "rust_toolchain_sha256", "scenario_count",
    "process_count", "delivery_order_count", "canonical_process_bytes",
    "canonical_output_sha256", "serialized_run_sha256",
    "deliberate_expectation_mismatch", "compatibility_rebinding",
    "result_identity_sha256",
)
EXPECTED = {
    "schema": "nostr_automerge.rust_conformance.v12.v1",
    "status": "pass",
    "source_candidate": SOURCE_CANDIDATE,
    "manifest_sha256": "29d1304aae027d33ff66b39b2cc499cca0e40fb24e5d4f5d749e33bf7dafd7c0",
    "distribution_schema_sha256": "2c2f1272559bd05f216107bc387870d7204f2cb7afd8bcc76c42bd5ea0a29e9d",
    "generator_sha256": "42e8e8173fb3cbe96722ed2fdec994c854c6ca8cf0f4e1f1b8002589b2a9b569",
    "runner_sha256": "0db1d0c80dd524d4f6f4ef7fb5554db5b49bd83f8c83b204bd43fda4c6486a1c",
    "cargo_lock_sha256": "6d1b886ff74637ba6682d349ab81424b0792f2cbc61cf0f213dfcf16af4f6744",
    "rust_toolchain_sha256": "5d959dfcc98b53886ee772ba216c4f9a1b31f093b46b5b263c0d084af54e821d",
    "scenario_count": 198,
    "process_count": 2,
    "delivery_order_count": 8,
    "canonical_process_bytes": "identical",
    "canonical_output_sha256": "ac1d326a2fe6fbc3ba495ecd7635250efd72179ac50985392757c1784cf59372",
    "serialized_run_sha256": "27e2febf15d800a81a9b87066ec9a4989d861fa8b8938b73c7a4fc3e87881932",
    "deliberate_expectation_mismatch": "rejected",
    "compatibility_rebinding": {
        "fixture_id": "canonical_derivation_exact_budget",
        "prior_max_items": 335,
        "required_max_items": 371,
        "prior_bytes_preserved": True,
        "delivery_orders_identical": True,
    },
    "result_identity_sha256": "e9ab4602f209a03a9366ec9ac2953fcd4d41b9aaab55f94ca6b1e3d5a3158967",
}


class EvidenceError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise EvidenceError(code)


def digest(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def canonical(value: Any) -> bytes:
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode()


def identity(value: dict[str, Any]) -> str:
    projection = {key: value[key] for key in FIELDS[:-1]}
    return hashlib.sha256(canonical(projection)[:-1]).hexdigest()


def validate_report(value: Any) -> None:
    require(type(value) is dict and tuple(value) == FIELDS, "report:keys")
    require(value == EXPECTED, "report:binding")
    require(value["result_identity_sha256"] == identity(value), "report:identity")


def validate_schema(value: Any) -> None:
    require(type(value) is dict and digest(SCHEMA_PATH) == SCHEMA_SHA256, "schema:sha256")
    require(value.get("type") == "object" and value.get("additionalProperties") is False, "schema:closed")
    require(value.get("required") == list(FIELDS), "schema:required")
    require(tuple(value.get("properties", {})) == FIELDS, "schema:properties")
    compatibility = value["properties"]["compatibility_rebinding"]
    require(
        compatibility.get("additionalProperties") is False
        and compatibility.get("required") == [
            "fixture_id", "prior_max_items", "required_max_items",
            "prior_bytes_preserved", "delivery_orders_identical",
        ],
        "schema:compatibility",
    )


def validate_sources() -> None:
    for relative, field in (
        (MANIFEST_PATH, "manifest_sha256"),
        ("tools/validation/distribution_v12.schema.json", "distribution_schema_sha256"),
        ("scripts/generate_distribution_v12.py", "generator_sha256"),
        (RUNNER_PATH, "runner_sha256"),
        ("Cargo.lock", "cargo_lock_sha256"),
        ("rust-toolchain.toml", "rust_toolchain_sha256"),
    ):
        require(digest(relative) == EXPECTED[field], f"source:{field}")
    ancestor = subprocess.run(
        ("git", "merge-base", "--is-ancestor", SOURCE_CANDIDATE, "HEAD"),
        cwd=ROOT,
        check=False,
        capture_output=True,
    )
    require(ancestor.returncode == 0, "source:candidate")
    manifest = json.loads((ROOT / MANIFEST_PATH).read_text(encoding="utf-8"))
    require(
        manifest.get("distribution_schema") == "nostr_automerge.fixture_distribution.v12"
        and manifest.get("transition_stage") == "distribution_complete"
        and manifest.get("complete") is True
        and manifest.get("fixture_count") == 198
        and len(manifest.get("fixtures", [])) == 198,
        "source:manifest",
    )
    rebound = next(
        (row for row in manifest["fixtures"] if row.get("fixture_id") == "canonical_derivation_exact_budget"),
        None,
    )
    require(
        type(rebound) is dict
        and rebound.get("metadata_path")
        == "fixtures/v12/scenarios/resource_followup/canonical_derivation_exact_budget.fixture.json",
        "source:rebinding",
    )
    prior = json.loads(
        (ROOT / "fixtures/v1_draft/scenarios/resource/canonical_derivation_exact_budget.input.json").read_text()
    )
    current = json.loads(
        (ROOT / "fixtures/v12/scenarios/resource_followup/canonical_derivation_exact_budget.input.json").read_text()
    )
    require(prior["budget"]["max_items"] == 335, "source:prior_budget")
    require(current["budget"]["max_items"] == 371, "source:current_budget")
    prior["budget"]["max_items"] = 371
    require(prior == current, "source:budget_only")


def mutation_self_test(report: dict[str, Any], schema: dict[str, Any]) -> int:
    mutations: list[dict[str, Any]] = []
    replacements = (
        ("source_candidate", "0" * 40), ("manifest_sha256", "0" * 64),
        ("runner_sha256", "0" * 64), ("scenario_count", 197),
        ("process_count", 1), ("delivery_order_count", 7),
        ("canonical_process_bytes", "different"),
        ("canonical_output_sha256", "0" * 64),
        ("serialized_run_sha256", "0" * 64),
        ("deliberate_expectation_mismatch", "accepted"),
        ("result_identity_sha256", "0" * 64),
    )
    for field, replacement in replacements:
        candidate = copy.deepcopy(report)
        candidate[field] = replacement
        mutations.append(candidate)
    for field, replacement in (
        ("prior_max_items", 334), ("required_max_items", 370),
        ("prior_bytes_preserved", False), ("delivery_orders_identical", False),
    ):
        candidate = copy.deepcopy(report)
        candidate["compatibility_rebinding"][field] = replacement
        mutations.append(candidate)
    missing = copy.deepcopy(report)
    missing.pop("status")
    mutations.append(missing)
    extra = copy.deepcopy(report)
    extra["extra"] = False
    mutations.append(extra)
    coordinated = copy.deepcopy(report)
    coordinated["canonical_output_sha256"] = "1" * 64
    coordinated["serialized_run_sha256"] = "2" * 64
    coordinated["result_identity_sha256"] = identity(coordinated)
    mutations.append(coordinated)
    reordered = {"status": report["status"], **report}
    mutations.append(reordered)
    for candidate in mutations:
        try:
            validate_report(candidate)
        except EvidenceError:
            continue
        raise EvidenceError("mutation:report")
    schema_mutations = []
    for mutate in (
        lambda value: value.update(additionalProperties=True),
        lambda value: value["required"].pop(),
        lambda value: value["properties"].pop("compatibility_rebinding"),
        lambda value: value["properties"]["compatibility_rebinding"].update(additionalProperties=True),
    ):
        candidate = copy.deepcopy(schema)
        mutate(candidate)
        schema_mutations.append(candidate)
    for candidate in schema_mutations:
        try:
            validate_schema(candidate)
        except EvidenceError:
            continue
        raise EvidenceError("mutation:schema")
    return len(mutations) + len(schema_mutations)


def distribution_command() -> tuple[str, ...]:
    return (
        "cargo", "run", "--quiet", "-p", "nostr_automerge_conformance",
        "--locked", "--", "run_distribution", MANIFEST_PATH,
    )


def run_distribution_twice() -> None:
    outputs = [
        subprocess.run(distribution_command(), cwd=ROOT, check=True, capture_output=True).stdout
        for _ in range(2)
    ]
    require(outputs[0] == outputs[1], "run:process_identity")
    require(hashlib.sha256(outputs[0]).hexdigest() == EXPECTED["serialized_run_sha256"], "run:serialized")
    result = json.loads(outputs[0])
    require(
        result.get("status") == "pass"
        and result.get("fixture_count") == 198
        and result.get("delivery_permutations") == 8
        and result.get("canonical_output_sha256") == EXPECTED["canonical_output_sha256"]
        and len(result.get("reports", [])) == 198,
        "run:coverage",
    )


def run_deliberate_mismatch() -> None:
    root = "fixtures/v12/scenarios/resource_followup/canonical_derivation_exact_budget"
    fixture = json.loads((ROOT / f"{root}.fixture.json").read_text())
    scenario = json.loads((ROOT / f"{root}.input.json").read_text())
    expected = json.loads((ROOT / f"{root}.expected.json").read_text())
    expected["history_digest"] = "0" * 64
    scenario["expected_report"] = copy.deepcopy(expected)
    input_bytes = canonical(scenario)
    expected_bytes = canonical(expected)
    fixture["inputs"][0]["sha256"] = hashlib.sha256(input_bytes).hexdigest()
    fixture["expected"]["sha256"] = hashlib.sha256(expected_bytes).hexdigest()
    with tempfile.TemporaryDirectory() as temporary:
        directory = Path(temporary) / "scenarios" / "mismatch"
        directory.mkdir(parents=True)
        (directory / "canonical_derivation_exact_budget.input.json").write_bytes(input_bytes)
        (directory / "canonical_derivation_exact_budget.expected.json").write_bytes(expected_bytes)
        path = directory / "canonical_derivation_exact_budget.fixture.json"
        path.write_bytes(canonical(fixture))
        command = (
            "cargo", "run", "--quiet", "-p", "nostr_automerge_conformance",
            "--locked", "--", "run_fixture", str(path),
        )
        completed = subprocess.run(command, cwd=ROOT, check=False, capture_output=True)
    require(
        completed.returncode == 1
        and completed.stdout == b""
        and completed.stderr == b"fixture result does not match expected report\n",
        "run:mismatch",
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run", action="store_true")
    args = parser.parse_args()
    report = json.loads((ROOT / REPORT_PATH).read_text(encoding="utf-8"))
    schema = json.loads((ROOT / SCHEMA_PATH).read_text(encoding="utf-8"))
    validate_report(report)
    validate_schema(schema)
    validate_sources()
    mutations = mutation_self_test(report, schema)
    if args.run:
        run_distribution_twice()
        run_deliberate_mismatch()
    print("PASS: Rust distribution-v12 conformance evidence")
    print("- scenarios=198")
    print("- delivery_orders=8")
    print("- processes=2")
    print(f"- negative_mutations={mutations}")
    print(f"- executed={int(args.run) * 2}")
    print(f"- deliberate_mismatch={int(args.run)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
