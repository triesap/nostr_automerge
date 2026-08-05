#!/usr/bin/env python3
"""Generate execution-bound Rust conformance evidence from the checked-in corpus."""

from __future__ import annotations

import hashlib
import json
import subprocess
from collections import Counter
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "fixtures"


def command(*arguments: str) -> str:
    return subprocess.run(
        arguments, cwd=ROOT, check=True, capture_output=True, text=True
    ).stdout.strip()


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def main() -> int:
    metadata_paths = sorted(FIXTURES.rglob("*.fixture.json"))
    fixtures = [json.loads(path.read_text(encoding="utf-8")) for path in metadata_paths]
    corpus_bytes = command(
        "cargo",
        "run",
        "--quiet",
        "-p",
        "nostr_automerge_conformance",
        "--locked",
        "--",
        "run_corpus",
        "fixtures",
    ).encode() + b"\n"
    corpus = json.loads(corpus_bytes)
    if corpus["failed"] != 0 or corpus["total"] != len(fixtures):
        raise AssertionError("conformance corpus did not complete successfully")

    gates = [
        ["cargo", "test", "-p", "nostr_automerge", "--test", "nip01_conformance", "--locked"],
        ["cargo", "test", "-p", "nostr_automerge", "--test", "public_engine_api", "--locked"],
        ["cargo", "test", "-p", "nostr_automerge", "--lib", "graph::", "--locked"],
        ["cargo", "test", "-p", "nostr_automerge", "--test", "checkpoint_replay_agreement", "--locked"],
    ]
    for gate in gates:
        command(*gate)

    family_sources = {
        "nip01": ROOT / "fixtures/v1_draft/nip01/scenario_nip01_boundaries.input.json",
        "manifest": ROOT / "fixtures/v1_draft/manifests/cases.json",
        "control": ROOT / "fixtures/v1_draft/controls/scenarios.json",
        "change_graph": ROOT / "fixtures/v1_draft/changes/cases.json",
        "integrity": ROOT / "fixtures/v1_draft/integrity/cases.json",
        "checkpoint": ROOT / "fixtures/v1_draft/checkpoints/cases.json",
    }
    families = Counter(path.parent.name for path in metadata_paths)
    family_source_sha256 = {
        name: digest(path.read_bytes()) for name, path in family_sources.items()
    }
    requirements = sorted(
        {identifier for fixture in fixtures for identifier in fixture["requirements"]}
    )
    fixture_digests = {}
    for path, fixture in zip(metadata_paths, fixtures, strict=True):
        report = command(
            "cargo",
            "run",
            "--quiet",
            "-p",
            "nostr_automerge_conformance",
            "--locked",
            "--",
            "run_fixture",
            str(path),
        ).encode() + b"\n"
        fixture_digests[fixture["fixture_id"]] = digest(report)

    report = {
        "commit": command("git", "rev-parse", "HEAD"),
        "completion": "complete",
        "corpus_sha256": digest(corpus_bytes),
        "failures": [],
        "families": dict(sorted(families.items())),
        "family_source_sha256": family_source_sha256,
        "fixture_count": len(fixtures),
        "fixture_report_sha256": fixture_digests,
        "requirement_count": len(requirements),
        "requirement_ids": requirements,
        "schema": "nostr_automerge.rust_conformance.v1",
        "test_gates": [{"command": " ".join(gate), "status": "passed"} for gate in gates],
        "toolchain": {
            "cargo": command("cargo", "--version"),
            "rustc": command("rustc", "--version"),
        },
    }
    encoded = json.dumps(report, sort_keys=True, separators=(",", ":")) + "\n"
    (ROOT / "reports/rust_conformance.json").write_text(encoded, encoding="utf-8")
    (ROOT / "reports/rust_conformance.md").write_text(
        "# Rust conformance\n\n"
        f"Completion: complete. Executed {len(fixtures)} fixtures across "
        f"{len(family_sources)} protocol scenario families with zero failures. The JSON companion binds "
        f"{len(requirements)} requirement identifiers, every canonical fixture-report digest, "
        "the corpus digest, source commit, and Rust toolchain.\n",
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
