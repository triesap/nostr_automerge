#!/usr/bin/env python3
"""Validate the closed semantic-proof vocabularies and catalog schemas."""

from __future__ import annotations

import copy
import hashlib
import json
import sys
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True

from validate_report_contract_v9 import EXPECTED_CLAUSES


ROOT = Path(__file__).resolve().parents[1]
AUTHORITY_PATH = "spec/semantic_proof_catalog_v10.json"
CATALOG_SCHEMA_PATH = "tools/validation/semantic_proof_catalog_v10.schema.json"
FINDING_SCHEMA_PATH = "tools/validation/finding_closure_catalog_v10.schema.json"
AUTHORITY_SHA256 = "92c0346c808047c27532da8422d737c72e6414ba3a4067d4af5515cd135ee913"
CATALOG_SCHEMA_SHA256 = "6acb7d9331188c3231832a177911c399fb5877c6ea68ede63e63386d00a72bfa"
FINDING_SCHEMA_SHA256 = "d7ca5fead5ad82cca201a97edcc8fed9d989a83c60392637ddd8c574f4f1e5ff"
AUTHORITY_FIELDS = (
    "schema", "status", "protocol_revision", "subject_kinds",
    "semantic_categories", "applicability_classes", "proof_kinds",
    "report_clauses", "finding_ids", "source_substring_proof",
    "generic_command_only_proof", "skipped_or_filtered_proof", "result",
)
SUBJECT_KINDS = ("requirement", "report_clause", "finding")
SEMANTIC_CATEGORIES = (
    "authority", "wire_ingress", "control_history", "change_application",
    "checkpoint_verification", "report_contract", "resource_accounting",
    "signed_conformance", "evidence_integrity", "external_hold",
)
APPLICABILITY_CLASSES = (
    "rust_only", "rust_and_opaque", "opaque_only", "external_hold",
)
PROOF_KINDS = (
    "rust_test", "signed_fixture", "validator", "opaque_fixture",
    "opaque_test", "hold_record",
)
FINDING_IDS = tuple(f"FINDING_{number:03d}" for number in range(73, 94))


class CatalogError(ValueError):
    """One proof-catalog authority invariant failed."""


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise CatalogError(diagnostic)


def load(relative: str) -> dict[str, Any]:
    value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    require(isinstance(value, dict), f"object:{relative}")
    return value


