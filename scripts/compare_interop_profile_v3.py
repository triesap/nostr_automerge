#!/usr/bin/env python3
"""Compare exact signed-v4 Rust and TypeScript profile reports."""

from __future__ import annotations

import argparse
import copy
import json
import sys
from pathlib import Path

sys.dont_write_bytecode = True
from compare_interop_profile import CanonicalMismatch, MISMATCH_CLASS, compare


PROFILES = ("core", "checkpoint", "malformed", "property")


def load(path: Path, schema: str) -> dict[str, object]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict) or value.get("schema") != schema:
        raise CanonicalMismatch("profile_schema")
    return value


def self_test() -> None:
    baseline = {
        "reports": [{"fixture_id": "fixture", "report_sha256": "11" * 32, "report": {"completion": "complete"}}]
    }
    mutation = copy.deepcopy(baseline)
    mutation["reports"][0]["report"]["completion"] = "cancelled"
    try:
        compare(baseline, mutation)
    except CanonicalMismatch as error:
        if error.classification == MISMATCH_CLASS:
            return
    raise AssertionError("canonical report mismatch unexpectedly passed")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("profile", nargs="?", choices=PROFILES)
    parser.add_argument("--rust-evidence", type=Path)
    parser.add_argument("--typescript-evidence", type=Path)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        print(f"PASS: deliberate mismatch classified as {MISMATCH_CLASS}")
        return 0
    if args.profile is None or args.rust_evidence is None or args.typescript_evidence is None:
        parser.error("profile and both evidence roots are required")
    rust = load(
        args.rust_evidence / f"rust_signed_{args.profile}.json",
        "nostr_automerge.rust_signed_profile.v4",
    )
    typescript = load(
        args.typescript_evidence / f"typescript_signed_{args.profile}.json",
        "nostr_automerge.typescript_signed_profile.v4",
    )
    if rust.get("fixture_manifest_sha256") != typescript.get("fixture_manifest_sha256"):
        raise CanonicalMismatch("fixture_manifest_sha256")
    print(f"PASS: {args.profile} signed-v4 profile {compare(rust, typescript)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
