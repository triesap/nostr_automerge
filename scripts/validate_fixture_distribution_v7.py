#!/usr/bin/env python3
"""Validate the remediation-v6 signed fixture distribution v7."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "fixtures/distribution/manifest_v7.json"
PROFILES = {"core", "checkpoint", "malformed", "property"}


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def fail(message: str) -> None:
    raise SystemExit(message)


def main() -> None:
    if sys.argv[1:]:
        fail("usage: validate_fixture_distribution_v7.py")
    manifest = json.loads(MANIFEST.read_text())
    if manifest.get("distribution_schema") != "nostr_automerge.fixture_distribution.v7":
        fail("invalid fixture distribution schema")
    if manifest.get("supersedes") != "fixtures/distribution/manifest_v6.json":
        fail("fixture distribution does not supersede v6")
    if manifest.get("complete") is not True or manifest.get("fixture_count") != 157:
        fail("signed distribution is incomplete")
    if manifest.get("target_fixture_count") != 157 or manifest.get("missing_v6_fixtures") != []:
        fail("signed distribution target is inconsistent")
    if set(manifest.get("profiles", {})) != PROFILES:
        fail("fixture distribution profiles are incomplete")
    authorities = {
        "requirements_sha256": "dc86a3c713166e4137693254e111dd51932a0b0659dafe95695a22241997aded",
        "authority_sha256": "67019c8ea680714052c65226f620a8e1a60b9b10a8f158603063a835a7bbc7a3",
        "companion_sha256": "8d7fe07de3ba699d7a944003c9fbf4f52e7e865945f0c11d1bd13db5937da5f4",
    }
    for field, expected in authorities.items():
        if manifest.get(field) != expected:
            fail(f"fixture distribution {field} changed")
    fixture_ids: set[str] = set()
    paths: set[str] = set()
    historical_files = {
        "spec/NOSTR_AUTOMERGE_V1_SPEC.md": authorities["companion_sha256"],
        "spec/requirements.json": authorities["requirements_sha256"],
    }
    for item in manifest.get("files", []):
        relative = item.get("path", "")
        if not relative or relative in paths or relative.startswith(("/", "../")):
            fail(f"invalid or duplicate distribution path: {relative!r}")
        paths.add(relative)
        if relative in historical_files:
            if item.get("sha256") != historical_files[relative]:
                fail(f"changed historical distribution file: {relative}")
            continue
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
        metadata = json.loads((ROOT / entry["metadata_path"]).read_text())
        if metadata.get("fixture_id") != fixture_id or len(metadata.get("inputs", [])) != 1:
            fail(f"invalid signed fixture metadata: {fixture_id}")
        input_path = (ROOT / entry["metadata_path"]).parent / metadata["inputs"][0]["path"]
        scenario = json.loads(input_path.read_text())
        if scenario.get("scenario_schema") != "nostr_automerge.signed_scenario.v2":
            fail(f"fixture is not a signed scenario: {fixture_id}")
        if any(key in scenario for key in ("operations", "valid", "selected", "accepted")):
            fail(f"fixture contains abstract protocol truth: {fixture_id}")
    assigned = [identifier for values in manifest["profiles"].values() for identifier in values]
    required = manifest.get("v6_fixtures", [])
    if len(fixture_ids) != 157 or len(assigned) != len(set(assigned)) or set(assigned) != fixture_ids:
        fail("profiles do not cover exactly 157 signed fixtures")
    if len(required) != 33 or not set(required).issubset(fixture_ids):
        fail("remediation-v6 fixture coverage is incomplete")
    print("PASS: fixture distribution v7 (157 signed fixtures)")


if __name__ == "__main__":
    main()
