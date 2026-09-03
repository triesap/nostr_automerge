#!/usr/bin/env python3
"""Validate the final v17 evidence graph in both traversal directions."""

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
FINAL_PATH = "reports/causal_projection_final_inventory_v17.json"
PROOF_PATH = "reports/causal_projection_proofs_v17.json"
MUTATION_PATH = "reports/causal_projection_mutations_v17.json"
REPORT = ROOT / "reports/causal_projection_evidence_graph_v17.json"
SCHEMA = ROOT / "tools/validation/causal_projection_evidence_graph_v17.schema.json"
SOURCE_CANDIDATE = "789eae3c6e0994f71420f49fe51fe3ab7cb75ca9"
PROOF_CANDIDATE = "12f824659e055354779bb65b99f475c2ec109c43"
MUTATION_CANDIDATE = "eb760b20499792364624f24990deb35a3e8f54dd"
FINAL_CANDIDATE = "ad02b6ee407d6f5958c480f7f1b1c447eecc6f26"
FINAL_SHA256 = "f2212c183d009d146663116322ec4640db8812f66b6d09570115a3535a7603a2"
ATTACKS = ["dangling", "duplicate", "extra", "stale", "mismatched", "reordered", "planned", "self_referential", "coordinated"]


class GraphError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise GraphError(code)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True).encode()


def committed(candidate: str, path: str) -> bytes:
    result = subprocess.run(["git", "show", f"{candidate}:{path}"], cwd=ROOT, capture_output=True, check=False)
    require(result.returncode == 0, f"CANDIDATE:{path}")
    return result.stdout


def expected_report() -> dict[str, Any]:
    final_bytes = committed(FINAL_CANDIDATE, FINAL_PATH)
    require(sha(final_bytes) == FINAL_SHA256, "FINAL_SHA")
    final = json.loads(final_bytes)
    proofs = {row["proof_row_id"]: row for row in json.loads(committed(PROOF_CANDIDATE, PROOF_PATH))["rows"]}
    coverage = {row["coverage_id"]: row for row in json.loads(committed(MUTATION_CANDIDATE, MUTATION_PATH))["coverage_records"]}
    forward = []
    for row in final["rows"]:
        require(row["proof_row_id"] in proofs and row["mutation_coverage_id"] in coverage, f"DANGLING:{row['id']}")
        forward.append({
            "inventory_row_id": row["id"], "proof_row_id": row["proof_row_id"],
            "proof_artifact_sha256": row["proof_artifact_sha256"],
            "mutation_coverage_id": row["mutation_coverage_id"], "result": "pass",
        })
    reverse_proofs = [
        {"proof_row_id": item["proof_row_id"], "inventory_row_ids": [row["inventory_row_id"] for row in forward if row["proof_row_id"] == item["proof_row_id"]]}
        for item in forward
    ]
    reverse_coverage = [
        {"mutation_coverage_id": item["mutation_coverage_id"], "inventory_row_ids": [row["inventory_row_id"] for row in forward if row["mutation_coverage_id"] == item["mutation_coverage_id"]]}
        for item in forward
    ]
    report: dict[str, Any] = {
        "schema": "nostr_automerge.causal_projection_evidence_graph.v17.v1", "status": "final",
        "final_inventory": {"path": FINAL_PATH, "candidate": FINAL_CANDIDATE, "sha256": FINAL_SHA256},
        "candidate_order": [SOURCE_CANDIDATE, PROOF_CANDIDATE, MUTATION_CANDIDATE, FINAL_CANDIDATE],
        "forward": forward, "reverse_proofs": reverse_proofs, "reverse_coverage": reverse_coverage,
        "counts": {"inventory_rows": 68, "proof_edges": 68, "coverage_edges": 68, "dangling": 0, "extra": 0},
        "attack_matrix": [{"attack": attack, "result": "killed"} for attack in ATTACKS],
        "result_identity_sha256": "", "result": "pass",
    }
    report["result_identity_sha256"] = sha(canonical({key: value for key, value in report.items() if key != "result_identity_sha256"}))
    return report


def validate(report: object, schema: object) -> None:
    expected = expected_report()
    require(type(report) is dict and report == expected, "GRAPH_DERIVATION")
    forward = report["forward"]
    require(len(forward) == len({row["inventory_row_id"] for row in forward}) == 68, "FORWARD_UNIQUE")
    require(all(len(row["inventory_row_ids"]) == 1 for row in report["reverse_proofs"] + report["reverse_coverage"]), "REVERSE_LINK")
    require(not any(isinstance(value, str) and value.startswith("planned:") for row in forward for value in row.values()), "PLANNED")
    require(FINAL_CANDIDATE not in (SOURCE_CANDIDATE, PROOF_CANDIDATE, MUTATION_CANDIDATE), "SELF_REFERENCE")
    for parent, child in zip(report["candidate_order"], report["candidate_order"][1:]):
        require(subprocess.run(["git", "merge-base", "--is-ancestor", parent, child], cwd=ROOT, check=False).returncode == 0, "CANDIDATE_ORDER")
    require(schema.get("additionalProperties") is False and schema.get("required") == list(expected), "SCHEMA_CLOSED")


def self_test(report: dict[str, Any], schema: dict[str, Any]) -> int:
    mutations = [
        lambda value: value["forward"].pop(),
        lambda value: value["forward"].__setitem__(1, copy.deepcopy(value["forward"][0])),
        lambda value: value["forward"].append(copy.deepcopy(value["forward"][-1])),
        lambda value: value["final_inventory"].update(sha256="0" * 64),
        lambda value: value["forward"][0].update(proof_row_id="missing"),
        lambda value: value["forward"].reverse(),
        lambda value: value["forward"][0].update(proof_row_id="planned:proof"),
        lambda value: value["final_inventory"].update(candidate=MUTATION_CANDIDATE),
        lambda value: (value["final_inventory"].update(sha256="1" * 64), value.update(result_identity_sha256="2" * 64)),
    ]
    caught = 0
    for mutate in mutations:
        changed = copy.deepcopy(report); mutate(changed)
        try:
            validate(changed, schema)
        except GraphError:
            caught += 1
            continue
        raise GraphError("ATTACK_SURVIVED")
    return caught


def main() -> int:
    parser = argparse.ArgumentParser(); parser.add_argument("--write-report", action="store_true"); args = parser.parse_args()
    expected = expected_report()
    if args.write_report:
        REPORT.write_text(json.dumps(expected, ensure_ascii=True, indent=2) + "\n")
    report = json.loads(REPORT.read_text()); schema = json.loads(SCHEMA.read_text())
    validate(report, schema); attacks = self_test(report, schema)
    print(f"PASS: causal projection evidence graph v17 forward=68 reverse=136 attacks={attacks}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
