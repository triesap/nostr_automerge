#!/usr/bin/env python3
"""Fail-closed validation for remediation-v5 authority and execution state."""

from __future__ import annotations

import argparse
import copy
import json
import re
import subprocess
from pathlib import Path

from validate_requirement_matrix_v6 import validate as validate_requirement_evidence


ROOT = Path(__file__).resolve().parents[1]
HEX40 = re.compile(r"^[0-9a-f]{40}$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")
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
    if registry.get("schema") not in {
        "nostr_automerge.requirements.v4",
        "nostr_automerge.requirements.v5",
    } or not isinstance(rows, list):
        raise AssertionError("unexpected append-only requirement registry")
    if registry.get("requirement_count") != len(rows) or len(rows) < 119:
        raise AssertionError("append-only requirement count is inconsistent")
    expected = [
        "NCRDT-DUP-003", "NCRDT-DISPOSITION-003", "NCRDT-EPOCH-002",
        "NCRDT-EPOCH-003", "NCRDT-CPTRUST-003", "NCRDT-SCOPE-003",
        "NCRDT-RESOURCE-003", "NCRDT-RESOURCE-004", "NCRDT-NIP-001",
        "NCRDT-CONF-006",
    ]
    identifiers = [row.get("id") for row in rows]
    if len(set(identifiers)) != len(identifiers) or identifiers[96:106] != expected:
        raise AssertionError("v3 requirements are duplicate, missing, or reordered")
    authority = load("reports/remediation_v5_authority.json")
    if authority.get("status") != "local_companion_authority_pass_external_nip_hold":
        raise AssertionError("authority status overclaims")
    if authority.get("appended_requirement_ids") != expected:
        raise AssertionError("authority requirement delta is inconsistent")
    if authority.get("nip_edited") is not False or authority.get("wire_constants_changed") is not False:
        raise AssertionError("authority claims a prohibited change")


def sha256(relative: str) -> str:
    import hashlib

    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def validate_final(value: dict) -> None:
    if value.get("schema") != "nostr_automerge.remediation_v5_final.v1":
        raise AssertionError("unexpected final evidence schema")
    if value.get("status") != "implementation_remediation_required" or value.get("local_implementation") != "pass":
        raise AssertionError("final status is inconsistent")
    if value.get("publication_authorized") is not False or value.get("nip_edited") is not False:
        raise AssertionError("final evidence grants forbidden authority")
    rust = value.get("rust", {})
    typescript = value.get("typescript", {})
    for commit in (rust.get("candidate"), rust.get("source_candidate")):
        if not isinstance(commit, str) or not HEX40.fullmatch(commit) or git("cat-file", "-t", commit) != "commit":
            raise AssertionError("invalid Rust final candidate")
    if not isinstance(typescript.get("candidate"), str) or not HEX40.fullmatch(typescript["candidate"]):
        raise AssertionError("invalid opaque TypeScript candidate")
    if typescript.get("dependency_lock_sha256") != "d881757529b805b8ae4da935127730fe901b8b13a71382023be161016cd35a7d":
        raise AssertionError("stale opaque TypeScript lock binding")
    if rust.get("cargo_lock_sha256") != sha256("Cargo.lock"):
        raise AssertionError("stale Rust lock binding")
    authority = value.get("authority", {})
    if authority.get("nip_draft_sha256") != sha256("spec/NIP_DRAFT.md"):
        raise AssertionError("stale final NIP binding")
    if authority.get("fixture_distribution_sha256") != sha256("fixtures/distribution/manifest_v6.json"):
        raise AssertionError("stale final fixture binding")
    if any(not isinstance(authority.get(key), str) or not HEX64.fullmatch(authority[key]) for key in ("companion_spec_sha256", "requirements_sha256")):
        raise AssertionError("stale final authority binding")
    evidence = value.get("evidence", {})
    for field in ("requirements", "interop", "resource", "rust_mutation"):
        relative = evidence.get(field)
        if not isinstance(relative, str) or not (ROOT / relative).is_file():
            raise AssertionError(f"missing final evidence: {field}")
    historical_requirements = load(evidence["requirements"])
    if (
        historical_requirements.get("schema") != "nostr_automerge.requirement_coverage.v6"
        or historical_requirements.get("requirement_count") != 106
        or len(historical_requirements.get("rows", [])) != 106
    ):
        raise AssertionError("invalid historical remediation-v5 requirement evidence")
    interop = load(evidence["interop"])
    if interop.get("status") != "pass" or interop.get("fixture_count") != 124 or interop.get("passed") != 124 or interop.get("failed") != 0:
        raise AssertionError("invalid final interop result")
    if interop.get("cross_language") != "byte-identical" or interop.get("deliberate_mismatch") != "detected":
        raise AssertionError("interop comparison is incomplete")
    if interop.get("rust_candidate") != rust.get("candidate") or interop.get("typescript_candidate") != typescript.get("candidate"):
        raise AssertionError("interop candidates are stale")
    resource = load(evidence["resource"])
    if resource.get("status") != "pass" or set(resource.get("qualifications", {}).values()) != {"pass"}:
        raise AssertionError("resource qualification is incomplete")
    if evidence.get("rust_mutations_caught") != 6 or evidence.get("typescript_mutations_caught") != 10:
        raise AssertionError("mutation evidence is incomplete")
    gates = value.get("ordinary_gates", {})
    if len(gates) != 6 or gates.get("source_only_and_private_boundaries") != "pass":
        raise AssertionError("ordinary gates are incomplete")
    if any(result not in {"pass", "pass_with_documented_warnings"} for result in gates.values()):
        raise AssertionError("ordinary gate failed")
    holds = value.get("external_holds", [])
    if not isinstance(holds, list) or len(holds) != 4:
        raise AssertionError("external holds are incomplete")


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
    final = load("reports/remediation_v5_final.json")
    mutations = []
    for path, replacement in (
        (("authority", "requirements_sha256"), "0" * 64),
        (("rust", "candidate"), "0" * 40),
        (("rust", "cargo_lock_sha256"), "0" * 64),
        (("authority", "fixture_distribution_sha256"), "0" * 64),
        (("evidence", "requirements"), "reports/missing.json"),
        (("ordinary_gates", "source_only_and_private_boundaries"), "fail"),
    ):
        mutation = copy.deepcopy(final)
        mutation[path[0]][path[1]] = replacement
        mutations.append(mutation)
    mutation = copy.deepcopy(final); mutation["publication_authorized"] = True; mutations.append(mutation)
    caught = 0
    for mutation in mutations:
        try:
            validate_final(mutation)
        except (AssertionError, ValueError):
            caught += 1
        else:
            raise AssertionError("final evidence mutation survived")
    if caught != len(mutations):
        raise AssertionError("final mutation count is inconsistent")


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
        validate_final(load("reports/remediation_v5_final.json"))
    if args.self_test:
        self_test()
    print("PASS: remediation v5 authority")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
