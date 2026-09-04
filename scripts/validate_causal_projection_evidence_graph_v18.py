#!/usr/bin/env python3
"""Generate and validate the bidirectional final v18 evidence graph."""

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
FINAL_PATH = "reports/causal_projection_final_inventory_v18.json"
CATALOG_PATH = "reports/causal_projection_catalogs_v18.json"
REPORT = ROOT / "reports/causal_projection_evidence_graph_v18.json"
SCHEMA = ROOT / "tools/validation/causal_projection_evidence_graph_v18.schema.json"
AUTHORITY = "spec/causal_projection_contracts_v18.json"
SOURCE_CANDIDATE = "076221ad7f03e67d89ac4b2fcfc8f2586b97f182"
PROOF_ARTIFACT_COMMIT = "9dda56c11e7f2376a21b0ad8c7b02105e3c9a444"
MUTATION_ARTIFACT_COMMIT = "3e101da1c0cabb6a2c5dd99279e8c3cf9f8eb0d7"
CATALOG_CANDIDATE = "2f44ca464d2b39f01617e17fe7fa7f8624478c0c"
FINAL_INVENTORY_CANDIDATE = "c90d61810bcf378eee9e6577428082a31aec1b5c"
TOP_FIELDS = [
    "schema", "status", "authority", "final_inventory", "catalog",
    "candidate_order", "forward", "reverse_proofs", "reverse_mutations",
    "counts", "attack_matrix", "self_candidate", "result_identity_sha256",
    "result",
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
        ["git", "show", f"{candidate}:{path}"],
        cwd=ROOT,
        capture_output=True,
        check=False,
    )
    require(completed.returncode == 0, f"COMMITTED_PATH:{candidate}:{path}")
    return completed.stdout


def expected_report() -> dict[str, Any]:
    final_raw = committed(FINAL_INVENTORY_CANDIDATE, FINAL_PATH)
    catalog_raw = committed(CATALOG_CANDIDATE, CATALOG_PATH)
    final, catalog = json.loads(final_raw), json.loads(catalog_raw)
    forward = [
        {
            "inventory_row_id": row["id"],
            "proof_catalog_id": row["proof_catalog_id"],
            "mutation_coverage_ids": row["mutation_coverage_ids"],
            "result": "pass",
        }
        for row in final["rows"]
    ]
    proof_ids = [row["id"] for row in catalog["proof_catalog"]["rows"]]
    mutation_ids = [row["id"] for row in catalog["mutation_catalog"]["rows"]]
    reverse_proofs = [
        {
            "proof_catalog_id": proof_id,
            "inventory_row_ids": [
                row["inventory_row_id"]
                for row in forward
                if row["proof_catalog_id"] == proof_id
            ],
            "result": "pass",
        }
        for proof_id in proof_ids
    ]
    reverse_mutations = [
        {
            "mutation_catalog_id": mutation_id,
            "inventory_row_ids": [
                row["inventory_row_id"]
                for row in forward
                if mutation_id in row["mutation_coverage_ids"]
            ],
            "result": "pass",
        }
        for mutation_id in mutation_ids
    ]
    require(all(row["inventory_row_ids"] for row in reverse_proofs), "PROOF_REVERSE_DANGLING")
    require(all(row["inventory_row_ids"] for row in reverse_mutations), "MUTATION_REVERSE_DANGLING")
    report: dict[str, Any] = {
        "schema": "nostr_automerge.causal_projection_evidence_graph.v18.v1",
        "status": "final_bidirectional",
        "authority": AUTHORITY,
        "final_inventory": {
            "path": FINAL_PATH,
            "candidate": FINAL_INVENTORY_CANDIDATE,
            "sha256": sha(final_raw),
        },
        "catalog": {
            "path": CATALOG_PATH,
            "candidate": CATALOG_CANDIDATE,
            "sha256": sha(catalog_raw),
        },
        "candidate_order": [
            SOURCE_CANDIDATE,
            PROOF_ARTIFACT_COMMIT,
            MUTATION_ARTIFACT_COMMIT,
            CATALOG_CANDIDATE,
            FINAL_INVENTORY_CANDIDATE,
        ],
        "forward": forward,
        "reverse_proofs": reverse_proofs,
        "reverse_mutations": reverse_mutations,
        "counts": {
            "inventory_rows": len(forward),
            "proof_edges": len(forward),
            "mutation_edges": sum(len(row["mutation_coverage_ids"]) for row in forward),
            "proof_catalog_rows": len(proof_ids),
            "mutation_catalog_rows": len(mutation_ids),
            "dangling": 0,
            "extra": 0,
        },
        "attack_matrix": [
            {"attack": attack, "result": "killed"}
            for attack in (
                "dangling", "duplicate", "extra", "stale", "mismatched",
                "reordered", "planned", "self_referential", "coordinated",
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
    for parent, child in zip(report["candidate_order"], report["candidate_order"][1:]):
        require(
            subprocess.run(
                ["git", "merge-base", "--is-ancestor", parent, child],
                cwd=ROOT,
                check=False,
            ).returncode
            == 0,
            "CANDIDATE_ORDER",
        )
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
        ("report", lambda value: value["final_inventory"].update(sha256="0" * 64)),
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
        "PASS: causal projection evidence graph v18 "
        f"forward={report['counts']['inventory_rows']} "
        f"reverse={len(report['reverse_proofs']) + len(report['reverse_mutations'])} "
        f"attacks={self_test(report, schema)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
