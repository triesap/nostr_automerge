#!/usr/bin/env python3
"""Validate exact resource qualifications without weakening held campaigns."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
HEX40 = re.compile(r"^[0-9a-f]{40}$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")
QUALIFICATIONS = {
    "target_scaling", "unrelated_control_flood", "exact_budget_boundaries",
    "cancellation_boundaries", "partial_report_settlement",
    "constant_no_progress_fallback", "peak_memory_observed",
}
HOLDS = {
    "rust_source_mutation", "typescript_source_mutation", "sustained_fuzzing",
    "sustained_generative_campaign",
}


def main() -> int:
    report = json.loads((ROOT / "reports/resource_qualification_v9.json").read_text())
    if report.get("schema") != "nostr_automerge.resource_qualification.v9" or report.get("status") != "pass_with_explicit_holds":
        raise AssertionError("qualification_status")
    rust = report.get("rust", {})
    source = rust.get("source_candidate", "")
    evidence = rust.get("evidence_candidate", "")
    if not HEX40.fullmatch(source) or not HEX40.fullmatch(evidence):
        raise AssertionError("rust_candidates")
    for candidate in (source, evidence):
        if subprocess.run(("git", "merge-base", "--is-ancestor", candidate, "HEAD"), cwd=ROOT, check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode:
            raise AssertionError("stale_candidate")
    smoke = rust.get("resource_smoke", {})
    if (
        smoke.get("result") != "pass"
        or not str(smoke.get("elapsed_ns", "")).isdigit()
        or not isinstance(smoke.get("maximum_resident_set_bytes"), int)
        or smoke["maximum_resident_set_bytes"] <= 0
        or not HEX64.fullmatch(str(smoke.get("raw_evidence_sha256", "")))
        or smoke.get("measurement_scope") != "operator-local child-process upper bound"
    ):
        raise AssertionError("resource_measurement")
    qualifications = rust.get("qualifications", {})
    if set(qualifications) != QUALIFICATIONS or set(qualifications.values()) != {"pass"}:
        raise AssertionError("qualification_coverage")
    commands = rust.get("commands")
    if not isinstance(commands, list) or len(commands) != 5 or len(set(commands)) != 5:
        raise AssertionError("qualification_commands")
    typescript_path = ROOT / "reports/interop_typescript_v9.json"
    typescript = report.get("typescript", {})
    attestation = json.loads(typescript_path.read_text())
    if (
        typescript.get("implementation_candidate") != attestation["commit"]
        or typescript.get("evidence_candidate") != attestation["evidence_commit"]
        or typescript.get("attestation_sha256") != hashlib.sha256(typescript_path.read_bytes()).hexdigest()
        or typescript.get("ordinary_resource_lane") != "pass"
        or typescript.get("source_only") is not True
    ):
        raise AssertionError("typescript_resource_binding")
    held = report.get("held", {})
    if set(held) != HOLDS or set(held.values()) != {"held_operator_safety"}:
        raise AssertionError("held_campaigns")
    print("PASS: resource qualification covers all ordinary v8 resource claims and holds")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
