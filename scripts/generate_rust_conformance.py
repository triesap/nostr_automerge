#!/usr/bin/env python3
"""Generate fresh-process Rust reports for every signed distribution profile."""

from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MANIFEST_PATH = ROOT / "fixtures" / "distribution" / "manifest_v3.json"
PROFILE_OUTPUTS = {
    "core": ROOT / "reports" / "rust_signed_core.json",
    "checkpoint": ROOT / "reports" / "rust_signed_checkpoint.json",
    "malformed": ROOT / "reports" / "rust_signed_malformed.json",
    "property": ROOT / "reports" / "rust_signed_property.json",
}


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def command(*arguments: str) -> bytes:
    return subprocess.run(
        arguments, cwd=ROOT, check=True, capture_output=True
    ).stdout


def fixture_report(path: str) -> bytes:
    return command(
        "cargo", "run", "--quiet", "-p", "nostr_automerge_conformance",
        "--locked", "--", "run_fixture", path,
    )


def main() -> int:
    manifest_bytes = MANIFEST_PATH.read_bytes()
    manifest = json.loads(manifest_bytes)
    entries = {entry["fixture_id"]: entry for entry in manifest["fixtures"]}
    binding = {
        "cargo_lock_sha256": sha256(ROOT.joinpath("Cargo.lock").read_bytes()),
        "fixture_manifest_sha256": sha256(manifest_bytes),
        "rust_toolchain_sha256": sha256(ROOT.joinpath("rust-toolchain.toml").read_bytes()),
        "source_commit": command("git", "rev-parse", "HEAD").decode().strip(),
        "toolchain": {
            "cargo": command("cargo", "--version").decode().strip(),
            "rustc": command("rustc", "--version").decode().strip(),
        },
    }
    for profile, output_path in PROFILE_OUTPUTS.items():
        results = []
        for fixture_id in manifest["profiles"][profile]:
            metadata_path = entries[fixture_id]["metadata_path"]
            first = fixture_report(metadata_path)
            second = fixture_report(metadata_path)
            if first != second:
                raise AssertionError(f"fresh-process output mismatch: {fixture_id}")
            results.append(
                {
                    "fixture_id": fixture_id,
                    "report": json.loads(first),
                    "report_sha256": sha256(first),
                    "requirements": entries[fixture_id]["requirements"],
                }
            )
        canonical_results = (
            json.dumps(results, sort_keys=True, separators=(",", ":")) + "\n"
        ).encode()
        report = {
            **binding,
            "fixture_count": len(results),
            "output_sha256": sha256(canonical_results),
            "process_runs_per_fixture": 2,
            "profile": profile,
            "reports": results,
            "schema": "nostr_automerge.rust_signed_profile.v3",
            "status": "pass",
        }
        output_path.write_text(
            json.dumps(report, sort_keys=True, separators=(",", ":")) + "\n",
            encoding="utf-8",
        )
    print(f"PASS: generated {len(entries)} signed Rust fixture reports twice")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
