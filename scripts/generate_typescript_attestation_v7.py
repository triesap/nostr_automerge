#!/usr/bin/env python3
"""Generate the opaque private-TypeScript signed-distribution v7 attestation."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
HEX_40 = re.compile(r"^[0-9a-f]{40}$")
HEX_64 = re.compile(r"^[0-9a-f]{64}$")


def hash_value(value: str) -> str:
    if not HEX_64.fullmatch(value):
        raise SystemExit(f"invalid SHA-256 value: {value}")
    return value


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--candidate", required=True)
    parser.add_argument("--dependency-lock", required=True)
    parser.add_argument("--corpus-run", action="append", required=True)
    parser.add_argument("--profile", action="append", required=True)
    arguments = parser.parse_args()
    if not HEX_40.fullmatch(arguments.candidate):
        raise SystemExit("candidate must be a lowercase 40-hex Git identity")
    if len(arguments.corpus_run) != 2:
        raise SystemExit("exactly two corpus runs are required")
    corpus_runs = [hash_value(value) for value in arguments.corpus_run]
    if len(set(corpus_runs)) != 1:
        raise SystemExit("TypeScript corpus runs are not byte-identical")
    profiles: dict[str, str] = {}
    for value in arguments.profile:
        name, separator, digest = value.partition("=")
        if not separator or name not in {"checkpoint", "core", "malformed", "property"}:
            raise SystemExit(f"invalid profile binding: {value}")
        profiles[name] = hash_value(digest)
    if set(profiles) != {"checkpoint", "core", "malformed", "property"}:
        raise SystemExit("all four signed profiles are required")
    manifest_path = ROOT / "fixtures/distribution/manifest_v7.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    fixture_ids = sorted(item["fixture_id"] for item in manifest["fixtures"])
    if len(fixture_ids) != 157 or len(set(fixture_ids)) != 157:
        raise SystemExit("signed distribution v7 must contain exactly 157 unique fixtures")
    report = {
        "boundaries": {
            "private_paths_included": False,
            "private_runner_state_included": False,
            "rust_bindings_used": False,
            "tracked_workflows_present": False,
        },
        "candidate": arguments.candidate,
        "commands": [
            "pnpm check",
            "pnpm signed:profiles",
            "node dist/src/cli.js run_corpus <signed-fixture-root> (pass 1)",
            "node dist/src/cli.js run_corpus <signed-fixture-root> (pass 2)",
        ],
        "corpus_run_sha256": corpus_runs,
        "dependency_lock_sha256": hash_value(arguments.dependency_lock),
        "distribution_id": manifest["distribution_id"],
        "executed_fixture_ids": fixture_ids,
        "fixture_count": len(fixture_ids),
        "fixture_distribution_sha256": hashlib.sha256(manifest_path.read_bytes()).hexdigest(),
        "implementation": "typescript",
        "profile_output_sha256": dict(sorted(profiles.items())),
        "result": "pass",
        "schema": "nostr_automerge.private_typescript_attestation.v7",
    }
    output = ROOT / "reports/interop_typescript_v7.json"
    output.write_text(
        json.dumps(report, sort_keys=True, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )
    print("PASS: generated opaque TypeScript signed-distribution v7 attestation")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
