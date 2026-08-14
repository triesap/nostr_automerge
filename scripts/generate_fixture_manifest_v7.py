#!/usr/bin/env python3
"""Generate the canonical remediation-v6 signed fixture distribution v7."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "fixtures"
SCENARIOS = FIXTURES / "v1_draft" / "scenarios"
OUTPUT = FIXTURES / "distribution" / "manifest_v7.json"
REQUIREMENTS = ROOT / "spec" / "requirements.json"
AUTHORITY = ROOT / "spec" / "NIP_DRAFT.md"
COMPANION = ROOT / "spec" / "NOSTR_AUTOMERGE_V1_SPEC.md"
TARGET_COUNT = 157
PROFILE_BY_FAMILY = {
    "checkpoint": "checkpoint",
    "checkpoints": "checkpoint",
    "projection": "property",
    "versioning": "malformed",
}
V6_FIXTURES = [
    "change_references_unsupported_control",
    "unauthorized_change_under_noncanonical_control",
    "change_under_terminal_control",
    "pending_and_noncanonical_claims_same_hash",
    "pending_and_invalid_claims_same_hash",
    "pruned_and_pending_claims_same_hash",
    "equivocation_excluded_and_pending_claims_same_hash",
    "child_references_unsupported_parent_control",
    "child_references_wrong_kind_parent",
    "child_references_static_invalid_parent",
    "child_references_wrong_coordinate_parent",
    "child_base_head_is_known_invalid",
    "child_base_head_is_known_excluded",
    "child_base_head_is_known_unsupported",
    "child_base_head_is_known_other_control",
    "descendant_of_pending_control_is_pending",
    "descendant_of_invalid_control_is_invalid",
    "deep_noncanonical_branch_control_validation",
    "dependency_known_through_other_control",
    "dependency_known_through_unsupported_control",
    "dependency_known_through_prior_equivocation_exclusion",
    "dependency_known_through_invalid_control",
    "checkpoint_descriptor_references_pending_control",
    "checkpoint_descriptor_references_wrong_kind_control",
    "checkpoint_descriptor_references_wrong_coordinate_control",
    "checkpoint_descriptor_references_unsupported_control",
    "checkpoint_descriptor_references_invalid_control",
    "chunk_references_wrong_kind_descriptor",
    "chunk_references_wrong_coordinate_descriptor",
    "chunk_references_invalid_descriptor",
    "chunk_references_unsupported_descriptor",
    "chunk_references_pending_descriptor",
    "orphan_chunk_promotes_after_descriptor_delivery",
]


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    entries: list[dict[str, object]] = []
    paths = set(FIXTURES.joinpath("schema").glob("*.json"))
    paths.update((REQUIREMENTS, AUTHORITY, COMPANION))
    profiles = {name: [] for name in ("checkpoint", "core", "malformed", "property")}
    seen: set[str] = set()
    for metadata_path in sorted(SCENARIOS.rglob("*.fixture.json")):
        metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
        fixture_id = metadata["fixture_id"]
        if fixture_id in seen:
            raise AssertionError(f"duplicate fixture id: {fixture_id}")
        seen.add(fixture_id)
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
            "requirements": metadata["requirements"],
            "metadata_path": metadata_path.relative_to(ROOT).as_posix(),
            "input_paths": [path.relative_to(ROOT).as_posix() for path in inputs],
            "expected_path": expected.relative_to(ROOT).as_posix(),
        })
    missing = sorted(set(V6_FIXTURES) - seen, key=str.encode)
    if len(entries) > TARGET_COUNT:
        raise AssertionError(f"signed fixture count exceeds {TARGET_COUNT}: {len(entries)}")
    entries.sort(key=lambda item: str(item["fixture_id"]).encode())
    for fixture_ids in profiles.values():
        fixture_ids.sort(key=str.encode)
    files = [
        {"path": path.relative_to(ROOT).as_posix(), "sha256": sha256(path)}
        for path in sorted(paths, key=lambda path: path.relative_to(ROOT).as_posix().encode())
    ]
    complete = len(entries) == TARGET_COUNT and not missing
    manifest = {
        "distribution_schema": "nostr_automerge.fixture_distribution.v7",
        "distribution_id": "draft_2026_08_signed_neutral_7",
        "protocol_revision": "draft_2026_08",
        "status": "canonical_signed_neutral_corpus" if complete else "incomplete_fail_closed",
        "target_fixture_count": TARGET_COUNT,
        "fixture_count": len(entries),
        "complete": complete,
        "missing_v6_fixtures": missing,
        "requirements_sha256": sha256(REQUIREMENTS),
        "authority_sha256": sha256(AUTHORITY),
        "companion_sha256": sha256(COMPANION),
        "supersedes": "fixtures/distribution/manifest_v6.json",
        "v6_fixtures": V6_FIXTURES,
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
            f"signed distribution incomplete: {len(entries)}/{TARGET_COUNT}; missing={missing}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
