#!/usr/bin/env python3
"""Execute canonical fixture profiles and emit Rust attestation inputs."""

from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "fixtures/distribution/manifest.json"


def command(*arguments: str) -> str:
    return subprocess.run(arguments, cwd=ROOT, check=True, capture_output=True, text=True).stdout.strip()


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    manifest = json.loads(MANIFEST.read_text())
    fixture_paths = {
        json.loads(path.read_text())["fixture_id"]: path
        for path in ROOT.joinpath("fixtures").rglob("*.fixture.json")
    }
    for profile in ("core", "checkpoint"):
        reports = []
        for identifier in manifest["profiles"][profile]:
            output = command(
                "cargo", "run", "--quiet", "-p", "nostr_automerge_conformance", "--locked", "--",
                "run_fixture", str(fixture_paths[identifier]),
            )
            reports.append(json.loads(output))
        report_path = ROOT / f"reports/interop_rust_{profile}.canonical.json"
        report_path.write_text(json.dumps(reports, sort_keys=True, separators=(",", ":")) + "\n", encoding="utf-8")
        attestation = {
            "schema": "nostr_automerge.interop_attestation.v1",
            "implementation": {
                "repository": "triesap/nostr_automerge",
                "language": "rust",
                "runner": "nostr_automerge_conformance",
            },
            "commit": command("git", "rev-parse", "HEAD"),
            "toolchain": {
                "cargo": command("cargo", "--version"),
                "rustc": command("rustc", "--version"),
            },
            "dependency_lock_sha256": digest(ROOT / "Cargo.lock"),
            "fixture_manifest_sha256": digest(MANIFEST),
            "profile": profile,
            "report_sha256": digest(report_path),
            "result": "pass",
        }
        (ROOT / f"reports/interop_rust_{profile}.attestation.json").write_text(
            json.dumps(attestation, sort_keys=True, separators=(",", ":")) + "\n",
            encoding="utf-8",
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
