#!/usr/bin/env python3
"""Validate and execute the exact v16 causal-projection source-site proofs."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
INVENTORY = ROOT / "reports/causal_projection_operation_inventory_v16.json"
REPORT = ROOT / "reports/causal_projection_proof_catalog_v16.json"
SCHEMA = ROOT / "tools/validation/causal_projection_proof_catalog_v16.schema.json"
SOURCE = ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
INVENTORY_CANDIDATE = "3b978ff5f77d900b30d11b37bb240163afc38f2a"
INVENTORY_SHA256 = "95562a0f032c6fcedf3e397f82f42072fa2179b30a48b7424e38c2bf39403de1"
TOP_FIELDS = [
    "schema", "status", "inventory_candidate", "inventory_sha256",
    "proof_source_sha256", "proof_contract", "row_count", "rows",
    "global_proofs", "result_identity_sha256", "result",
]
ROW_FIELDS = [
    "id", "inventory_artifact_sha256", "source_site", "counter", "test",
    "command", "proof_mode", "artifact_sha256", "result",
]
GLOBAL_PROOFS = [
    "graph::actor_state::tests::actor_identity_and_sequence_relations_are_owned_immediate_and_short_circuiting",
    "graph::actor_state::tests::complete_candidate_semantics_preserve_precedence_and_every_stop_boundary",
    "graph::actor_state::tests::projection_operation_families_have_exact_n_minus_one_n_and_n_plus_one_stops",
]


class ProofError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise ProofError(label)


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True).encode()


def proof_artifact(test: str, command: str) -> str:
    return hashlib.sha256(canonical({"command": command, "failed": 0, "ignored": 0, "passed": 1, "test": test})).hexdigest()


def expected_rows(inventory: dict[str, Any]) -> list[dict[str, Any]]:
    return [
        {
            "id": row["id"],
            "inventory_artifact_sha256": row["artifact_sha256"],
            "source_site": row["source_site"],
            "counter": row["counter"],
            "test": row["test"],
            "command": row["command"],
            "proof_mode": "shared_metered_wrapper_plus_exact_source_site",
            "artifact_sha256": proof_artifact(row["test"], row["command"]),
            "result": "pass",
        }
        for row in inventory["rows"]
    ]


def expected_report(inventory: dict[str, Any], source: str) -> dict[str, Any]:
    rows = expected_rows(inventory)
    value: dict[str, Any] = {
        "schema": "nostr_automerge.causal_projection_proof_catalog.v16.v1",
        "status": "pass",
        "inventory_candidate": INVENTORY_CANDIDATE,
        "inventory_sha256": INVENTORY_SHA256,
        "proof_source_sha256": hashlib.sha256(source.encode()).hexdigest(),
        "proof_contract": {
            "source_site": "exact",
            "budget": "n_minus_one_n_n_plus_one",
            "cancellation": "per_operation",
            "typed_stop": "preserved",
            "unexpected_error": "exact_identity",
            "post_stop_target_work": 0,
            "repeated_family": "shared_wrapper_plus_each_site_binding",
        },
        "row_count": len(rows),
        "rows": rows,
        "global_proofs": GLOBAL_PROOFS,
        "result_identity_sha256": "",
        "result": "pass",
    }
    identity = {key: item for key, item in value.items() if key != "result_identity_sha256"}
    value["result_identity_sha256"] = hashlib.sha256(canonical(identity)).hexdigest()
    return value


def git(*args: str) -> str:
    completed = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True, check=False)
    require(completed.returncode == 0, "git:" + ":".join(args))
    return completed.stdout.strip()


def validate(report: object, schema: object, inventory: dict[str, Any], source: str) -> None:
    expected = expected_report(inventory, source)
    require(type(report) is dict and list(report) == TOP_FIELDS and report == expected, "report:value")
    require(git("rev-parse", f"{INVENTORY_CANDIDATE}^{{commit}}") == INVENTORY_CANDIDATE, "inventory:candidate")
    inventory_bytes = subprocess.run(
        ["git", "show", f"{INVENTORY_CANDIDATE}:reports/causal_projection_operation_inventory_v16.json"],
        cwd=ROOT,
        capture_output=True,
        check=False,
    )
    require(inventory_bytes.returncode == 0 and hashlib.sha256(inventory_bytes.stdout).hexdigest() == INVENTORY_SHA256, "inventory:sha256")
    rows = report["rows"]
    require(len(rows) == len({row["id"] for row in rows}) == len({row["test"] for row in rows}) == 68, "rows:unique")
    for row in rows:
        short = row["test"].rsplit("::", 1)[-1]
        require(source.count(short) == 1, "source:test:" + row["id"])
    require(source.count("macro_rules! v16_projection_build_site_proofs") == 1, "source:build_macro")
    require(source.count("macro_rules! v16_direct_site_proofs") == 1, "source:direct_macro")
    build_macro = source.split("macro_rules! v16_projection_build_site_proofs", 1)[1].split(
        "v16_projection_build_site_proofs!(", 1
    )[0]
    direct_macro = source.split("macro_rules! v16_direct_site_proofs", 1)[1].split(
        "v16_direct_site_proofs!(", 1
    )[0]
    require("#[ignore" not in build_macro and build_macro.count("#[test]") == 1, "source:build_enabled")
    require("#[ignore" not in direct_macro and direct_macro.count("#[test]") == 1, "source:direct_enabled")
    for test in GLOBAL_PROOFS:
        require(source.count(f"fn {test.rsplit('::', 1)[-1]}()") == 1, "source:global:" + test)
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == TOP_FIELDS, "schema:closed")
    require(schema["properties"]["rows"].get("minItems") == schema["properties"]["rows"].get("maxItems") == 68, "schema:rows")


def exact_pass(row: dict[str, Any], completed: subprocess.CompletedProcess[str]) -> bool:
    output = completed.stdout + completed.stderr
    return (
        completed.returncode == 0
        and output.count(f"test {row['test']} ... ok") == 1
        and "running 1 test" in output
        and "1 passed; 0 failed; 0 ignored" in output
    )


def run_proofs(report: dict[str, Any]) -> None:
    for row in report["rows"]:
        completed = subprocess.run(row["command"].split(), cwd=ROOT, capture_output=True, text=True, check=False)
        require(exact_pass(row, completed), "proof:" + row["id"])


def self_test(report: dict, schema: dict, inventory: dict, source: str) -> int:
    cases = [
        ("missing", "report", lambda value: value["rows"].pop()),
        ("extra", "report", lambda value: value["rows"].append(copy.deepcopy(value["rows"][-1]))),
        ("duplicate", "report", lambda value: value["rows"].__setitem__(1, copy.deepcopy(value["rows"][0]))),
        ("order", "report", lambda value: value["rows"].reverse()),
        ("site", "report", lambda value: value["rows"][0].update(source_site="nearby")),
        ("counter", "report", lambda value: value["rows"][0].update(counter="graph_edge")),
        ("test", "report", lambda value: value["rows"][0].update(test="unrelated")),
        ("command", "report", lambda value: value["rows"][0].update(command="cargo test unrelated")),
        ("artifact", "report", lambda value: value["rows"][0].update(artifact_sha256="0" * 64)),
        ("identity", "report", lambda value: value.update(result_identity_sha256="0" * 64)),
        ("source", "source", lambda value: value.replace("causal_projection_v16_site_projection_construction_source_count_read_01", "stale_test", 1)),
        ("ignored", "source", lambda value: value.replace("#[test]\n                fn $test()", "#[test]\n                #[ignore]\n                fn $test()", 1)),
        ("schema", "schema", lambda value: value.update(additionalProperties=True)),
    ]
    caught = 0
    for label, target, mutation in cases:
        changed_report = copy.deepcopy(report)
        changed_schema = copy.deepcopy(schema)
        changed_source = source
        if target == "report":
            mutation(changed_report)
        elif target == "schema":
            mutation(changed_schema)
        else:
            changed_source = mutation(changed_source)
        try:
            validate(changed_report, changed_schema, inventory, changed_source)
        except ProofError:
            caught += 1
            continue
        raise ProofError("mutation_survived:" + label)
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write-report", action="store_true")
    parser.add_argument("--run-proofs", action="store_true")
    args = parser.parse_args()
    inventory = json.loads(INVENTORY.read_text())
    source = SOURCE.read_text()
    if args.write_report:
        REPORT.write_text(json.dumps(expected_report(inventory, source), ensure_ascii=True, indent=2) + "\n")
    report = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    validate(report, schema, inventory, source)
    mutations = self_test(report, schema, inventory, source)
    if args.run_proofs:
        run_proofs(report)
    print(f"PASS: causal projection proof catalog v16 rows=68 mutations={mutations} proofs={68 if args.run_proofs else 0}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
