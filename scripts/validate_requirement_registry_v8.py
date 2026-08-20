#!/usr/bin/env python3
"""Validate the atomic remediation-v7 requirement-registry transition."""

from __future__ import annotations

import json
from pathlib import Path

from validate_requirements import validate


ROOT = Path(__file__).resolve().parents[1]
EXPECTED_IDS = (
    "NCRDT-BRANCH-001",
    "NCRDT-BRANCH-002",
    "NCRDT-SCOPE-004",
    "NCRDT-SCOPE-005",
    "NCRDT-SCOPE-006",
    "NCRDT-RESOURCE-009",
    "NCRDT-RESOURCE-010",
    "NCRDT-NIP-002",
    "NCRDT-CONF-008",
    "NCRDT-EVIDENCE-004",
)


def load(relative: str) -> dict:
    value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise AssertionError(f"{relative} must contain an object")
    return value


def main() -> int:
    registry = load("spec/requirements.json")
    validate(registry, resolve_sources=True)
    rows = registry["requirements"]
    if registry["schema"] != "nostr_automerge.requirements.v5":
        raise AssertionError("requirement registry schema is not v5")
    if registry["requirement_count"] != 129:
        raise AssertionError("requirement registry does not contain exactly 129 rows")
    if tuple(row["id"] for row in rows[-10:]) != EXPECTED_IDS:
        raise AssertionError("remediation-v7 rows are not appended in canonical order")

    additions = load("spec/remediation_v7_requirements.json")["requirements"]
    for canonical, proposed in zip(rows[-10:], additions, strict=True):
        for field in ("id", "section", "text", "source"):
            if canonical[field] != proposed[field]:
                raise AssertionError(f"canonical row differs from approved {field}")
        source = ROOT / canonical["source"]
        heading = f"## {canonical['section']}"
        source_text = source.read_text(encoding="utf-8")
        if heading not in source_text and f"### {canonical['section']}" not in source_text:
            raise AssertionError(
                f"remediation-v7 authority section is absent: {canonical['id']}"
            )

    applicability = load("spec/requirements_applicability.json")
    if applicability.get("schema") != "nostr_automerge.requirements_applicability.v5":
        raise AssertionError("applicability schema is not v5")
    classifications = applicability.get("classifications")
    if not isinstance(classifications, dict):
        raise AssertionError("applicability classifications are missing")
    if tuple(classifications) != tuple(row["id"] for row in rows):
        raise AssertionError("applicability rows do not exactly match registry order")
    for proposed in additions:
        if classifications[proposed["id"]] != proposed["applicability"]:
            raise AssertionError(f"applicability differs for {proposed['id']}")

    registry_schema = load("tools/validation/requirements_schema.json")
    if registry_schema["properties"]["requirement_count"].get("const") != 129:
        raise AssertionError("registry schema does not require exactly 129 rows")
    evidence_schema = load("reports/schema/requirement_evidence_v8.schema.json")
    if evidence_schema["properties"]["requirement_count"].get("const") != 129:
        raise AssertionError("evidence schema does not require exactly 129 rows")
    if evidence_schema["properties"]["phase"].get("enum") != [
        "rust-complete-typescript-pending",
        "complete",
    ]:
        raise AssertionError("evidence schema does not define the two exact phases")

    print("PASS: remediation-v7 canonical requirement registry")
    print("- requirements=129")
    print("- appended=10")
    print("- authority_sources=companion,conformance,portable-delta")
    print("- nip_applicability=explicitly-deferred")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
