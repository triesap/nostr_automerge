#!/usr/bin/env python3
"""Generate the canonical immutable neutral fixture distribution manifest."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "fixtures"


def main() -> int:
    metadata_paths = sorted(FIXTURES.rglob("*.fixture.json"))
    metadata = [json.loads(path.read_text(encoding="utf-8")) for path in metadata_paths]
    fixture_ids = {item["fixture_id"] for item in metadata}
    profiles = {
        "core": ["actor_derivation_001", "interop_core_001"],
        "checkpoint": ["interop_checkpoint_001"],
        "malformed": ["interop_malformed_001", "scenario_nip01_boundaries"],
        "property": ["interop_property_001"],
    }
    profiled = {identifier for values in profiles.values() for identifier in values}
    if profiled != fixture_ids:
        raise AssertionError("every fixture must appear in exactly one canonical profile")
    paths = {
        Path("fixtures/schema/fixture.schema.json"),
        Path("fixtures/schema/report.schema.json"),
        Path("fixtures/schema/scenario.schema.json"),
        Path("fixtures/schema/interop_attestation.schema.json"),
        Path("fixtures/v1_draft/manifests/cases.json"),
        Path("fixtures/v1_draft/controls/scenarios.json"),
        Path("fixtures/v1_draft/changes/cases.json"),
        Path("fixtures/v1_draft/integrity/cases.json"),
        Path("fixtures/v1_draft/checkpoints/cases.json"),
    }
    for path, item in zip(metadata_paths, metadata, strict=True):
        relative = path.relative_to(ROOT)
        paths.add(relative)
        paths.update(relative.parent / source["path"] for source in item["inputs"])
        paths.add(relative.parent / item["expected"]["report_path"])
    files = []
    for relative in sorted(paths, key=lambda value: value.as_posix().encode()):
        data = (ROOT / relative).read_bytes()
        files.append({"path": relative.as_posix(), "sha256": hashlib.sha256(data).hexdigest()})
    manifest = {
        "distribution_schema": "nostr_automerge.fixture_distribution.v2",
        "distribution_id": "draft_2026_08_interop_2",
        "protocol_revision": "draft_2026_08",
        "status": "canonical_neutral_corpus",
        "profiles": profiles,
        "files": files,
    }
    (FIXTURES / "distribution/manifest.json").write_text(
        json.dumps(manifest, sort_keys=True, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
