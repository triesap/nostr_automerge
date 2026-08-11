#!/usr/bin/env python3
"""Validate finding source anchors against the reviewed Git commit."""

from __future__ import annotations

import argparse
import copy
import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def git_object(commit: str, path: str) -> str:
    result = subprocess.run(
        ["git", "rev-parse", f"{commit}:{path}"],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode:
        raise AssertionError(f"source anchor does not resolve: {commit}:{path}")
    return result.stdout.strip()


def validate(report: dict, findings: dict) -> None:
    if report.get("schema") != "nostr_automerge.source_anchors.v2":
        raise AssertionError("unexpected source-anchor schema")
    commit = findings.get("reviewed_rust_commit")
    if report.get("reviewed_commit") != commit:
        raise AssertionError("source anchors do not bind the reviewed commit")
    expected = sorted({path for finding in findings["findings"] for path in finding["paths"]})
    anchors = report.get("anchors", [])
    paths = [anchor.get("path") for anchor in anchors]
    if paths != expected or len(paths) != len(set(paths)):
        raise AssertionError("finding source anchors are missing, duplicated, or reordered")
    for anchor in anchors:
        path = anchor["path"]
        if git_object(commit, path) != anchor.get("baseline_git_object"):
            raise AssertionError(f"stale baseline object for source anchor: {path}")


def expect_rejected(report: dict, findings: dict, reason: str) -> None:
    try:
        validate(report, findings)
    except AssertionError:
        return
    raise AssertionError(f"invalid source anchors accepted: {reason}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    report = json.loads((ROOT / "reports/source_anchors_v2.json").read_text())
    findings = json.loads((ROOT / "spec/remediation_v2_findings.json").read_text())
    validate(report, findings)
    if args.self_test:
        missing = copy.deepcopy(report)
        missing["anchors"].pop()
        expect_rejected(missing, findings, "deleted path")
        stale = copy.deepcopy(report)
        stale["anchors"][0]["baseline_git_object"] = "0" * 40
        expect_rejected(stale, findings, "stale hash")
    print(f"PASS: {len(report['anchors'])} source anchors resolve at {report['reviewed_commit']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
