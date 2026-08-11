#!/usr/bin/env python3
"""Validate all durable remediation authority and negative invariants."""

from __future__ import annotations

import copy
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def validate_findings(value: dict[str, object]) -> None:
    findings = value.get("findings")
    if not isinstance(findings, list) or len(findings) != 13:
        raise AssertionError("missing remediation finding")
    ids = [item.get("id") for item in findings if isinstance(item, dict)]
    if ids != [f"FINDING_{index:03d}" for index in range(1, 14)] or len(ids) != len(set(ids)):
        raise AssertionError("missing, duplicate, or reordered remediation finding")
    statuses = [item.get("status") for item in findings if isinstance(item, dict)]
    if statuses != ["closed"] * 11 + ["closed_with_release_hold", "closed"]:
        raise AssertionError("invalid remediation disposition")

def expect_rejected(value: dict[str, object], message: str) -> None:
    try:
        validate_findings(value)
    except AssertionError:
        return
    raise AssertionError(message)

def main() -> int:
    for script in (
        "validate_remediation_baseline.py",
        "validate_remediation_findings.py",
        "validate_remediation_ledger.py",
    ):
        subprocess.run([sys.executable, str(ROOT / "scripts" / script)], cwd=ROOT, check=True)
    value = json.loads((ROOT / "spec/remediation_findings.json").read_text())
    validate_findings(value)
    missing = copy.deepcopy(value)
    missing["findings"].pop()
    expect_rejected(missing, "missing finding passed")
    duplicate = copy.deepcopy(value)
    duplicate["findings"][1]["id"] = "FINDING_001"
    expect_rejected(duplicate, "duplicate finding passed")
    invalid_status = copy.deepcopy(value)
    invalid_status["findings"][11]["status"] = "closed"
    expect_rejected(invalid_status, "release hold was erased")
    misplaced_hold = copy.deepcopy(value)
    misplaced_hold["findings"][10]["status"] = "closed_with_release_hold"
    expect_rejected(misplaced_hold, "release hold was assigned to the wrong finding")
    closure = json.loads((ROOT / "reports/remediation_closure.json").read_text())
    closure_findings = closure.get("findings", [])
    closure_ids = [item.get("id") for item in closure_findings]
    closure_results = [item.get("result") for item in closure_findings]
    if closure.get("schema") != "nostr_automerge.remediation_closure.v2":
        raise AssertionError("remediation closure schema is invalid")
    if closure_ids != [f"FINDING_{index:03d}" for index in range(14, 28)]:
        raise AssertionError("remediation closure findings are incomplete or reordered")
    if closure_results != ["closed"] * 13 + ["resolved_with_release_holds"]:
        raise AssertionError("remediation closure dispositions are invalid")
    for finding in closure_findings:
        if not finding.get("commits") or not finding.get("evidence"):
            raise AssertionError(f"closure evidence is missing: {finding.get('id')}")
        for path in finding["evidence"]:
            if not (ROOT / path).exists():
                raise AssertionError(f"stale closure evidence: {finding['id']}:{path}")
    rcld = (ROOT / "docs/execution/rcl/nostr_automerge_v1_14_engine_remediation_rcld.md").read_text()
    steps = [int(item) for item in re.findall(r"^\| `step_(\d{3})` \|", rcld, re.MULTILINE)]
    if steps != list(range(193, 308)):
        raise AssertionError("remediation RCLD steps are incomplete or reordered")
    adrs = sorted((ROOT / "docs/adr").glob("adr_[0-9][0-9][0-9][0-9]_*.md"))
    if len(adrs) != 32 or any("## Decision" not in path.read_text() for path in adrs[12:]):
        raise AssertionError("remediation ADR is missing or invalid")
    report_path = ROOT / "reports/remediation_phase_00.json"
    if report_path.exists():
        report = json.loads(report_path.read_text())
        if set(report) != {"adaptations", "completed_steps", "finding_status", "next_step", "result", "schema", "through_commit", "verification"}:
            raise AssertionError("phase 00 report fields are incomplete or unknown")
        if report["completed_steps"] != [f"step_{number:03d}" for number in range(193, 200)]:
            raise AssertionError("phase 00 completion range is invalid")
        if report["result"] != "pass" or report["finding_status"] != "13_open" or report["next_step"] != "step_201":
            raise AssertionError("phase 00 result is inconsistent")
        if report["verification"].get("tracked_workflows") != 0:
            raise AssertionError("tracked workflow claim is invalid")
    print("PASS: remediation authority and negative invariants")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
