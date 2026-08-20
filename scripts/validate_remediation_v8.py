#!/usr/bin/env python3
"""Fail-closed authority and execution validation for remediation v8."""

from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
FINDINGS = tuple(f"FINDING_{number:03d}" for number in range(66, 73))
REQUIREMENTS = (
    "NCRDT-BRANCH-003",
    "NCRDT-BRANCH-004",
    "NCRDT-SCOPE-007",
    "NCRDT-RESOURCE-011",
    "NCRDT-RESOURCE-012",
    "NCRDT-DISPOSITION-004",
    "NCRDT-DISPOSITION-005",
    "NCRDT-NIP-003",
    "NCRDT-CONF-009",
    "NCRDT-EVIDENCE-005",
)


def load(relative: str) -> dict[str, object]:
    value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise AssertionError(f"{relative} must contain an object")
    return value


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def validate_baseline() -> None:
    baseline = load("reports/remediation_v8_baseline.json")
    require(
        baseline.get("schema") == "nostr_automerge.remediation_v8_baseline.v1",
        "baseline schema",
    )
    require(baseline.get("remote_actions_authorized") is False, "remote boundary")
    require(baseline.get("maximum_local_status") == "code_complete_publication_held", "status ceiling")
    counts = baseline.get("counts")
    require(isinstance(counts, dict), "baseline counts")
    require(counts == {"requirements": 129, "signed_fixtures": 171}, "baseline counts")


def validate_findings_and_requirements() -> None:
    findings = load("spec/remediation_findings_v8.json").get("findings")
    require(isinstance(findings, list), "findings array")
    require(tuple(row.get("id") for row in findings if isinstance(row, dict)) == FINDINGS, "finding order")
    require(isinstance(findings[-1], dict) and findings[-1].get("status") == "held", "finding 072 hold")

    additions = load("spec/remediation_v8_requirements.json")
    rows = additions.get("requirements")
    require(isinstance(rows, list) and len(rows) == 10, "addition count")
    require(tuple(row.get("id") for row in rows if isinstance(row, dict)) == REQUIREMENTS, "addition order")
    registry = load("spec/requirements.json")
    canonical = registry.get("requirements")
    require(registry.get("schema") == "nostr_automerge.requirements.v6", "registry schema")
    require(isinstance(canonical, list) and len(canonical) == registry.get("requirement_count") == 139, "registry count")
    require(tuple(row.get("id") for row in canonical[-10:] if isinstance(row, dict)) == REQUIREMENTS, "registry append")

    applicability = load("spec/requirements_applicability.json")
    classifications = applicability.get("classifications")
    require(applicability.get("schema") == "nostr_automerge.requirements_applicability.v6", "applicability schema")
    require(isinstance(classifications, dict), "applicability map")
    require(tuple(classifications) == tuple(row.get("id") for row in canonical if isinstance(row, dict)), "applicability order")


def validate_plan_and_ledger() -> None:
    plan = (ROOT / "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v8.md").read_text(encoding="utf-8")
    steps = {int(value) for value in re.findall(r"`step_(\d+)`", plan)}
    require(set(range(1096, 1158)).issubset(steps), "contiguous checkpoint inventory")
    for rcld in range(73, 81):
        require(f"## RCLD {rcld} " in plan, f"missing RCLD {rcld}")
    ledger = (ROOT / "docs/execution/remediation_v8/ledger.md").read_text(encoding="utf-8")
    deviations = (ROOT / "docs/execution/remediation_v8/deviations.md").read_text(encoding="utf-8")
    for value in range(1096, 1102):
        require(f"`step_{value}`" in ledger, f"missing ledger checkpoint {value}")
    for value in range(1, 6):
        require(f"`DEV-V8-00{value}`" in deviations, f"missing deviation {value}")


def validate_boundaries_and_reproductions() -> None:
    tracked = subprocess.run(
        ("git", "ls-files"), cwd=ROOT, check=True, capture_output=True, text=True
    ).stdout.splitlines()
    require(not any(path.startswith(".github/workflows/") or path.startswith(".act/") for path in tracked), "tracked workflows")
    public_files = (
        "docs/execution/remediation_v8/baseline.md",
        "docs/execution/remediation_v8/deviations.md",
        "docs/execution/remediation_v8/ledger.md",
        "docs/execution/remediation_v8/reproductions.md",
        "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v8.md",
    )
    forbidden = (
        "/" + "Users/",
        "docs/" + "handoff",
        "domains/" + "triesap",
        "triesap/" + "dev",
    )
    for relative in public_files:
        text = (ROOT / relative).read_text(encoding="utf-8")
        require(not any(marker in text for marker in forbidden), f"private boundary: {relative}")
    reproduction = (ROOT / "scripts/reproduce_remediation_v8.py").read_text(encoding="utf-8")
    for number in range(66, 72):
        require(f"finding_{number:03d}" in reproduction, f"missing reproduction {number}")


def main() -> int:
    validate_baseline()
    validate_findings_and_requirements()
    validate_plan_and_ledger()
    validate_boundaries_and_reproductions()
    print("PASS: remediation-v8 authority and execution plan")
    print("- findings=7")
    print("- requirements=139")
    print("- rclds=8")
    print("- steps=62")
    print("- remote_actions=unauthorized")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
