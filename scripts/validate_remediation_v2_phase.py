#!/usr/bin/env python3
"""Validate the follow-up authority and baseline phase report."""

from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    report = json.loads((ROOT / "reports/remediation_v2_phase_00.json").read_text())
    if report.get("phase") != "phase_00_authority_and_baseline" or report.get("status") != "pass":
        raise AssertionError("authority and baseline phase is not passing")
    expected_steps = [f"step_{step:03d}" for step in range(308, 318)]
    if report.get("completed_steps") != expected_steps:
        raise AssertionError("phase report checkpoint coverage is incomplete")
    if set(report.get("commits", {})) != set(expected_steps):
        raise AssertionError("phase report commit binding is incomplete")
    if any(command.get("result") != "pass" for command in report.get("commands", [])):
        raise AssertionError("phase report contains a nonpassing command")
    required_categories = ("fmt", "check", "test", "clippy", "doc", "validate")
    command_text = " ".join(command["command"] for command in report["commands"])
    standard = "scripts/local_gate.py standard"
    gate_source = (ROOT / "scripts/local_gate.py").read_text()
    if standard not in command_text or not all(category in gate_source for category in required_categories):
        raise AssertionError("phase report does not bind the complete standard gate")
    for relative, expected in report.get("artifact_sha256", {}).items():
        if sha256(ROOT / relative) != expected:
            raise AssertionError(f"phase artifact hash mismatch: {relative}")
    baseline = report.get("reviewed_baseline_commit")
    commit = subprocess.run(
        ["git", "cat-file", "-e", f"{baseline}^{{commit}}"], cwd=ROOT, check=False
    )
    if commit.returncode:
        raise AssertionError("reviewed baseline commit does not exist")
    if report.get("next") != {"rcld": 16, "checkpoint": "step_318"}:
        raise AssertionError("phase report does not activate the exact next checkpoint")
    if report.get("publication_authorized") is not False:
        raise AssertionError("phase report cannot authorize publication")
    print("PASS: authority and baseline phase report activates step_318")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
