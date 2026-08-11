#!/usr/bin/env python3
"""Validate a fresh-process signed profile artifact without retaining it."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


def canonical(value: object) -> bytes:
    return (canonical_value(value) + "\n").encode()


def canonical_value(value: object) -> str:
    if isinstance(value, dict):
        keys = sorted(value, key=lambda item: item.encode("utf-16-be", "surrogatepass"))
        return "{" + ",".join(
            json.dumps(key, ensure_ascii=False) + ":" + canonical_value(value[key]) for key in keys
        ) + "}"
    if isinstance(value, list):
        return "[" + ",".join(canonical_value(item) for item in value) + "]"
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("artifact", type=Path)
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--contract", required=True, type=Path)
    parser.add_argument("--profile", required=True, choices=("core", "checkpoint", "malformed", "property"))
    parser.add_argument("--projection-alias", action="store_true")
    arguments = parser.parse_args()
    artifact = json.loads(arguments.artifact.read_text(encoding="utf-8"))
    manifest_bytes = arguments.manifest.read_bytes()
    manifest = json.loads(manifest_bytes)
    contract = json.loads(arguments.contract.read_text(encoding="utf-8"))
    profile = arguments.profile
    if artifact["schema"] != "nostr_automerge.typescript_signed_profile.v4":
        raise AssertionError("profile schema")
    if artifact["profile"] != profile or artifact["status"] != "pass":
        raise AssertionError("profile result")
    if artifact["process_runs_per_fixture"] != 2:
        raise AssertionError("fresh process count")
    if artifact["source_commit"] != contract["implementation_commit"]:
        raise AssertionError("stale implementation commit")
    if artifact["package_lock_sha256"] != contract["dependency_lock_sha256"]:
        raise AssertionError("dependency lock substitution")
    manifest_hash = hashlib.sha256(manifest_bytes).hexdigest()
    if artifact["fixture_manifest_sha256"] != manifest_hash:
        raise AssertionError("distribution substitution")
    reports = artifact["reports"]
    expected_ids = manifest["profiles"][profile]
    if artifact["fixture_count"] != len(expected_ids) or [item["fixture_id"] for item in reports] != expected_ids:
        raise AssertionError("profile fixture membership")
    entries = {entry["fixture_id"]: entry for entry in manifest["fixtures"]}
    for item in reports:
        if item["requirements"] != entries[item["fixture_id"]]["requirements"]:
            raise AssertionError("profile requirement binding")
        if item["report_sha256"] != hashlib.sha256(canonical(item["report"])).hexdigest():
            raise AssertionError("canonical report digest")
    report_bytes = canonical(reports)
    if artifact["output_sha256"] != hashlib.sha256(report_bytes).hexdigest():
        raise AssertionError("profile output digest")
    if arguments.projection_alias:
        if profile != "property" or not all(
            item["fixture_id"].startswith("projection_") and item["report"]["state_assertions"]
            for item in reports
        ):
            raise AssertionError("projection profile coverage")
    print(f"PASS: signed {profile} profile artifact")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
