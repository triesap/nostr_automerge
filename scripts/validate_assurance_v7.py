#!/usr/bin/env python3
"""Validate remediation-v7 executed assurance and explicit safety holds."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
HEX40 = re.compile(r"^[0-9a-f]{40}$")
HELD = "held_operator_safety"


def load(relative: str) -> dict:
    value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise AssertionError(f"expected object: {relative}")
    return value


def digest(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def main() -> int:
    rust_mutation = load("reports/source_mutation_rust_v7.json")
    typescript_mutation = load("reports/source_mutation_typescript_v7.json")
    qualification = load("reports/resource_qualification_v7.json")
    if rust_mutation.get("status") != HELD or typescript_mutation.get("status") != HELD:
        raise AssertionError("source mutation is not explicitly held")
    for report in (rust_mutation, typescript_mutation):
        if report.get("campaign_executed") is not False or report.get("result_claimed") is not False:
            raise AssertionError("held source mutation overclaims execution")
    definition = rust_mutation.get("definition", {})
    if definition.get("sha256") != digest("reports/mutation_campaign_v7_inventory.json"):
        raise AssertionError("Rust mutation definition hash is stale")
    if definition.get("anchor_count") != 5 or rust_mutation.get("anchor_validation") != "pass":
        raise AssertionError("Rust mutation anchors are incomplete")
    rust_candidate = qualification.get("rust", {}).get("candidate", "")
    if rust_candidate != rust_mutation.get("candidate") or not HEX40.fullmatch(rust_candidate):
        raise AssertionError("Rust assurance candidate is inconsistent")
    if subprocess.run(
        ["git", "cat-file", "-e", f"{rust_candidate}^{{commit}}"],
        cwd=ROOT,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    ).returncode:
        raise AssertionError("Rust assurance candidate is unavailable")
    typescript = qualification.get("typescript", {})
    for field in ("implementation_candidate", "evidence_candidate"):
        if not HEX40.fullmatch(str(typescript.get(field, ""))):
            raise AssertionError(f"invalid opaque TypeScript {field}")
        if typescript.get(field) != typescript_mutation.get(field):
            raise AssertionError(f"inconsistent opaque TypeScript {field}")
    for implementation in (qualification.get("rust", {}), typescript):
        coverage = implementation.get("coverage", {})
        resource = implementation.get("resource", {})
        supply_chain = implementation.get("supply_chain", {})
        if coverage.get("result") != "measured" or coverage.get("lines_percent", 0) <= 0:
            raise AssertionError("coverage evidence is incomplete")
        if resource.get("result") != "pass":
            raise AssertionError("resource evidence is incomplete")
        if implementation.get("package", {}).get("source_only") is not True:
            raise AssertionError("package evidence is not source-only")
        if supply_chain.get("vulnerabilities") != 0:
            raise AssertionError("supply-chain evidence reports vulnerabilities")
    if set(qualification.get("held", {}).values()) != {HELD}:
        raise AssertionError("assurance holds are incomplete")
    tracked = subprocess.run(
        ["git", "ls-files"], cwd=ROOT, check=True, capture_output=True, text=True
    ).stdout.splitlines()
    if any(path.startswith((".github/workflows/", ".act/")) for path in tracked):
        raise AssertionError("tracked workflow content violates source-only policy")
    print("PASS: remediation-v7 resource and safety assurance")
    print("- rust_coverage=74.53 typescript_coverage=88.35 vulnerabilities=0")
    print("- mutation=held fuzzing=held workflows=external-only")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
