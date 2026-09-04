#!/usr/bin/env python3
"""Generate and validate the source-derived final v18 Rust inventory."""

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
SOURCE_PATH = "crates/nostr_automerge/src/graph/actor_state.rs"
CATALOG_PATH = "reports/causal_projection_catalogs_v18.json"
REPORT = ROOT / "reports/causal_projection_final_inventory_v18.json"
SCHEMA = ROOT / "tools/validation/causal_projection_final_inventory_v18.schema.json"
AUTHORITY = "spec/causal_projection_contracts_v18.json"
SOURCE_CANDIDATE = "076221ad7f03e67d89ac4b2fcfc8f2586b97f182"
CATALOG_CANDIDATE = "2f44ca464d2b39f01617e17fe7fa7f8624478c0c"
ROW_FIELDS = [
    "id", "phase", "family", "language", "applicability", "source_path",
    "source_symbol", "site_id", "counter", "owner_mode",
    "abstract_owner_class", "reachability_sha256", "proof_catalog_id",
    "proof_trace_sha256", "mutation_coverage_ids", "source_candidate",
    "catalog_candidate", "result",
]
TOP_FIELDS = [
    "schema", "status", "authority", "row_contract", "rows", "counts",
    "source_candidate", "catalog_candidate", "source_sha256",
    "catalog_sha256", "self_candidate", "result_identity_sha256", "result",
]

sys.path.insert(0, str(ROOT / "scripts"))
from validate_causal_projection_inventory_v18 import derive_rows, production  # noqa: E402


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
        ["git", "show", f"{candidate}:{path}"],
        cwd=ROOT,
        capture_output=True,
        check=False,
    )
    require(completed.returncode == 0, f"COMMITTED_PATH:{candidate}:{path}")
    return completed.stdout


def expected_report() -> dict[str, Any]:
    source = committed(SOURCE_CANDIDATE, SOURCE_PATH)
    catalog_raw = committed(CATALOG_CANDIDATE, CATALOG_PATH)
    catalog = json.loads(catalog_raw)
    source_rows = derive_rows(production(source.decode()))
    proofs = {
        row["inventory_row_id"]: row for row in catalog["proof_catalog"]["rows"]
    }
    mutations = catalog["mutation_catalog"]["rows"]
    rows = []
    for source_row in source_rows:
        row_id = source_row["id"]
        proof = proofs.get(row_id)
        require(proof is not None, "PROOF_COVERAGE:" + row_id)
        direct = [
            row["id"]
            for row in mutations
            if row["inventory_row_id"] == row_id
        ]
        helper = [
            row["id"]
            for row in mutations
            if row["id"] == f"catalog.helper.{source_row['phase']}.target_before_charge"
        ]
        coverage = list(dict.fromkeys(direct + helper))
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
        "schema": "nostr_automerge.causal_projection_final_inventory.v18.v1",
        "status": "final_source_derived",
        "authority": AUTHORITY,
        "row_contract": ROW_FIELDS,
        "rows": rows,
        "counts": {
            "rows": len(rows),
            "proofs": len(proofs),
            "mutation_catalog_rows": len(mutations),
            "covered_rows": sum(bool(row["mutation_coverage_ids"]) for row in rows),
            "planned_values": 0,
        },
        "source_candidate": SOURCE_CANDIDATE,
        "catalog_candidate": CATALOG_CANDIDATE,
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
    require(all(row["proof_catalog_id"] and row["mutation_coverage_ids"] for row in rows), "EVIDENCE_COVERAGE")
    require(not any(isinstance(value, str) and value.startswith("planned:") for row in rows for value in row.values()), "PLANNED_VALUE")
    require(report["self_candidate"] is None, "SELF_REFERENCE")
    require(
        subprocess.run(
            ["git", "merge-base", "--is-ancestor", SOURCE_CANDIDATE, CATALOG_CANDIDATE],
            cwd=ROOT,
            check=False,
        ).returncode
        == 0,
        "CANDIDATE_ORDER",
    )
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == TOP_FIELDS, "SCHEMA_CLOSED")
    row_schema = schema["properties"]["rows"]
    require(row_schema.get("minItems") == 1 and "maxItems" not in row_schema, "SCHEMA_SOURCE_DERIVED")
    require(row_schema["items"].get("required") == ROW_FIELDS and row_schema["items"].get("additionalProperties") is False, "SCHEMA_ROW_CLOSED")


def self_test(report: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks = [
        ("report", lambda value: value["rows"].pop()),
        ("report", lambda value: value["rows"][0].update(proof_catalog_id="missing")),
        ("report", lambda value: value["rows"][0].update(mutation_coverage_ids=[])),
        ("report", lambda value: value["rows"][0].update(catalog_candidate="0" * 40)),
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
        "PASS: causal projection final inventory v18 "
        f"rows={report['counts']['rows']} planned=0 attacks={self_test(report, schema)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
