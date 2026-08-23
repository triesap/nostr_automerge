#!/usr/bin/env python3
"""Reject cross-model semantic-proof mutations before catalog publication."""

from __future__ import annotations

import copy
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
EXPECTED_PROJECTION = "bb79ec8f0b489c1d9c60af09b33a6992765cf7676000e4f29d814b051b1dbeac"
OPAQUE_APPLICABILITY = {
    "rust-and-typescript",
    "rust-only-evidence-with-opaque-typescript-overlay",
}


class MutationError(ValueError):
    """One combined semantic-proof invariant failed."""


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise MutationError(diagnostic)


def canonical(value: Any) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def projection(value: dict[str, Any]) -> str:
    return hashlib.sha256(canonical(value)).hexdigest()


def build() -> dict[str, Any]:
    rust_rows, rust_proofs = public_requirements.build_rows()
    clauses = public_subjects.clause_rows()
    findings = public_subjects.finding_rows(public_subjects.FINDING_BINDINGS)
    opaque_report = opaque.load(opaque.REPORT)
    opaque_schema = opaque.load(opaque.SCHEMA)
    public_requirements.validate(rust_rows, rust_proofs)
    public_subjects.validate(clauses, findings, public_subjects.FINDING_BINDINGS)
    opaque.validate(opaque_report, opaque_schema)
    return {
        "requirements": [
            {
                "id": row["id"],
                "applicability": row["applicability"],
                "status": row["status"],
                "public_proof_ids": row["rust_proof_ids"],
                "opaque_proof_ids": next(
                    (
                        item["proof_ids"]
                        for item in opaque_report["requirements"]
                        if item["id"] == row["id"]
                    ),
                    [],
                ),
            }
            for row in rust_rows
        ],
        "report_clauses": [
            {
                "id": row["id"],
                "status": row["status"],
                "public_proof_ids": row["closure_proofs"],
                "opaque_proof_ids": opaque_row["proof_ids"],
            }
            for row, opaque_row in zip(
                clauses, opaque_report["report_clauses"], strict=True
            )
        ],
        "findings": [
            {
                "id": row["id"],
                "status": row["status"],
                "public_proof_ids": row["closure_proofs"],
                "opaque_proof_ids": opaque_row["proof_ids"],
            }
            for row, opaque_row in zip(
                findings, opaque_report["findings"], strict=True
            )
        ],
    }


def validate(value: dict[str, Any], *, bind_projection: bool = True) -> None:
    require(tuple(value) == ("requirements", "report_clauses", "findings"), "keys")
    requirements = value["requirements"]
    clauses = value["report_clauses"]
    findings = value["findings"]
    authority = public_requirements.load("spec/requirements.json")["requirements"]
    require([row["id"] for row in requirements] == [row["id"] for row in authority], "requirements:order")
    require(len(requirements) == 148, "requirements:count")
    for row in requirements:
        require(tuple(row) == ("id", "applicability", "status", "public_proof_ids", "opaque_proof_ids"), f"requirement:keys:{row.get('id')}")
        held = row["status"] == "held"
        require(bool(row["public_proof_ids"]) != held, f"requirement:public:{row['id']}")
        expected_opaque = row["applicability"] in OPAQUE_APPLICABILITY
        require(bool(row["opaque_proof_ids"]) == expected_opaque, f"requirement:opaque:{row['id']}")
        require(all(item.startswith(("rust_test:", "signed_fixture:", "validator:")) for item in row["public_proof_ids"]), f"requirement:public_kind:{row['id']}")
        require(all(opaque.PROOF_ID.fullmatch(item) is not None for item in row["opaque_proof_ids"]), f"requirement:opaque_kind:{row['id']}")
    require([row["id"] for row in clauses] == list(public_subjects.EXPECTED_CLAUSES), "clauses:order")
    require(len(clauses) == 21, "clauses:count")
    for row in clauses:
        require(tuple(row) == ("id", "status", "public_proof_ids", "opaque_proof_ids"), f"clause:keys:{row.get('id')}")
        require(row["status"] == "pass", f"clause:status:{row['id']}")
        require(len(row["public_proof_ids"]) == len(row["opaque_proof_ids"]) == 1, f"clause:proof:{row['id']}")
        require(row["public_proof_ids"][0].startswith("rust_test:"), f"clause:public_kind:{row['id']}")
        require(row["opaque_proof_ids"][0].startswith("opaque_test_"), f"clause:opaque_kind:{row['id']}")
    require([row["id"] for row in findings] == list(public_subjects.FINDING_IDS), "findings:order")
    require(len(findings) == 21, "findings:count")
    for row in findings:
        require(tuple(row) == ("id", "status", "public_proof_ids", "opaque_proof_ids"), f"finding:keys:{row.get('id')}")
        held = row["id"] == public_subjects.HELD_FINDING
        require((row["status"] == "held") == held, f"finding:status:{row['id']}")
        require(bool(row["public_proof_ids"]) and len(row["opaque_proof_ids"]) == 1, f"finding:proof:{row['id']}")
        require(row["opaque_proof_ids"][0].startswith("opaque_hold_") == held, f"finding:opaque_kind:{row['id']}")
    if bind_projection:
        require(projection(value) == EXPECTED_PROJECTION, "projection")


