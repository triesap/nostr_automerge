#!/usr/bin/env python3
"""Run the closed Rust fuzz campaign and publish evidence only after success."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / ".act/output/rust_fuzz_summary.json"
TARGETS = [
    "raw_nip01",
    "automerge_framing",
    "automerge_decode",
    "automerge_reencode",
    "control_transition",
    "reference_evaluator",
    "checkpoint",
    "projection",
    "smoke",
]


def main() -> int:
    for target in TARGETS:
        subprocess.run(
            [
                "cargo",
                "+nightly-2026-07-16",
                "fuzz",
                "run",
                target,
                "--",
                "-runs=10000",
                "-seed=20260804",
                "-max_len=4096",
            ],
            cwd=ROOT,
            check=True,
        )
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text(
        json.dumps(
            {
                "crashes": 0,
                "executions_per_target": 10_000,
                "schema": "nostr_automerge.rust_fuzz.v1",
                "seed": 20260804,
                "status": "pass",
                "targets": TARGETS,
                "timeouts": 0,
                "tool": "cargo-fuzz 0.13.2",
                "toolchain": "nightly-2026-07-16",
                "total_executions": 10_000 * len(TARGETS),
            },
            sort_keys=True,
            separators=(",", ":"),
        )
        + "\n",
        encoding="utf-8",
    )
    print("PASS: sustained Rust fuzz campaign")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
