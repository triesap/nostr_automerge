#!/usr/bin/env python3
"""Validate and optionally execute Rust conformance via the v17 transition."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/rust_conformance_v17.json"
SCHEMA = ROOT / "tools/validation/rust_conformance_v17.schema.json"
TRANSITION_CANDIDATE = "10be9bc3d9a5bf653338c3b30195d0c8299c2dac"
RESOURCE_EVIDENCE_CANDIDATE = "e74dcdb3fdaa30aeeb59bab53126bbee82a64557"
TRANSITION_PATH = "spec/distribution_v17_transition.json"
RESOURCE_EVIDENCE_PATH = "reports/causal_projection_evidence_graph_v17.json"
FIELDS = [
    "schema", "status", "transition_candidate", "transition_path", "transition_sha256",
    "transition_schema_sha256", "transition_validator_sha256", "resource_contract_sha256",
    "resource_evidence_candidate", "resource_evidence_sha256", "resource_evidence_schema_sha256",
    "resource_evidence_validator_sha256", "manifest_path", "manifest_sha256", "manifest_lock_path",
    "manifest_lock_sha256", "fixture_generator_sha256", "runner_sha256", "main_sha256",
    "cargo_lock_sha256", "rust_toolchain_sha256", "scenario_count", "resource_site_count",
    "transition_affected_count", "selected_manifest_historical_rebinding_count", "unaffected_v17_count",
    "signed_event_count", "process_count", "delivery_order_count", "canonical_process_bytes",
    "canonical_output_sha256", "serialized_run_sha256", "deliberate_expectation_mismatch", "result",
    "result_identity_sha256",
]


class EvidenceError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise EvidenceError(label)


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def committed(candidate: str, path: str) -> bytes:
    completed = subprocess.run(
        ["git", "show", f"{candidate}:{path}"], cwd=ROOT, capture_output=True, check=False
    )
    require(completed.returncode == 0, "candidate:" + path)
    return completed.stdout


def digest(candidate: str, path: str) -> str:
    data = committed(candidate, path)
    require((ROOT / path).read_bytes() == data, "working_tree_drift:" + path)
    return hashlib.sha256(data).hexdigest()


def resolved_inputs() -> tuple[dict[str, object], dict[str, object], dict[str, object]]:
    transition = json.loads(committed(TRANSITION_CANDIDATE, TRANSITION_PATH))
    graph = json.loads(committed(RESOURCE_EVIDENCE_CANDIDATE, RESOURCE_EVIDENCE_PATH))
    manifest_path = transition["selected_manifest"]["path"]
    lock_path = transition["selected_lock"]["path"]
    require(manifest_path == "fixtures/distribution/manifest_v16.json", "transition:manifest_path")
    require(lock_path == "fixtures/distribution/manifest_v16.lock.json", "transition:lock_path")
    manifest = json.loads((ROOT / manifest_path).read_text())
    lock = json.loads((ROOT / lock_path).read_text())
    require(transition["status"] == "immutable_reuse" and transition["result"] == "pass", "transition:state")
    require(transition["affected_fixture_ids"] == [] and transition["counts"]["affected"] == 0, "transition:affected")
    require(transition["selected_manifest"]["sha256"] == hashlib.sha256((ROOT / manifest_path).read_bytes()).hexdigest(), "transition:manifest_hash")
    require(transition["selected_lock"]["sha256"] == hashlib.sha256((ROOT / lock_path).read_bytes()).hexdigest(), "transition:lock_hash")
    require(graph["status"] == "final" and graph["result"] == "pass", "resource:state")
    require(graph["counts"] == {"inventory_rows": 68, "proof_edges": 68, "coverage_edges": 68, "dangling": 0, "extra": 0}, "resource:coverage")
    require(manifest["fixture_count"] == 204 and lock["scenario_count"] == 204, "manifest:scenarios")
    require(lock["signed_event_count"] == 771, "manifest:events")
    return transition, manifest, lock


def expected_report() -> dict[str, object]:
    transition, manifest, lock = resolved_inputs()
    sources = {
        "transition_sha256": (TRANSITION_CANDIDATE, TRANSITION_PATH),
        "transition_schema_sha256": (TRANSITION_CANDIDATE, "tools/validation/distribution_v17_transition.schema.json"),
        "transition_validator_sha256": (TRANSITION_CANDIDATE, "scripts/validate_distribution_v17_transition.py"),
        "resource_contract_sha256": (TRANSITION_CANDIDATE, "spec/causal_projection_contracts_v17.json"),
        "resource_evidence_sha256": (RESOURCE_EVIDENCE_CANDIDATE, RESOURCE_EVIDENCE_PATH),
        "resource_evidence_schema_sha256": (RESOURCE_EVIDENCE_CANDIDATE, "tools/validation/causal_projection_evidence_graph_v17.schema.json"),
        "resource_evidence_validator_sha256": (RESOURCE_EVIDENCE_CANDIDATE, "scripts/validate_causal_projection_evidence_graph_v17.py"),
        "fixture_generator_sha256": (TRANSITION_CANDIDATE, "tools/nostr_automerge_conformance/src/fixture_generation.rs"),
        "runner_sha256": (TRANSITION_CANDIDATE, "tools/nostr_automerge_conformance/src/runner.rs"),
        "main_sha256": (TRANSITION_CANDIDATE, "tools/nostr_automerge_conformance/src/main.rs"),
        "cargo_lock_sha256": (TRANSITION_CANDIDATE, "Cargo.lock"),
        "rust_toolchain_sha256": (TRANSITION_CANDIDATE, "rust-toolchain.toml"),
    }
    value = {
        "schema": "nostr_automerge.rust_conformance.v17.v1",
        "status": "pass",
        "transition_candidate": TRANSITION_CANDIDATE,
        "transition_path": TRANSITION_PATH,
        **{field: digest(candidate, path) for field, (candidate, path) in sources.items()},
        "resource_evidence_candidate": RESOURCE_EVIDENCE_CANDIDATE,
        "manifest_path": transition["selected_manifest"]["path"],
        "manifest_sha256": transition["selected_manifest"]["sha256"],
        "manifest_lock_path": transition["selected_lock"]["path"],
        "manifest_lock_sha256": transition["selected_lock"]["sha256"],
        "scenario_count": manifest["fixture_count"],
        "resource_site_count": 68,
        "transition_affected_count": transition["counts"]["affected"],
        "selected_manifest_historical_rebinding_count": len(manifest["authorized_v15_fixture_rebindings"]),
        "unaffected_v17_count": manifest["fixture_count"] - transition["counts"]["affected"],
        "signed_event_count": lock["signed_event_count"],
        "process_count": 2,
        "delivery_order_count": transition["counts"]["delivery_orders"],
        "canonical_process_bytes": "identical",
        "canonical_output_sha256": transition["identity"]["canonical_output_sha256"],
        "serialized_run_sha256": "000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344",
        "deliberate_expectation_mismatch": "rejected",
        "result": "pass",
        "result_identity_sha256": "",
    }
    value = {field: value[field] for field in FIELDS}
    value["result_identity_sha256"] = hashlib.sha256(canonical({key: value[key] for key in FIELDS[:-1]})).hexdigest()
    return value


def validate(report: object, schema: object) -> None:
    expected = expected_report()
    require(type(report) is dict and list(report) == FIELDS and report == expected, "report:value")
    resolved = subprocess.run(
        ["git", "rev-parse", "--verify", TRANSITION_CANDIDATE + "^{commit}"],
        cwd=ROOT, capture_output=True, text=True, check=False,
    )
    require(resolved.returncode == 0 and resolved.stdout.strip() == TRANSITION_CANDIDATE, "transition:candidate")
    require(
        type(schema) is dict
        and schema.get("additionalProperties") is False
        and schema.get("required") == FIELDS
        and list(schema.get("properties", {})) == FIELDS,
        "schema:closed",
    )


def self_test(report: dict[str, object], schema: dict[str, object]) -> int:
    cases = [
        ("report", lambda value: value.update(transition_candidate="0" * 40)),
        ("report", lambda value: value.update(transition_sha256="0" * 64)),
        ("report", lambda value: value.update(resource_evidence_sha256="0" * 64)),
        ("report", lambda value: value.update(manifest_path="fixtures/distribution/manifest_v15.json")),
        ("report", lambda value: value.update(manifest_sha256="0" * 64)),
        ("report", lambda value: value.update(resource_site_count=67)),
        ("report", lambda value: value.update(transition_affected_count=1)),
        ("report", lambda value: value.update(signed_event_count=770)),
        ("report", lambda value: value.update(process_count=1)),
        ("report", lambda value: value.update(canonical_output_sha256="0" * 64)),
        ("report", lambda value: value.update(serialized_run_sha256="0" * 64)),
        ("report", lambda value: value.update(deliberate_expectation_mismatch="accepted")),
        ("report", lambda value: value.update(result_identity_sha256="0" * 64)),
        ("report", lambda value: value.update(extra=False)),
        ("schema", lambda value: value.update(additionalProperties=True)),
    ]
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


def command(manifest_path: str) -> list[str]:
    return [
        "cargo", "run", "--quiet", "-p", "nostr_automerge_conformance", "--locked", "--",
        "run_distribution", manifest_path,
    ]


def run_twice(report: dict[str, object]) -> None:
    outputs = [
        subprocess.run(command(str(report["manifest_path"])), cwd=ROOT, capture_output=True, check=True).stdout
        for _ in range(2)
    ]
    require(outputs[0] == outputs[1], "run:process_bytes")
    require(hashlib.sha256(outputs[0]).hexdigest() == report["serialized_run_sha256"], "run:identity")
    value = json.loads(outputs[0])
    require(
        value["status"] == "pass"
        and value["fixture_count"] == 204
        and value["delivery_permutations"] == 8
        and len(value["reports"]) == 204,
        "run:coverage",
    )
    require(value["canonical_output_sha256"] == report["canonical_output_sha256"], "run:canonical")


def run_mismatch() -> None:
    root = ROOT / "fixtures/v16/rebindings/causal_projection/deep_actor_predecessor_exact_budget"
    fixture, scenario, expected = (
        json.loads(root.with_suffix(suffix).read_text())
        for suffix in (".fixture.json", ".input.json", ".expected.json")
    )
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
        completed = subprocess.run(
            ["cargo", "run", "--quiet", "-p", "nostr_automerge_conformance", "--locked", "--", "run_fixture", str(path)],
            cwd=ROOT, capture_output=True, check=False,
        )
    require(completed.returncode == 1, "run:mismatch_status")
    require(completed.stderr == b"fixture result does not match expected report\n", "run:mismatch_result")


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
    print(
        "PASS: Rust distribution-v17 transition scenarios=204 affected=0 "
        f"sites=68 mutations={mutations} executed={2 if args.run else 0}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
