#!/usr/bin/env python3
"""Fail-closed validation for remediation-v6 authority and execution state."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
HEX40 = re.compile(r"^[0-9a-f]{40}$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")
PHASES = [
    (56, 861, 870),
    (57, 871, 888),
    (58, 889, 916),
    (59, 917, 936),
    (60, 937, 964),
    (61, 965, 1001),
    (62, 1002, 1018),
    (63, 1019, 1035),
    (64, 1036, 1058),
]


def load(relative: str) -> dict:
    value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise AssertionError(f"expected object: {relative}")
    return value


def digest(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def git(*args: str) -> str:
    result = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True)
    if result.returncode:
        raise AssertionError(result.stderr.strip() or "git failed")
    return result.stdout.strip()


def validate_baseline(value: dict) -> None:
    if value.get("schema") != "nostr_automerge.remediation_v6_baseline.v1":
        raise AssertionError("unexpected baseline schema")
    if value.get("status") != "implementation_remediation_required":
        raise AssertionError("baseline status overclaims")
    rust = value.get("rust", {})
    for field in ("review_head", "prior_implementation_candidate", "prior_source_candidate"):
        commit = rust.get(field)
        if not isinstance(commit, str) or not HEX40.fullmatch(commit):
            raise AssertionError(f"invalid Rust identity: {field}")
        if git("cat-file", "-t", commit) != "commit":
            raise AssertionError(f"missing Rust commit: {field}")
    typescript = value.get("typescript", {})
    if not isinstance(typescript.get("opaque_import_identity"), str) or not HEX40.fullmatch(
        typescript["opaque_import_identity"]
    ):
        raise AssertionError("invalid opaque TypeScript identity")
    if typescript.get("private_source") is not True:
        raise AssertionError("private TypeScript boundary missing")
    authority = value.get("authority", {})
    expected = {
        "nip_draft_sha256": digest("spec/NIP_DRAFT.md"),
        "companion_spec_sha256": digest("spec/NOSTR_AUTOMERGE_V1_SPEC.md"),
        "requirements_sha256": digest("spec/requirements.json"),
        "fixture_distribution_sha256": digest("fixtures/distribution/manifest_v6.json"),
    }
    for field, checksum in expected.items():
        if authority.get(field) != checksum or not HEX64.fullmatch(checksum):
            raise AssertionError(f"stale baseline authority: {field}")
    if authority.get("nip_read_only") is not True:
        raise AssertionError("NIP is not bound read-only")
    if value.get("external_actions_authorized") is not False:
        raise AssertionError("baseline grants external authority")
    if len(value.get("external_holds", [])) != 4:
        raise AssertionError("baseline external holds are incomplete")


def validate_findings(value: dict) -> None:
    rows = value.get("findings")
    expected = [f"FINDING_{number:03d}" for number in range(51, 59)]
    if value.get("schema") != "nostr_automerge.remediation_v6.findings.v1" or not isinstance(rows, list):
        raise AssertionError("unexpected findings schema")
    if value.get("review_head") != "e1a6d1cc9f046b5129ad699488fcb034a70f9b4a":
        raise AssertionError("findings review head is stale")
    if [row.get("id") for row in rows] != expected:
        raise AssertionError("findings missing, duplicate, or reordered")
    for row in rows:
        if not row.get("paths") or not row.get("reproduction") or not row.get("closure"):
            raise AssertionError(f"incomplete finding: {row.get('id')}")
        for relative in row["paths"]:
            if not (ROOT / relative).exists():
                raise AssertionError(f"missing affected path: {relative}")


def validate_ledger() -> None:
    text = (ROOT / "docs/execution/remediation_v6/ledger.md").read_text(encoding="utf-8")
    rows = re.findall(r"^\| (\d+) \| (\d{3,4})–(\d{3,4}) \| (active|pending|complete) \|", text, re.M)
    if [(int(a), int(b), int(c)) for a, b, c, _ in rows] != PHASES:
        raise AssertionError("ledger phases are inconsistent")
    authority = (ROOT / "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v6.md").read_text(encoding="utf-8")
    steps = [int(value) for value in re.findall(r"^\| `step_(\d{3,4})` \|", authority, re.M)]
    if steps != list(range(861, 1059)):
        raise AssertionError("authority steps are missing or reordered")
    if len(re.findall(r"^## RCLD \d+", authority, re.M)) != 9:
        raise AssertionError("RCLD count is inconsistent")


def validate_adrs() -> None:
    index = (ROOT / "docs/adr/README.md").read_text(encoding="utf-8")
    for number in range(53, 58):
        paths = list((ROOT / "docs/adr").glob(f"adr_{number:04d}_*.md"))
        if len(paths) != 1 or f"[{number:04d}]" not in index:
            raise AssertionError(f"ADR {number:04d} is missing")
        text = paths[0].read_text(encoding="utf-8")
        for heading in ("Status: Approved", "## Context", "## Decision", "## Consequences"):
            if heading not in text:
                raise AssertionError(f"ADR {number:04d} lacks {heading}")


def validate_boundaries() -> None:
    tracked = git("ls-files")
    for forbidden in (".github/workflows/", ".act/", "/Users/", "docs/handoff/"):
        if forbidden in tracked:
            raise AssertionError(f"forbidden tracked path: {forbidden}")
    authority = (ROOT / "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v6.md").read_text(encoding="utf-8")
    if "NIP document remains externally authored and read-only" not in authority:
        raise AssertionError("read-only NIP boundary is missing")
    if "private TypeScript" not in authority or "opaque" not in authority:
        raise AssertionError("private evidence boundary is missing")


def self_test() -> None:
    baseline = load("reports/remediation_v6_baseline.json")
    mutations = []
    for path, replacement in (
        (("authority", "requirements_sha256"), "0" * 64),
        (("rust", "review_head"), "0" * 40),
        (("authority", "fixture_distribution_sha256"), "0" * 64),
        (("typescript", "private_source"), False),
    ):
        mutation = copy.deepcopy(baseline)
        mutation[path[0]][path[1]] = replacement
        mutations.append(mutation)
    mutation = copy.deepcopy(baseline)
    mutation["external_actions_authorized"] = True
    mutations.append(mutation)
    for mutation in mutations:
        try:
            validate_baseline(mutation)
        except AssertionError:
            pass
        else:
            raise AssertionError("baseline mutation survived")
    findings = load("spec/remediation_findings_v6.json")
    mutation = copy.deepcopy(findings)
    mutation["findings"].reverse()
    try:
        validate_findings(mutation)
    except AssertionError:
        pass
    else:
        raise AssertionError("finding-order mutation survived")


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
        validate_baseline(load("reports/remediation_v6_baseline.json"))
    if selected or args.findings:
        validate_findings(load("spec/remediation_findings_v6.json"))
    if selected or args.ledger:
        validate_ledger()
    if selected:
        validate_adrs()
        validate_boundaries()
    if args.self_test:
        self_test()
    print("PASS: remediation v6 authority")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
