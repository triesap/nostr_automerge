#!/usr/bin/env python3
"""Generate or check the staged signed distribution-v10 manifest."""

from __future__ import annotations

import argparse
from pathlib import Path

import validate_authority_transition_v10 as authority


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / authority.V10_MANIFEST_PATH


def canonical_bytes() -> bytes:
    authority.validate_historical_v10_manifest()
    return authority.v10_manifest_bytes()


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
