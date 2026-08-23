#!/usr/bin/env python3
"""Generate or check the staged signed distribution-v10 manifest."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import validate_authority_transition_v10 as authority


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / authority.V10_MANIFEST_PATH


def canonical_bytes() -> bytes:
    state = authority.load_object(authority.STATE_PATH)
    stage = state.get("current_stage")
    authority.require(
        isinstance(stage, str) and stage in authority.STAGES,
        "generator_stage",
    )
    authority.require(
        authority.STAGES.index(stage)
        >= authority.STAGES.index("distribution_locked"),
        "generator_stage_unlocked",
    )
    manifest = authority.expected_v10_manifest(
        stage,
        authority.discover_fixture_metadata(),
    )
    authority.validate_v10_manifest(
        stage,
        manifest,
        authority.discover_fixture_metadata(),
    )
    return (
        json.dumps(
            manifest,
            ensure_ascii=False,
            sort_keys=True,
            separators=(",", ":"),
        )
        + "\n"
    ).encode("utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    expected = canonical_bytes()
    if args.write:
        OUTPUT.write_bytes(expected)
        print(f"WROTE: {OUTPUT.relative_to(ROOT)}")
    elif not OUTPUT.is_file() or OUTPUT.read_bytes() != expected:
        raise SystemExit("FAIL: stale signed distribution-v10 manifest")
    else:
        print("PASS: signed distribution-v10 manifest is deterministic")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