def mutation_self_test(value: dict[str, Any]) -> int:
    mutations: list[tuple[str, dict[str, Any]]] = []
    opaque_row = next(
        index
        for index, row in enumerate(value["requirements"])
        if row["opaque_proof_ids"]
    )
    for field in ("requirements", "report_clauses", "findings"):
        missing = copy.deepcopy(value); missing[field].pop(); mutations.append((f"missing:{field}", missing))
        reordered = copy.deepcopy(value); reordered[field].reverse(); mutations.append((f"reordered:{field}", reordered))
        duplicate = copy.deepcopy(value); duplicate[field][-1] = duplicate[field][0]; mutations.append((f"duplicate:{field}", duplicate))
    generic = copy.deepcopy(value); generic["requirements"][0]["public_proof_ids"] = ["complete package check"]; mutations.append(("generic", generic))
    stale = copy.deepcopy(value); stale["requirements"][0]["opaque_proof_ids"] = ["opaque_test_" + "0" * 64]; mutations.append(("stale", stale))
    skipped = copy.deepcopy(value); skipped["report_clauses"][0]["public_proof_ids"] = ["rust_test:runner:ignored"]; mutations.append(("skipped", skipped))
    category = copy.deepcopy(value); category["requirements"][opaque_row]["applicability"] = "out-of-core"; mutations.append(("category", category))
    unrelated = copy.deepcopy(value); unrelated["requirements"][0]["public_proof_ids"] = value["requirements"][1]["public_proof_ids"]; mutations.append(("unrelated", unrelated))
    false_hold = copy.deepcopy(value); false_hold["findings"][0]["status"] = "held"; mutations.append(("false_hold", false_hold))
    false_close = copy.deepcopy(value); false_close["findings"][7]["status"] = "closed"; mutations.append(("false_close", false_close))
    wrong_namespace = copy.deepcopy(value); wrong_namespace["report_clauses"][0]["opaque_proof_ids"] = ["opaque_fixture_" + "a" * 64]; mutations.append(("wrong_namespace", wrong_namespace))
    leaking = copy.deepcopy(value); leaking["findings"][0]["opaque_proof_ids"] = [chr(47).join(("private", "test", "path"))]; mutations.append(("leaking", leaking))
    extra = copy.deepcopy(value); extra["unapproved"] = False; mutations.append(("extra", extra))
    coordinated = copy.deepcopy(value); coordinated["requirements"][0]["opaque_proof_ids"] = ["opaque_test_" + "f" * 64]; mutations.append(("coordinated", coordinated))
    caught = 0
    for name, mutation in mutations:
        try:
            validate(mutation)
        except MutationError:
            caught += 1
            continue
        raise MutationError(f"mutation_survived:{name}")
    require(caught == 20, "mutation_count")
    return caught


def main() -> int:
    value = build()
    validate(value)
    mutations = mutation_self_test(value)
    print("PASS: combined semantic proof mutation gate v10")
    print("- requirements=148")
    print("- report_clauses=21")
    print("- findings=21")
    print(f"- negative_mutations={mutations}")
    print(f"- projection_sha256={projection(value)}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except MutationError as error:
        raise SystemExit(f"FAIL: {error}") from error
