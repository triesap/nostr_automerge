#!/usr/bin/env python3
"""Generate and validate the source-derived final v17 Rust inventory."""

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
PROOF_PATH = "reports/causal_projection_proofs_v17.json"
MUTATION_PATH = "reports/causal_projection_mutations_v17.json"
REPORT = ROOT / "reports/causal_projection_final_inventory_v17.json"
SCHEMA = ROOT / "tools/validation/causal_projection_final_inventory_v17.schema.json"
SOURCE_CANDIDATE = "789eae3c6e0994f71420f49fe51fe3ab7cb75ca9"
PROOF_CANDIDATE = "12f824659e055354779bb65b99f475c2ec109c43"
MUTATION_CANDIDATE = "eb760b20499792364624f24990deb35a3e8f54dd"
ROW_FIELDS = [
    "id", "abstract_family", "phase", "language", "applicability",
    "source_path", "source_symbol", "source_site_id", "source_site_description",
    "owner_mode", "counter", "abstract_owner_class", "reachability_artifact",
    "proof_row_id", "proof_artifact_sha256", "mutation_coverage_id",
    "source_candidate", "proof_candidate", "mutation_candidate", "result",
]
TOP_FIELDS = [
    "schema", "status", "authority", "row_contract", "rows", "counts",
    "source_candidate", "proof_candidate", "mutation_candidate",
    "source_sha256", "proof_report_sha256", "mutation_report_sha256",
    "self_candidate", "result_identity_sha256", "result",
]

sys.path.insert(0, str(ROOT / "scripts"))
from validate_causal_projection_inventory_v17 import derive_rows, production, snake  # noqa: E402


class FinalInventoryError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise FinalInventoryError(code)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True).encode()


def committed(candidate: str, path: str) -> bytes:
    result = subprocess.run(["git", "show", f"{candidate}:{path}"], cwd=ROOT, capture_output=True, check=False)
    require(result.returncode == 0, f"CANDIDATE:{path}")
    return result.stdout


def expected_report() -> dict[str, Any]:
    source = committed(SOURCE_CANDIDATE, SOURCE_PATH)
    proof_bytes = committed(PROOF_CANDIDATE, PROOF_PATH)
    mutation_bytes = committed(MUTATION_CANDIDATE, MUTATION_PATH)
    source_rows = derive_rows(production(source.decode()))
    proofs = {row["inventory_row_id"]: row for row in json.loads(proof_bytes)["rows"]}
    coverage = {row["inventory_row_ids"][0]: row for row in json.loads(mutation_bytes)["coverage_records"]}
    rows = []
    for source_row in source_rows:
        row_id = source_row["id"]
        require(row_id in proofs and row_id in coverage, f"EVIDENCE_ROW:{row_id}")
        proof, mutation = proofs[row_id], coverage[row_id]
        rows.append({
            "id": row_id,
            "abstract_family": f"{source_row['phase']}.{snake(source_row['operation'])}",
            "phase": source_row["phase"], "language": "rust", "applicability": "required",
            "source_path": source_row["source_path"], "source_symbol": source_row["source_symbol"],
            "source_site_id": source_row["site_id"],
            "source_site_description": f"{source_row['phase']}:{source_row['operation']}",
            "owner_mode": source_row["owner_mode"], "counter": source_row["counter"],
            "abstract_owner_class": source_row["abstract_owner_class"],
            "reachability_artifact": source_row["reachability_artifact"],
            "proof_row_id": proof["proof_row_id"], "proof_artifact_sha256": proof["transcript_sha256"],
            "mutation_coverage_id": mutation["coverage_id"],
            "source_candidate": SOURCE_CANDIDATE, "proof_candidate": PROOF_CANDIDATE,
            "mutation_candidate": MUTATION_CANDIDATE, "result": "pass",
        })
    report: dict[str, Any] = {
        "schema": "nostr_automerge.causal_projection_final_inventory.v17.v1",
        "status": "final", "authority": "spec/causal_projection_contracts_v17.json",
        "row_contract": ROW_FIELDS, "rows": rows,
        "counts": {"rows": len(rows), "proofs": len(proofs), "coverage": len(coverage), "planned_values": 0},
        "source_candidate": SOURCE_CANDIDATE, "proof_candidate": PROOF_CANDIDATE,
        "mutation_candidate": MUTATION_CANDIDATE,
        "source_sha256": sha(source), "proof_report_sha256": sha(proof_bytes),
        "mutation_report_sha256": sha(mutation_bytes), "self_candidate": None,
        "result_identity_sha256": "", "result": "pass",
    }
    report["result_identity_sha256"] = sha(canonical({key: value for key, value in report.items() if key != "result_identity_sha256"}))
    return report


def validate(report: object, schema: object) -> None:
    expected = expected_report()
    require(type(report) is dict and list(report) == TOP_FIELDS and report == expected, "REPORT_DERIVATION")
    rows = report["rows"]
    require(report["status"] == "final" and report["self_candidate"] is None, "FINAL_STATUS")
    require(len(rows) == len({row["id"] for row in rows}) == len({row["source_site_id"] for row in rows}) == 68, "ROW_UNIQUE")
    require(all(list(row) == ROW_FIELDS and row["result"] == "pass" for row in rows), "ROW_SHAPE")
    require(not any(isinstance(value, str) and value.startswith("planned:") for row in rows for value in row.values()), "PLANNED_VALUE")
    require(schema.get("additionalProperties") is False and schema.get("required") == TOP_FIELDS, "SCHEMA_CLOSED")
    require(schema["properties"]["rows"].get("minItems") == schema["properties"]["rows"].get("maxItems") == 68, "SCHEMA_ROWS")


def self_test(report: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks = [
        lambda value: value["rows"].pop(),
        lambda value: value["rows"][0].update(proof_candidate="0" * 40),
        lambda value: value["rows"][0].update(mutation_coverage_id="missing"),
        lambda value: value["rows"][0].update(proof_row_id="planned:proof"),
        lambda value: value.update(self_candidate=MUTATION_CANDIDATE),
        lambda value: value["rows"].reverse(),
        lambda value: value.update(result_identity_sha256="0" * 64),
    ]
    caught = 0
    for attack in attacks:
        changed = copy.deepcopy(report); attack(changed)
        try:
            validate(changed, schema)
        except FinalInventoryError:
            caught += 1
            continue
        raise FinalInventoryError("ATTACK_SURVIVED")
    return caught


def main() -> int:
    parser = argparse.ArgumentParser(); parser.add_argument("--write-report", action="store_true"); args = parser.parse_args()
    expected = expected_report()
    if args.write_report:
        REPORT.write_text(json.dumps(expected, ensure_ascii=True, indent=2) + "\n")
    report = json.loads(REPORT.read_text()); schema = json.loads(SCHEMA.read_text())
    validate(report, schema); attacks = self_test(report, schema)
    print(f"PASS: causal projection final inventory v17 rows=68 planned=0 self_candidate=none attacks={attacks}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
