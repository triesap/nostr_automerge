#!/usr/bin/env python3
"""Validate direct and root-private workflow proof for remediation v8."""

from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
HEX40 = re.compile(r"^[0-9a-f]{40}$")
FORBIDDEN = ("/" + "Users/", "/" + "home/", "domains/" + "labs", "triesap/" + "dev", ".act" + "/", ".github/" + "workflows")


def main() -> int:
    path = ROOT / "reports/local_gate_summary_v8.json"
    report = json.loads(path.read_text())
    identity = json.loads((ROOT / "reports/final_candidate_identity_v8.json").read_text())
    if report.get("schema") != "nostr_automerge.local_gate_summary.v8" or report.get("result") != "pass_with_explicit_holds":
        raise AssertionError("local_gate_status")
    if report.get("act_version") != "0.2.89" or report.get("ownership") != "root-private-untracked":
        raise AssertionError("workflow_runtime")
    if set(report.get("direct_gates", {}).values()) != {"pass"}:
        raise AssertionError("direct_gates")
    workflows = report.get("operator_workflows", {})
    if (
        workflows.get("remediation") != "pass"
        or workflows.get("interop") != "pass"
        or workflows.get("readiness") != "pass_holds_validated"
        or workflows.get("local_suite") != {"job_count": 14, "result": "pass"}
    ):
        raise AssertionError("operator_workflows")
    if report.get("source_repositories") != {"rust_tracked_workflows": 0, "typescript_tracked_workflows": 0}:
        raise AssertionError("source_workflows")
    tracked = subprocess.run(("git", "ls-files"), cwd=ROOT, check=True, capture_output=True, text=True).stdout.splitlines()
    if any(item.startswith(".act/") or item.startswith(".github/workflows/") for item in tracked):
        raise AssertionError("tracked_rust_workflow")
    for field in ("public_source_candidate", "public_evidence_candidate", "typescript_attestation_candidate"):
        if not HEX40.fullmatch(str(report.get(field, ""))):
            raise AssertionError(field)
    if (
        report["public_source_candidate"] != identity["rust"]["source_candidate"]
        or report["public_evidence_candidate"] != identity["rust"]["evidence_base_candidate"]
        or report["typescript_attestation_candidate"] != identity["typescript"]["attestation_candidate"]
    ):
        raise AssertionError("candidate_binding")
    for candidate in (report["public_source_candidate"], report["public_evidence_candidate"]):
        if subprocess.run(("git", "merge-base", "--is-ancestor", candidate, "HEAD"), cwd=ROOT, check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode:
            raise AssertionError("candidate_ancestor")
    if report.get("held_campaigns") != {"source_mutation": "held", "sustained_fuzzing": "held"}:
        raise AssertionError("held_campaigns")
    if report.get("remote_actions_performed") is not False:
        raise AssertionError("remote_actions")
    if any(token in path.read_text() for token in FORBIDDEN):
        raise AssertionError("private_workflow_material")
    print("PASS: all direct and root-private local workflow gates passed with holds")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
