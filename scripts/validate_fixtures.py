#!/usr/bin/env python3
"""Validate language-neutral fixture metadata and referenced files."""

from __future__ import annotations

import hashlib
import json
import re
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
FIXTURE_ROOT = ROOT / "fixtures"
FIXTURE_ID = re.compile(r"^[a-z0-9][a-z0-9_]{2,127}$")
INPUT_NAME = re.compile(r"^[a-z][a-z0-9_]{0,63}$")
SHA256 = re.compile(r"^[0-9a-f]{64}$")
REQUIRED_TOP_LEVEL = {
    "fixture_schema", "fixture_id", "revision", "requirements", "seed",
    "provenance", "inputs", "expected",
}
ABSTRACT_TRUTH_FIELDS = {
    "accepted",
    "changes",
    "controls",
    "excluded",
    "selected",
    "synthetic_dependencies",
    "valid",
}


class FixtureError(Exception):
    """A stable fixture validation error."""


def load_json(path: Path) -> dict[str, Any]:
    """Load a JSON object."""

    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise FixtureError("json_not_object")
    return value


def safe_relative(base: Path, value: object) -> Path:
    """Resolve a safe repository-relative fixture artifact path."""

    if not isinstance(value, str) or not value or Path(value).is_absolute():
        raise FixtureError("invalid_relative_path")
    if ".." in Path(value).parts:
        raise FixtureError("path_traversal")
    resolved = base / value
    try:
        resolved.resolve().relative_to(base.resolve())
    except ValueError as error:
        raise FixtureError("path_traversal") from error
    return resolved


def digest(path: Path) -> str:
    """Return a file SHA-256 digest."""

    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate(metadata: dict[str, Any], *, base: Path, resolve_files: bool) -> None:
    """Validate one fixture metadata object."""

    if set(metadata) != REQUIRED_TOP_LEVEL:
        raise FixtureError("invalid_fixture_fields")
    if metadata["fixture_schema"] != "nostr_automerge.fixture.v1":
        raise FixtureError("invalid_fixture_schema")
    if not isinstance(metadata["fixture_id"], str) or FIXTURE_ID.fullmatch(
        metadata["fixture_id"]
    ) is None:
        raise FixtureError("invalid_fixture_id")
    if metadata["revision"] != "draft_2026_08":
        raise FixtureError("invalid_revision")
    seed = metadata["seed"]
    if seed is not None and (not isinstance(seed, int) or isinstance(seed, bool) or seed < 0):
        raise FixtureError("invalid_seed")

    requirements = metadata["requirements"]
    if not isinstance(requirements, list) or not requirements:
        raise FixtureError("invalid_requirements")
    if len(set(requirements)) != len(requirements):
        raise FixtureError("duplicate_requirement")
    registry_text = (ROOT / "spec/requirements.json").read_text(encoding="utf-8")
    for identifier in requirements:
        if not isinstance(identifier, str) or f'"id": "{identifier}"' not in registry_text:
            raise FixtureError("unknown_requirement")

    provenance = metadata["provenance"]
    required_provenance = {"generator", "generator_revision", "created_at", "source_versions"}
    if not isinstance(provenance, dict) or set(provenance) != required_provenance:
        raise FixtureError("invalid_provenance")
    for field in ("generator", "generator_revision", "created_at"):
        if not isinstance(provenance[field], str) or not provenance[field]:
            raise FixtureError("invalid_provenance")
    if not isinstance(provenance["source_versions"], dict):
        raise FixtureError("invalid_provenance")

    inputs = metadata["inputs"]
    if not isinstance(inputs, list) or not inputs:
        raise FixtureError("invalid_inputs")
    names: set[str] = set()
    paths: set[str] = set()
    for item in inputs:
        if not isinstance(item, dict) or not {"name", "path", "sha256"} <= set(item):
            raise FixtureError("invalid_input")
        if set(item) - {"name", "path", "sha256", "media_type"}:
            raise FixtureError("invalid_input")
        name = item["name"]
        if not isinstance(name, str) or INPUT_NAME.fullmatch(name) is None or name in names:
            raise FixtureError("invalid_input_name")
        names.add(name)
        path_value = item["path"]
        if not isinstance(path_value, str) or path_value in paths:
            raise FixtureError("duplicate_input_path")
        paths.add(path_value)
        path = safe_relative(base, path_value)
        expected_hash = item["sha256"]
        if not isinstance(expected_hash, str) or SHA256.fullmatch(expected_hash) is None:
            raise FixtureError("invalid_input_hash")
        if resolve_files and (not path.is_file() or digest(path) != expected_hash):
            raise FixtureError("input_hash_mismatch")

    expected = metadata["expected"]
    if not isinstance(expected, dict) or set(expected) != {"report_path", "sha256"}:
        raise FixtureError("invalid_expected")
    report = safe_relative(base, expected["report_path"])
    expected_hash = expected["sha256"]
    if not isinstance(expected_hash, str) or SHA256.fullmatch(expected_hash) is None:
        raise FixtureError("invalid_expected_hash")
    if resolve_files and (not report.is_file() or digest(report) != expected_hash):
        raise FixtureError("expected_hash_mismatch")


