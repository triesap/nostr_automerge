#!/usr/bin/env python3
"""Validate final remediation-v3 evidence and its truthful claim boundary."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
from pathlib import Path

from generate_final_evidence_v3 import BASELINE, NIP_SHA256, authority_reconciliation, implementation_commit


ROOT = Path(__file__).resolve().parents[1]


class EvidenceError(ValueError):
    """One final closure invariant failed."""


def load(relative: str) -> dict[str, object]:
    value = json.loads((ROOT / relative).read_text())
    if not isinstance(value, dict):
        raise EvidenceError(f"shape:{relative}")
    return value


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_candidate(candidate: dict[str, object]) -> None:
    rust = candidate.get("rust", {})
    ts = candidate.get("typescript", {})
    attestation = load("reports/interop_typescript_v3.json")
    if candidate.get("schema") != "nostr_automerge.final_candidate_identity.v1" or candidate.get("status") != "pass":
        raise EvidenceError("candidate_shape")
    if rust.get("implementation_commit") != implementation_commit() or rust.get("cargo_lock_sha256") != sha256(ROOT / "Cargo.lock") or rust.get("protected_changes_after_implementation") != []:
        raise EvidenceError("rust_candidate")
    evidence_commit = rust.get("evidence_commit", "")
    if subprocess.run(("git", "merge-base", "--is-ancestor", rust["implementation_commit"], evidence_commit), cwd=ROOT).returncode != 0:
        raise EvidenceError("evidence_not_descendant")
    if ts.get("implementation_commit") != attestation.get("commit") or ts.get("dependency_lock_sha256") != attestation.get("dependency_lock_sha256") or ts.get("attestation_sha256") != sha256(ROOT / "reports/interop_typescript_v3.json"):
        raise EvidenceError("typescript_candidate")
    if candidate.get("fixture_distribution_sha256") != sha256(ROOT / "fixtures/distribution/manifest_v4.json"):
        raise EvidenceError("candidate_distribution")


def validate() -> None:
    candidate = load("reports/final_candidate_identity.json")
    validate_candidate(candidate)
    authority = load("reports/authority_reconciliation_v4.json")
    if authority != authority_reconciliation() or authority.get("baseline_commit") != BASELINE or authority.get("external_nip_sha256") != NIP_SHA256 or authority.get("external_nip_modified") is not False:
        raise EvidenceError("authority_reconciliation")
    legacy = load("reports/interop_combined.json")
    if legacy.get("schema") != "nostr_automerge.superseded_evidence.v1" or legacy.get("status") != "superseded" or legacy.get("authoritative_artifact_sha256") != sha256(ROOT / "reports/interop_combined_v3.json"):
        raise EvidenceError("legacy_interop_not_superseded")
    leak = load("reports/typescript_leak_boundary_v3.json")
    if leak.get("status") != "pass" or leak.get("attestation_sha256") != sha256(ROOT / "reports/interop_typescript_v3.json"):
        raise EvidenceError("leak_boundary")
    closure = load("reports/remediation_closure_v3.json")
    findings = {item["id"]: item for item in closure.get("findings", [])}
    if closure.get("status") != "code_complete_publication_held" or closure.get("candidate") != candidate or set(findings) != {f"FINDING_{index:03d}" for index in range(28, 36)}:
        raise EvidenceError("closure_shape")
    if any(findings[f"FINDING_{index:03d}"].get("result") != "closed" for index in range(28, 35)):
        raise EvidenceError("implementation_finding_open")
    if findings["FINDING_035"].get("result") != "resolved_with_release_holds" or set(findings["FINDING_035"].get("holds", [])) != {"sustained_native_fuzzing", "independent_external_review", "publication_authority"}:
        raise EvidenceError("release_holds")
    readiness = load("reports/release_readiness.json")
    if readiness.get("decision") != "code_complete_publication_held" or readiness.get("sustained_fuzzing") != "not_completed_release_hold" or readiness.get("external_review") != "not_completed_release_hold" or readiness.get("publication_authority") != "not_authorized":
        raise EvidenceError("readiness_claim")
    fuzz = load("reports/fuzz_campaign.json")
    review = load("reports/external_review.json")
    if fuzz.get("status") != "not_completed_release_hold" or review.get("status") != "not_completed_release_hold":
        raise EvidenceError("assurance_hold_erased")


def self_test() -> None:
    baseline = load("reports/final_candidate_identity.json")
    mutations = []
    commit = copy.deepcopy(baseline); commit["rust"]["implementation_commit"] = "00" * 20; mutations.append(("rust_commit", commit))
    protected = copy.deepcopy(baseline); protected["rust"]["protected_changes_after_implementation"] = ["crates/private.rs"]; mutations.append(("protected_change", protected))
    lock = copy.deepcopy(baseline); lock["typescript"]["dependency_lock_sha256"] = "00" * 32; mutations.append(("typescript_lock", lock))
    distribution = copy.deepcopy(baseline); distribution["fixture_distribution_sha256"] = "00" * 32; mutations.append(("distribution", distribution))
    caught = []
    for name, mutation in mutations:
        try:
            validate_candidate(mutation)
        except EvidenceError as error:
            caught.append({"mutation": name, "diagnostic": str(error), "result": "caught"})
            continue
        raise AssertionError(f"final evidence mutation survived: {name}")
    report = {"schema": "nostr_automerge.final_evidence_mutations.v3", "generated": len(caught), "caught": len(caught), "survived": 0, "status": "pass", "mutations": caught}
    (ROOT / "reports/final_evidence_mutations_v3.json").write_text(json.dumps(report, sort_keys=True, separators=(",", ":")) + "\n")


def main() -> int:
    parser = argparse.ArgumentParser(); parser.add_argument("--self-test", action="store_true"); args = parser.parse_args()
    validate()
    if args.self_test:
        self_test()
    print("PASS: final remediation evidence is exact and publication-held")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
