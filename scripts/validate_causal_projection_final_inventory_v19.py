#!/usr/bin/env python3
"""Generate and validate the source-derived final v19 Rust inventory."""

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
sys.path.insert(0, str(ROOT / "scripts"))
import validate_causal_projection_inventory_v18 as discovery  # noqa: E402

SOURCE_PATH = "crates/nostr_automerge/src/graph/actor_state.rs"
CATALOG_PATH = "reports/causal_projection_catalogs_v19.json"
REPORT = ROOT / "reports/causal_projection_final_inventory_v19.json"
SCHEMA = ROOT / "tools/validation/causal_projection_final_inventory_v19.schema.json"
AUTHORITY = "spec/causal_projection_contracts_v19.json"
SOURCE_CANDIDATE = "a8f8651c26a0ac7822e1e5d61f3aafa3fec53d1f"
CATALOG_CANDIDATE = "266e80b67e13ea377ea874bbd4b75ac4ca5898fa"
ROW_FIELDS = [
    "id", "phase", "family", "language", "applicability", "source_path",
    "source_symbol", "site_id", "counter", "owner_mode",
    "abstract_owner_class", "reachability_sha256", "proof_catalog_id",
    "proof_trace_sha256", "mutation_coverage_ids", "source_candidate",
    "catalog_candidate", "result",
]
TOP_FIELDS = [
    "schema", "status", "authority", "row_contract", "rows", "counts",
    "candidate_roles", "source_sha256", "catalog_sha256", "self_candidate",
    "result_identity_sha256", "result",
]


class FinalInventoryError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise FinalInventoryError(code)


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
    source = committed(SOURCE_CANDIDATE, SOURCE_PATH)
    catalog_raw = committed(CATALOG_CANDIDATE, CATALOG_PATH)
    catalog = json.loads(catalog_raw)
    discovery.SOURCE_CANDIDATE = SOURCE_CANDIDATE
    source_rows = discovery.derive_rows(discovery.production(source.decode()))
    proofs = {row["inventory_row_id"]: row for row in catalog["proof_catalog"]["rows"]}
    mutations = catalog["mutation_catalog"]["rows"]
    helper_by_phase = {
        "projection_construction": "perform_projection_build_operation",
        "actor_sequence": "perform_actor_decision_operation",
        "causal_counter": "perform_causal_next_operation",
        "frontier_comparison": "metered_frontier_operation",
    }
    rows = []
    for source_row in source_rows:
        row_id = source_row["id"]
        proof = proofs.get(row_id)
        require(proof is not None, "PROOF_COVERAGE:" + row_id)
        helper = [
            row["id"] for row in mutations
            if row["campaign"] == "helper"
            and row["helper"] == helper_by_phase[source_row["phase"]]
        ]
        local = [row["id"] for row in mutations if row["campaign"] != "helper" and row["inventory_row_id"] == row_id]
        coverage = helper + local
        require(len(helper) == 7, "HELPER_MATRIX_COVERAGE:" + row_id)
        require(coverage, "MUTATION_COVERAGE:" + row_id)
        rows.append({
            "id": row_id,
            "phase": source_row["phase"],
            "family": source_row["family"],
            "language": "rust",
            "applicability": "required",
            "source_path": source_row["source_path"],
            "source_symbol": source_row["source_symbol"],
            "site_id": source_row["site_id"],
            "counter": source_row["counter"],
            "owner_mode": source_row["owner_mode"],
            "abstract_owner_class": source_row["abstract_owner_class"],
            "reachability_sha256": source_row["reachability_sha256"],
            "proof_catalog_id": proof["id"],
            "proof_trace_sha256": proof["trace_sha256"],
            "mutation_coverage_ids": coverage,
            "source_candidate": SOURCE_CANDIDATE,
            "catalog_candidate": CATALOG_CANDIDATE,
            "result": "pass",
        })
    report: dict[str, Any] = {
        "schema": "nostr_automerge.causal_projection_final_inventory.v19.v1",
        "status": "final_source_derived",
        "authority": AUTHORITY,
        "row_contract": ROW_FIELDS,
        "rows": rows,
        "counts": {
            "rows": len(rows),
            "proofs": len(proofs),
            "mutation_catalog_rows": len(mutations),
            "helper_matrix_cells": sum(row["campaign"] == "helper" for row in mutations),
            "covered_rows": sum(bool(row["mutation_coverage_ids"]) for row in rows),
            "planned_values": 0,
        },
        "candidate_roles": {
            "source_candidate": SOURCE_CANDIDATE,
            "catalog_candidate": CATALOG_CANDIDATE,
        },
        "source_sha256": sha(source),
        "catalog_sha256": sha(catalog_raw),
        "self_candidate": None,
        "result_identity_sha256": "",
        "result": "pass",
    }
    identity = {key: value for key, value in report.items() if key != "result_identity_sha256"}
    report["result_identity_sha256"] = sha(canonical(identity))
    return report


