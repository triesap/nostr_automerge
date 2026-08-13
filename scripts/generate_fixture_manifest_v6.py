#!/usr/bin/env python3
"""Generate the canonical remediation-v5 signed fixture distribution v6."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "fixtures"
SCENARIOS = FIXTURES / "v1_draft" / "scenarios"
OUTPUT = FIXTURES / "distribution" / "manifest_v6.json"
REQUIREMENTS = ROOT / "spec" / "requirements.json"
AUTHORITY = ROOT / "spec" / "NIP_DRAFT.md"
COMPANION = ROOT / "spec" / "NOSTR_AUTOMERGE_V1_SPEC.md"
PROFILE_BY_FAMILY = {"checkpoints": "checkpoint", "projection": "property", "versioning": "malformed"}
COVERAGE = {
    "mixed_claims": [
        "accepted_base_pruned_duplicate_carrier",
        "change_before_pending_control",
        "change_under_noncanonical_control",
        "cross_control_duplicate_accepted_dominance",
        "invalid_claim_does_not_poison_valid_hash",
    ],
    "dependency_knowledge": [
        "change_under_invalid_control",
        "change_under_wrong_coordinate_control",
        "change_under_wrong_kind_control",
        "change_with_missing_control",
        "child_change_depends_on_pruned_parent_change",
        "equivocation_descendants",
        "versioning_unknown",
    ],
    "checkpoint_controls": [
        "checkpoints_missing_control_dynamic",
        "checkpoints_single_chunk",
        "checkpoints_unauthorized",
    ],
    "coordinate_resources": [
        "interrupted_cancel_at_ingress",
        "unrelated_changes_do_not_consume_target_budget",
        "unrelated_checkpoint_does_not_change_target",
        "unrelated_manifest_does_not_change_target",
    ],
    "finalization": [
        "interrupted_report_reservation_after",
        "interrupted_report_reservation_before",
    ],
}


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
    for family, fixture_ids in COVERAGE.items():
        missing = set(fixture_ids) - seen
        if missing:
            raise AssertionError(f"missing {family} coverage fixtures: {sorted(missing)}")
    entries.sort(key=lambda item: str(item["fixture_id"]).encode())
    for fixture_ids in profiles.values():
        fixture_ids.sort(key=str.encode)
    files = [
        {"path": path.relative_to(ROOT).as_posix(), "sha256": sha256(path)}
        for path in sorted(paths, key=lambda path: path.relative_to(ROOT).as_posix().encode())
    ]
    manifest = {
        "distribution_schema": "nostr_automerge.fixture_distribution.v6",
        "distribution_id": "draft_2026_08_signed_neutral_6",
        "protocol_revision": "draft_2026_08",
        "status": "canonical_signed_neutral_corpus",
        "requirements_sha256": sha256(REQUIREMENTS),
        "authority_sha256": sha256(AUTHORITY),
        "companion_sha256": sha256(COMPANION),
        "supersedes": "fixtures/distribution/manifest_v5.json",
        "coverage": COVERAGE,
        "profiles": profiles,
        "fixtures": entries,
        "files": files,
    }
    OUTPUT.write_text(json.dumps(manifest, sort_keys=True, separators=(",", ":")) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
