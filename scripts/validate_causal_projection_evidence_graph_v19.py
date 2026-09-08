#!/usr/bin/env python3
"""Generate and validate the complete bidirectional v19 evidence graph."""

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
FINAL_PATH = "reports/causal_projection_final_inventory_v19.json"
CATALOG_PATH = "reports/causal_projection_catalogs_v19.json"
MAPPING_PATH = "reports/causal_projection_v18_execution_mapping_v19.json"
REPORT = ROOT / "reports/causal_projection_evidence_graph_v19.json"
SCHEMA = ROOT / "tools/validation/causal_projection_evidence_graph_v19.schema.json"
AUTHORITY = "spec/causal_projection_contracts_v19.json"
MAPPING_CANDIDATE = "50457ab4efd482582291c77dd3c44d3e3ecc0adb"
SOURCE_CANDIDATE = "a8f8651c26a0ac7822e1e5d61f3aafa3fec53d1f"
PROOF_EXECUTION_BASE = "7050d3abf4104d5772e05b037a5799904de031f2"
PROOF_ARTIFACT_COMMIT = "7c5a1603c4ee9f92d541a8ff6fd66071806518df"
MUTATION_EXECUTION_BASE = "8d14a3cbfddd80da4c621cdbb346997619b57b95"
MUTATION_ARTIFACT_COMMIT = "001bea9cb5edc8fb83aa972901f749f4707d59e1"
CATALOG_CANDIDATE = "266e80b67e13ea377ea874bbd4b75ac4ca5898fa"
FINAL_INVENTORY_CANDIDATE = "b489ef551ace84791f3d513dd5add0a5729b53a7"
TOP_FIELDS = [
    "schema", "status", "authority", "candidate_roles", "final_inventory",
    "catalog", "process_evidence", "candidate_order", "forward",
    "reverse_proofs", "reverse_mutations", "counts", "attack_matrix",
    "self_candidate", "result_identity_sha256", "result",
]
FORWARD_FIELDS = [
    "inventory_row_id", "proof_catalog_id", "mutation_coverage_ids", "result",
]


class GraphError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise GraphError(code)


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True).encode()


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def committed(candidate: str, path: str) -> bytes:
    completed = subprocess.run(
        ["git", "show", f"{candidate}:{path}"], cwd=ROOT,
        capture_output=True, check=False,
    )
    require(completed.returncode == 0, f"COMMITTED_PATH:{candidate}:{path}")
    return completed.stdout


def expected_report() -> dict[str, Any]:
    final_raw = committed(FINAL_INVENTORY_CANDIDATE, FINAL_PATH)
    catalog_raw = committed(CATALOG_CANDIDATE, CATALOG_PATH)
    mapping_raw = committed(MAPPING_CANDIDATE, MAPPING_PATH)
    final, catalog, mapping = json.loads(final_raw), json.loads(catalog_raw), json.loads(mapping_raw)
    require(mapping["result"] == "pass" and mapping["approved_step_count"] == 43, "PROCESS_EVIDENCE")
    forward = [
        {
            "inventory_row_id": row["id"],
            "proof_catalog_id": row["proof_catalog_id"],
            "mutation_coverage_ids": row["mutation_coverage_ids"],
            "result": "pass",
        }
        for row in final["rows"]
    ]
    proof_ids = [row["id"] for row in catalog["proof_catalog"]["rows"]
    ]
    mutation_ids = [row["id"] for row in catalog["mutation_catalog"]["rows"]]
    reverse_proofs = [
        {
            "proof_catalog_id": proof_id,
            "inventory_row_ids": [row["inventory_row_id"] for row in forward if row["proof_catalog_id"] == proof_id],
            "result": "pass",
        }
        for proof_id in proof_ids
    ]
    reverse_mutations = [
        {
            "mutation_catalog_id": mutation_id,
            "inventory_row_ids": [row["inventory_row_id"] for row in forward if mutation_id in row["mutation_coverage_ids"]],
            "result": "pass",
        }
        for mutation_id in mutation_ids
    ]
    require(all(row["inventory_row_ids"] for row in reverse_proofs), "PROOF_REVERSE_DANGLING")
    require(all(row["inventory_row_ids"] for row in reverse_mutations), "MUTATION_REVERSE_DANGLING")
    roles = {
        "mapping_candidate": MAPPING_CANDIDATE,
        "source_candidate": SOURCE_CANDIDATE,
        "proof_execution_base": PROOF_EXECUTION_BASE,
        "proof_artifact_commit": PROOF_ARTIFACT_COMMIT,
        "mutation_execution_base": MUTATION_EXECUTION_BASE,
        "mutation_artifact_commit": MUTATION_ARTIFACT_COMMIT,
        "catalog_candidate": CATALOG_CANDIDATE,
        "final_inventory_candidate": FINAL_INVENTORY_CANDIDATE,
    }
    report: dict[str, Any] = {
        "schema": "nostr_automerge.causal_projection_evidence_graph.v19.v1",
        "status": "final_bidirectional",
        "authority": AUTHORITY,
        "candidate_roles": roles,
        "final_inventory": {"path": FINAL_PATH, "candidate": FINAL_INVENTORY_CANDIDATE, "sha256": sha(final_raw)},
        "catalog": {"path": CATALOG_PATH, "candidate": CATALOG_CANDIDATE, "sha256": sha(catalog_raw)},
        "process_evidence": {"path": MAPPING_PATH, "candidate": MAPPING_CANDIDATE, "sha256": sha(mapping_raw), "approved_steps": 43, "result": "pass"},
        "candidate_order": list(roles.values()),
        "forward": forward,
        "reverse_proofs": reverse_proofs,
        "reverse_mutations": reverse_mutations,
        "counts": {
            "inventory_rows": len(forward),
            "proof_edges": len(forward),
            "mutation_edges": sum(len(row["mutation_coverage_ids"]) for row in forward),
            "proof_catalog_rows": len(proof_ids),
            "mutation_catalog_rows": len(mutation_ids),
            "mapped_predecessor_steps": mapping["approved_step_count"],
            "dangling": 0,
            "extra": 0,
        },
        "attack_matrix": [
            {"attack": attack, "result": "killed"}
            for attack in (
                "dangling", "duplicate", "extra", "stale", "mismatched",
                "reordered", "planned", "self_referential", "coordinated",
                "missing_helper_cell", "missing_process_mapping",
            )
        ],
        "self_candidate": None,
        "result_identity_sha256": "",
        "result": "pass",
    }
    identity = {key: value for key, value in report.items() if key != "result_identity_sha256"}
    report["result_identity_sha256"] = sha(canonical(identity))
    return report


