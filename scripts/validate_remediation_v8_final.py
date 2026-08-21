#!/usr/bin/env python3
"""Validate all 62 remediation-v8 checkpoints and truthful final status."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
HEX40 = re.compile(r"^[0-9a-f]{40}$")
HOLDS = [
    "source-mutating campaigns", "sustained fuzzing", "independent external review",
    "production-readiness authorization", "NIP submission and event-kind allocation", "publication",
]
FORBIDDEN = ("/" + "Users/", "/" + "home/", "domains/" + "labs", "triesap/" + "dev", ".act" + "/", ".github/" + "workflows")


def digest(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def main() -> int:
    path = ROOT / "reports/remediation_v8_final.json"
    report = json.loads(path.read_text())
    identity = json.loads((ROOT / "reports/final_candidate_identity_v8.json").read_text())
    gates = json.loads((ROOT / "reports/local_gate_summary_v8.json").read_text())
    findings = json.loads((ROOT / "spec/remediation_findings_v8.json").read_text())["findings"]
    plan = (ROOT / "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v8.md").read_text()
    ledger = (ROOT / "docs/execution/remediation_v8/ledger.md").read_text()
    if (
        report.get("schema") != "nostr_automerge.remediation_v8_final.v1"
        or report.get("status") != "code_complete_publication_held"
        or report.get("local_implementation") != "pass"
        or report.get("publication_authorized") is not False
        or report.get("remote_actions_performed") is not False
    ):
        raise AssertionError("closure_status")
    sequence = report.get("sequence")
    if sequence != {"first_step": 1096, "last_step": 1157, "checkpoint_count": 62, "completed_rclds": list(range(73, 81)), "unfinished_rclds": []}:
        raise AssertionError("sequence")
    expected_steps = set(range(1096, 1158))
    if {int(value) for value in re.findall(r"`step_(\d+)`", plan)} != expected_steps:
        raise AssertionError("plan_steps")
    if not all(f"`step_{value}`" in ledger for value in expected_steps):
        raise AssertionError("ledger_steps")
    if (
        "Status: complete — `code_complete_publication_held`" not in plan
        or "None. All 62 checkpoints" not in plan
        or "Status: `code_complete_publication_held`" not in ledger
        or "None. Remediation v8 is complete" not in ledger
    ):
        raise AssertionError("execution_status")
    statuses = {row["id"]: row["status"] for row in findings}
    if statuses != {**{f"FINDING_{number:03d}": "closed" for number in range(66, 72)}, "FINDING_072": "held"}:
        raise AssertionError("finding_status")
    if report.get("findings") != {"closed": [f"FINDING_{number:03d}" for number in range(66, 72)], "held": ["FINDING_072"]}:
        raise AssertionError("finding_binding")
    candidates = report.get("candidates", {})
    for field in ("rust_source", "rust_evidence_base", "final_identity", "final_gate_evidence", "typescript_implementation", "typescript_evidence", "typescript_attestation"):
        if not HEX40.fullmatch(str(candidates.get(field, ""))):
            raise AssertionError(f"candidate:{field}")
    for field in ("rust_source", "rust_evidence_base", "final_identity", "final_gate_evidence"):
        if subprocess.run(("git", "merge-base", "--is-ancestor", candidates[field], "HEAD"), cwd=ROOT, check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode:
            raise AssertionError(f"candidate_ancestor:{field}")
    if (
        candidates["rust_source"] != identity["rust"]["source_candidate"]
        or candidates["rust_evidence_base"] != identity["rust"]["evidence_base_candidate"]
        or candidates["typescript_implementation"] != identity["typescript"]["implementation_candidate"]
        or candidates["typescript_evidence"] != identity["typescript"]["evidence_candidate"]
        or candidates["typescript_attestation"] != identity["typescript"]["attestation_candidate"]
    ):
        raise AssertionError("candidate_binding")
    if report.get("authority") != identity["authority"]:
        raise AssertionError("authority")
    evidence = report.get("evidence")
    if not isinstance(evidence, dict) or any(digest(relative) != value for relative, value in evidence.items()):
        raise AssertionError("evidence")
    if report.get("ordinary_direct_gates") != gates["direct_gates"] or report.get("operator_local_workflows") != gates["operator_workflows"] or report.get("source_repository_workflows") != gates["source_repositories"]:
        raise AssertionError("gate_binding")
    if report.get("held") != HOLDS:
        raise AssertionError("holds")
    if any(token in path.read_text() for token in FORBIDDEN):
        raise AssertionError("private_material")
    print("PASS: all 62 remediation-v8 checkpoints are complete with finding 072 held")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
