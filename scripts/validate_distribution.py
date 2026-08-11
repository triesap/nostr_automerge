#!/usr/bin/env python3
"""Validate the immutable language-neutral fixture distribution."""

from __future__ import annotations

import hashlib
import json
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "fixtures" / "distribution" / "manifest.json"
SCHEMA = "nostr_automerge.fixture_distribution.v2"
PROFILES = {"core", "checkpoint", "malformed", "property"}
SIGNED_SCHEMA = "nostr_automerge.signed_scenario.v2"


def validate_signed_input(value: object) -> None:
    if not isinstance(value, dict) or value.get("scenario_schema") != SIGNED_SCHEMA:
        raise ValueError("normative fixture input is not signed scenario v2")
    if not isinstance(value.get("raw_events"), list):
        raise ValueError("normative fixture has no raw signed events")
    forbidden = {"operations", "valid", "selected", "accepted", "excluded", "controls", "changes"}
    if forbidden.intersection(value):
        raise ValueError("normative fixture contains abstract protocol truth")


def main() -> None:
    if sys.argv[1:] == ["--self-test"]:
        self_test()
        print("PASS: abstract normative fixture rejection")
        return
    if sys.argv[1:]:
        raise SystemExit("usage: validate_distribution.py [--self-test]")
    validate(ROOT, MANIFEST)


def _validate_manifest(root: Path, manifest: dict[str, object]) -> None:
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
        path = root / relative
        if not path.is_file():
            raise SystemExit(f"missing distribution file: {relative}")
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != item.get("sha256"):
            raise SystemExit(f"distribution checksum mismatch: {relative}")
        if relative.endswith(".fixture.json"):
            fixture = json.loads(path.read_text(encoding="utf-8"))
            fixture_ids.add(fixture["fixture_id"])
            inputs = fixture.get("inputs", [])
            if len(inputs) != 1 or inputs[0].get("name") != "signed_scenario":
                raise SystemExit("normative fixture must have one signed_scenario input")
            input_path = path.parent / inputs[0]["path"]
            try:
                validate_signed_input(json.loads(input_path.read_text(encoding="utf-8")))
            except (OSError, json.JSONDecodeError, ValueError) as error:
                raise SystemExit(f"abstract or invalid normative input: {relative}") from error

    assigned = [
        fixture_id
        for profile in manifest["profiles"].values()
        for fixture_id in profile
    ]
    if len(assigned) != len(set(assigned)):
        raise SystemExit("fixture is assigned to more than one interop profile")
    if set(assigned) != fixture_ids:
        raise SystemExit("profiles do not cover exactly the distributed fixtures")


def validate(root: Path, manifest_path: Path) -> None:
    _validate_manifest(root, json.loads(manifest_path.read_text(encoding="utf-8")))


def self_test() -> None:
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary)
        fixture_dir = root / "fixtures" / "interop"
        fixture_dir.mkdir(parents=True)
        abstract = {"operations": [{"valid": True, "selected": True}]}
        input_path = fixture_dir / "interop_core_001.input.json"
        input_path.write_text(json.dumps(abstract), encoding="utf-8")
        fixture = {
            "fixture_id": "interop_core_001",
            "inputs": [{"name": "signed_scenario", "path": input_path.name}],
        }
        fixture_path = fixture_dir / "interop_core_001.fixture.json"
        fixture_path.write_text(json.dumps(fixture), encoding="utf-8")
        manifest = {
            "distribution_schema": SCHEMA,
            "profiles": {"core": ["interop_core_001"], "checkpoint": [], "malformed": [], "property": []},
            "files": [
                {"path": fixture_path.relative_to(root).as_posix(), "sha256": hashlib.sha256(fixture_path.read_bytes()).hexdigest()},
                {"path": input_path.relative_to(root).as_posix(), "sha256": hashlib.sha256(input_path.read_bytes()).hexdigest()},
            ],
        }
        try:
            _validate_manifest(root, manifest)
        except SystemExit as error:
            if "abstract or invalid normative input" not in str(error):
                raise AssertionError("unexpected self-test rejection") from error
        else:
            raise AssertionError("abstract interop_core_001 input unexpectedly passed")


if __name__ == "__main__":
    main()
