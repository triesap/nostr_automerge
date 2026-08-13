#!/usr/bin/env python3
"""Fail-closed validation for remediation-v5 authority and execution state."""

from __future__ import annotations

import argparse
import copy
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
HEX40 = re.compile(r"^[0-9a-f]{40}$")
PHASES = [(47, 738, 746), (48, 747, 758), (49, 759, 775),
          (50, 776, 790), (51, 791, 802), (52, 803, 816),
          (53, 817, 828), (54, 829, 839), (55, 840, 860)]


def load(relative: str) -> dict:
    value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise AssertionError(f"expected object: {relative}")
    return value


def git(*args: str) -> str:
    result = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True)
    if result.returncode:
        raise AssertionError(result.stderr.strip() or "git failed")
    return result.stdout.strip()


def validate_baseline(value: dict) -> None:
    if value.get("schema") != "nostr_automerge.remediation_v5_baseline.v1":
        raise AssertionError("unexpected baseline schema")
    rust = value.get("rust", {})
    for field in ("review_head", "implementation_candidate"):
        commit = rust.get(field)
        if not isinstance(commit, str) or not HEX40.fullmatch(commit):
            raise AssertionError(f"invalid Rust identity: {field}")
        if git("cat-file", "-t", commit) != "commit":
            raise AssertionError(f"missing Rust commit: {field}")
    typescript = value.get("typescript", {}).get("candidate")
    if not isinstance(typescript, str) or not HEX40.fullmatch(typescript):
        raise AssertionError("invalid opaque TypeScript identity")
    if value.get("active_rcld") != 47 or value.get("active_step") != "step_738":
        raise AssertionError("baseline start is inconsistent")
    if value.get("publication_authorized") is not False or value.get("nip_edit_authorized") is not False:
        raise AssertionError("baseline grants forbidden authority")


def validate_findings(value: dict) -> None:
    rows = value.get("findings")
    expected = [f"FINDING_{number:03d}" for number in range(44, 51)]
    if value.get("schema") != "nostr_automerge.remediation_v5.findings.v1" or not isinstance(rows, list):
        raise AssertionError("unexpected findings schema")
    if [row.get("id") for row in rows] != expected:
        raise AssertionError("findings missing or reordered")
    for row in rows:
        if not row.get("paths") or not row.get("symbols") or not row.get("reproductions") or not row.get("closure"):
            raise AssertionError(f"incomplete finding: {row.get('id')}")
        for relative in row["paths"]:
            if not (ROOT / relative).exists():
                raise AssertionError(f"missing affected path: {relative}")


def validate_ledger() -> None:
    text = (ROOT / "docs/execution/remediation_v5/ledger.md").read_text(encoding="utf-8")
    rows = re.findall(r"^\| (\d+) \| (\d{3})–(\d{3}) \| (active|pending|complete) \|", text, re.M)
    if [(int(a), int(b), int(c)) for a, b, c, _ in rows] != PHASES:
        raise AssertionError("ledger phases are inconsistent")
    authority = (ROOT / "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v5.md").read_text(encoding="utf-8")
    steps = [int(value) for value in re.findall(r"^\| `step_(\d{3})` \|", authority, re.M)]
    if steps != list(range(738, 861)):
        raise AssertionError("authority steps are missing or reordered")


def validate_adrs() -> None:
    index = (ROOT / "docs/adr/README.md").read_text(encoding="utf-8")
    for number in range(48, 53):
        paths = list((ROOT / "docs/adr").glob(f"adr_{number:04d}_*.md"))
        if len(paths) != 1 or f"[{number:04d}]" not in index:
            raise AssertionError(f"ADR {number:04d} is missing")
        text = paths[0].read_text(encoding="utf-8")
        for heading in ("Status: Approved", "## Context", "## Decision", "## Consequences"):
            if heading not in text:
                raise AssertionError(f"ADR {number:04d} lacks {heading}")


def validate_boundaries() -> None:
    tracked = git("ls-files")
    forbidden = (
        ".github/workflows/",
        ".act/",
        "/" + "Users/",
        "docs/" + "handoff/",
    )
    if any(token in tracked for token in forbidden):
        raise AssertionError("forbidden private or workflow path is tracked")
    authority = (ROOT / "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v5.md").read_text(encoding="utf-8")
    if "NIP document remains externally authored and read-only" not in authority:
        raise AssertionError("read-only NIP boundary is missing")


def validate_registry() -> None:
    registry = load("spec/requirements.json")
    rows = registry.get("requirements")
    if registry.get("schema") != "nostr_automerge.requirements.v3" or not isinstance(rows, list):
        raise AssertionError("unexpected v3 requirement registry")
    if registry.get("requirement_count") != 106 or len(rows) != 106:
        raise AssertionError("v3 requirement count is inconsistent")
    expected = [
        "NCRDT-DUP-003", "NCRDT-DISPOSITION-003", "NCRDT-EPOCH-002",
        "NCRDT-EPOCH-003", "NCRDT-CPTRUST-003", "NCRDT-SCOPE-003",
        "NCRDT-RESOURCE-003", "NCRDT-RESOURCE-004", "NCRDT-NIP-001",
        "NCRDT-CONF-006",
    ]
    identifiers = [row.get("id") for row in rows]
    if len(set(identifiers)) != 106 or identifiers[96:] != expected:
        raise AssertionError("v3 requirements are duplicate, missing, or reordered")
    authority = load("reports/remediation_v5_authority.json")
    if authority.get("status") != "local_companion_authority_pass_external_nip_hold":
        raise AssertionError("authority status overclaims")
    if authority.get("appended_requirement_ids") != expected:
        raise AssertionError("authority requirement delta is inconsistent")
    if authority.get("nip_edited") is not False or authority.get("wire_constants_changed") is not False:
        raise AssertionError("authority claims a prohibited change")


def self_test() -> None:
    findings = load("spec/remediation_findings_v5.json")
    mutated = copy.deepcopy(findings)
    mutated["findings"].reverse()
    try:
        validate_findings(mutated)
    except AssertionError:
        pass
    else:
        raise AssertionError("reordered findings mutation survived")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--all", action="store_true")
    parser.add_argument("--baseline", action="store_true")
    parser.add_argument("--findings", action="store_true")
    parser.add_argument("--ledger", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    selected = args.all or not any((args.baseline, args.findings, args.ledger))
    if selected or args.baseline:
        validate_baseline(load("reports/remediation_v5_baseline.json"))
    if selected or args.findings:
        validate_findings(load("spec/remediation_findings_v5.json"))
    if selected or args.ledger:
        validate_ledger()
    if selected:
        validate_adrs()
        validate_boundaries()
        validate_registry()
    if args.self_test:
        self_test()
    print("PASS: remediation v5 authority")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
