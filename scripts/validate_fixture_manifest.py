#!/usr/bin/env python3
"""Validate the canonical signed fixture distribution v5."""

from __future__ import annotations

import copy
import hashlib
import json
import sys
from pathlib import Path, PurePosixPath


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "fixtures" / "distribution" / "manifest_v5.json"
PROFILES = {"checkpoint", "core", "malformed", "property"}
SCHEMA = "nostr_automerge.fixture_distribution.v5"
SIGNED_SCHEMA = "nostr_automerge.signed_scenario.v2"


def fail(message: str) -> None:
    raise ValueError(message)


def safe_path(root: Path, relative: object) -> Path:
    if not isinstance(relative, str):
        fail("distribution path is not a string")
    pure = PurePosixPath(relative)
    if pure.is_absolute() or not pure.parts or any(part in {"", ".", ".."} for part in pure.parts):
        fail("distribution path escapes repository")
    return root.joinpath(*pure.parts)


def validate(manifest: dict[str, object], root: Path = ROOT) -> None:
    expected_keys = {
        "distribution_schema", "distribution_id", "protocol_revision", "status",
        "profiles", "fixtures", "files", "requirements_sha256", "authority_sha256",
        "supersedes",
    }
    if set(manifest) != expected_keys or manifest["distribution_schema"] != SCHEMA:
        fail("invalid distribution v5 shape")
    if manifest["status"] != "canonical_signed_neutral_corpus":
        fail("distribution is not canonical signed neutral corpus")
    for field, relative in [
        ("requirements_sha256", "spec/requirements.json"),
        ("authority_sha256", "spec/NOSTR_AUTOMERGE_V1_SPEC.md"),
    ]:
        if manifest[field] != hashlib.sha256(safe_path(root, relative).read_bytes()).hexdigest():
            fail(f"stale {field}")
    if manifest["supersedes"] != "fixtures/distribution/manifest_v4.json":
        fail("distribution does not supersede v4")
    profiles = manifest["profiles"]
    if not isinstance(profiles, dict) or set(profiles) != PROFILES:
        fail("fixture profiles are incomplete")

    file_entries = manifest["files"]
    if not isinstance(file_entries, list):
        fail("distribution files must be an array")
    paths = [item.get("path") for item in file_entries if isinstance(item, dict)]
    if len(paths) != len(file_entries) or paths != sorted(paths, key=str.encode) or len(paths) != len(set(paths)):
        fail("distribution files are duplicated or noncanonical")
    hashes: dict[str, str] = {}
    for item in file_entries:
        relative = item["path"]
        path = safe_path(root, relative)
        if not path.is_file():
            fail(f"missing distribution file: {relative}")
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if item.get("sha256") != actual:
            fail(f"distribution checksum mismatch: {relative}")
        hashes[relative] = actual

    fixtures = manifest["fixtures"]
    if not isinstance(fixtures, list) or not fixtures:
        fail("distribution contains no fixtures")
    fixture_ids: list[str] = []
    assigned_paths: set[str] = set()
    for entry in fixtures:
        if not isinstance(entry, dict) or set(entry) != {
            "fixture_id", "profile", "requirements", "metadata_path", "input_paths", "expected_path"
        }:
            fail("invalid fixture entry shape")
        fixture_id = entry["fixture_id"]
        profile = entry["profile"]
        if not isinstance(fixture_id, str) or profile not in PROFILES:
            fail("invalid fixture identity or profile")
        fixture_ids.append(fixture_id)
        metadata_path = safe_path(root, entry["metadata_path"])
        input_paths = entry["input_paths"]
        if not isinstance(input_paths, list) or len(input_paths) != 1:
            fail("signed fixture must have exactly one input")
        expected_path = safe_path(root, entry["expected_path"])
        relative_artifacts = [entry["metadata_path"], *input_paths, entry["expected_path"]]
        if any(relative not in hashes for relative in relative_artifacts):
            fail("fixture artifact is absent from file checksums")
        assigned_paths.update(relative_artifacts)

        metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
        input_value = json.loads(safe_path(root, input_paths[0]).read_text(encoding="utf-8"))
        if metadata.get("fixture_id") != fixture_id or metadata.get("requirements") != entry["requirements"]:
            fail("fixture entry disagrees with metadata")
        if input_value.get("scenario_schema") != SIGNED_SCHEMA or not isinstance(input_value.get("raw_events"), list):
            fail("fixture input is not signed scenario v2")
        forbidden = {"operations", "valid", "selected", "accepted", "excluded", "controls", "changes"}
        if forbidden.intersection(input_value):
            fail("fixture input contains abstract protocol truth")
        if not expected_path.is_file():
            fail("fixture expected report is missing")

    if fixture_ids != sorted(fixture_ids, key=str.encode) or len(fixture_ids) != len(set(fixture_ids)):
        fail("fixture ids are duplicated or noncanonical")
    assigned = [fixture_id for values in profiles.values() for fixture_id in values]
    if len(assigned) != len(set(assigned)) or set(assigned) != set(fixture_ids):
        fail("profile membership is missing or duplicated")
    for entry in fixtures:
        if entry["fixture_id"] not in profiles[entry["profile"]]:
            fail("fixture profile entry disagrees with profile membership")
    required_schemas = {
        "fixtures/schema/fixture.schema.json", "fixtures/schema/report.schema.json",
        "fixtures/schema/scenario.schema.json", "fixtures/schema/scenario_v2.schema.json",
        "fixtures/schema/fixture.schema.v5.json", "fixtures/schema/report.schema.v2.json",
        "fixtures/schema/scenario_v3.schema.json",
    }
    if not required_schemas.issubset(hashes):
        fail("required fixture schemas are absent")


def expect_failure(manifest: dict[str, object], fragment: str) -> None:
    try:
        validate(manifest)
    except ValueError as error:
        if fragment not in str(error):
            raise AssertionError(f"unexpected validation failure: {error}") from error
    else:
        raise AssertionError(f"mutation unexpectedly passed: {fragment}")


def self_test(manifest: dict[str, object]) -> None:
    tampered = copy.deepcopy(manifest)
    tampered["files"][0]["sha256"] = "0" * 64
    expect_failure(tampered, "checksum mismatch")
    missing = copy.deepcopy(manifest)
    missing["files"][0]["path"] = "fixtures/schema/missing.schema.json"
    missing["files"].sort(key=lambda item: item["path"].encode())
    expect_failure(missing, "missing distribution file")
    duplicate = copy.deepcopy(manifest)
    duplicate["fixtures"].append(copy.deepcopy(duplicate["fixtures"][0]))
    expect_failure(duplicate, "fixture ids are duplicated")
    escaped = copy.deepcopy(manifest)
    escaped["files"][0]["path"] = "../escape"
    escaped["files"].sort(key=lambda item: item["path"].encode())
    expect_failure(escaped, "escapes repository")
    omitted = copy.deepcopy(manifest)
    fixture_id = omitted["fixtures"][0]["fixture_id"]
    profile = omitted["fixtures"][0]["profile"]
    omitted["profiles"][profile].remove(fixture_id)
    expect_failure(omitted, "profile membership is missing")


def main() -> int:
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    validate(manifest)
    if sys.argv[1:] == ["--self-test"]:
        self_test(manifest)
    elif sys.argv[1:]:
        raise SystemExit("usage: validate_fixture_manifest.py [--self-test]")
    print(f"PASS: signed fixture distribution v5 ({len(manifest['fixtures'])} fixtures)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