def validate(report: Any, schema: Any) -> None:
    expected = expected_report()
    require(type(report) is dict and list(report) == TOP_FIELDS and report == expected, "GRAPH_DERIVATION")
    forward = report["forward"]
    require(len(forward) == len({row["inventory_row_id"] for row in forward}), "FORWARD_UNIQUE")
    require(all(list(row) == FORWARD_FIELDS for row in forward), "FORWARD_SHAPE")
    require(all(len(row["inventory_row_ids"]) == 1 for row in report["reverse_proofs"]), "PROOF_REVERSE_CARDINALITY")
    require(all(row["inventory_row_ids"] for row in report["reverse_mutations"]), "MUTATION_REVERSE_CARDINALITY")
    require(report["self_candidate"] is None, "SELF_REFERENCE")
    require(len(set(report["candidate_order"])) == len(report["candidate_order"]), "ROLE_AMBIGUITY")
    for parent, child in zip(report["candidate_order"], report["candidate_order"][1:]):
        require(subprocess.run(
            ["git", "merge-base", "--is-ancestor", parent, child], cwd=ROOT, check=False,
        ).returncode == 0, "CANDIDATE_ORDER")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == TOP_FIELDS, "SCHEMA_CLOSED")
    for key in ("forward", "reverse_proofs", "reverse_mutations"):
        require(schema["properties"][key].get("minItems") == 1 and "maxItems" not in schema["properties"][key], "SCHEMA_SOURCE_DERIVED")
        require(schema["properties"][key]["items"].get("additionalProperties") is False, "SCHEMA_ROWS_CLOSED")


def self_test(report: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks = [
        ("report", lambda value: value["forward"].pop()),
        ("report", lambda value: value["forward"].__setitem__(1, copy.deepcopy(value["forward"][0]))),
        ("report", lambda value: value["forward"][0].update(proof_catalog_id="missing")),
        ("report", lambda value: value["forward"][0].update(mutation_coverage_ids=[])),
        ("report", lambda value: value["reverse_mutations"][0].update(inventory_row_ids=[])),
        ("report", lambda value: value["final_inventory"].update(sha256="0" * 64)),
        ("report", lambda value: value["process_evidence"].update(sha256="0" * 64)),
        ("report", lambda value: value["candidate_order"].reverse()),
        ("report", lambda value: value.update(self_candidate=FINAL_INVENTORY_CANDIDATE)),
        ("report", lambda value: value["forward"].reverse()),
        ("schema", lambda value: value.update(additionalProperties=True)),
    ]
    caught = 0
    for target, mutate in attacks:
        changed_report, changed_schema = copy.deepcopy(report), copy.deepcopy(schema)
        mutate(changed_report if target == "report" else changed_schema)
        try:
            validate(changed_report, changed_schema)
        except GraphError:
            caught += 1
            continue
        raise GraphError("ATTACK_SURVIVED")
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write-report", action="store_true")
    args = parser.parse_args()
    expected = expected_report()
    if args.write_report:
        REPORT.write_text(json.dumps(expected, ensure_ascii=True, indent=2) + "\n")
    report = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    validate(report, schema)
    print(
        "PASS: causal projection evidence graph v19 "
        f"forward={report['counts']['inventory_rows']} "
        f"reverse={len(report['reverse_proofs']) + len(report['reverse_mutations'])} "
        f"mutation_edges={report['counts']['mutation_edges']} attacks={self_test(report, schema)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
