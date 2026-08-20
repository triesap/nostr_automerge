#!/usr/bin/env python3
"""Validate final remediation-v7 candidates, evidence, supersession, and holds."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
HEX40 = re.compile(r"^[0-9a-f]{40}$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")


def load(relative: str) -> dict:
    value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise AssertionError(f"expected object: {relative}")
    return value


def digest(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def main() -> int:
    identity = load("reports/final_candidate_identity_v7.json")
    matrix = load("reports/requirements_coverage_v8.json")
    attestation = load("reports/interop_typescript_v8.json")
    supersession = load("reports/evidence_supersession_v7.json")
    holds = load("reports/external_holds_v7.json")
    if (
        identity.get("schema") != "nostr_automerge.final_candidate_identity.v7"
        or identity.get("result") != "bound"
        or identity.get("status") != "implementation_remediation_required"
        or identity.get("publication_authorized") is not False
    ):
        raise AssertionError("invalid final candidate identity")
    expected_authority = {
        "nip_sha256": digest("spec/NIP_DRAFT.md"),
        "companion_sha256": digest("spec/NOSTR_AUTOMERGE_V1_SPEC.md"),
        "requirements_sha256": digest("spec/requirements.json"),
        "applicability_sha256": digest("spec/requirements_applicability.json"),
        "fixture_distribution_sha256": digest("fixtures/distribution/manifest_v8.json"),
    }
    if identity.get("authority") != expected_authority:
        raise AssertionError("final authority hash is stale")
    rust = identity.get("rust", {})
    rust_candidate = rust.get("source_candidate", "")
    if (
        not HEX40.fullmatch(rust_candidate)
        or rust_candidate != matrix.get("rust_candidate")
        or rust.get("cargo_lock_sha256") != digest("Cargo.lock")
        or rust.get("requirement_matrix_sha256")
        != digest("reports/requirements_coverage_v8.json")
    ):
        raise AssertionError("final Rust candidate is stale")
    if subprocess.run(
        ["git", "cat-file", "-e", f"{rust_candidate}^{{commit}}"],
        cwd=ROOT,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    ).returncode:
        raise AssertionError("final Rust candidate is unavailable")
    typescript = identity.get("typescript", {})
    if (
        typescript.get("implementation_candidate") != attestation.get("candidate")
        or typescript.get("evidence_candidate") != attestation.get("evidence_candidate")
        or typescript.get("attestation_sha256") != digest("reports/interop_typescript_v8.json")
        or matrix.get("typescript_candidate") != attestation.get("candidate")
    ):
        raise AssertionError("final opaque TypeScript candidate is stale")
    for value in (
        typescript.get("implementation_candidate", ""),
        typescript.get("evidence_candidate", ""),
    ):
        if not HEX40.fullmatch(value):
            raise AssertionError("invalid opaque TypeScript candidate")
    if matrix.get("phase") != "complete" or matrix.get("requirement_count") != 129:
        raise AssertionError("final requirement matrix is incomplete")
    rows = matrix.get("rows", [])
    if sum("typescript_proof" in row for row in rows) != 96:
        raise AssertionError("final TypeScript proof count is incomplete")
    conformance = identity.get("conformance", {})
    if (
        conformance.get("fixture_count") != 171
        or conformance.get("permutations_per_fixture") != 8
        or conformance.get("process_runs_per_implementation") != 2
        or conformance.get("canonical_report_bytes") != "identical"
        or conformance.get("deliberate_mismatch") != "detected"
        or conformance.get("differential_corpus_sha256")
        != attestation.get("corpus_sha256")
    ):
        raise AssertionError("final conformance binding is incomplete")
    if supersession.get("schema") != "nostr_automerge.evidence_supersession.v7":
        raise AssertionError("invalid evidence supersession schema")
    for group in ("authoritative", "superseded"):
        rows = supersession.get(group)
        if not isinstance(rows, list) or not rows:
            raise AssertionError(f"empty evidence supersession group: {group}")
        for row in rows:
            if row.get("sha256") != digest(row.get("path", "")):
                raise AssertionError(f"stale {group} evidence: {row.get('path')}")
    hold_rows = holds.get("holds", [])
    if (
        holds.get("status") != "implementation_remediation_required"
        or len(hold_rows) != 7
        or any(row.get("local_result_claimed") is not False for row in hold_rows)
        or holds.get("remote_actions_performed") is not False
    ):
        raise AssertionError("external holds overclaim completion")
    public_attestation = (ROOT / "reports/interop_typescript_v8.json").read_text()
    forbidden = (
        "/" + "Users/",
        "domains/labs",
        "triesap/" + "dev",
        ".act/workflows",
        ".github/workflows",
    )
    if any(value in public_attestation for value in forbidden):
        raise AssertionError("opaque TypeScript attestation leaks private material")
    if any(
        not HEX64.fullmatch(row.get("sha256", ""))
        for group in ("authoritative", "superseded")
        for row in supersession[group]
    ):
        raise AssertionError("invalid evidence digest")
    print("PASS: remediation-v7 final candidate evidence")
    print("- requirements=129 typescript_proofs=96 fixtures=171 permutations=8")
    print("- status=implementation_remediation_required publication=false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
