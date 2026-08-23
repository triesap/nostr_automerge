#!/usr/bin/env python3
"""Validate exact opaque fixture and test identifiers without private-path access."""

from __future__ import annotations

import copy
import hashlib
import json
import re
import sys
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True

from validate_report_contract_v9 import EXPECTED_CLAUSES


ROOT = Path(__file__).resolve().parents[1]
REPORT = "reports/opaque_semantic_proofs_v10.json"
SCHEMA = "tools/validation/opaque_semantic_proofs_v10.schema.json"
REPORT_SHA256 = "594b2510b9302ac040efa5a1225e9a07a90fc60045c9db941272f269c83796e2"
SCHEMA_SHA256 = "0a7611451b17ce0888490cfb38da60801c4729088f4ef93ffe7bcac343baacbb"
ROW_PROJECTION_SHA256 = "8af38778aa19c7ecb555ba0494e5aa7853c3c04aa5a120cf29646e901523f807"
RESULT_IDENTITY = "10f3954ebbe75de9f161ab029c2497b4af0bf264322a3d0ec7a78d51346f676c"
PRIVATE_ARTIFACT_SHA256 = "ec1f10d92ab050cd1ab8d8917e85f7b0f0762b7341e3c24f9ff4c3dc9bf66443"
PUBLIC_CANDIDATE = "920c768946a2d33449905a0b0891942fa8fb9afe"
FINDING_IDS = tuple(f"FINDING_{number:03d}" for number in range(73, 94))
APPLICABLE = {"rust-and-typescript", "rust-only-evidence-with-opaque-typescript-overlay"}
PROOF_ID = re.compile(r"^opaque_(?:fixture|test|hold)_[0-9a-f]{64}$")


class OpaqueProofError(ValueError):
    """One opaque semantic-proof invariant failed."""


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise OpaqueProofError(diagnostic)


