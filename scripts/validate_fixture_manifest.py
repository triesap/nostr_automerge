#!/usr/bin/env python3
"""Validate the immutable canonical fixture distribution."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    manifest = json.loads((ROOT / "fixtures/distribution/manifest.json").read_text())
    if set(manifest) != {"distribution_schema", "distribution_id", "protocol_revision", "status", "profiles", "files"}:
        raise AssertionError("invalid fixture manifest shape")
    if manifest["distribution_schema"] != "nostr_automerge.fixture_distribution.v2" or manifest["status"] != "canonical_neutral_corpus":
        raise AssertionError("fixture manifest is not canonical")
    if set(manifest["profiles"]) != {"core", "checkpoint", "malformed", "property"}:
        raise AssertionError("fixture profiles are incomplete")
    fixture_ids = {
        json.loads(path.read_text())["fixture_id"]
        for path in ROOT.joinpath("fixtures").rglob("*.fixture.json")
    }
    profiled = [identifier for values in manifest["profiles"].values() for identifier in values]
    if len(profiled) != len(set(profiled)) or set(profiled) != fixture_ids:
        raise AssertionError("fixture profile membership is missing or duplicated")
    paths = [item["path"] for item in manifest["files"]]
    if paths != sorted(paths, key=str.encode) or len(paths) != len(set(paths)):
        raise AssertionError("fixture files are noncanonical")
    for item in manifest["files"]:
        path = (ROOT / item["path"]).resolve()
        if ROOT.resolve() not in path.parents or not path.is_file():
            raise AssertionError("fixture manifest path escaped or is stale")
        if hashlib.sha256(path.read_bytes()).hexdigest() != item["sha256"]:
            raise AssertionError("fixture manifest checksum mismatch")
    print(f"PASS: canonical fixture manifest ({len(fixture_ids)} fixtures, {len(paths)} files)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
