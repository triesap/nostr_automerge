#!/usr/bin/env python3
"""Validate the final semantic-proof and finding-closure catalogs."""

from __future__ import annotations

import copy
import hashlib
import json
import sys
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True

import generate_semantic_proof_catalog_final_v10 as generator
import validate_semantic_proof_catalog_v10 as authority


ROOT = Path(__file__).resolve().parents[1]
CATALOG = "reports/semantic_proof_catalog_v10.json"
FINDINGS = "reports/finding_closure_catalog_v10.json"
CATALOG_SHA256 = "48f27ffff08756b7567c83fe3025efd4aac5cc0da9c4c2055d5cc8373168574a"
FINDINGS_SHA256 = "b4fabc6486c78aa745548d269cc0119a1668e90b5cd3a13dd8c266be3e2d7e29"
GENERATOR_SHA256 = "be8613f80ef4bf9db50a75633f4fb3004ca7fa335bfa7a939523a1877f553dac"
CATALOG_IDENTITY = "0357f8f558f22096611bf08d197e0e46b30cd53618ea40ac49d2d057d1931c82"
FINDINGS_IDENTITY = "0eb24b686f6ac30ff308981822d490525574a95e0f4cd7f9e752c191efe1a10d"


class FinalCatalogError(ValueError):
    """One final semantic-proof catalog invariant failed."""


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise FinalCatalogError(diagnostic)


def load(relative: str) -> dict[str, Any]:
    value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    require(isinstance(value, dict), f"object:{relative}")
    return value


