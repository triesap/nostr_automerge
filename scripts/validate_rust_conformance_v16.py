#!/usr/bin/env python3
"""Validate and optionally execute closed Rust distribution-v16 evidence."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/rust_conformance_v16.json"
SCHEMA = ROOT / "tools/validation/rust_conformance_v16.schema.json"
MANIFEST = "fixtures/distribution/manifest_v16.json"
SOURCE_CANDIDATE = "d2653edc718b002b7fe13b18d01bfe09df1fa02d"
FIELDS = ["schema","status","source_candidate","manifest_sha256","manifest_lock_sha256","transition_sha256","transition_schema_sha256","distribution_schema_sha256","lock_schema_sha256","generator_sha256","distribution_validator_sha256","fixture_generator_sha256","runner_sha256","main_sha256","cargo_lock_sha256","rust_toolchain_sha256","scenario_count","fixture_rebinding_count","unaffected_fixture_count","signed_event_count","process_count","delivery_order_count","canonical_process_bytes","canonical_output_sha256","serialized_run_sha256","deliberate_expectation_mismatch","result_identity_sha256"]
SOURCES = {
    "manifest_sha256": ("fixtures/distribution/manifest_v16.json", "7890fe2532da48ca84e54f5b1b883a38fd1a60ff58bb2999a056025335a4b5d3"),
    "manifest_lock_sha256": ("fixtures/distribution/manifest_v16.lock.json", "9e09dfd2de706d320c3bcd7cfe45b2f9a7560d5e9354809d2a41e5f52a2fba90"),
    "transition_sha256": ("spec/distribution_v16_transition.json", "ebb74b8a7a930a1eff9c2c10ac84499f65c145242e49d4dd5fa5ead1a2cbd7ad"),
    "transition_schema_sha256": ("tools/validation/distribution_v16_transition.schema.json", "1d593bdd57767084cd00bdde16e54005e4dc2e239f4144e02a08cbbf56f45bc3"),
    "distribution_schema_sha256": ("tools/validation/distribution_v16.schema.json", "2ccc3ba07b3a46c674b4e43f491157e6756a78a6793ff6ffeb710cfcf089f015"),
    "lock_schema_sha256": ("tools/validation/distribution_v16_lock.schema.json", "47328d31b3f60ff3c6cc4ed372bdcad109915118505dd2b3358af896f31fb149"),
    "generator_sha256": ("scripts/generate_distribution_v16.py", "0a4d9056d8267befd0026e5c2313ef41efbc77e63423f2d38c3e001b45a7cb72"),
    "distribution_validator_sha256": ("scripts/validate_distribution_v16.py", "c64168923d731a2e68db6cc0c3895efa23da2d99c22a901e3f77dc440b25c58d"),
    "fixture_generator_sha256": ("tools/nostr_automerge_conformance/src/fixture_generation.rs", "25fb8936fa69aa631e82afe03b44f2fae272ab504ffd5fac52b2e4c37942f21f"),
    "runner_sha256": ("tools/nostr_automerge_conformance/src/runner.rs", "dc4079c4a08b38568d92208e70515c6fcf84a6e2936d2e1b4e1705111587b5b5"),
    "main_sha256": ("tools/nostr_automerge_conformance/src/main.rs", "dc27965f9e98c59e0e52ee26e170b4248a052e75525780561c58a3047ab70108"),
    "cargo_lock_sha256": ("Cargo.lock", "9dd7897a8b729fb687c6f9001bfdeb09e441db69a36f8f86ab4260a30d75ca19"),
    "rust_toolchain_sha256": ("rust-toolchain.toml", "5d959dfcc98b53886ee772ba216c4f9a1b31f093b46b5b263c0d084af54e821d"),
}


class EvidenceError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise EvidenceError(label)


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def expected_report() -> dict[str, object]:
    for path, expected in SOURCES.values():
        require(hashlib.sha256((ROOT / path).read_bytes()).hexdigest() == expected, "source:" + path)
    value = {
        "schema": "nostr_automerge.rust_conformance.v16.v1", "status": "pass", "source_candidate": SOURCE_CANDIDATE,
        **{field: expected for field, (_, expected) in SOURCES.items()},
        "scenario_count": 204, "fixture_rebinding_count": 8, "unaffected_fixture_count": 196,
        "signed_event_count": 771, "process_count": 2, "delivery_order_count": 8,
        "canonical_process_bytes": "identical",
        "canonical_output_sha256": "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415",
        "serialized_run_sha256": "000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344",
        "deliberate_expectation_mismatch": "rejected", "result_identity_sha256": "",
    }
    value["result_identity_sha256"] = hashlib.sha256(canonical({key: value[key] for key in FIELDS[:-1]})).hexdigest()
    return value


def validate(report: object, schema: object) -> None:
    expected = expected_report()
    require(type(report) is dict and list(report) == FIELDS and report == expected, "report:value")
    resolved = subprocess.run(["git", "rev-parse", "--verify", SOURCE_CANDIDATE + "^{commit}"], cwd=ROOT, capture_output=True, text=True, check=False)
    require(resolved.returncode == 0 and resolved.stdout.strip() == SOURCE_CANDIDATE, "report:candidate")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == FIELDS and list(schema.get("properties", {})) == FIELDS, "schema:closed")


def self_test(report: dict, schema: dict) -> int:
    cases = [("report", lambda value: value.update(source_candidate="0" * 40)), ("report", lambda value: value.update(manifest_sha256="0" * 64)), ("report", lambda value: value.update(transition_sha256="0" * 64)), ("report", lambda value: value.update(fixture_rebinding_count=9)), ("report", lambda value: value.update(signed_event_count=770)), ("report", lambda value: value.update(process_count=1)), ("report", lambda value: value.update(canonical_output_sha256="0" * 64)), ("report", lambda value: value.update(serialized_run_sha256="0" * 64)), ("report", lambda value: value.update(deliberate_expectation_mismatch="accepted")), ("report", lambda value: value.update(result_identity_sha256="0" * 64)), ("report", lambda value: value.update(extra=False)), ("schema", lambda value: value.update(additionalProperties=True))]
    caught = 0
    for target, mutate in cases:
        changed_report, changed_schema = copy.deepcopy(report), copy.deepcopy(schema)
        mutate(changed_report if target == "report" else changed_schema)
        try:
            validate(changed_report, changed_schema)
        except EvidenceError:
            caught += 1
            continue
        raise EvidenceError("mutation_survived:" + target)
    return caught


def command() -> list[str]:
    return ["cargo", "run", "--quiet", "-p", "nostr_automerge_conformance", "--locked", "--", "run_distribution", MANIFEST]


def run_twice(report: dict) -> None:
    outputs = [subprocess.run(command(), cwd=ROOT, capture_output=True, check=True).stdout for _ in range(2)]
    require(outputs[0] == outputs[1] and hashlib.sha256(outputs[0]).hexdigest() == report["serialized_run_sha256"], "run:identity")
    value = json.loads(outputs[0])
    require(value["status"] == "pass" and value["fixture_count"] == 204 and value["delivery_permutations"] == 8 and len(value["reports"]) == 204, "run:coverage")
    require(value["canonical_output_sha256"] == report["canonical_output_sha256"], "run:canonical")


def run_mismatch() -> None:
    root = ROOT / "fixtures/v16/rebindings/causal_projection/deep_actor_predecessor_exact_budget"
    fixture, scenario, expected = (json.loads(root.with_suffix(suffix).read_text()) for suffix in (".fixture.json", ".input.json", ".expected.json"))
    expected["history_digest"] = "0" * 64
    scenario["expected_report"] = copy.deepcopy(expected)
    input_bytes, expected_bytes = canonical(scenario) + b"\n", canonical(expected) + b"\n"
    fixture["inputs"][0]["sha256"] = hashlib.sha256(input_bytes).hexdigest()
    fixture["expected"]["sha256"] = hashlib.sha256(expected_bytes).hexdigest()
    with tempfile.TemporaryDirectory() as temporary:
        directory = Path(temporary)
        (directory / (root.name + ".input.json")).write_bytes(input_bytes)
        (directory / (root.name + ".expected.json")).write_bytes(expected_bytes)
        path = directory / (root.name + ".fixture.json")
        path.write_bytes(canonical(fixture) + b"\n")
        completed = subprocess.run(["cargo", "run", "--quiet", "-p", "nostr_automerge_conformance", "--locked", "--", "run_fixture", str(path)], cwd=ROOT, capture_output=True, check=False)
    require(completed.returncode == 1 and completed.stderr == b"fixture result does not match expected report\n", "run:mismatch")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write-report", action="store_true")
    parser.add_argument("--run", action="store_true")
    args = parser.parse_args()
    expected = expected_report()
    if args.write_report:
        REPORT.write_text(json.dumps(expected, ensure_ascii=True, indent=2) + "\n")
    report, schema = json.loads(REPORT.read_text()), json.loads(SCHEMA.read_text())
    validate(report, schema)
    mutations = self_test(report, schema)
    if args.run:
        run_twice(report)
        run_mismatch()
    print(f"PASS: Rust distribution-v16 scenarios=204 rebindings=8 mutations={mutations} executed={2 if args.run else 0}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
