#!/usr/bin/env python3
"""Generate and validate later-bound v18 proof and mutation catalogs."""

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
REPORT = ROOT / "reports/causal_projection_catalogs_v18.json"
SCHEMA = ROOT / "tools/validation/causal_projection_catalogs_v18.schema.json"
AUTHORITY = "spec/causal_projection_contracts_v18.json"
PROOF_PATH = "reports/causal_projection_proofs_v18.json"
MUTATION_PATH = "reports/causal_projection_mutations_v18.json"
SOURCE_CANDIDATE = "076221ad7f03e67d89ac4b2fcfc8f2586b97f182"
PROOF_ARTIFACT_COMMIT = "9dda56c11e7f2376a21b0ad8c7b02105e3c9a444"
MUTATION_ARTIFACT_COMMIT = "3e101da1c0cabb6a2c5dd99279e8c3cf9f8eb0d7"
TOP_FIELDS = [
    "schema", "status", "authority", "source_candidate",
    "proof_artifact_commit", "mutation_artifact_commit", "proof_catalog",
    "mutation_catalog", "candidate_order", "counts", "binding",
    "result_identity_sha256", "result",
]
PROOF_FIELDS = [
    "id", "inventory_row_id", "trace_artifact", "trace_sha256",
    "artifact_commit", "result",
]
MUTATION_FIELDS = [
    "id", "inventory_row_id", "patch_artifact", "patch_sha256",
    "transcript_artifact", "transcript_sha256", "artifact_commit", "result",
]


class CatalogError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise CatalogError(code)


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


def proof_catalog(report: dict[str, Any]) -> dict[str, Any]:
    rows = [
        {
            "id": "catalog." + row["proof_row_id"],
            "inventory_row_id": row["inventory_row_id"],
            "trace_artifact": row["trace_artifact"],
            "trace_sha256": row["trace_sha256"],
            "artifact_commit": PROOF_ARTIFACT_COMMIT,
            "result": "pass",
        }
        for row in report["rows"]
    ]
    return {
        "report_path": PROOF_PATH,
        "report_sha256": sha(committed(PROOF_ARTIFACT_COMMIT, PROOF_PATH)),
        "artifact_commit": PROOF_ARTIFACT_COMMIT,
        "rows": rows,
        "result": "pass",
    }


def mutation_catalog(report: dict[str, Any]) -> dict[str, Any]:
    rows = [
        {
            "id": "catalog." + row["mutation_id"],
            "inventory_row_id": row["inventory_row_id"],
            "patch_artifact": row["patch_artifact"],
            "patch_sha256": row["patch_sha256"],
            "transcript_artifact": row["transcript_artifact"],
            "transcript_sha256": row["transcript_sha256"],
            "artifact_commit": MUTATION_ARTIFACT_COMMIT,
            "result": "pass",
        }
        for row in report["rows"]
    ]
    return {
        "report_path": MUTATION_PATH,
        "report_sha256": sha(committed(MUTATION_ARTIFACT_COMMIT, MUTATION_PATH)),
        "artifact_commit": MUTATION_ARTIFACT_COMMIT,
        "rows": rows,
        "result": "pass",
    }


