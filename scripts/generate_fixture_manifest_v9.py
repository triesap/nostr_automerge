#!/usr/bin/env python3
"""Generate the checksum-bound remediation-v8 signed distribution v9."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "fixtures"
SCENARIOS = FIXTURES / "v1_draft" / "scenarios"
OUTPUT = FIXTURES / "distribution" / "manifest_v9.json"
BASE = FIXTURES / "distribution" / "manifest_v8.json"
REQUIREMENTS = ROOT / "spec" / "requirements.json"
AUTHORITY = ROOT / "spec" / "NIP_DRAFT.md"
COMPANION = ROOT / "spec" / "NOSTR_AUTOMERGE_V1_SPEC.md"
CONFORMANCE = ROOT / "spec" / "CONFORMANCE.md"
TARGET_COUNT = 180
BASE_COUNT = 171
BASE_SIGNED_EVENTS_SHA256 = "50313da01a212e25fcab49e27882d5e9ed11110cfe1ab1b69d6771f83f6e8844"
PROFILE_BY_FAMILY = {
    "checkpoint": "checkpoint",
    "checkpoints": "checkpoint",
    "projection": "property",
    "versioning": "malformed",
}
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


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def signed_event_set_sha256(entries: list[dict[str, object]]) -> str:
    digest = hashlib.sha256()
    for entry in entries:
        identifier = str(entry["fixture_id"]).encode()
        digest.update(len(identifier).to_bytes(4, "big"))
        digest.update(identifier)
        scenario = json.loads((ROOT / str(entry["input_paths"][0])).read_text())
        for raw in scenario["raw_events"]:
            encoding = raw["encoding"].encode()
            data = raw["data"].encode()
            digest.update(len(encoding).to_bytes(4, "big"))
            digest.update(encoding)
            digest.update(len(data).to_bytes(8, "big"))
            digest.update(data)
    return digest.hexdigest()


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--allow-locked-transition", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    base = json.loads(BASE.read_text(encoding="utf-8"))
    base_entries = {item["fixture_id"]: item for item in base["fixtures"]}
    entries: list[dict[str, object]] = []
    paths = set(FIXTURES.joinpath("schema").glob("*.json"))
    paths.update((REQUIREMENTS, AUTHORITY, COMPANION, CONFORMANCE))
    profiles = {name: [] for name in ("checkpoint", "core", "malformed", "property", "resource")}
    seen: set[str] = set()
    known_requirements = {
        row["id"] for row in json.loads(REQUIREMENTS.read_text())["requirements"]
    }
    for metadata_path in sorted(SCENARIOS.rglob("*.fixture.json")):
        metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
        fixture_id = metadata["fixture_id"]
        if fixture_id in seen:
            raise AssertionError(f"duplicate fixture id: {fixture_id}")
        seen.add(fixture_id)
        requirements = metadata["requirements"]
        if requirements != sorted(set(requirements), key=str.encode):
            raise AssertionError(f"noncanonical requirement list: {fixture_id}")
        if not set(requirements).issubset(known_requirements):
            raise AssertionError(f"unknown fixture requirement: {fixture_id}")
        if fixture_id in V9_REQUIREMENTS and requirements != V9_REQUIREMENTS[fixture_id]:
            raise AssertionError(f"incorrect remediation-v8 requirements: {fixture_id}")
        profile = (
            "resource"
            if fixture_id == "unrelated_control_flood_exact_budget"
            else PROFILE_BY_FAMILY.get(metadata_path.parent.name, "core")
        )
        inputs = [metadata_path.parent / item["path"] for item in metadata["inputs"]]
        expected = metadata_path.parent / metadata["expected"]["report_path"]
        fixture_paths = [metadata_path, *inputs, expected]
        if any(not path.is_file() for path in fixture_paths):
            raise AssertionError(f"fixture has a missing artifact: {fixture_id}")
        paths.update(fixture_paths)
        profiles[profile].append(fixture_id)
        entries.append(
            {
                "fixture_id": fixture_id,
                "profile": profile,
                "requirements": requirements,
                "metadata_path": metadata_path.relative_to(ROOT).as_posix(),
                "input_paths": [path.relative_to(ROOT).as_posix() for path in inputs],
                "expected_path": expected.relative_to(ROOT).as_posix(),
            }
        )
    entries.sort(key=lambda item: str(item["fixture_id"]).encode())
    for fixture_ids in profiles.values():
        fixture_ids.sort(key=str.encode)
    current = {item["fixture_id"]: item for item in entries}
    missing_base = sorted(set(base_entries) - set(current), key=str.encode)
    changed_base = sorted(
        (identifier for identifier, item in base_entries.items() if current.get(identifier) != item),
        key=str.encode,
    )
    missing_v9 = sorted(set(V9_REQUIREMENTS) - seen, key=str.encode)
    if missing_base or changed_base:
        raise AssertionError(
            f"v8 fixture entries changed: missing={missing_base}; changed={changed_base}"
        )
    base_files = {item["path"]: item["sha256"] for item in base["files"]}
    intentional_v8_report_changes = sorted(
        (
            identifier
            for identifier, entry in base_entries.items()
            if base_files.get(entry["expected_path"])
            != sha256(ROOT / str(entry["expected_path"]))
        ),
        key=str.encode,
    )
    preserved_entries = [current[identifier] for identifier in sorted(base_entries, key=str.encode)]
    signed_events_sha256 = signed_event_set_sha256(preserved_entries)
    if signed_events_sha256 != BASE_SIGNED_EVENTS_SHA256:
        raise AssertionError("v8 signed event set changed")
    if len(entries) > TARGET_COUNT:
        raise AssertionError(f"signed fixture count exceeds {TARGET_COUNT}: {len(entries)}")
    files = [
        {"path": path.relative_to(ROOT).as_posix(), "sha256": sha256(path)}
        for path in sorted(paths, key=lambda path: path.relative_to(ROOT).as_posix().encode())
    ]
    complete = len(entries) == TARGET_COUNT and not missing_v9
    locked_transition = (
        args.allow_locked_transition
        and len(entries) == BASE_COUNT
        and set(missing_v9) == set(V9_REQUIREMENTS)
    )
    if not complete and not locked_transition:
        raise AssertionError(
            f"signed distribution incomplete: {len(entries)}/{TARGET_COUNT}; missing_v9={missing_v9}"
        )
    manifest = {
        "distribution_schema": "nostr_automerge.fixture_distribution.v9",
        "distribution_id": "draft_2026_08_signed_neutral_9",
        "protocol_revision": "draft_2026_08",
        "status": "canonical_signed_neutral_corpus" if complete else "locked_transition",
        "target_fixture_count": TARGET_COUNT,
        "fixture_count": len(entries),
        "complete": complete,
        "base_manifest_sha256": sha256(BASE),
        "preserved_v8_fixture_count": len(base_entries),
        "preserved_v8_signed_events_sha256": signed_events_sha256,
        "missing_v8_fixtures": missing_base,
        "missing_v9_fixtures": missing_v9,
        "intentional_v8_report_changes": intentional_v8_report_changes,
        "requirements_sha256": sha256(REQUIREMENTS),
        "authority_sha256": sha256(AUTHORITY),
        "companion_sha256": sha256(COMPANION),
        "conformance_sha256": sha256(CONFORMANCE),
        "supersedes": "fixtures/distribution/manifest_v8.json",
        "v9_fixtures": list(V9_REQUIREMENTS),
        "profiles": profiles,
        "fixtures": entries,
        "files": files,
    }
    OUTPUT.write_text(
        json.dumps(manifest, sort_keys=True, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )
    state = "complete" if complete else "locked transition"
    print(f"PASS: generated fixture distribution v9 {state} ({len(entries)} signed fixtures)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
