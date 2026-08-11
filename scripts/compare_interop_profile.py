#!/usr/bin/env python3
"""Compare canonical signed profile report arrays without normalization."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PROFILES = ("core", "checkpoint", "malformed", "property")
MISMATCH_CLASS = "canonical_report_bytes"


class CanonicalMismatch(ValueError):
    """An exact canonical profile comparison failed."""

    def __init__(self, classification: str) -> None:
        super().__init__(classification)
        self.classification = classification


def canonical_reports(value: object) -> bytes:
    if not isinstance(value, dict) or not isinstance(value.get("reports"), list):
        raise ValueError("profile artifact has no canonical reports array")
    return (json.dumps(value["reports"], sort_keys=True, separators=(",", ":")) + "\n").encode()


def compare(left: object, right: object) -> str:
    if canonical_reports(left) != canonical_reports(right):
        raise CanonicalMismatch(MISMATCH_CLASS)
    return hashlib.sha256(canonical_reports(left)).hexdigest()


def self_test() -> None:
    path = ROOT / "reports" / "rust_signed_property.json"
    original_bytes = path.read_bytes()
    original = json.loads(original_bytes)
    mutated = copy.deepcopy(original)
    mutated["reports"][0]["report"]["completion"] = "cancelled"
    try:
        compare(original, mutated)
    except CanonicalMismatch as error:
        if error.classification != MISMATCH_CLASS:
            raise AssertionError("unexpected mismatch classification") from error
    else:
        raise AssertionError("deliberate canonical report mutation was not detected")
    if path.read_bytes() != original_bytes:
        raise AssertionError("mismatch self-test altered the source report")


def compare_profile(profile: str, typescript_root: Path) -> None:
    rust_path = ROOT / "reports" / f"rust_signed_{profile}.json"
    typescript_path = typescript_root / "reports" / f"typescript_signed_{profile}.json"
    rust = json.loads(rust_path.read_text(encoding="utf-8"))
    typescript = json.loads(typescript_path.read_text(encoding="utf-8"))
    digest = compare(rust, typescript)
    if rust["fixture_manifest_sha256"] != typescript["fixture_manifest_sha256"]:
        raise CanonicalMismatch("fixture_manifest_sha256")
    print(f"PASS: {profile} signed profile canonical bytes {digest}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("profile", nargs="?", choices=PROFILES)
    parser.add_argument("--typescript-root", type=Path)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        if args.profile is not None or args.typescript_root is not None:
            parser.error("--self-test does not accept a profile or TypeScript root")
        self_test()
        print(f"PASS: deliberate mismatch classified as {MISMATCH_CLASS}")
        return 0
    if args.profile is None or args.typescript_root is None:
        parser.error("profile and --typescript-root are required")
    compare_profile(args.profile, args.typescript_root.resolve(strict=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