def expected_report() -> dict[str, Any]:
    proof_raw = committed(PROOF_ARTIFACT_COMMIT, PROOF_PATH)
    mutation_raw = committed(MUTATION_ARTIFACT_COMMIT, MUTATION_PATH)
    proof, mutation = json.loads(proof_raw), json.loads(mutation_raw)
    require(proof["status"] == mutation["status"] == "actual_execution_raw_unbound", "RAW_STATUS")
    require(proof["source_candidate"] == mutation["source_candidate"] == SOURCE_CANDIDATE, "RAW_SOURCE")
    proofs, mutations = proof_catalog(proof), mutation_catalog(mutation)
    report: dict[str, Any] = {
        "schema": "nostr_automerge.causal_projection_catalogs.v18.v1",
        "status": "committed_later_binding",
        "authority": AUTHORITY,
        "source_candidate": SOURCE_CANDIDATE,
        "proof_artifact_commit": PROOF_ARTIFACT_COMMIT,
        "mutation_artifact_commit": MUTATION_ARTIFACT_COMMIT,
        "proof_catalog": proofs,
        "mutation_catalog": mutations,
        "candidate_order": [
            SOURCE_CANDIDATE,
            proof["execution_base_candidate"],
            PROOF_ARTIFACT_COMMIT,
            mutation["execution_base_candidate"],
            MUTATION_ARTIFACT_COMMIT,
        ],
        "counts": {
            "proof_rows": len(proofs["rows"]),
            "mutation_rows": len(mutations["rows"]),
            "unbound_rows": 0,
        },
        "binding": {
            "mode": "later_catalog",
            "raw_artifact_self_reference": False,
            "catalog_self_reference": False,
        },
        "result_identity_sha256": "",
        "result": "pass",
    }
    identity = {key: value for key, value in report.items() if key != "result_identity_sha256"}
    report["result_identity_sha256"] = sha(canonical(identity))
    return report


def validate(report: Any, schema: Any) -> None:
    expected = expected_report()
    require(type(report) is dict and list(report) == TOP_FIELDS and report == expected, "REPORT_DERIVATION")
    proofs = report["proof_catalog"]["rows"]
    mutations = report["mutation_catalog"]["rows"]
    require(len(proofs) == len({row["id"] for row in proofs}) == len({row["inventory_row_id"] for row in proofs}), "PROOF_UNIQUE")
    require(len(mutations) == len({row["id"] for row in mutations}), "MUTATION_UNIQUE")
    require(all(list(row) == PROOF_FIELDS for row in proofs), "PROOF_ROW_SHAPE")
    require(all(list(row) == MUTATION_FIELDS for row in mutations), "MUTATION_ROW_SHAPE")
    for candidate in report["candidate_order"]:
        resolved = subprocess.run(
            ["git", "rev-parse", f"{candidate}^{{commit}}"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        require(resolved.returncode == 0 and resolved.stdout.strip() == candidate, "CANDIDATE")
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
    for name, fields in (("proof_catalog", PROOF_FIELDS), ("mutation_catalog", MUTATION_FIELDS)):
        row_schema = schema["properties"][name]["properties"]["rows"]
        require(row_schema.get("minItems") == 1 and "maxItems" not in row_schema, "SCHEMA_SOURCE_DERIVED")
        require(row_schema["items"].get("required") == fields and row_schema["items"].get("additionalProperties") is False, "SCHEMA_ROW_CLOSED")


def self_test(report: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks = [
        ("report", lambda value: value["proof_catalog"]["rows"].pop()),
        ("report", lambda value: value["mutation_catalog"]["rows"].pop()),
        ("report", lambda value: value["proof_catalog"]["rows"][0].update(artifact_commit=MUTATION_ARTIFACT_COMMIT)),
        ("report", lambda value: value["mutation_catalog"]["rows"][0].update(transcript_sha256="0" * 64)),
        ("report", lambda value: value["candidate_order"].reverse()),
        ("report", lambda value: value["binding"].update(raw_artifact_self_reference=True)),
        ("schema", lambda value: value.update(additionalProperties=True)),
    ]
    caught = 0
    for target, mutate in attacks:
        changed_report, changed_schema = copy.deepcopy(report), copy.deepcopy(schema)
        mutate(changed_report if target == "report" else changed_schema)
        try:
            validate(changed_report, changed_schema)
        except CatalogError:
            caught += 1
            continue
        raise CatalogError("ATTACK_SURVIVED")
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
        "PASS: causal projection catalogs v18 "
        f"proofs={report['counts']['proof_rows']} mutations={report['counts']['mutation_rows']} "
        f"attacks={self_test(report, schema)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
