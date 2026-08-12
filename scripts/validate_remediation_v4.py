#!/usr/bin/env python3
"""Fail-closed validation entry point for draft-v1 remediation v4."""

from __future__ import annotations

import argparse
import copy
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
HEX40 = re.compile(r"^[0-9a-f]{40}$")
PHASES = [(39, 660, 667), (40, 668, 675), (41, 676, 684), (42, 685, 698),
          (43, 699, 706), (44, 707, 715), (45, 716, 727), (46, 728, 737)]


def load(relative: str) -> dict:
    value = json.loads((ROOT / relative).read_text())
    if not isinstance(value, dict):
        raise AssertionError(f"expected object: {relative}")
    return value


def git(*args: str) -> str:
    result = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True)
    if result.returncode:
        raise AssertionError(result.stderr.strip() or "git failed")
    return result.stdout.strip()


def baseline(value: dict) -> None:
    if value.get("schema") != "nostr_automerge.remediation_v4_baseline.v1":
        raise AssertionError("unexpected baseline schema")
    rust = value.get("rust", {})
    for field in ("head", "reviewed_public_head", "implementation_candidate"):
        commit = rust.get(field)
        if not isinstance(commit, str) or not HEX40.fullmatch(commit):
            raise AssertionError(f"invalid Rust identity: {field}")
        if git("cat-file", "-t", commit) != "commit":
            raise AssertionError(f"missing Rust commit: {field}")
    if value.get("active_rcld") != 39 or value.get("active_step") != "step_660":
        raise AssertionError("baseline start is inconsistent")
    if value.get("publication_authorized") is not False or value.get("nip_edit_authorized") is not False:
        raise AssertionError("baseline grants forbidden authority")


def findings(value: dict) -> None:
    expected = [f"FINDING_{number:03d}" for number in range(36, 44)]
    rows = value.get("findings")
    if value.get("schema") != "nostr_automerge.remediation_findings.v4" or not isinstance(rows, list):
        raise AssertionError("unexpected findings schema")
    if value.get("finding_count") != 8 or [row.get("id") for row in rows] != expected:
        raise AssertionError("findings missing or reordered")
    for row in rows:
        if not row.get("affected") or not row.get("requirements") or not row.get("closure"):
            raise AssertionError(f"incomplete finding: {row.get('id')}")
        for relative in row["affected"]:
            if not (ROOT / relative).exists():
                raise AssertionError(f"missing affected path: {relative}")


def ledger() -> None:
    text = (ROOT / "docs/execution/remediation_v4/ledger.md").read_text()
    rows = re.findall(r"^\| (\d+) \| `step_(\d{3})`–`step_(\d{3})` \| (active|pending|complete) \|", text, re.M)
    if [(int(a), int(b), int(c)) for a, b, c, _ in rows] != PHASES:
        raise AssertionError("ledger phases are inconsistent")
    authority = (ROOT / "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v4.md").read_text()
    steps = sorted(set(re.findall(r"step_(\d{3})", authority)))
    if steps != [f"{number:03d}" for number in range(660, 738)]:
        raise AssertionError("authority steps are inconsistent")


def adrs() -> None:
    index = (ROOT / "docs/adr/README.md").read_text()
    for number in range(41, 48):
        matches = list((ROOT / "docs/adr").glob(f"adr_{number:04d}_*.md"))
        if len(matches) != 1 or f"[{number:04d}]" not in index:
            raise AssertionError(f"ADR {number:04d} is missing")
        text = matches[0].read_text()
        for heading in ("Status: Approved", "## Context", "## Decision", "## Consequences"):
            if heading not in text:
                raise AssertionError(f"ADR {number:04d} lacks {heading}")


def anchors(value: dict) -> None:
    if value.get("schema") != "nostr_automerge.remediation_v4_source_manifest.v1":
        raise AssertionError("unexpected source manifest schema")
    commit = value.get("reviewed_commit")
    rows = value.get("anchors")
    if not isinstance(commit, str) or not HEX40.fullmatch(commit) or not isinstance(rows, list):
        raise AssertionError("invalid source manifest")
    paths = [row.get("path") for row in rows]
    if paths != sorted(paths) or len(paths) != len(set(paths)):
        raise AssertionError("source anchors are missing or reordered")
    for row in rows:
        source = subprocess.run(["git", "show", f"{commit}:{row['path']}"], cwd=ROOT, capture_output=True, text=True)
        if source.returncode or git("rev-parse", f"{commit}:{row['path']}") != row.get("baseline_git_object"):
            raise AssertionError(f"stale anchor: {row['path']}")
        if any(symbol not in source.stdout for symbol in row.get("symbols", [])):
            raise AssertionError(f"missing source symbol: {row['path']}")


def authority(value: dict) -> None:
    if value.get("schema") != "nostr_automerge.remediation_v4_authority.v1":
        raise AssertionError("unexpected authority schema")
    if value.get("status") != "local_authority_pass_external_nip_hold":
        raise AssertionError("authority status overclaims")
    registry = load("spec/requirements.json")
    rows = registry.get("requirements", [])
    if value.get("requirement_count") != 96 or len(rows) != 96:
        raise AssertionError("v2 registry count is inconsistent")
    appended = [row.get("id") for row in rows[87:]]
    if appended != value.get("appended_requirement_ids"):
        raise AssertionError("v2 registry append order is inconsistent")
    if value.get("holds") != ["external_nip_reconciliation"]:
        raise AssertionError("external NIP hold is missing")


def self_test() -> None:
    value = load("spec/remediation_findings_v4.json")
    mutated = copy.deepcopy(value)
    mutated["findings"].reverse()
    try:
        findings(mutated)
    except AssertionError:
        return
    raise AssertionError("reordered findings mutation survived")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    baseline(load("reports/remediation_v4_baseline.json"))
    findings(load("spec/remediation_findings_v4.json"))
    ledger()
    adrs()
    anchors(load("reports/remediation_v4_source_manifest.json"))
    authority_path = ROOT / "reports/remediation_v4_authority.json"
    if authority_path.exists():
        authority(load("reports/remediation_v4_authority.json"))
    if args.self_test:
        self_test()
    print("PASS: remediation v4 authority")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
