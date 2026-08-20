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
    closure = load("reports/remediation_v7_final.json")
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
    if (
        closure.get("schema") != "nostr_automerge.remediation_v7_final.v1"
        or closure.get("status") != "implementation_remediation_required"
        or closure.get("local_implementation") != "pass"
        or closure.get("publication_authorized") is not False
        or closure.get("remote_actions_performed") is not False
    ):
        raise AssertionError("invalid remediation-v7 closure")
    expected_authority = {
        "nip_sha256": digest("spec/NIP_DRAFT.md"),
        "companion_sha256": digest("spec/NOSTR_AUTOMERGE_V1_SPEC.md"),
        "requirements_sha256": digest("spec/requirements.json"),
        "applicability_sha256": digest("spec/requirements_applicability.json"),
        "fixture_distribution_sha256": digest("fixtures/distribution/manifest_v8.json"),
    }
    if identity.get("authority") != expected_authority:
        raise AssertionError("final authority hash is stale")
    closure_authority = closure.get("authority", {})
    if (
        closure_authority
        != {
            **expected_authority,
            "nip_edited": False,
        }
    ):
        raise AssertionError("closure authority hash is stale")
    rust = identity.get("rust", {})
    typescript = identity.get("typescript", {})
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
    candidates = closure.get("candidates", {})
    rust_evidence = candidates.get("rust_evidence", "")
    if (
        candidates.get("rust_source") != rust_candidate
        or candidates.get("typescript_implementation")
        != typescript.get("implementation_candidate")
        or candidates.get("typescript_evidence") != typescript.get("evidence_candidate")
        or not HEX40.fullmatch(rust_evidence)
    ):
        raise AssertionError("closure candidate binding is stale")
    for candidate in (rust_candidate, rust_evidence):
        if subprocess.run(
            ["git", "merge-base", "--is-ancestor", candidate, "HEAD"],
            cwd=ROOT,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        ).returncode:
            raise AssertionError("closure Rust candidate is not an ancestor")
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
    sequence = closure.get("sequence", {})
    if (
        sequence.get("first_step") != 1059
        or sequence.get("last_step") != 1095
        or sequence.get("checkpoint_count") != 37
        or sequence.get("completed_rclds") != list(range(65, 73))
        or sequence.get("unfinished_rclds") != []
    ):
        raise AssertionError("invalid remediation-v7 sequence closure")
    workflows = closure.get("operator_local_workflows", {})
    if (
        workflows.get("ownership") != "private_untracked"
        or workflows.get("definitions_tracked_in_source_repositories") is not False
        or workflows.get("outputs_tracked_in_source_repositories") is not False
        or any(
            workflows.get(name) != "pass"
            for name in (
                "remediation",
                "held_campaign_readiness",
                "interoperability",
                "complete_local_suite",
            )
        )
    ):
        raise AssertionError("operator-local workflow evidence is incomplete")
    held = {row.get("id") for row in hold_rows}
    if set(closure.get("held_campaigns", [])) | set(
        closure.get("external_holds", [])
    ) != held:
        raise AssertionError("closure hold set is incomplete")
    evidence = closure.get("evidence", {})
    if set(evidence.values()) != {
        "reports/final_candidate_identity_v7.json",
        "reports/requirements_coverage_v8.json",
        "reports/interop_typescript_v8.json",
        "reports/resource_qualification_v7.json",
        "reports/external_holds_v7.json",
    }:
        raise AssertionError("closure evidence set is incomplete")
    if any(not (ROOT / path).is_file() for path in evidence.values()):
        raise AssertionError("closure evidence file is unavailable")
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
