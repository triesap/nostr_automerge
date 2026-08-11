#!/usr/bin/env python3
"""Collapse matching signed-v4 profiles into source-free v3 attestations."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

sys.dont_write_bytecode = True
from compare_interop_profile import compare


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "fixtures/distribution/manifest_v4.json"
PROFILES = ("core", "checkpoint", "malformed", "property")
HEX40 = re.compile(r"^[0-9a-f]{40}$")


def canonical(value: object) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode()


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load(path: Path) -> dict[str, object]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise AssertionError("profile_shape")
    return value


def git_source_commit() -> str:
    return subprocess.run(
        (
            "git", "log", "-1", "--format=%H", "--", "crates", "tools", "Cargo.toml",
            "Cargo.lock", "rust-toolchain.toml", "fixtures",
        ),
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rust-evidence", required=True, type=Path)
    parser.add_argument("--typescript-evidence", required=True, type=Path)
    parser.add_argument("--output-root", default=ROOT / "reports", type=Path)
    args = parser.parse_args()
    rust_root = args.rust_evidence.resolve(strict=True)
    typescript_root = args.typescript_evidence.resolve(strict=True)
    output_root = args.output_root.resolve()
    output_root.mkdir(parents=True, exist_ok=True)
    manifest_hash = sha256(MANIFEST.read_bytes())
    cargo_lock_hash = sha256((ROOT / "Cargo.lock").read_bytes())
    rust_commit = git_source_commit()
    profile_results: dict[str, dict[str, str]] = {}
    rust_toolchain: object | None = None
    typescript_toolchain: object | None = None
    typescript_commit: str | None = None
    typescript_lock: str | None = None
    for profile in PROFILES:
        rust = load(rust_root / f"rust_signed_{profile}.json")
        typescript = load(typescript_root / f"typescript_signed_{profile}.json")
        if rust.get("schema") != "nostr_automerge.rust_signed_profile.v4":
            raise AssertionError("rust_profile_schema")
        if typescript.get("schema") != "nostr_automerge.typescript_signed_profile.v4":
            raise AssertionError("typescript_profile_schema")
        if rust.get("source_commit") != rust_commit or rust.get("cargo_lock_sha256") != cargo_lock_hash:
            raise AssertionError("stale_rust_profile")
        if rust.get("fixture_manifest_sha256") != manifest_hash:
            raise AssertionError("stale_rust_distribution")
        if typescript.get("fixture_manifest_sha256") != manifest_hash:
            raise AssertionError("stale_typescript_distribution")
        current_commit = typescript.get("source_commit")
        current_lock = typescript.get("package_lock_sha256")
        if not isinstance(current_commit, str) or not HEX40.fullmatch(current_commit):
            raise AssertionError("typescript_commit")
        if typescript_commit not in (None, current_commit) or typescript_lock not in (None, current_lock):
            raise AssertionError("typescript_profile_binding")
        typescript_commit = current_commit
        typescript_lock = str(current_lock)
        rust_toolchain = rust.get("toolchain")
        typescript_toolchain = typescript.get("toolchain")
        profile_results[profile] = {"report_sha256": compare(rust, typescript), "result": "pass"}
    profile_results["projection"] = dict(profile_results["property"])
    rust_attestation = {
        "schema": "nostr_automerge.rust_interop_attestation.v3",
        "implementation_identity": "triesap/nostr_automerge",
        "commit": rust_commit,
        "dependency_lock_sha256": cargo_lock_hash,
        "toolchain": rust_toolchain,
        "fixture_distribution_sha256": manifest_hash,
        "profiles": profile_results,
        "result": "pass",
        "provenance": "operator-local",
    }
    typescript_attestation = {
        "schema": "nostr_automerge.interop_attestation.v3",
        "implementation_identity": "triesap/nostr_automerge_typescript",
        "commit": typescript_commit,
        "dependency_lock_sha256": typescript_lock,
        "toolchain": typescript_toolchain,
        "fixture_distribution_sha256": manifest_hash,
        "profiles": profile_results,
        "result": "pass",
        "deliberate_mismatch": "detected",
        "provenance": "operator-local",
    }
    rust_bytes = canonical(rust_attestation)
    typescript_bytes = canonical(typescript_attestation)
    combined = {
        "schema": "nostr_automerge.interop_combined.v3",
        "fixture_distribution_sha256": manifest_hash,
        "rust_attestation_sha256": sha256(rust_bytes),
        "typescript_attestation_sha256": sha256(typescript_bytes),
        "profiles": profile_results,
        "comparison": "byte_exact_canonical_reports_without_normalization",
        "deliberate_mismatch": "detected",
        "result": "pass",
        "provenance": "operator-local",
    }
    (output_root / "interop_rust_v3.json").write_bytes(rust_bytes)
    (output_root / "interop_typescript_v3.json").write_bytes(typescript_bytes)
    (output_root / "interop_combined_v3.json").write_bytes(canonical(combined))
    print("PASS: final Rust and TypeScript signed-v4 profiles match byte-for-byte")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
