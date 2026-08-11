#!/usr/bin/env python3
"""Generate the canonical signed neutral fixture distribution v4."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "fixtures"
SCENARIOS = FIXTURES / "v1_draft" / "scenarios"
OUTPUT = FIXTURES / "distribution" / "manifest_v4.json"
PROFILE_BY_FAMILY = {
    "checkpoints": "checkpoint",
    "projection": "property",
    "versioning": "malformed",
}


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    entries: list[dict[str, object]] = []
    distributed_paths = set(FIXTURES.joinpath("schema").glob("*.schema.json"))
    seen_ids: set[str] = set()
    profiles = {name: [] for name in ("checkpoint", "core", "malformed", "property")}

    for metadata_path in sorted(SCENARIOS.rglob("*.fixture.json")):
        metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
        fixture_id = metadata["fixture_id"]
        if fixture_id in seen_ids:
            raise AssertionError(f"duplicate fixture id: {fixture_id}")
        seen_ids.add(fixture_id)
        family = metadata_path.parent.name
        profile = PROFILE_BY_FAMILY.get(family, "core")
        inputs = [metadata_path.parent / item["path"] for item in metadata["inputs"]]
        expected = metadata_path.parent / metadata["expected"]["report_path"]
        paths = [metadata_path, *inputs, expected]
        if any(not path.is_file() for path in paths):
            raise AssertionError(f"fixture has a missing artifact: {fixture_id}")
        distributed_paths.update(paths)
        profiles[profile].append(fixture_id)
        entries.append(
            {
                "fixture_id": fixture_id,
                "profile": profile,
                "requirements": metadata["requirements"],
                "metadata_path": metadata_path.relative_to(ROOT).as_posix(),
                "input_paths": [path.relative_to(ROOT).as_posix() for path in inputs],
                "expected_path": expected.relative_to(ROOT).as_posix(),
            }
        )

    files = [
        {"path": path.relative_to(ROOT).as_posix(), "sha256": sha256(path)}
        for path in sorted(distributed_paths, key=lambda value: value.relative_to(ROOT).as_posix().encode())
    ]
    manifest = {
        "distribution_schema": "nostr_automerge.fixture_distribution.v4",
        "distribution_id": "draft_2026_08_signed_neutral_4",
        "protocol_revision": "draft_2026_08",
        "status": "canonical_signed_neutral_corpus",
        "profiles": profiles,
        "fixtures": entries,
        "files": files,
    }
    OUTPUT.write_text(
        json.dumps(manifest, sort_keys=True, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