def validate(report: Any, schema: Any) -> None:
    expected = expected_report()
    require(type(report) is dict and list(report) == TOP_FIELDS and report == expected, "REPORT_DERIVATION")
    rows = report["rows"]
    require(len(rows) == len({row["id"] for row in rows}) == len({row["site_id"] for row in rows}), "ROW_UNIQUE")
    require(all(list(row) == ROW_FIELDS for row in rows), "ROW_SHAPE")
    require(all(row["proof_catalog_id"] and len(row["mutation_coverage_ids"]) >= 7 for row in rows), "EVIDENCE_COVERAGE")
    catalog = json.loads(committed(CATALOG_CANDIDATE, CATALOG_PATH))
    mutation_ids = {row["id"] for row in catalog["mutation_catalog"]["rows"]}
    covered_mutations = {item for row in rows for item in row["mutation_coverage_ids"]}
    require(covered_mutations == mutation_ids, "MUTATION_BIDIRECTIONAL_COVERAGE")
    require(not any(isinstance(value, str) and value.startswith("planned:") for row in rows for value in row.values()), "PLANNED_VALUE")
    require(report["self_candidate"] is None, "SELF_REFERENCE")
    require(subprocess.run(
        ["git", "merge-base", "--is-ancestor", SOURCE_CANDIDATE, CATALOG_CANDIDATE],
        cwd=ROOT, check=False,
    ).returncode == 0, "CANDIDATE_ORDER")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == TOP_FIELDS, "SCHEMA_CLOSED")
    row_schema = schema["properties"]["rows"]
    require(row_schema.get("minItems") == 1 and "maxItems" not in row_schema, "SCHEMA_SOURCE_DERIVED")
    require(row_schema["items"].get("required") == ROW_FIELDS and row_schema["items"].get("additionalProperties") is False, "SCHEMA_ROW_CLOSED")


def self_test(report: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks = [
        ("report", lambda value: value["rows"].pop()),
        ("report", lambda value: value["rows"][0].update(proof_catalog_id="missing")),
        ("report", lambda value: value["rows"][0].update(mutation_coverage_ids=[])),
        ("report", lambda value: value["rows"][0]["mutation_coverage_ids"].pop()),
        ("report", lambda value: value["candidate_roles"].update(catalog_candidate="0" * 40)),
        ("report", lambda value: value.update(self_candidate=CATALOG_CANDIDATE)),
        ("report", lambda value: value["rows"].reverse()),
        ("schema", lambda value: value.update(additionalProperties=True)),
    ]
    caught = 0
    for target, mutate in attacks:
        changed_report, changed_schema = copy.deepcopy(report), copy.deepcopy(schema)
        mutate(changed_report if target == "report" else changed_schema)
        try:
            validate(changed_report, changed_schema)
        except FinalInventoryError:
            caught += 1
            continue
        raise FinalInventoryError("ATTACK_SURVIVED")
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
        "PASS: causal projection final inventory v19 "
        f"rows={report['counts']['rows']} mutations={report['counts']['mutation_catalog_rows']} "
        f"planned=0 attacks={self_test(report, schema)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
