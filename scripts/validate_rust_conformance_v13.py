#!/usr/bin/env python3
"""Validate and optionally execute the closed Rust distribution-v13 evidence."""

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
REPORT_PATH = "reports/rust_conformance_v13.json"
SCHEMA_PATH = "tools/validation/rust_conformance_v13.schema.json"
MANIFEST_PATH = "fixtures/distribution/manifest_v13.json"
LOCK_PATH = "fixtures/distribution/manifest_v13.lock.json"
SOURCE_CANDIDATE = "378f15e7af474e34884b9b25a19960d37b0c02f6"
SCHEMA_SHA256 = "c9c96287080435bfe1f368d330769569e63055d696bbc8b7ce3b010b4ccc0ad4"
FIELDS = (
    "schema", "status", "source_candidate", "manifest_sha256", "manifest_lock_sha256",
    "distribution_schema_sha256", "generator_sha256", "fixture_generator_sha256",
    "runner_sha256", "cargo_lock_sha256", "rust_toolchain_sha256", "scenario_count",
    "fixture_rebinding_count", "process_count", "delivery_order_count",
    "canonical_process_bytes", "canonical_output_sha256", "serialized_run_sha256",
    "deliberate_expectation_mismatch", "result_identity_sha256",
)
EXPECTED = {
    "schema": "nostr_automerge.rust_conformance.v13.v1",
    "status": "pass",
    "source_candidate": SOURCE_CANDIDATE,
    "manifest_sha256": "12aa1b1f806ce810463768d566cc63d2ceba6126014d4da9fe5688df518bef3f",
    "manifest_lock_sha256": "c8145bbbd84d5d149b7ae7712f2ee4d16e3c8f5f367348c7119b41ccac40333f",
    "distribution_schema_sha256": "fa5f016bbadb83fe63e613ec0399a44eadd4dbb90d6d63b87f8b0e29f986cc34",
    "generator_sha256": "5375666085c5c09e3b343ed1cf5eaf350dada9619fb5b13267140c7118d4167c",
    "fixture_generator_sha256": "afe76f150b03852fd3aa5659b59e480c846de96ccfced42cdc6dbd5cf0606b51",
    "runner_sha256": "62d0fed49f73b942039728b748ac963aec00a1aa3c073cdec505b1da78020c5f",
    "cargo_lock_sha256": "6d1b886ff74637ba6682d349ab81424b0792f2cbc61cf0f213dfcf16af4f6744",
    "rust_toolchain_sha256": "5d959dfcc98b53886ee772ba216c4f9a1b31f093b46b5b263c0d084af54e821d",
    "scenario_count": 204,
    "fixture_rebinding_count": 4,
    "process_count": 2,
    "delivery_order_count": 8,
    "canonical_process_bytes": "identical",
    "canonical_output_sha256": "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415",
    "serialized_run_sha256": "000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344",
    "deliberate_expectation_mismatch": "rejected",
    "result_identity_sha256": "67d21245aaddbf5fe487a8e763063e428f7f71ec2ef0ac9b0ed3d682ede76007",
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
    require(value["properties"]["scenario_count"] == {"const": 204}, "schema:scenarios")
    require(value["properties"]["fixture_rebinding_count"] == {"const": 4}, "schema:rebindings")


def validate_sources() -> None:
    for relative, field in (
        (MANIFEST_PATH, "manifest_sha256"),
        (LOCK_PATH, "manifest_lock_sha256"),
        ("tools/validation/distribution_v13.schema.json", "distribution_schema_sha256"),
        ("scripts/generate_distribution_v13.py", "generator_sha256"),
        ("tools/nostr_automerge_conformance/src/fixture_generation.rs", "fixture_generator_sha256"),
        ("tools/nostr_automerge_conformance/src/runner.rs", "runner_sha256"),
        ("Cargo.lock", "cargo_lock_sha256"),
        ("rust-toolchain.toml", "rust_toolchain_sha256"),
    ):
        require(digest(relative) == EXPECTED[field], "source:" + field)
    candidate = subprocess.run(
        ("git", "rev-parse", SOURCE_CANDIDATE + "^{commit}"),
        cwd=ROOT, check=False, capture_output=True, text=True,
    )
    require(candidate.returncode == 0 and candidate.stdout.strip() == SOURCE_CANDIDATE, "source:candidate")
    manifest = json.loads((ROOT / MANIFEST_PATH).read_text())
    require(
        manifest.get("distribution_schema") == "nostr_automerge.fixture_distribution.v13"
        and manifest.get("transition_stage") == "distribution_complete"
        and manifest.get("complete") is True
        and manifest.get("fixture_count") == 204
        and len(manifest.get("fixtures", [])) == 204
        and len(manifest.get("authorized_v12_fixture_rebindings", [])) == 4,
        "source:manifest",
    )
    for row in manifest["authorized_v12_fixture_rebindings"]:
        prior = json.loads((ROOT / row["prior_metadata_path"].replace(".fixture.json", ".input.json")).read_text())
        current = json.loads((ROOT / row["current_metadata_path"].replace(".fixture.json", ".input.json")).read_text())
        require(prior["budget"]["max_items"] == row["prior_max_items"], "source:prior_budget")
        require(current["budget"]["max_items"] == row["required_max_items"], "source:required_budget")
        prior["budget"]["max_items"] = row["required_max_items"]
        require(prior == current and row["raw_events_preserved"] is True, "source:budget_only")


def mutation_self_test(report: dict[str, Any], schema: dict[str, Any]) -> int:
    mutations: list[dict[str, Any]] = []
    for field, replacement in (
        ("source_candidate", "0" * 40), ("manifest_sha256", "0" * 64),
        ("manifest_lock_sha256", "0" * 64), ("fixture_generator_sha256", "0" * 64),
        ("runner_sha256", "0" * 64), ("scenario_count", 203),
        ("fixture_rebinding_count", 3), ("process_count", 1),
        ("delivery_order_count", 7), ("canonical_process_bytes", "different"),
        ("canonical_output_sha256", "0" * 64), ("serialized_run_sha256", "0" * 64),
        ("deliberate_expectation_mismatch", "accepted"), ("result_identity_sha256", "0" * 64),
    ):
        candidate = copy.deepcopy(report)
        candidate[field] = replacement
        mutations.append(candidate)
    missing = copy.deepcopy(report)
    missing.pop("status")
    mutations.append(missing)
    extra = copy.deepcopy(report)
    extra["extra"] = False
    mutations.append(extra)
    reordered = {"status": report["status"], **report}
    mutations.append(reordered)
    coordinated = copy.deepcopy(report)
    coordinated["canonical_output_sha256"] = "1" * 64
    coordinated["serialized_run_sha256"] = "2" * 64
    coordinated["result_identity_sha256"] = identity(coordinated)
    mutations.append(coordinated)
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
        lambda value: value["properties"].pop("fixture_rebinding_count"),
        lambda value: value["properties"]["scenario_count"].update(const=203),
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
    outputs = [subprocess.run(distribution_command(), cwd=ROOT, check=True, capture_output=True).stdout for _ in range(2)]
    require(outputs[0] == outputs[1], "run:process_identity")
    require(hashlib.sha256(outputs[0]).hexdigest() == EXPECTED["serialized_run_sha256"], "run:serialized")
    result = json.loads(outputs[0])
    require(
        result.get("status") == "pass"
        and result.get("fixture_count") == 204
        and result.get("delivery_permutations") == 8
        and result.get("canonical_output_sha256") == EXPECTED["canonical_output_sha256"]
        and len(result.get("reports", [])) == 204,
        "run:coverage",
    )


def run_deliberate_mismatch() -> None:
    root = "fixtures/v13/scenarios/epoch_semantics/deep_actor_predecessor_exact_budget"
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
        (directory / "deep_actor_predecessor_exact_budget.input.json").write_bytes(input_bytes)
        (directory / "deep_actor_predecessor_exact_budget.expected.json").write_bytes(expected_bytes)
        path = directory / "deep_actor_predecessor_exact_budget.fixture.json"
        path.write_bytes(canonical(fixture))
        command = (
            "cargo", "run", "--quiet", "-p", "nostr_automerge_conformance",
            "--locked", "--", "run_fixture", str(path),
        )
        completed = subprocess.run(command, cwd=ROOT, check=False, capture_output=True)
    require(
        completed.returncode == 1 and completed.stdout == b""
        and completed.stderr == b"fixture result does not match expected report\n",
        "run:mismatch",
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run", action="store_true")
    args = parser.parse_args()
    report = json.loads((ROOT / REPORT_PATH).read_text())
    schema = json.loads((ROOT / SCHEMA_PATH).read_text())
    validate_report(report)
    validate_schema(schema)
    validate_sources()
    mutations = mutation_self_test(report, schema)
    if args.run:
        run_distribution_twice()
        run_deliberate_mismatch()
    print("PASS: Rust distribution-v13 conformance evidence")
    print("- scenarios=204 delivery_orders=8 processes=2")
    print(f"- negative_mutations={mutations}")
    print(f"- executed={int(args.run) * 2}")
    print(f"- deliberate_mismatch={int(args.run)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