def digest(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def identity(value: dict[str, Any]) -> str:
    body = {key: item for key, item in value.items() if key != "result_identity_sha256"}
    return hashlib.sha256(json.dumps(body, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def validate(catalog: dict[str, Any], findings: dict[str, Any], *, bind_files: bool = True) -> None:
    expected_catalog, expected_findings = generator.build()
    require(catalog == expected_catalog, "catalog:projection")
    require(findings == expected_findings, "findings:projection")
    require(tuple(catalog) == ("authority_candidate", "finding_count", "protocol_revision", "report_clause_count", "requirement_count", "result_identity_sha256", "rows", "schema", "status"), "catalog:keys")
    require(tuple(findings) == ("candidate", "finding_count", "result_identity_sha256", "rows", "schema"), "findings:keys")
    rows = catalog["rows"]
    require(len(rows) == 190, "catalog:count")
    require([row["subject_kind"] for row in rows] == ["requirement"] * 148 + ["report_clause"] * 21 + ["finding"] * 21, "catalog:subjects")
    require(sum(row["status"] == "pass" for row in rows) == 165, "catalog:pass_count")
    require(sum(row["status"] == "held" for row in rows) == 25, "catalog:held_count")
    for index, row in enumerate(rows):
        require(tuple(row) == ("applicability", "id", "opaque_proofs", "public_proofs", "semantic_category", "status", "subject_kind"), f"row:keys:{index}")
        for item in (*row["public_proofs"], *row["opaque_proofs"]):
            require(tuple(item) == ("artifact_sha256", "candidate", "id", "kind", "result"), f"proof:keys:{index}")
            require(len(item["candidate"]) == 40 and len(item["artifact_sha256"]) == 64, f"proof:identity:{index}")
            require(item["result"] == row["status"], f"proof:result:{index}")
        require((not row["public_proofs"] and not row["opaque_proofs"]) == (row["subject_kind"] == "requirement" and row["status"] == "held"), f"proof:presence:{index}")
    require(identity(catalog) == CATALOG_IDENTITY == catalog["result_identity_sha256"], "catalog:identity")
    require(identity(findings) == FINDINGS_IDENTITY == findings["result_identity_sha256"], "findings:identity")
    require(findings["rows"] == [
        {
            "id": row["id"],
            "semantic_category": row["semantic_category"],
            "status": row["status"],
            "closure_proofs": [item["id"] for item in (*row["public_proofs"], *row["opaque_proofs"])],
        }
        for row in rows[-21:]
    ], "findings:cross_binding")
    catalog_schema = authority.load(authority.CATALOG_SCHEMA_PATH)
    finding_schema = authority.load(authority.FINDING_SCHEMA_PATH)
    authority.validate(authority.load(authority.AUTHORITY_PATH), catalog_schema, finding_schema)
    if bind_files:
        require(digest(CATALOG) == CATALOG_SHA256, "catalog:file")
        require(digest(FINDINGS) == FINDINGS_SHA256, "findings:file")
        require(digest("scripts/generate_semantic_proof_catalog_final_v10.py") == GENERATOR_SHA256, "generator:file")


def mutation_self_test(catalog: dict[str, Any], findings: dict[str, Any]) -> int:
    mutations: list[tuple[str, dict[str, Any], dict[str, Any]]] = []
    passing_requirement = next(
        index
        for index, row in enumerate(catalog["rows"][:148])
        if row["status"] == "pass"
    )
    for offset, name in ((0, "requirement"), (148, "clause"), (169, "finding")):
        missing = copy.deepcopy(catalog); missing["rows"].pop(offset); mutations.append((f"missing:{name}", missing, findings))
        reordered = copy.deepcopy(catalog); reordered["rows"][offset:] = reversed(reordered["rows"][offset:]); mutations.append((f"reordered:{name}", reordered, findings))
        duplicate = copy.deepcopy(catalog); duplicate["rows"][offset] = duplicate["rows"][offset + 1]; mutations.append((f"duplicate:{name}", duplicate, findings))
    wrong_category = copy.deepcopy(catalog); wrong_category["rows"][passing_requirement]["semantic_category"] = "external_hold"; mutations.append(("category", wrong_category, findings))
    wrong_applicability = copy.deepcopy(catalog); wrong_applicability["rows"][passing_requirement]["applicability"] = "opaque_only"; mutations.append(("applicability", wrong_applicability, findings))
    wrong_status = copy.deepcopy(catalog); wrong_status["rows"][passing_requirement]["status"] = "held"; mutations.append(("status", wrong_status, findings))
    proof_candidate = copy.deepcopy(catalog); proof_candidate["rows"][passing_requirement]["public_proofs"][0]["candidate"] = "0" * 40; mutations.append(("proof_candidate", proof_candidate, findings))
    proof_artifact = copy.deepcopy(catalog); proof_artifact["rows"][passing_requirement]["public_proofs"][0]["artifact_sha256"] = "0" * 64; mutations.append(("proof_artifact", proof_artifact, findings))
    proof_result = copy.deepcopy(catalog); proof_result["rows"][passing_requirement]["public_proofs"][0]["result"] = "held"; mutations.append(("proof_result", proof_result, findings))
    proof_id = copy.deepcopy(catalog); proof_id["rows"][passing_requirement]["public_proofs"][0]["id"] = "generic check"; mutations.append(("proof_id", proof_id, findings))
    extra = copy.deepcopy(catalog); extra["unapproved"] = False; mutations.append(("catalog_extra", extra, findings))
    identity_drift = copy.deepcopy(catalog); identity_drift["result_identity_sha256"] = "f" * 64; mutations.append(("catalog_identity", identity_drift, findings))
    finding_missing = copy.deepcopy(findings); finding_missing["rows"].pop(); mutations.append(("finding_missing", catalog, finding_missing))
    finding_proof = copy.deepcopy(findings); finding_proof["rows"][0]["closure_proofs"] = ["generic check"]; mutations.append(("finding_proof", catalog, finding_proof))
    finding_identity = copy.deepcopy(findings); finding_identity["result_identity_sha256"] = "f" * 64; mutations.append(("finding_identity", catalog, finding_identity))
    caught = 0
    for name, changed_catalog, changed_findings in mutations:
        try:
            validate(changed_catalog, changed_findings, bind_files=False)
        except FinalCatalogError:
            caught += 1
            continue
        raise FinalCatalogError(f"mutation_survived:{name}")
    require(caught == 21, "mutation_count")
    return caught


def main() -> int:
    catalog = load(CATALOG)
    findings = load(FINDINGS)
    validate(catalog, findings)
    mutations = mutation_self_test(catalog, findings)
    print("PASS: final semantic proof catalogs v10")
    print("- rows=190")
    print("- requirements=148")
    print("- report_clauses=21")
    print("- findings=21")
    print("- passing=165")
    print("- held=25")
    print(f"- negative_mutations={mutations}")
    print(f"- catalog_identity_sha256={CATALOG_IDENTITY}")
    print(f"- finding_identity_sha256={FINDINGS_IDENTITY}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except FinalCatalogError as error:
        raise SystemExit(f"FAIL: {error}") from error
