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


def sha256_at_commit(commit: str, relative: str) -> str:
    blob = subprocess.run(
        ["git", "show", f"{commit}:{relative}"],
        cwd=ROOT,
        check=True,
        stdout=subprocess.PIPE,
    ).stdout
    return hashlib.sha256(blob).hexdigest()


def artifact_matches(
    report: dict[str, object], report_path: str, relative: str, expected: str
) -> bool:
    candidates = [sha256(ROOT / relative), sha256_at_commit(str(report["phase_input_head"]), relative)]
    report_commit = subprocess.run(
        ["git", "log", "--diff-filter=A", "-1", "--format=%H", "--", report_path],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if report_commit:
        candidates.append(sha256_at_commit(report_commit, relative))
    return expected in candidates


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
        if not artifact_matches(report, "reports/remediation_v2_phase_00.json", relative, expected):
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

    control = json.loads((ROOT / "reports/remediation_v2_phase_01.json").read_text())
    if control.get("phase") != "phase_01_stateful_control_candidates" or control.get("status") != "pass":
        raise AssertionError("stateful control candidate phase is not passing")
    control_steps = [f"step_{step:03d}" for step in range(318, 337)]
    if control.get("completed_steps") != control_steps:
        raise AssertionError("control candidate phase checkpoint coverage is incomplete")
    if set(control.get("commits", {})) != set(control_steps):
        raise AssertionError("control candidate phase commit binding is incomplete")
    if control.get("findings") != ["FINDING_014", "FINDING_015"]:
        raise AssertionError("control candidate phase findings are incomplete")
    tests = control.get("tests", {})
    if any(len(tests.get(finding, [])) < 4 for finding in control["findings"]):
        raise AssertionError("control candidate phase lacks direct test evidence")
    if any(command.get("result") != "pass" for command in control.get("commands", [])):
        raise AssertionError("control candidate phase contains a nonpassing command")
    for relative, expected in control.get("artifact_sha256", {}).items():
        if not artifact_matches(control, "reports/remediation_v2_phase_01.json", relative, expected):
            raise AssertionError(f"control candidate artifact hash mismatch: {relative}")
    for commit_hash in control["commits"].values():
        if commit_hash == "self":
            continue
        commit = subprocess.run(
            ["git", "cat-file", "-e", f"{commit_hash}^{{commit}}"], cwd=ROOT, check=False
        )
        if commit.returncode:
            raise AssertionError(f"control candidate commit does not exist: {commit_hash}")
    if control.get("next") != {"rcld": 17, "checkpoint": "step_337"}:
        raise AssertionError("control candidate report does not activate step_337")
    if control.get("publication_authorized") is not False:
        raise AssertionError("control candidate report cannot authorize publication")

    interleaved = json.loads((ROOT / "reports/remediation_v2_phase_02.json").read_text())
    if interleaved.get("phase") != "phase_02_interleaved_epoch_control_engine" or interleaved.get("status") != "pass":
        raise AssertionError("interleaved epoch/control phase is not passing")
    interleaved_steps = [f"step_{step:03d}" for step in range(337, 356)]
    if interleaved.get("completed_steps") != interleaved_steps:
        raise AssertionError("interleaved phase checkpoint coverage is incomplete")
    if set(interleaved.get("commits", {})) != set(interleaved_steps):
        raise AssertionError("interleaved phase commit binding is incomplete")
    expected_findings = ["FINDING_014", "FINDING_015", "FINDING_016", "FINDING_018", "FINDING_025"]
    if interleaved.get("findings") != expected_findings:
        raise AssertionError("interleaved phase findings are incomplete")
    tests = interleaved.get("tests", {})
    if any(len(tests.get(finding, [])) < 2 for finding in expected_findings):
        raise AssertionError("interleaved phase lacks direct test evidence")
    if any(command.get("result") != "pass" for command in interleaved.get("commands", [])):
        raise AssertionError("interleaved phase contains a nonpassing command")
    for relative, expected in interleaved.get("artifact_sha256", {}).items():
        if not artifact_matches(interleaved, "reports/remediation_v2_phase_02.json", relative, expected):
            raise AssertionError(f"interleaved artifact hash mismatch: {relative}")
    for commit_hash in interleaved["commits"].values():
        if commit_hash == "self":
            continue
        commit = subprocess.run(
            ["git", "cat-file", "-e", f"{commit_hash}^{{commit}}"], cwd=ROOT, check=False
        )
        if commit.returncode:
            raise AssertionError(f"interleaved commit does not exist: {commit_hash}")
    if interleaved.get("next") != {"rcld": 18, "checkpoint": "step_356"}:
        raise AssertionError("interleaved report does not activate step_356")
    if interleaved.get("publication_authorized") is not False:
        raise AssertionError("interleaved phase report cannot authorize publication")
    print("PASS: phase reports activate interleaved controls then step_356")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
