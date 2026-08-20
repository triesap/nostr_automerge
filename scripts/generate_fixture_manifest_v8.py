#!/usr/bin/env python3
"""Generate the canonical remediation-v7 signed fixture distribution v8."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "fixtures"
SCENARIOS = FIXTURES / "v1_draft" / "scenarios"
OUTPUT = FIXTURES / "distribution" / "manifest_v8.json"
REQUIREMENTS = ROOT / "spec" / "requirements.json"
AUTHORITY = ROOT / "spec" / "NIP_DRAFT.md"
COMPANION = ROOT / "spec" / "NOSTR_AUTOMERGE_V1_SPEC.md"
TARGET_COUNT = 171
PROFILE_BY_FAMILY = {
    "checkpoint": "checkpoint",
    "checkpoints": "checkpoint",
    "projection": "property",
    "versioning": "malformed",
}
V8_FIXTURES = [
    "change_references_invalid_noncanonical_child",
    "manifest_references_invalid_noncanonical_child",
    "noncanonical_child_excluded_base_head",
    "noncanonical_child_invalid_base_head",
    "noncanonical_child_pending_base_head",
    "noncanonical_grandchild_invalid_parent_epoch",
    "cross_coordinate_descriptor_reference_isolated",
    "foreign_change_references_target_control",
    "foreign_chunk_excluded_from_target_digest",
    "foreign_chunk_references_target_descriptor",
    "foreign_claim_flood_exact_budget",
    "unrelated_valid_checkpoints_exact_budget",
    "interrupted_finalization_forfeiture",
    "parent_propagation_exact_budget",
]


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    prior = json.loads((FIXTURES / "distribution" / "manifest_v7.json").read_text())
    prior_ids = {item["fixture_id"] for item in prior["fixtures"]}
    entries: list[dict[str, object]] = []
    paths = set(FIXTURES.joinpath("schema").glob("*.json"))
    paths.update((REQUIREMENTS, AUTHORITY, COMPANION))
    profiles = {name: [] for name in ("checkpoint", "core", "malformed", "property")}
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
        profile = PROFILE_BY_FAMILY.get(metadata_path.parent.name, "core")
        inputs = [metadata_path.parent / item["path"] for item in metadata["inputs"]]
        expected = metadata_path.parent / metadata["expected"]["report_path"]
        fixture_paths = [metadata_path, *inputs, expected]
        if any(not path.is_file() for path in fixture_paths):
            raise AssertionError(f"fixture has a missing artifact: {fixture_id}")
        paths.update(fixture_paths)
        profiles[profile].append(fixture_id)
        entries.append({
            "fixture_id": fixture_id,
            "profile": profile,
            "requirements": requirements,
            "metadata_path": metadata_path.relative_to(ROOT).as_posix(),
            "input_paths": [path.relative_to(ROOT).as_posix() for path in inputs],
            "expected_path": expected.relative_to(ROOT).as_posix(),
        })
    missing_prior = sorted(prior_ids - seen, key=str.encode)
    missing_v8 = sorted(set(V8_FIXTURES) - seen, key=str.encode)
    if len(entries) > TARGET_COUNT:
        raise AssertionError(f"signed fixture count exceeds {TARGET_COUNT}: {len(entries)}")
    entries.sort(key=lambda item: str(item["fixture_id"]).encode())
    for fixture_ids in profiles.values():
        fixture_ids.sort(key=str.encode)
    files = [
        {"path": path.relative_to(ROOT).as_posix(), "sha256": sha256(path)}
        for path in sorted(paths, key=lambda path: path.relative_to(ROOT).as_posix().encode())
    ]
    complete = len(entries) == TARGET_COUNT and not missing_prior and not missing_v8
    manifest = {
        "distribution_schema": "nostr_automerge.fixture_distribution.v8",
        "distribution_id": "draft_2026_08_signed_neutral_8",
        "protocol_revision": "draft_2026_08",
        "status": "canonical_signed_neutral_corpus" if complete else "incomplete_fail_closed",
        "target_fixture_count": TARGET_COUNT,
        "fixture_count": len(entries),
        "complete": complete,
        "missing_v7_fixtures": missing_prior,
        "missing_v8_fixtures": missing_v8,
        "requirements_sha256": sha256(REQUIREMENTS),
        "authority_sha256": sha256(AUTHORITY),
        "companion_sha256": sha256(COMPANION),
        "supersedes": "fixtures/distribution/manifest_v7.json",
        "v8_fixtures": V8_FIXTURES,
        "profiles": profiles,
        "fixtures": entries,
        "files": files,
    }
    OUTPUT.write_text(
        json.dumps(manifest, sort_keys=True, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )
    if not complete:
        raise AssertionError(
            f"signed distribution incomplete: {len(entries)}/{TARGET_COUNT}; "
            f"missing_v7={missing_prior}; missing_v8={missing_v8}"
        )
    print(f"PASS: generated fixture distribution v8 ({len(entries)} signed fixtures)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
