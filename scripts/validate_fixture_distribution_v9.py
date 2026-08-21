#!/usr/bin/env python3
"""Validate the signed-v9 distribution and its byte-preserving v8 transition."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "fixtures/distribution/manifest_v9.json"
BASE = ROOT / "fixtures/distribution/manifest_v8.json"
PROFILES = {"checkpoint", "core", "malformed", "property", "resource"}
V9_REQUIREMENTS = {
    "invalid_change_under_valid_noncanonical_control": ["NCRDT-BRANCH-003", "NCRDT-BRANCH-004", "NCRDT-DISPOSITION-004"],
    "pending_change_under_valid_noncanonical_control": ["NCRDT-BRANCH-003", "NCRDT-BRANCH-004"],
    "equivocation_excluded_change_under_valid_noncanonical_control": ["NCRDT-BRANCH-003", "NCRDT-BRANCH-004"],
    "noncanonical_bad_start_op_is_invalid": ["NCRDT-BRANCH-004"],
    "same_hash_valid_and_noncanonical_invalid_carriers": ["NCRDT-BRANCH-004", "NCRDT-DISPOSITION-004", "NCRDT-DISPOSITION-005"],
    "unrelated_control_flood_exact_budget": ["NCRDT-RESOURCE-011", "NCRDT-SCOPE-007"],
    "unrelated_control_flood_does_not_change_digest": ["NCRDT-SCOPE-007"],
    "change_carrier_mixed_outcomes": ["NCRDT-DISPOSITION-004", "NCRDT-DISPOSITION-005"],
    "change_carrier_event_order_stability": ["NCRDT-CONF-009", "NCRDT-DISPOSITION-005"],
}
REQUIRED_SCHEMAS = {
    "fixtures/schema/distribution.schema.v9.json",
    "fixtures/schema/fixture.schema.v8.json",
    "fixtures/schema/interop_attestation_v3.schema.json",
}


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--allow-locked-transition", action="store_true")
    return parser.parse_args()


def fail(message: str) -> None:
    raise SystemExit(message)


def main() -> None:
    args = parse_args()
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    base = json.loads(BASE.read_text(encoding="utf-8"))
    if manifest.get("distribution_schema") != "nostr_automerge.fixture_distribution.v9":
        fail("invalid fixture distribution schema")
    if manifest.get("distribution_id") != "draft_2026_08_signed_neutral_9":
        fail("invalid fixture distribution id")
    if manifest.get("supersedes") != "fixtures/distribution/manifest_v8.json":
        fail("fixture distribution does not supersede v8")
    if manifest.get("base_manifest_sha256") != digest(BASE):
        fail("v8 base manifest identity is stale")
    if manifest.get("target_fixture_count") != 180:
        fail("signed-v9 target count is not exact")
    if manifest.get("requirements_sha256") != digest(ROOT / "spec/requirements.json"):
        fail("fixture distribution requirements identity is stale")
    for field, relative in (
        ("authority_sha256", "spec/NIP_DRAFT.md"),
        ("companion_sha256", "spec/NOSTR_AUTOMERGE_V1_SPEC.md"),
    ):
        if manifest.get(field) != digest(ROOT / relative):
            fail(f"fixture distribution {field} is stale")

    base_entries = {item["fixture_id"]: item for item in base["fixtures"]}
    entries = manifest.get("fixtures")
    if not isinstance(entries, list) or entries != sorted(
        entries, key=lambda item: item["fixture_id"].encode()
    ):
        fail("fixture entries are not canonically ordered")
    current = {item["fixture_id"]: item for item in entries}
    if len(current) != len(entries):
        fail("duplicate fixture id")
    if manifest.get("preserved_v8_fixture_count") != len(base_entries) or len(base_entries) != 171:
        fail("v8 fixture count is not preserved")
    for identifier, entry in base_entries.items():
        if current.get(identifier) != entry:
            fail(f"v8 fixture entry changed: {identifier}")
    base_files = {
        item["path"]: item["sha256"]
        for item in base["files"]
        if item["path"].startswith("fixtures/v1_draft/scenarios/")
    }
    current_files = {item["path"]: item["sha256"] for item in manifest.get("files", [])}
    for relative, checksum in base_files.items():
        if current_files.get(relative) != checksum:
            fail(f"v8 signed artifact changed: {relative}")
    if not REQUIRED_SCHEMAS.issubset(current_files):
        fail("signed-v9 schema set is incomplete")
    for relative, checksum in current_files.items():
        path = ROOT / relative
        if not path.is_file() or digest(path) != checksum:
            fail(f"missing or stale distribution file: {relative}")

    if manifest.get("v9_fixtures") != list(V9_REQUIREMENTS):
        fail("signed-v9 fixture inventory changed")
    missing = sorted(set(V9_REQUIREMENTS) - set(current), key=str.encode)
    if manifest.get("missing_v8_fixtures") != [] or manifest.get("missing_v9_fixtures") != missing:
        fail("signed-v9 missing fixture inventory is stale")
    complete = manifest.get("complete") is True
    if complete:
        if manifest.get("status") != "canonical_signed_neutral_corpus":
            fail("complete signed-v9 status is invalid")
        if len(entries) != 180 or missing:
            fail("signed-v9 distribution is incomplete")
    elif not args.allow_locked_transition:
        fail("signed-v9 distribution remains in a locked transition")
    elif (
        manifest.get("status") != "locked_transition"
        or len(entries) != 171
        or set(missing) != set(V9_REQUIREMENTS)
    ):
        fail("invalid signed-v9 locked transition")

    assigned = [identifier for values in manifest.get("profiles", {}).values() for identifier in values]
    if set(manifest.get("profiles", {})) != PROFILES or len(assigned) != len(set(assigned)):
        fail("signed-v9 profiles are invalid")
    if set(assigned) != set(current):
        fail("signed-v9 profiles do not cover the distribution")
    known_requirements = {
        row["id"]
        for row in json.loads((ROOT / "spec/requirements.json").read_text())["requirements"]
    }
    for identifier in set(V9_REQUIREMENTS) & set(current):
        entry = current[identifier]
        if entry.get("requirements") != V9_REQUIREMENTS[identifier]:
            fail(f"incorrect remediation-v8 requirements: {identifier}")
        if not set(entry["requirements"]).issubset(known_requirements):
            fail(f"unknown remediation-v8 requirement: {identifier}")
        scenario = json.loads((ROOT / entry["input_paths"][0]).read_text())
        if scenario.get("scenario_schema") != "nostr_automerge.signed_scenario.v2":
            fail(f"fixture is not a signed scenario: {identifier}")
        if scenario.get("requirements") != entry["requirements"]:
            fail(f"scenario requirements differ from metadata: {identifier}")
        if any(key in scenario for key in ("operations", "valid", "selected", "accepted")):
            fail(f"fixture contains abstract protocol truth: {identifier}")
    state = "complete" if complete else "locked transition"
    print(f"PASS: fixture distribution v9 {state} ({len(entries)} signed fixtures)")
    print("- preserved_v8_fixtures=171")
    print(f"- missing_v9_fixtures={len(missing)}")


if __name__ == "__main__":
    main()
