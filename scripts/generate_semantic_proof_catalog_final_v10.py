#!/usr/bin/env python3
"""Generate the final public semantic-proof and finding-closure catalogs."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True

import validate_opaque_semantic_proofs_v10 as opaque
import validate_report_finding_proofs_v10 as public_subjects
import validate_rust_requirement_proofs_v10 as public_requirements


ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "reports/semantic_proof_catalog_v10.json"
FINDINGS = ROOT / "reports/finding_closure_catalog_v10.json"
AUTHORITY_CANDIDATE = "ebf8d1ecc75cf5eee2741ec61b80f0dbe5283df5"
RUST_PROOF_CANDIDATE = public_requirements.SOURCE_CANDIDATE
SUBJECT_PROOF_CANDIDATE = public_subjects.SOURCE_CANDIDATE
OPAQUE_PROOF_CANDIDATE = "b812328043cd514ad8909c0f926435005d27d1fd"
RUST_PROOF_ARTIFACT = "75e66350c10bbcfc382e5eb0f21acff998cfa026b99c5937565ca4dda9d4c462"
SUBJECT_PROOF_ARTIFACT = "a29cf269b3a8a2b0c77c1f937cd9d5bbad931db61655c36fdd7fda4d5835957e"
OPAQUE_PROOF_ARTIFACT = opaque.REPORT_SHA256
OPAQUE_APPLICABILITY = {
    "rust-and-typescript",
    "rust-only-evidence-with-opaque-typescript-overlay",
}


class GenerationError(ValueError):
    """One final catalog generation invariant failed."""


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise GenerationError(diagnostic)


def canonical(value: Any) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def with_identity(value: dict[str, Any]) -> dict[str, Any]:
    return {
        **value,
        "result_identity_sha256": hashlib.sha256(canonical(value)).hexdigest(),
    }


def proof(kind: str, identifier: str, candidate: str, artifact: str, result: str) -> dict[str, Any]:
    return {
        "kind": kind,
        "id": identifier,
        "candidate": candidate,
        "artifact_sha256": artifact,
        "result": result,
    }


def public_requirement_proof(identifier: str) -> dict[str, Any]:
    kind = identifier.split(":", 1)[0]
    require(kind in {"rust_test", "signed_fixture", "validator"}, "requirement:proof_kind")
    return proof(kind, identifier, RUST_PROOF_CANDIDATE, RUST_PROOF_ARTIFACT, "pass")


def public_subject_proof(identifier: str, result: str) -> dict[str, Any]:
    kind = identifier.split(":", 1)[0]
    require(kind in {"rust_test", "validator", "hold_record"}, "subject:proof_kind")
    return proof(kind, identifier, SUBJECT_PROOF_CANDIDATE, SUBJECT_PROOF_ARTIFACT, result)


def opaque_proof(identifier: str, result: str) -> dict[str, Any]:
    kind = identifier.split("_", 2)[1]
    mapped = {"fixture": "opaque_fixture", "test": "opaque_test", "hold": "hold_record"}
    require(kind in mapped, "opaque:proof_kind")
    return proof(mapped[kind], identifier, OPAQUE_PROOF_CANDIDATE, OPAQUE_PROOF_ARTIFACT, result)


def build() -> tuple[dict[str, Any], dict[str, Any]]:
    requirement_rows, requirement_proofs = public_requirements.build_rows()
    clauses = public_subjects.clause_rows()
    findings = public_subjects.finding_rows(public_subjects.FINDING_BINDINGS)
    opaque_report = opaque.load(opaque.REPORT)
    opaque_schema = opaque.load(opaque.SCHEMA)
    public_requirements.validate(requirement_rows, requirement_proofs)
    public_subjects.validate(clauses, findings, public_subjects.FINDING_BINDINGS)
    opaque.validate(opaque_report, opaque_schema)
    opaque_requirements = {row["id"]: row for row in opaque_report["requirements"]}
    opaque_clauses = {row["id"]: row for row in opaque_report["report_clauses"]}
    opaque_findings = {row["id"]: row for row in opaque_report["findings"]}
    rows: list[dict[str, Any]] = []
    for row in requirement_rows:
        held = row["status"] == "held"
        has_opaque = row["applicability"] in OPAQUE_APPLICABILITY
        opaque_ids = opaque_requirements.get(row["id"], {}).get("proof_ids", [])
        require(bool(opaque_ids) == has_opaque, f"requirement:opaque:{row['id']}")
        rows.append(
            {
                "subject_kind": "requirement",
                "id": row["id"],
                "semantic_category": row["semantic_category"],
                "applicability": "external_hold" if held else ("rust_and_opaque" if has_opaque else "rust_only"),
                "status": row["status"],
                "public_proofs": [public_requirement_proof(item) for item in row["rust_proof_ids"]],
                "opaque_proofs": [opaque_proof(item, "pass") for item in opaque_ids],
            }
        )
    for row in clauses:
        opaque_ids = opaque_clauses[row["id"]]["proof_ids"]
        rows.append(
            {
                "subject_kind": "report_clause",
                "id": row["id"],
                "semantic_category": "report_contract",
                "applicability": "rust_and_opaque",
                "status": "pass",
                "public_proofs": [public_subject_proof(item, "pass") for item in row["closure_proofs"]],
                "opaque_proofs": [opaque_proof(item, "pass") for item in opaque_ids],
            }
        )
    for row in findings:
        held = row["status"] == "held"
        opaque_ids = opaque_findings[row["id"]]["proof_ids"]
        result = "held" if held else "pass"
        rows.append(
            {
                "subject_kind": "finding",
                "id": row["id"],
                "semantic_category": row["semantic_category"],
                "applicability": "external_hold" if held else "rust_and_opaque",
                "status": result,
                "public_proofs": [public_subject_proof(item, result) for item in row["closure_proofs"]],
                "opaque_proofs": [opaque_proof(item, result) for item in opaque_ids],
            }
        )
    require(len(rows) == 190, "catalog:rows")
    catalog = with_identity(
        {
            "schema": "nostr_automerge.semantic_proof_catalog.v10.v1",
            "status": "pass",
            "protocol_revision": "draft_2026_08",
            "authority_candidate": AUTHORITY_CANDIDATE,
            "requirement_count": 148,
            "report_clause_count": 21,
            "finding_count": 21,
            "rows": rows,
        }
    )
    finding_catalog = with_identity(
        {
            "schema": "nostr_automerge.finding_closure_catalog.v10.v1",
            "candidate": AUTHORITY_CANDIDATE,
            "finding_count": 21,
            "rows": [
                {
                    "id": row["id"],
                    "semantic_category": row["semantic_category"],
                    "status": row["status"],
                    "closure_proofs": [
                        item["id"] for item in (*row["public_proofs"], *row["opaque_proofs"])
                    ],
                }
                for row in rows[-21:]
            ],
        }
    )
    return catalog, finding_catalog


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    arguments = parser.parse_args()
    catalog, findings = build()
    outputs = ((CATALOG, canonical(catalog) + b"\n"), (FINDINGS, canonical(findings) + b"\n"))
    for path, encoded in outputs:
        if arguments.check:
            require(path.read_bytes() == encoded, f"stale:{path.name}")
        else:
            path.write_bytes(encoded)
    print("PASS: final semantic proof catalog generation v10" if arguments.check else "WROTE: final semantic proof catalogs v10")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except GenerationError as error:
        raise SystemExit(f"FAIL: {error}") from error