def digest(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def canonical(value: Any) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def load(relative: str) -> dict[str, Any]:
    value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    require(isinstance(value, dict), f"object:{relative}")
    return value


def row_projection(value: dict[str, Any]) -> str:
    return digest(
        canonical(
            {
                "requirements": value["requirements"],
                "report_clauses": value["report_clauses"],
                "findings": value["findings"],
            }
        )
    )


def result_identity(value: dict[str, Any]) -> str:
    return digest(canonical({key: item for key, item in value.items() if key != "result_identity_sha256"}))


def opaque_fixture(identifier: str) -> str:
    return f"opaque_fixture_{digest(identifier.encode())}"


def validate_closed_schema(value: Any, diagnostic: str) -> None:
    if isinstance(value, dict):
        if value.get("type") == "object":
            require(value.get("additionalProperties") is False, f"{diagnostic}:open")
            properties = value.get("properties")
            required = value.get("required")
            require(isinstance(properties, dict) and isinstance(required, list), f"{diagnostic}:shape")
            require(set(properties) == set(required), f"{diagnostic}:required")
        for key, child in value.items():
            validate_closed_schema(child, f"{diagnostic}:{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            validate_closed_schema(child, f"{diagnostic}:{index}")


def validate(value: dict[str, Any], schema: dict[str, Any], *, bind_files: bool = True) -> None:
    fields = (
        "schema", "status", "checkpoint", "public_candidate",
        "opaque_evidence_candidate", "opaque_implementation_candidate",
        "opaque_record_sha256", "opaque_record_identity_sha256",
        "requirement_count", "fixture_requirement_count", "assertion_requirement_count",
        "report_clause_count", "finding_count", "closed_finding_count",
        "held_finding_count", "requirements", "report_clauses", "findings",
        "result_identity_sha256",
    )
    require(tuple(value) == tuple(sorted(fields)), "report:keys")
    require(value["schema"] == "nostr_automerge.opaque_semantic_proofs.v10.v1", "report:schema")
    require(value["status"] == "pass" and value["checkpoint"] == "step_1279", "report:status")
    require(value["public_candidate"] == PUBLIC_CANDIDATE, "report:public_candidate")
    require(value["opaque_record_sha256"] == PRIVATE_ARTIFACT_SHA256, "report:opaque_record")
    require(value["requirement_count"] == 113, "report:requirement_count")
    require(value["fixture_requirement_count"] == 49, "report:fixture_count")
    require(value["assertion_requirement_count"] == 64, "report:assertion_count")
    require(value["report_clause_count"] == 21 and value["finding_count"] == 21, "report:subject_counts")
    require(value["closed_finding_count"] == 20 and value["held_finding_count"] == 1, "report:finding_counts")
    applicability = load("spec/requirements_applicability.json")["classifications"]
    requirements = load("spec/requirements.json")["requirements"]
    manifest = load("fixtures/distribution/manifest_v10.json")["fixtures"]
    fixture_map: dict[str, list[str]] = {}
    for fixture in manifest:
        for identifier in fixture["requirements"]:
            fixture_map.setdefault(identifier, []).append(fixture["fixture_id"])
    expected_requirements = [row["id"] for row in requirements if applicability[row["id"]] in APPLICABLE]
    rows = value["requirements"]
    require([row.get("id") for row in rows] == expected_requirements, "requirements:order")
    fixture_count = 0
    test_count = 0
    for row in rows:
        require(tuple(row) == ("id", "proof_ids", "proof_kind"), f"requirement:keys:{row.get('id')}")
        proof_ids = row["proof_ids"]
        require(isinstance(proof_ids, list) and len(proof_ids) == 1, f"requirement:proof_count:{row['id']}")
        require(PROOF_ID.fullmatch(proof_ids[0]) is not None, f"requirement:proof_id:{row['id']}")
        fixtures = sorted(fixture_map.get(row["id"], ()), key=str.encode)
        if row["proof_kind"] == "opaque_fixture":
            fixture_count += 1
            require(bool(fixtures), f"requirement:fixture_missing:{row['id']}")
            require(proof_ids == [opaque_fixture(fixtures[0])], f"requirement:fixture_binding:{row['id']}")
        else:
            test_count += 1
            require(row["proof_kind"] == "opaque_test", f"requirement:proof_kind:{row['id']}")
            require(not fixtures, f"requirement:unnecessary_test:{row['id']}")
            require(proof_ids[0].startswith("opaque_test_"), f"requirement:test_binding:{row['id']}")
    require((fixture_count, test_count) == (49, 64), "requirements:partition")
    clauses = value["report_clauses"]
    require([row.get("id") for row in clauses] == list(EXPECTED_CLAUSES), "clauses:order")
    for row in clauses:
        require(tuple(row) == ("id", "proof_ids"), f"clause:keys:{row.get('id')}")
        require(len(row["proof_ids"]) == 1 and row["proof_ids"][0].startswith("opaque_test_"), f"clause:proof:{row['id']}")
    findings = value["findings"]
    require([row.get("id") for row in findings] == list(FINDING_IDS), "findings:order")
    for row in findings:
        require(tuple(row) == ("id", "proof_ids", "status"), f"finding:keys:{row.get('id')}")
        require(len(row["proof_ids"]) == 1 and PROOF_ID.fullmatch(row["proof_ids"][0]) is not None, f"finding:proof:{row['id']}")
        require((row["status"] == "held") == (row["id"] == "FINDING_080"), f"finding:status:{row['id']}")
        require(row["proof_ids"][0].startswith("opaque_hold_") == (row["id"] == "FINDING_080"), f"finding:kind:{row['id']}")
    require(row_projection(value) == ROW_PROJECTION_SHA256, "report:row_projection")
    require(value["result_identity_sha256"] == RESULT_IDENTITY, "report:identity_literal")
    require(result_identity(value) == RESULT_IDENTITY, "report:identity")
    validate_closed_schema(schema, "schema")
    if bind_files:
        require(digest((ROOT / REPORT).read_bytes()) == REPORT_SHA256, "report:file")
        require(digest((ROOT / SCHEMA).read_bytes()) == SCHEMA_SHA256, "schema:file")


def mutation_self_test(value: dict[str, Any], schema: dict[str, Any]) -> int:
    mutations: list[dict[str, Any]] = []
    for field in ("requirements", "report_clauses", "findings"):
        missing = copy.deepcopy(value); missing[field].pop(); mutations.append(missing)
        reordered = copy.deepcopy(value); reordered[field].reverse(); mutations.append(reordered)
        duplicate = copy.deepcopy(value); duplicate[field][-1] = duplicate[field][0]; mutations.append(duplicate)
    stale = copy.deepcopy(value); stale["requirements"][0]["proof_ids"] = ["opaque_test_" + "0" * 64]; mutations.append(stale)
    generic = copy.deepcopy(value); generic["requirements"][0]["proof_ids"] = ["complete package check"]; mutations.append(generic)
    skipped = copy.deepcopy(value); skipped["requirements"][0]["proof_ids"] = ["opaque_test_skipped"]; mutations.append(skipped)
    wrong_kind = copy.deepcopy(value); wrong_kind["requirements"][0]["proof_kind"] = "opaque_fixture"; mutations.append(wrong_kind)
    false_hold = copy.deepcopy(value); false_hold["findings"][0]["status"] = "held"; mutations.append(false_hold)
    false_close = copy.deepcopy(value); false_close["findings"][7]["status"] = "closed"; mutations.append(false_close)
    extra = copy.deepcopy(value); extra["unapproved"] = False; mutations.append(extra)
    coordinated = copy.deepcopy(value); coordinated["requirements"][0]["proof_ids"] = ["opaque_test_" + "f" * 64]; coordinated["result_identity_sha256"] = result_identity(coordinated); mutations.append(coordinated)
    caught = 0
    for mutation in mutations:
        try:
            validate(mutation, schema, bind_files=False)
        except OpaqueProofError:
            caught += 1
            continue
        raise OpaqueProofError("mutation_survived")
    opened = copy.deepcopy(schema)
    opened["additionalProperties"] = True
    try:
        validate(value, opened, bind_files=False)
    except OpaqueProofError:
        caught += 1
    else:
        raise OpaqueProofError("schema_mutation_survived")
    require(caught == 18, "mutation_count")
    return caught


def main() -> int:
    value = load(REPORT)
    schema = load(SCHEMA)
    validate(value, schema)
    mutations = mutation_self_test(value, schema)
    unique = {
        proof
        for field in ("requirements", "report_clauses", "findings")
        for row in value[field]
        for proof in row["proof_ids"]
    }
    print("PASS: opaque semantic proof identifiers v10")
    print("- applicable_requirements=113")
    print("- report_clauses=21")
    print("- findings=21")
    print(f"- unique_opaque_ids={len(unique)}")
    print(f"- negative_mutations={mutations}")
    print(f"- result_identity_sha256={RESULT_IDENTITY}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except OpaqueProofError as error:
        raise SystemExit(f"FAIL: {error}") from error
