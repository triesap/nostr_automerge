#!/usr/bin/env python3
"""Generate the truthful remediation-v8 local closure report."""

from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = [
    "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v8.md",
    "docs/execution/remediation_v8/ledger.md",
    "spec/remediation_findings_v8.json",
    "reports/final_candidate_identity_v8.json",
    "reports/local_gate_summary_v8.json",
    "reports/requirements_coverage_v9.json",
    "reports/interop_combined_v9.json",
    "reports/ordinary_assurance_v9.json",
    "reports/private_assurance_v9.json",
    "reports/resource_qualification_v9.json",
]
HOLDS = [
    "source-mutating campaigns", "sustained fuzzing", "independent external review",
    "production-readiness authorization", "NIP submission and event-kind allocation", "publication",
]


def sha256(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def git(*args: str) -> str:
    return subprocess.run(("git", *args), cwd=ROOT, check=True, capture_output=True, text=True).stdout.strip()


def main() -> int:
    identity = json.loads((ROOT / "reports/final_candidate_identity_v8.json").read_text())
    gates = json.loads((ROOT / "reports/local_gate_summary_v8.json").read_text())
    findings = json.loads((ROOT / "spec/remediation_findings_v8.json").read_text())["findings"]
    report = {
        "schema": "nostr_automerge.remediation_v8_final.v1",
        "status": "code_complete_publication_held",
        "local_implementation": "pass",
        "publication_authorized": False,
        "remote_actions_performed": False,
        "sequence": {
            "first_step": 1096, "last_step": 1157, "checkpoint_count": 62,
            "completed_rclds": list(range(73, 81)), "unfinished_rclds": [],
        },
        "findings": {
            "closed": [row["id"] for row in findings if row["status"] == "closed"],
            "held": [row["id"] for row in findings if row["status"] == "held"],
        },
        "candidates": {
            "rust_source": identity["rust"]["source_candidate"],
            "rust_evidence_base": identity["rust"]["evidence_base_candidate"],
            "final_identity": git("log", "-1", "--format=%H", "--", "reports/final_candidate_identity_v8.json"),
            "final_gate_evidence": git("rev-parse", "HEAD"),
            "typescript_implementation": identity["typescript"]["implementation_candidate"],
            "typescript_evidence": identity["typescript"]["evidence_candidate"],
            "typescript_attestation": identity["typescript"]["attestation_candidate"],
        },
        "authority": identity["authority"],
        "evidence": {path: sha256(path) for path in EVIDENCE},
        "ordinary_direct_gates": gates["direct_gates"],
        "operator_local_workflows": gates["operator_workflows"],
        "source_repository_workflows": gates["source_repositories"],
        "held": HOLDS,
    }
    (ROOT / "reports/remediation_v8_final.json").write_text(
        json.dumps(report, sort_keys=True, separators=(",", ":")) + "\n", encoding="utf-8"
    )
    print("PASS: generated remediation-v8 code-complete publication-held closure")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
