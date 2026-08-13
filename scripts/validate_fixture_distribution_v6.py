#!/usr/bin/env python3
"""Validate the remediation-v5 signed fixture distribution v6."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "fixtures" / "distribution" / "manifest_v6.json"
SCHEMA = "nostr_automerge.fixture_distribution.v6"
PROFILES = {"core", "checkpoint", "malformed", "property"}
COVERAGE = {"mixed_claims", "dependency_knowledge", "checkpoint_controls", "coordinate_resources", "finalization"}


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def fail(message: str) -> None:
    raise SystemExit(message)


def main() -> None:
    if sys.argv[1:]:
        fail("usage: validate_fixture_distribution_v6.py")
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    if manifest.get("distribution_schema") != SCHEMA:
        fail("invalid fixture distribution schema")
    if manifest.get("supersedes") != "fixtures/distribution/manifest_v5.json":
        fail("fixture distribution does not supersede v5")
    if set(manifest.get("profiles", {})) != PROFILES:
        fail("fixture distribution profiles are incomplete")
    coverage = manifest.get("coverage", {})
    if set(coverage) != COVERAGE or any(not value for value in coverage.values()):
        fail("remediation-v5 fixture coverage is incomplete")
    authorities = {
        "requirements_sha256": ROOT / "spec" / "requirements.json",
        "authority_sha256": ROOT / "spec" / "NIP_DRAFT.md",
        "companion_sha256": ROOT / "spec" / "NOSTR_AUTOMERGE_V1_SPEC.md",
    }
    for field, path in authorities.items():
        if manifest.get(field) != digest(path):
            fail(f"fixture distribution {field} is stale")
    fixture_ids: set[str] = set()
    paths: set[str] = set()
    for item in manifest.get("files", []):
        relative = item.get("path", "")
        if not relative or relative in paths or relative.startswith(("/", "../")):
            fail(f"invalid or duplicate distribution path: {relative!r}")
        paths.add(relative)
        path = ROOT / relative
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
        metadata = json.loads((ROOT / entry["metadata_path"]).read_text(encoding="utf-8"))
        if metadata.get("fixture_id") != fixture_id or len(metadata.get("inputs", [])) != 1:
            fail(f"invalid signed fixture metadata: {fixture_id}")
        input_path = (ROOT / entry["metadata_path"]).parent / metadata["inputs"][0]["path"]
        scenario = json.loads(input_path.read_text(encoding="utf-8"))
        if scenario.get("scenario_schema") != "nostr_automerge.signed_scenario.v2":
            fail(f"fixture is not a signed scenario: {fixture_id}")
        if any(key in scenario for key in ("operations", "valid", "selected", "accepted")):
            fail(f"fixture contains abstract protocol truth: {fixture_id}")
    assigned = [identifier for values in manifest["profiles"].values() for identifier in values]
    covered = [identifier for values in coverage.values() for identifier in values]
    if len(assigned) != len(set(assigned)) or set(assigned) != fixture_ids:
        fail("profiles do not cover exactly the signed fixtures")
    if not set(covered).issubset(fixture_ids):
        fail("coverage references an unknown fixture")
    print(f"PASS: fixture distribution v6 ({len(fixture_ids)} signed fixtures)")


if __name__ == "__main__":
    main()