def digest(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def validate_closed_schema(value: Any, diagnostic: str) -> None:
    if isinstance(value, dict):
        if value.get("type") == "object":
            require(value.get("additionalProperties") is False, f"{diagnostic}:open")
            properties = value.get("properties")
            required = value.get("required")
            require(isinstance(properties, dict), f"{diagnostic}:properties")
            require(isinstance(required, list), f"{diagnostic}:required")
            require(set(required) == set(properties), f"{diagnostic}:shape")
        for key, child in value.items():
            validate_closed_schema(child, f"{diagnostic}:{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            validate_closed_schema(child, f"{diagnostic}:{index}")


def validate(
    authority: dict[str, Any],
    catalog_schema: dict[str, Any],
    finding_schema: dict[str, Any],
    *,
    bind_files: bool = True,
) -> None:
    require(tuple(authority) == AUTHORITY_FIELDS, "authority:keys")
    require(authority.get("schema") == "nostr_automerge.semantic_proof_catalog.authority.v10.v1", "authority:schema")
    require(authority.get("status") == "schema_defined", "authority:status")
    require(authority.get("protocol_revision") == "draft_2026_08", "authority:revision")
    require(authority.get("subject_kinds") == list(SUBJECT_KINDS), "authority:subjects")
    require(authority.get("semantic_categories") == list(SEMANTIC_CATEGORIES), "authority:categories")
    require(authority.get("applicability_classes") == list(APPLICABILITY_CLASSES), "authority:applicability")
    require(authority.get("proof_kinds") == list(PROOF_KINDS), "authority:kinds")
    require(authority.get("report_clauses") == list(EXPECTED_CLAUSES), "authority:clauses")
    require(authority.get("finding_ids") == list(FINDING_IDS), "authority:findings")
    require(authority.get("source_substring_proof") == "forbidden", "authority:substring")
    require(authority.get("generic_command_only_proof") == "forbidden", "authority:generic")
    require(authority.get("skipped_or_filtered_proof") == "forbidden", "authority:skipped")
    require(authority.get("result") == "pass", "authority:result")
    requirements = load("spec/requirements.json").get("requirements")
    findings = load("spec/remediation_findings_v9.json").get("findings")
    require(isinstance(requirements, list) and len(requirements) == 148, "requirements:count")
    require(isinstance(findings, list), "findings:type")
    require(tuple(row.get("id") for row in findings) == FINDING_IDS, "findings:inventory")
    validate_closed_schema(catalog_schema, "catalog_schema")
    validate_closed_schema(finding_schema, "finding_schema")
    require(catalog_schema.get("properties", {}).get("requirement_count") == {"const": 148}, "catalog_schema:requirements")
    require(catalog_schema.get("properties", {}).get("report_clause_count") == {"const": 21}, "catalog_schema:clauses")
    require(catalog_schema.get("properties", {}).get("finding_count") == {"const": 21}, "catalog_schema:findings")
    require(catalog_schema.get("properties", {}).get("rows", {}).get("minItems") == 190, "catalog_schema:row_minimum")
    require(catalog_schema.get("properties", {}).get("rows", {}).get("maxItems") == 190, "catalog_schema:row_maximum")
    row_properties = catalog_schema["properties"]["rows"]["items"]["properties"]
    require(tuple(row_properties["subject_kind"]["enum"]) == SUBJECT_KINDS, "catalog_schema:subjects")
    require(tuple(row_properties["semantic_category"]["enum"]) == SEMANTIC_CATEGORIES, "catalog_schema:categories")
    require(tuple(row_properties["applicability"]["enum"]) == APPLICABILITY_CLASSES, "catalog_schema:applicability")
    require(tuple(catalog_schema["$defs"]["proof"]["properties"]["kind"]["enum"]) == PROOF_KINDS, "catalog_schema:kinds")
    finding_row = finding_schema["properties"]["rows"]["items"]["properties"]
    require(tuple(finding_row["semantic_category"]["enum"]) == SEMANTIC_CATEGORIES, "finding_schema:categories")
    if bind_files:
        require(digest(AUTHORITY_PATH) == AUTHORITY_SHA256, "authority:file")
        require(digest(CATALOG_SCHEMA_PATH) == CATALOG_SCHEMA_SHA256, "catalog_schema:file")
        require(digest(FINDING_SCHEMA_PATH) == FINDING_SCHEMA_SHA256, "finding_schema:file")


def expect_rejected(work: Any, diagnostic: str) -> int:
    try:
        work()
    except CatalogError:
        return 1
    raise CatalogError(f"mutation_survived:{diagnostic}")


def mutation_self_test(
    authority: dict[str, Any], catalog_schema: dict[str, Any], finding_schema: dict[str, Any]
) -> int:
    mutations: list[tuple[str, dict[str, Any]]] = []
    for field in ("subject_kinds", "semantic_categories", "applicability_classes", "proof_kinds", "report_clauses", "finding_ids"):
        missing = copy.deepcopy(authority); missing[field].pop(); mutations.append((f"missing:{field}", missing))
        reordered = copy.deepcopy(authority); reordered[field].reverse(); mutations.append((f"reordered:{field}", reordered))
        duplicate = copy.deepcopy(authority); duplicate[field][-1] = duplicate[field][0]; mutations.append((f"duplicate:{field}", duplicate))
    for field in ("source_substring_proof", "generic_command_only_proof", "skipped_or_filtered_proof"):
        changed = copy.deepcopy(authority); changed[field] = "allowed"; mutations.append((f"allowed:{field}", changed))
    extra = copy.deepcopy(authority); extra["extra"] = False; mutations.append(("authority:extra", extra))
    missing_key = copy.deepcopy(authority); missing_key.pop("status"); mutations.append(("authority:missing", missing_key))
    caught = sum(
        expect_rejected(
            lambda item=item: validate(item, catalog_schema, finding_schema, bind_files=False), name
        )
        for name, item in mutations
    )
    schema_mutations: list[tuple[str, dict[str, Any], dict[str, Any]]] = []
    opened = copy.deepcopy(catalog_schema); opened["additionalProperties"] = True
    schema_mutations.append(("catalog:open", opened, finding_schema))
    weak_row = copy.deepcopy(catalog_schema); weak_row["properties"]["rows"]["items"]["required"].pop()
    schema_mutations.append(("catalog:weak_row", weak_row, finding_schema))
    weak_count = copy.deepcopy(catalog_schema); weak_count["properties"]["rows"].pop("maxItems")
    schema_mutations.append(("catalog:weak_count", weak_count, finding_schema))
    extra_category = copy.deepcopy(catalog_schema); extra_category["properties"]["rows"]["items"]["properties"]["semantic_category"]["enum"].append("generic")
    schema_mutations.append(("catalog:category", extra_category, finding_schema))
    source_kind = copy.deepcopy(catalog_schema); source_kind["$defs"]["proof"]["properties"]["kind"]["enum"].append("source_substring")
    schema_mutations.append(("catalog:source_kind", source_kind, finding_schema))
    open_finding = copy.deepcopy(finding_schema); open_finding["properties"]["rows"]["items"]["additionalProperties"] = True
    schema_mutations.append(("finding:open", catalog_schema, open_finding))
    weak_finding = copy.deepcopy(finding_schema); weak_finding["properties"]["rows"]["items"]["required"].pop()
    schema_mutations.append(("finding:weak", catalog_schema, weak_finding))
    for name, first, second in schema_mutations:
        caught += expect_rejected(
            lambda first=first, second=second: validate(authority, first, second, bind_files=False),
            name,
        )
    require(caught == 30, "mutation_count")
    return caught


def main() -> int:
    authority = load(AUTHORITY_PATH)
    catalog_schema = load(CATALOG_SCHEMA_PATH)
    finding_schema = load(FINDING_SCHEMA_PATH)
    validate(authority, catalog_schema, finding_schema)
    mutations = mutation_self_test(authority, catalog_schema, finding_schema)
    print("PASS: semantic proof-catalog v10 authority")
    print("- semantic_categories=10")
    print("- report_clauses=21")
    print("- findings=21")
    print(f"- negative_mutations={mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
