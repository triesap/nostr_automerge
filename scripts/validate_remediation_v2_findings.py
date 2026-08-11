#!/usr/bin/env python3
"""Validate follow-up findings and remediation requirements."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    finding_registry = json.loads((ROOT / "spec/remediation_v2_findings.json").read_text())
    requirement_registry = json.loads(
        (ROOT / "spec/remediation_v2_requirements.json").read_text()
    )
    findings = finding_registry["findings"]
    finding_ids = [item["id"] for item in findings]
    expected_findings = [f"FINDING_{index:03d}" for index in range(14, 28)]
    if finding_ids != expected_findings or len(set(finding_ids)) != 14:
        raise AssertionError("follow-up findings are missing, duplicated, or reordered")
    if any(item["status"] != "open" for item in findings):
        raise AssertionError("a follow-up finding was prematurely closed")
    severities = {"blocker", "critical", "high", "medium", "release_hold"}
    reviewed = finding_registry["reviewed_rust_commit"]
    for item in findings:
        if item["severity"] not in severities or not item["paths"] or not item["phases"]:
            raise AssertionError(f"incomplete finding: {item['id']}")
        for raw_path in item["paths"]:
            exists = subprocess.run(
                ["git", "cat-file", "-e", f"{reviewed}:{raw_path}"],
                cwd=ROOT,
                capture_output=True,
            ).returncode == 0
            if not exists:
                raise AssertionError(f"stale finding path: {item['id']}:{raw_path}")
    requirements = requirement_registry["requirements"]
    requirement_ids = [item["id"] for item in requirements]
    if requirement_registry["count"] != 68 or len(requirements) != 68:
        raise AssertionError("follow-up remediation requirement count must be 68")
    if len(set(requirement_ids)) != len(requirement_ids):
        raise AssertionError("duplicate follow-up remediation requirement")
    known = set(finding_ids)
    for item in requirements:
        if item["finding"] not in known or not all(
            isinstance(item[field], str) and item[field]
            for field in ("id", "title", "statement", "acceptance")
        ):
            raise AssertionError(f"invalid remediation requirement: {item.get('id')}")
    referenced = {item["finding"] for item in requirements}
    if referenced != known:
        raise AssertionError("every finding must own at least one requirement")
    print("PASS: 14 open findings and 68 remediation requirements are traceable")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
