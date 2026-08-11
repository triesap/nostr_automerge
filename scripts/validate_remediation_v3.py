#!/usr/bin/env python3
"""Fail-closed validation entry point for draft-v1 remediation v3."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
HEX_40 = re.compile(r"^[0-9a-f]{40}$")
HEX_64 = re.compile(r"^[0-9a-f]{64}$")
FINDING_IDS = [f"FINDING_{number:03d}" for number in range(28, 36)]
PHASES = [
    (29, 534, 541),
    (30, 542, 557),
    (31, 558, 569),
    (32, 570, 582),
    (33, 583, 596),
    (34, 597, 612),
    (35, 613, 621),
    (36, 622, 635),
    (37, 636, 647),
    (38, 648, 659),
]


class PendingArtifactError(AssertionError):
    """A later-phase artifact is explicitly not available yet."""


def load_json(relative: str) -> dict:
    value = json.loads((ROOT / relative).read_text())
    if not isinstance(value, dict):
        raise AssertionError(f"expected JSON object: {relative}")
    return value


def git_output(*arguments: str) -> str:
    result = subprocess.run(
        ["git", *arguments], cwd=ROOT, check=False, capture_output=True, text=True
    )
    if result.returncode:
        raise AssertionError(result.stderr.strip() or "git command failed")
    return result.stdout.strip()


def git_bytes(commit: str, relative: str) -> bytes:
    result = subprocess.run(
        ["git", "show", f"{commit}:{relative}"], cwd=ROOT, check=False, capture_output=True
    )
    if result.returncode:
        raise AssertionError(f"baseline path does not resolve: {commit}:{relative}")
    return result.stdout


def sha256(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def validate_baseline(value: dict) -> None:
    if value.get("schema") != "nostr_automerge.remediation_v3_baseline.v1":
        raise AssertionError("unexpected remediation v3 baseline schema")
    rust = value.get("rust", {})
    reviewed = rust.get("reviewed_public_head")
    if not isinstance(reviewed, str) or not HEX_40.fullmatch(reviewed):
        raise AssertionError("invalid reviewed public head")
    if rust.get("head") != reviewed or rust.get("implementation_candidate") != reviewed:
        raise AssertionError("baseline Rust identities diverge")
    if git_output("cat-file", "-t", reviewed) != "commit":
        raise AssertionError("reviewed Rust commit does not exist")
    expected_hashes = {
        "cargo_lock_sha256": ("Cargo.lock", rust),
        "fixture_manifest_sha256": ("fixtures/distribution/manifest_v3.json", rust),
        "nip_draft_sha256": ("spec/NIP_DRAFT.md", value.get("authority", {})),
        "companion_spec_sha256": (
            "spec/NOSTR_AUTOMERGE_V1_SPEC.md",
            value.get("authority", {}),
        ),
        "requirements_sha256": ("spec/requirements.json", value.get("authority", {})),
    }
    for field, (relative, owner) in expected_hashes.items():
        digest = owner.get(field)
        if not isinstance(digest, str) or not HEX_64.fullmatch(digest):
            raise AssertionError(f"invalid baseline digest: {field}")
        if sha256(git_bytes(reviewed, relative)) != digest:
            raise AssertionError(f"stale baseline digest: {relative}")
    reports = value.get("reports")
    if not isinstance(reports, dict) or not reports:
        raise AssertionError("baseline report hashes are missing")
    for relative, digest in reports.items():
        if not isinstance(digest, str) or not HEX_64.fullmatch(digest):
            raise AssertionError(f"invalid report digest: {relative}")
        if sha256(git_bytes(reviewed, relative)) != digest:
            raise AssertionError(f"stale reviewed report digest: {relative}")
    typescript = value.get("typescript", {})
    if typescript.get("implementation_id") != "typescript_v1_internal":
        raise AssertionError("unexpected TypeScript implementation identity")
    if not HEX_40.fullmatch(str(typescript.get("candidate", ""))):
        raise AssertionError("invalid TypeScript candidate")
    if value.get("active_rcld") != 29 or value.get("active_step") != "step_534":
        raise AssertionError("baseline does not begin at step_534")
    if value.get("publication_authorized") is not False:
        raise AssertionError("baseline authorizes publication")
    if value.get("nip_edit_authorized") is not False:
        raise AssertionError("baseline authorizes NIP editing")


def validate_findings(value: dict) -> None:
    if value.get("schema") != "nostr_automerge.remediation_findings.v3":
        raise AssertionError("unexpected remediation finding schema")
    findings = value.get("findings")
    if not isinstance(findings, list) or value.get("finding_count") != len(findings):
        raise AssertionError("finding count is inconsistent")
    if [finding.get("id") for finding in findings] != FINDING_IDS:
        raise AssertionError("findings are missing, duplicated, or reordered")
    for finding in findings:
        if not finding.get("severity") or not finding.get("title"):
            raise AssertionError("finding metadata is incomplete")
        if not finding.get("requirements") or len(finding["requirements"]) != len(
            set(finding["requirements"])
        ):
            raise AssertionError(f"invalid requirements for {finding['id']}")
        if not finding.get("affected") or not finding.get("closure"):
            raise AssertionError(f"incomplete closure for {finding['id']}")
        for relative in finding["affected"]:
            if not (ROOT / relative).exists():
                raise AssertionError(f"missing affected path: {relative}")
    if [finding.get("status") for finding in findings[:-1]] != ["open"] * 7:
        raise AssertionError("implementation findings must begin open")
    if findings[-1].get("status") != "held":
        raise AssertionError("release assurance must begin held")


def validate_ledger() -> None:
    ledger = (ROOT / "docs/execution/remediation_v3/ledger.md").read_text()
    authority = (ROOT / "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v3.md").read_text()
    rows = re.findall(
        r"^\| (\d+) \| `step_(\d{3})`–`step_(\d{3})` \| "
        r"(active|pending|complete) \|",
        ledger,
        flags=re.MULTILINE,
    )
    parsed = [(int(rcld), int(start), int(end)) for rcld, start, end, _ in rows]
    if parsed != PHASES:
        raise AssertionError("ledger RCLD ranges are missing, reordered, or noncontiguous")
    if sum(status == "active" for _, _, _, status in rows) != 1:
        raise AssertionError("ledger must contain exactly one active RCLD")
    authority_steps = sorted(set(re.findall(r"step_(\d{3})", authority)))
    if authority_steps != [f"{step:03d}" for step in range(534, 660)]:
        raise AssertionError("multi-RCLD steps are missing, duplicated, or out of range")
    denied = ["push", "publication", "deployment", "NIP submission"]
    nonauthorization = ledger.split("## Nonauthorization", maxsplit=1)
    if len(nonauthorization) != 2 or not all(word in nonauthorization[1] for word in denied):
        raise AssertionError("ledger publication nonauthorization is incomplete")


def validate_adrs() -> None:
    index = (ROOT / "docs/adr/README.md").read_text()
    for number in range(33, 41):
        matches = sorted((ROOT / "docs/adr").glob(f"adr_{number:04d}_*.md"))
        if len(matches) != 1:
            raise AssertionError(f"ADR {number:04d} is missing or duplicated")
        text = matches[0].read_text()
        for heading in ("Status: Approved", "## Context", "## Decision", "## Consequences"):
            if heading not in text:
                raise AssertionError(f"ADR {number:04d} lacks {heading}")
        if f"[{number:04d}]" not in index:
            raise AssertionError(f"ADR {number:04d} is absent from the index")


def validate_source_anchors(value: dict) -> None:
    if value.get("schema") != "nostr_automerge.remediation_v3_source_manifest.v1":
        raise AssertionError("unexpected source-manifest schema")
    reviewed = value.get("reviewed_commit")
    if not isinstance(reviewed, str) or not HEX_40.fullmatch(reviewed):
        raise AssertionError("invalid source-manifest reviewed commit")
    anchors = value.get("anchors")
    if not isinstance(anchors, list) or not anchors:
        raise AssertionError("source anchors are missing")
    paths = [anchor.get("path") for anchor in anchors]
    if paths != sorted(paths) or len(paths) != len(set(paths)):
        raise AssertionError("source anchors are duplicated or reordered")
    for anchor in anchors:
        relative = anchor["path"]
        if git_output("rev-parse", f"{reviewed}:{relative}") != anchor.get(
            "baseline_git_object"
        ):
            raise AssertionError(f"stale source object: {relative}")
        source = git_bytes(reviewed, relative).decode()
        if not anchor.get("findings") or not anchor.get("symbols"):
            raise AssertionError(f"incomplete source anchor: {relative}")
        for symbol in anchor["symbols"]:
            if symbol not in source:
                raise AssertionError(f"missing source symbol {symbol}: {relative}")
        if not isinstance(anchor.get("replacement_anchors"), list):
            raise AssertionError(f"invalid replacement anchors: {relative}")


def validate_private_boundary() -> None:
    validator = ROOT / "scripts/validate_typescript_private_boundary_v3.py"
    if not validator.exists():
        raise PendingArtifactError("private TypeScript boundary validator is pending step_540")
    result = subprocess.run([sys.executable, str(validator)], cwd=ROOT, check=False)
    if result.returncode:
        raise AssertionError("private TypeScript boundary validation failed")


def validate_phase(phase: str) -> None:
    phase_match = re.fullmatch(r"phase_(\d{2})(?:_[a-z0-9_]+)?", phase)
    if phase_match is None:
        raise AssertionError(f"invalid phase identifier: {phase}")
    report_path = ROOT / "reports" / f"remediation_v3_phase_{phase_match.group(1)}.json"
    if not report_path.exists():
        raise PendingArtifactError(f"phase artifact is pending: {report_path.name}")
    report = load_json(str(report_path.relative_to(ROOT)))
    if report.get("schema") != "nostr_automerge.remediation_v3_phase.v1":
        raise AssertionError("unexpected phase report schema")
    if report.get("phase") != phase or report.get("status") != "pass":
        raise AssertionError("phase report is not passing")
    phase_index = int(phase_match.group(1))
    if phase_index >= len(PHASES):
        raise AssertionError("phase index is outside the approved sequence")
    rcld, start, end = PHASES[phase_index]
    if report.get("rcld") != rcld or report.get("completed_steps") != [
        f"step_{step:03d}" for step in range(start, end + 1)
    ]:
        raise AssertionError("phase completion range is inconsistent")
    verified = report.get("verified_source_commit")
    if not isinstance(verified, str) or not HEX_40.fullmatch(verified):
        raise AssertionError("phase verified commit is invalid")
    if git_output("cat-file", "-t", verified) != "commit":
        raise AssertionError("phase verified commit does not exist")
    verification = report.get("verification")
    if not isinstance(verification, list) or not verification:
        raise AssertionError("phase verification is missing")
    if any(item.get("result") != "pass" for item in verification):
        raise AssertionError("phase verification contains a nonpassing command")


def validate_final() -> None:
    final_path = ROOT / "reports/remediation_v3_final_decision.json"
    if not final_path.exists():
        raise PendingArtifactError("final decision artifact is pending step_659")
    report = load_json(str(final_path.relative_to(ROOT)))
    if report.get("decision") != "code_complete_publication_held":
        raise AssertionError("final decision overclaims or is incomplete")


def expect_rejected(function: object, value: dict, reason: str) -> None:
    try:
        function(value)  # type: ignore[operator]
    except AssertionError:
        return
    raise AssertionError(f"invalid mutation accepted: {reason}")


def self_test() -> None:
    baseline = load_json("reports/remediation_v3_baseline.json")
    findings = load_json("spec/remediation_findings_v3.json")
    anchors = load_json("reports/remediation_v3_source_manifest.json")
    missing = copy.deepcopy(baseline)
    del missing["rust"]["cargo_lock_sha256"]
    expect_rejected(validate_baseline, missing, "missing baseline digest")
    stale = copy.deepcopy(baseline)
    stale["reports"]["reports/release_readiness.json"] = "0" * 64
    expect_rejected(validate_baseline, stale, "stale report digest")
    reordered = copy.deepcopy(findings)
    reordered["findings"][0], reordered["findings"][1] = (
        reordered["findings"][1],
        reordered["findings"][0],
    )
    expect_rejected(validate_findings, reordered, "reordered findings")
    missing_anchor = copy.deepcopy(anchors)
    missing_anchor["anchors"].pop()
    missing_anchor["anchors"][0]["baseline_git_object"] = "0" * 40
    expect_rejected(validate_source_anchors, missing_anchor, "stale source object")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--baseline", action="store_true")
    parser.add_argument("--findings", action="store_true")
    parser.add_argument("--ledger", action="store_true")
    parser.add_argument("--adr", action="store_true")
    parser.add_argument("--source-anchors", action="store_true")
    parser.add_argument("--private-boundary", action="store_true")
    parser.add_argument("--phase")
    parser.add_argument("--final", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    arguments = parser.parse_args()
    selected = any(
        (
            arguments.baseline,
            arguments.findings,
            arguments.ledger,
            arguments.adr,
            arguments.source_anchors,
            arguments.private_boundary,
            arguments.phase,
            arguments.final,
            arguments.self_test,
        )
    )
    checks: list[tuple[str, object]] = []
    if not selected or arguments.baseline:
        checks.append(("baseline", lambda: validate_baseline(load_json("reports/remediation_v3_baseline.json"))))
    if not selected or arguments.findings:
        checks.append(("findings", lambda: validate_findings(load_json("spec/remediation_findings_v3.json"))))
    if not selected or arguments.ledger:
        checks.append(("ledger", validate_ledger))
    if not selected or arguments.adr:
        checks.append(("adr", validate_adrs))
    if not selected or arguments.source_anchors:
        checks.append(("source-anchors", lambda: validate_source_anchors(load_json("reports/remediation_v3_source_manifest.json"))))
    if arguments.private_boundary:
        checks.append(("private-boundary", validate_private_boundary))
    if arguments.phase:
        checks.append((f"phase:{arguments.phase}", lambda: validate_phase(arguments.phase)))
    if arguments.final:
        checks.append(("final", validate_final))
    if arguments.self_test:
        checks.append(("self-test", self_test))
    try:
        for name, check in checks:
            check()  # type: ignore[operator]
            print(f"PASS: remediation v3 {name}")
    except PendingArtifactError as error:
        print(f"PENDING: {error}", file=sys.stderr)
        return 2
    except (AssertionError, KeyError, TypeError, ValueError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