def validate_signed_scenario(value: dict[str, Any]) -> None:
    """Reject caller-declared protocol truth from signed scenario v2 inputs."""

    if value.get("scenario_schema") != "nostr_automerge.signed_scenario.v2":
        raise FixtureError("invalid_signed_scenario_schema")
    input_fields = {key: item for key, item in value.items() if key != "expected_report"}
    pending: list[object] = [input_fields]
    while pending:
        item = pending.pop()
        if isinstance(item, dict):
            if ABSTRACT_TRUTH_FIELDS & set(item):
                raise FixtureError("abstract_protocol_truth")
            pending.extend(item.values())
        elif isinstance(item, list):
            pending.extend(item)


def main() -> int:
    """Validate schema metadata, examples, and required negative behavior."""

    schema = load_json(FIXTURE_ROOT / "schema/fixture.schema.json")
    if schema.get("$schema") != "https://json-schema.org/draft/2020-12/schema":
        raise AssertionError("fixture schema must use JSON Schema 2020-12")

    paths = sorted(FIXTURE_ROOT.rglob("*.fixture.json"))
    if not paths:
        raise AssertionError("fixture corpus is empty")
    for path in paths:
        validate(load_json(path), base=path.parent, resolve_files=True)
    signed_paths = sorted(FIXTURE_ROOT.rglob("*.input.json"))
    signed_count = 0
    for path in signed_paths:
        value = load_json(path)
        if value.get("scenario_schema") == "nostr_automerge.signed_scenario.v2":
            validate_signed_scenario(value)
            signed_count += 1

    if sys.argv[1:] == ["--self-test"]:
        candidate = {
            "scenario_schema": "nostr_automerge.signed_scenario.v2",
            "valid": True,
            "expected_report": {},
        }
        try:
            validate_signed_scenario(candidate)
        except FixtureError as error:
            if str(error) != "abstract_protocol_truth":
                raise
        else:
            raise AssertionError("abstract validity input unexpectedly passed")
    elif sys.argv[1:]:
        raise SystemExit("usage: validate_fixtures.py [--self-test]")

    candidate = load_json(paths[0])
    candidate["inputs"][0]["path"] = "../escape.json"
    try:
        validate(candidate, base=paths[0].parent, resolve_files=False)
    except FixtureError as error:
        if str(error) != "path_traversal":
            raise
    else:
        raise AssertionError("fixture path traversal unexpectedly passed")

    print("PASS: language-neutral fixture metadata")
    print(f"- fixtures={len(paths)}")
    print("- path_traversal=reject")
    print("- checksums=pass")
    print(f"- signed_scenarios={signed_count}")
    print("- abstract_protocol_truth=reject")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
