#!/usr/bin/env python3
"""Validate the immutable language-neutral fixture distribution."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "fixtures" / "distribution" / "manifest.json"
SCHEMA = "nostr_automerge.fixture_distribution.v1"
PROFILES = {"core", "checkpoint", "malformed", "property"}


def main() -> None:
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    if manifest.get("distribution_schema") != SCHEMA:
        raise SystemExit("invalid fixture distribution schema")
    if set(manifest.get("profiles", {})) != PROFILES:
        raise SystemExit("fixture distribution profiles are incomplete")

    paths: set[str] = set()
    fixture_ids: set[str] = set()
    for item in manifest.get("files", []):
        relative = item.get("path", "")
        if not relative or relative in paths or relative.startswith(("/", "../")):
            raise SystemExit(f"invalid or duplicate distribution path: {relative!r}")
        paths.add(relative)
        path = ROOT / relative
        if not path.is_file():
            raise SystemExit(f"missing distribution file: {relative}")
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != item.get("sha256"):
            raise SystemExit(f"distribution checksum mismatch: {relative}")
        if relative.endswith(".fixture.json"):
            fixture = json.loads(path.read_text(encoding="utf-8"))
            fixture_ids.add(fixture["fixture_id"])

    assigned = [
        fixture_id
        for profile in manifest["profiles"].values()
        for fixture_id in profile
    ]
    if len(assigned) != len(set(assigned)):
        raise SystemExit("fixture is assigned to more than one interop profile")
    unknown = set(assigned) - fixture_ids
    if unknown:
        raise SystemExit(f"profiles reference undistributed fixtures: {sorted(unknown)}")


if __name__ == "__main__":
    main()
