#!/usr/bin/env python3
"""Fail-closed authority and execution validation for remediation v7."""

from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
NIP_SHA256 = "67019c8ea680714052c65226f620a8e1a60b9b10a8f158603063a835a7bbc7a3"
FINDING_IDS = {f"FINDING_{number:03d}" for number in range(59, 66)}
REQUIREMENT_IDS = (
    "NCRDT-BRANCH-001",
    "NCRDT-BRANCH-002",
    "NCRDT-SCOPE-004",
    "NCRDT-SCOPE-005",
    "NCRDT-SCOPE-006",
    "NCRDT-RESOURCE-009",
    "NCRDT-RESOURCE-010",
    "NCRDT-NIP-002",
    "NCRDT-CONF-008",
    "NCRDT-EVIDENCE-004",
)


def load(relative: str) -> dict[str, object]:
    return json.loads((ROOT / relative).read_text(encoding="utf-8"))


def digest(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def validate_authority() -> None:
    baseline = load("reports/remediation_v7_baseline.json")
    require(baseline["schema"] == "nostr_automerge.remediation_v7_baseline.v1", "baseline schema")
    authority = baseline["authority"]
    require(isinstance(authority, dict), "baseline authority")
    require(authority["nip_read_only"] is True, "NIP boundary")
    require(authority["nip_draft_sha256"] == NIP_SHA256, "baseline NIP identity")
    require(digest("spec/NIP_DRAFT.md") == NIP_SHA256, "NIP changed")
    require(baseline["external_actions_authorized"] is False, "external action boundary")


def validate_findings_and_requirements() -> None:
    findings = load("spec/remediation_findings_v7.json")
    finding_rows = findings["findings"]
    require(isinstance(finding_rows, list) and len(finding_rows) == 7, "finding count")
    require({row["id"] for row in finding_rows} == FINDING_IDS, "finding IDs")

    additions = load("spec/remediation_v7_requirements.json")
    rows = additions["requirements"]
    require(isinstance(rows, list) and len(rows) == 10, "proposed requirement count")
    require(tuple(row["id"] for row in rows) == REQUIREMENT_IDS, "proposed requirement order")
    require(additions["base_requirement_count"] == 119, "base requirement count")
    require(additions["target_requirement_count"] == 129, "target requirement count")
    nip_row = next(row for row in rows if row["id"] == "NCRDT-NIP-002")
    require(nip_row["applicability"] == "explicitly-deferred", "NIP applicability")

    registry = load("spec/requirements.json")
    canonical = registry["requirements"]
    require(registry["schema"] == "nostr_automerge.requirements.v5", "canonical registry schema")
    require(registry["requirement_count"] == 129 == len(canonical), "canonical requirement count")
    require(tuple(row["id"] for row in canonical[-10:]) == REQUIREMENT_IDS, "canonical append order")
    applicability = load("spec/requirements_applicability.json")
    require(applicability["schema"] == "nostr_automerge.requirements_applicability.v5", "applicability schema")
    classifications = applicability["classifications"]
    require(tuple(classifications) == tuple(row["id"] for row in canonical), "applicability order")
    for row in rows:
        require(classifications[row["id"]] == row["applicability"], f"applicability {row['id']}")


def validate_plan() -> None:
    plan = (ROOT / "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v7.md").read_text(encoding="utf-8")
    steps = [int(value) for value in re.findall(r"`step_(\d+)`", plan)]
    require(set(range(1059, 1096)).issubset(steps), "contiguous checkpoint inventory")
    for rcld in range(65, 73):
        require(f"## RCLD {rcld} " in plan, f"missing RCLD {rcld}")
    require("spec/NIP_DRAFT.md` byte-identical" in plan, "read-only NIP plan")


def validate_boundaries() -> None:
    tracked = tuple(
        line
        for line in (ROOT / ".git").read_text(encoding="utf-8").splitlines()
    ) if (ROOT / ".git").is_file() else ()
    del tracked
    forbidden = [path for path in ROOT.rglob("*") if path.is_file() and (".github/workflows" in path.as_posix() or "/.act/" in path.as_posix())]
    require(not forbidden, f"tracked-or-present workflow content: {forbidden}")
    reproduction = (ROOT / "scripts/reproduce_remediation_v7.py").read_text(encoding="utf-8")
    for number in range(59, 64):
        require(f"finding_{number:03d}" in reproduction, f"missing reproduction {number}")
    mutation = load("reports/mutation_campaign_v7_inventory.json")
    require(
        mutation["schema"] == "nostr_automerge.mutation_campaign.v7.inventory.v1",
        "mutation inventory schema",
    )
    require(mutation["campaign_executed"] is False, "mutation execution boundary")
    require(mutation["execution_status"] == "held_operator_safety", "mutation hold")
    anchors = mutation["anchors"]
    require(isinstance(anchors, list) and len(anchors) == 5, "mutation anchor count")
    require({row["finding"] for row in anchors} == {f"FINDING_{value:03d}" for value in range(59, 64)}, "mutation findings")
    for row in anchors:
        source = (ROOT / row["path"]).read_text(encoding="utf-8")
        require(all(symbol in source for symbol in row["symbols"]), f"mutation anchor: {row['finding']}")


def main() -> int:
    validate_authority()
    validate_findings_and_requirements()
    validate_plan()
    validate_boundaries()
    print("PASS: remediation-v7 authority")
    print("- findings=7 requirements=10 checkpoints=37 rclds=8")
    print("- nip=read-only workflows=absent external-actions=unauthorized")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
