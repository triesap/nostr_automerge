#!/usr/bin/env python3
"""Validate the remediation-v7 signed fixture distribution v8."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "fixtures/distribution/manifest_v8.json"
PROFILES = {"core", "checkpoint", "malformed", "property"}
REQUIREMENTS_SHA256 = "95a80689b3e4d661a73867673994829e7060df67277120b2f16ee9f2dd16f9fd"
V8_REQUIREMENTS = {
    "change_references_invalid_noncanonical_child": ["NCRDT-BRANCH-001", "NCRDT-BRANCH-002", "NCRDT-CONF-008"],
    "manifest_references_invalid_noncanonical_child": ["NCRDT-BRANCH-001", "NCRDT-BRANCH-002", "NCRDT-CONF-008"],
    "noncanonical_child_excluded_base_head": ["NCRDT-BRANCH-001", "NCRDT-BRANCH-002", "NCRDT-CONF-008"],
    "noncanonical_child_invalid_base_head": ["NCRDT-BRANCH-001", "NCRDT-BRANCH-002", "NCRDT-CONF-008"],
    "noncanonical_child_pending_base_head": ["NCRDT-BRANCH-001", "NCRDT-BRANCH-002", "NCRDT-CONF-008"],
    "noncanonical_grandchild_invalid_parent_epoch": ["NCRDT-BRANCH-001", "NCRDT-BRANCH-002", "NCRDT-CONF-008"],
    "cross_coordinate_descriptor_reference_isolated": ["NCRDT-CONF-008", "NCRDT-SCOPE-005", "NCRDT-SCOPE-006"],
    "foreign_change_references_target_control": ["NCRDT-CONF-008", "NCRDT-SCOPE-004", "NCRDT-SCOPE-006"],
    "foreign_chunk_excluded_from_target_digest": ["NCRDT-CONF-008", "NCRDT-SCOPE-006"],
    "foreign_chunk_references_target_descriptor": ["NCRDT-CONF-008", "NCRDT-SCOPE-005", "NCRDT-SCOPE-006"],
    "foreign_claim_flood_exact_budget": ["NCRDT-CONF-008", "NCRDT-SCOPE-004", "NCRDT-SCOPE-006"],
    "unrelated_valid_checkpoints_exact_budget": ["NCRDT-CONF-008", "NCRDT-SCOPE-006"],
    "interrupted_finalization_forfeiture": ["NCRDT-CONF-008", "NCRDT-RESOURCE-010"],
    "parent_propagation_exact_budget": ["NCRDT-CONF-008", "NCRDT-RESOURCE-009"],
}


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def fail(message: str) -> None:
    raise SystemExit(message)


def main() -> None:
    if sys.argv[1:]:
        fail("usage: validate_fixture_distribution_v8.py")
    manifest = json.loads(MANIFEST.read_text())
    if manifest.get("distribution_schema") != "nostr_automerge.fixture_distribution.v8":
        fail("invalid fixture distribution schema")
    if manifest.get("supersedes") != "fixtures/distribution/manifest_v7.json":
        fail("fixture distribution does not supersede v7")
    if manifest.get("complete") is not True or manifest.get("fixture_count") != 171:
        fail("signed distribution is incomplete")
    if (
        manifest.get("target_fixture_count") != 171
        or manifest.get("missing_v7_fixtures") != []
        or manifest.get("missing_v8_fixtures") != []
    ):
        fail("signed distribution target is inconsistent")
    if set(manifest.get("profiles", {})) != PROFILES:
        fail("fixture distribution profiles are incomplete")
    if manifest.get("requirements_sha256") != REQUIREMENTS_SHA256:
        fail("fixture distribution requirements_sha256 is stale")
    authorities = {
        "authority_sha256": ROOT / "spec/NIP_DRAFT.md",
        "companion_sha256": ROOT / "spec/NOSTR_AUTOMERGE_V1_SPEC.md",
    }
    for field, path in authorities.items():
        if manifest.get(field) != digest(path):
            fail(f"fixture distribution {field} is stale")
    known_requirements = {
        row["id"]
        for row in json.loads((ROOT / "spec/requirements.json").read_text())["requirements"]
    }
    fixture_ids: set[str] = set()
    paths: set[str] = set()
    for item in manifest.get("files", []):
        relative = item.get("path", "")
        if not relative or relative in paths or relative.startswith(("/", "../")):
            fail(f"invalid or duplicate distribution path: {relative!r}")
        paths.add(relative)
        path = ROOT / relative
        if relative == "spec/requirements.json":
            if item.get("sha256") != REQUIREMENTS_SHA256:
                fail(f"missing or stale distribution file: {relative}")
            continue
        if not path.is_file() or item.get("sha256") != digest(path):
            fail(f"missing or stale distribution file: {relative}")
    entries = manifest.get("fixtures", [])
    if entries != sorted(entries, key=lambda item: item["fixture_id"].encode()):
        fail("fixture order is not deterministic")
    for entry in entries:
        fixture_id = entry["fixture_id"]
        if fixture_id in fixture_ids:
            fail(f"duplicate fixture id: {fixture_id}")
        fixture_ids.add(fixture_id)
        metadata_path = ROOT / entry["metadata_path"]
        metadata = json.loads(metadata_path.read_text())
        requirements = metadata.get("requirements")
        if metadata.get("fixture_id") != fixture_id or len(metadata.get("inputs", [])) != 1:
            fail(f"invalid signed fixture metadata: {fixture_id}")
        if requirements != sorted(set(requirements), key=str.encode):
            fail(f"noncanonical requirements: {fixture_id}")
        if not set(requirements).issubset(known_requirements) or entry.get("requirements") != requirements:
            fail(f"unknown or mismatched requirements: {fixture_id}")
        if fixture_id in V8_REQUIREMENTS and requirements != V8_REQUIREMENTS[fixture_id]:
            fail(f"incorrect remediation-v7 requirements: {fixture_id}")
        input_path = metadata_path.parent / metadata["inputs"][0]["path"]
        scenario = json.loads(input_path.read_text())
        if scenario.get("scenario_schema") != "nostr_automerge.signed_scenario.v2":
            fail(f"fixture is not a signed scenario: {fixture_id}")
        if scenario.get("requirements") != requirements:
            fail(f"scenario requirements differ from metadata: {fixture_id}")
        if any(key in scenario for key in ("operations", "valid", "selected", "accepted")):
            fail(f"fixture contains abstract protocol truth: {fixture_id}")
    assigned = [identifier for values in manifest["profiles"].values() for identifier in values]
    required = manifest.get("v8_fixtures", [])
    prior = json.loads((ROOT / "fixtures/distribution/manifest_v7.json").read_text())
    prior_ids = {item["fixture_id"] for item in prior["fixtures"]}
    if len(fixture_ids) != 171 or len(assigned) != len(set(assigned)) or set(assigned) != fixture_ids:
        fail("profiles do not cover exactly 171 signed fixtures")
    if required != list(V8_REQUIREMENTS) or not set(required).issubset(fixture_ids):
        fail("remediation-v7 fixture coverage is incomplete")
    if len(prior_ids) != 157 or not prior_ids.issubset(fixture_ids):
        fail("distribution v7 is not preserved")
    print("PASS: fixture distribution v8 (171 signed fixtures)")


if __name__ == "__main__":
    main()
