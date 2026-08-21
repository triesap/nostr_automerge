#!/usr/bin/env python3
"""Validate ordinary assurance and its source-only publication boundary."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
HEX40 = re.compile(r"^[0-9a-f]{40}$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")
HOLDS = [
    "source-mutating campaigns", "sustained fuzzing", "independent external review",
    "production-readiness authorization", "NIP submission and event-kind allocation", "publication",
]


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    report = json.loads((ROOT / "reports/ordinary_assurance_v9.json").read_text())
    if report.get("schema") != "nostr_automerge.ordinary_assurance.v9" or report.get("status") != "pass_with_explicit_holds":
        raise AssertionError("assurance_status")
    for field in ("source_candidate", "evidence_candidate"):
        candidate = report.get(field, "")
        if not HEX40.fullmatch(candidate) or subprocess.run(("git", "merge-base", "--is-ancestor", candidate, "HEAD"), cwd=ROOT, check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode:
            raise AssertionError(field)
    coverage = report.get("coverage", {})
    if coverage.get("result") != "measured" or not HEX64.fullmatch(str(coverage.get("raw_evidence_sha256", ""))):
        raise AssertionError("coverage")
    if any(not isinstance(coverage.get(field), float) or not 0.0 <= coverage[field] <= 100.0 for field in ("regions_percent", "functions_percent", "lines_percent", "branches_percent")):
        raise AssertionError("coverage_metrics")
    local_coverage = ROOT / ".local/evidence/rust_coverage.txt"
    if local_coverage.exists() and sha256(local_coverage) != coverage["raw_evidence_sha256"]:
        raise AssertionError("coverage_evidence_changed")
    if report.get("package") != {"result": "pass", "file_count": 132, "source_only": True}:
        raise AssertionError("package")
    dependency = report.get("dependency", {})
    if (
        dependency.get("cargo_lock_sha256") != sha256(ROOT / "Cargo.lock")
        or dependency.get("cargo_deny") != "pass_with_documented_warnings"
        or dependency.get("licenses") != "pass_with_two_unused_allowance_warnings"
        or dependency.get("sources") != "pass"
        or dependency.get("bans") != "pass_with_duplicate_version_warnings"
    ):
        raise AssertionError("dependency")
    if report.get("advisory") != {"result": "pass_after_fresh_temporary_database", "advisories_loaded": 1225, "dependencies_scanned": 113, "vulnerabilities": 0}:
        raise AssertionError("advisory")
    sbom = report.get("sbom", {})
    if sbom.get("result") != "generated" or sbom.get("format") != "CycloneDX" or sbom.get("spec_version") != "1.5" or sbom.get("component_count") != 110 or not HEX64.fullmatch(str(sbom.get("sha256", ""))):
        raise AssertionError("sbom")
    local_sbom = ROOT / ".local/evidence/nostr_automerge.cdx.json"
    if local_sbom.exists() and sha256(local_sbom) != sbom["sha256"]:
        raise AssertionError("sbom_evidence_changed")
    tracked = subprocess.run(("git", "ls-files"), cwd=ROOT, check=True, capture_output=True, text=True).stdout.splitlines()
    if any(path.startswith(".act/") or path.startswith(".github/workflows/") for path in tracked):
        raise AssertionError("tracked_workflows")
    if report.get("tracked_workflows_present") is not False or report.get("repository_policy") != "pass" or report.get("documentation") != "pass_with_denied_warnings":
        raise AssertionError("source_only_policy")
    if report.get("resource_qualification_sha256") != sha256(ROOT / "reports/resource_qualification_v9.json"):
        raise AssertionError("resource_binding")
    if report.get("holds") != HOLDS:
        raise AssertionError("external_holds")
    print("PASS: ordinary assurance is exact source-only and retains all external holds")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
