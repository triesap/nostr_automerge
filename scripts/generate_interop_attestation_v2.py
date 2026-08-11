#!/usr/bin/env python3
"""Collapse passing local profile evidence into an opaque v2 attestation."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

sys.dont_write_bytecode = True
from compare_interop_profile import compare
from validate_interop_attestation_v2 import validate


ROOT = Path(__file__).resolve().parents[1]
PROFILES = ("core", "checkpoint", "malformed", "property")


def load(path: Path) -> dict[str, object]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise AssertionError("evidence object")
    return value


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rust-evidence", required=True, type=Path)
    parser.add_argument("--typescript-evidence", required=True, type=Path)
    parser.add_argument("--handoff", required=True, type=Path)
    parser.add_argument("--contract", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    arguments = parser.parse_args()
    rust_root = arguments.rust_evidence.resolve(strict=True)
    typescript_root = arguments.typescript_evidence.resolve(strict=True)
    handoff = load(arguments.handoff)
    contract = load(arguments.contract)
    manifest_hash = hashlib.sha256(
        (ROOT / "fixtures/distribution/manifest_v3.json").read_bytes()
    ).hexdigest()
    if handoff["manifest_sha256"] != manifest_hash:
        raise AssertionError("stale distribution handoff")
    rust_lock = hashlib.sha256((ROOT / "Cargo.lock").read_bytes()).hexdigest()
    profile_hashes: dict[str, dict[str, str]] = {}
    for profile in PROFILES:
        rust = load(rust_root / f"rust_signed_{profile}.json")
        typescript = load(typescript_root / f"typescript_signed_{profile}.json")
        if rust["cargo_lock_sha256"] != rust_lock:
            raise AssertionError("stale Rust lock")
        if rust["fixture_manifest_sha256"] != manifest_hash:
            raise AssertionError("stale Rust distribution")
        if typescript["source_commit"] != contract["implementation_commit"]:
            raise AssertionError("stale TypeScript commit")
        if typescript["package_lock_sha256"] != contract["dependency_lock_sha256"]:
            raise AssertionError("stale TypeScript lock")
        if typescript["fixture_manifest_sha256"] != manifest_hash:
            raise AssertionError("stale TypeScript distribution")
        profile_hashes[profile] = {"report_sha256": compare(rust, typescript), "result": "pass"}
    profile_hashes["projection"] = dict(profile_hashes["property"])
    attestation = {
        "commit": contract["implementation_commit"],
        "deliberate_mismatch": "detected",
        "dependency_lock_sha256": contract["dependency_lock_sha256"],
        "fixture_distribution_sha256": manifest_hash,
        "implementation_identity": contract["implementation_identity"],
        "profiles": profile_hashes,
        "provenance": "operator-local",
        "result": "pass",
        "schema": "nostr_automerge.interop_attestation.v2",
        "toolchain": contract["toolchain"],
    }
    validate(attestation)
    arguments.output.parent.mkdir(parents=True, exist_ok=True)
    arguments.output.write_text(
        json.dumps(attestation, sort_keys=True, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )
    print("PASS: combined evidence bound to final commits and locks")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
