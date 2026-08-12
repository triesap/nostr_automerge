#!/usr/bin/env python3
"""Generate fresh-process Rust reports for every signed distribution profile."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MANIFEST_PATH = ROOT / "fixtures" / "distribution" / "manifest_v5.json"
PROFILES = ("core", "checkpoint", "malformed", "property")


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def command(*arguments: str) -> bytes:
    return subprocess.run(
        arguments, cwd=ROOT, check=True, capture_output=True
    ).stdout


def fixture_report(runner: Path, path: str) -> bytes:
    return command(str(runner), "run_fixture", path)


def prepared_runner(temporary: Path) -> Path:
    command("cargo", "build", "-p", "nostr_automerge_conformance", "--locked")
    target = os.environ.get("CARGO_TARGET_DIR")
    if not target:
        raise AssertionError("CARGO_TARGET_DIR must be supplied by the build router")
    source = Path(target) / "debug" / "nostr_automerge_conformance"
    if not source.is_file():
        raise AssertionError("routed conformance runner is missing")
    runner = temporary / source.name
    shutil.copy2(source, runner)
    runner.chmod(0o700)
    return runner


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output-root", type=Path, default=ROOT / "reports")
    arguments = parser.parse_args()
    output_root = arguments.output_root.resolve()
    output_root.mkdir(parents=True, exist_ok=True)
    manifest_bytes = MANIFEST_PATH.read_bytes()
    manifest = json.loads(manifest_bytes)
    entries = {entry["fixture_id"]: entry for entry in manifest["fixtures"]}
    binding = {
        "cargo_lock_sha256": sha256(ROOT.joinpath("Cargo.lock").read_bytes()),
        "fixture_manifest_sha256": sha256(manifest_bytes),
        "rust_toolchain_sha256": sha256(ROOT.joinpath("rust-toolchain.toml").read_bytes()),
        "source_commit": command(
            "git", "log", "-1", "--format=%H", "--", "crates", "tools", "Cargo.toml",
            "Cargo.lock", "rust-toolchain.toml", "fixtures",
        ).decode().strip(),
        "toolchain": {
            "cargo": command("cargo", "--version").decode().strip(),
            "rustc": command("rustc", "--version").decode().strip(),
        },
    }
    with tempfile.TemporaryDirectory(prefix="nostr-automerge-runner-") as directory:
        runner = prepared_runner(Path(directory))
        for profile in PROFILES:
            output_path = output_root / f"rust_signed_{profile}.json"
            results = []
            for fixture_id in manifest["profiles"][profile]:
                metadata_path = entries[fixture_id]["metadata_path"]
                first = fixture_report(runner, metadata_path)
                second = fixture_report(runner, metadata_path)
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
                "schema": "nostr_automerge.rust_signed_profile.v4",
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
