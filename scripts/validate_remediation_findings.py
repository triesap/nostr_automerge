#!/usr/bin/env python3
"""Validate the repository-owned remediation findings registry."""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def main() -> int:
    value = json.loads((ROOT / "spec/remediation_findings.json").read_text())
    findings = value["findings"]
    ids = [item["id"] for item in findings]
    expected = [f"FINDING_{index:03d}" for index in range(1, 14)]
    if value["schema"] != "nostr_automerge.remediation_findings.v2" or ids != expected:
        raise AssertionError("finding IDs or schema are invalid")
    expected_statuses = ["closed"] * 11 + ["closed_with_release_hold", "closed"]
    if len(ids) != len(set(ids)) or [item["status"] for item in findings] != expected_statuses:
        raise AssertionError("finding closure statuses are invalid")
    if any(item["severity"] not in {"blocker", "critical", "high", "medium"} for item in findings):
        raise AssertionError("invalid finding severity")
    for item in findings:
        if not item["title"] or not item["phase"] or not item["evidence_paths"]:
            raise AssertionError(f"incomplete finding: {item['id']}")
        for path in item["evidence_paths"]:
            if not (ROOT / path).exists():
                raise AssertionError(f"stale evidence path: {item['id']}:{path}")
    print("PASS: 13 unique remediation findings are dispositioned and traceable")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
