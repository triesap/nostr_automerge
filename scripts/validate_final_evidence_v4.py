#!/usr/bin/env python3
"""Fail closed on remediation-v4 final evidence and held decision."""

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
FORBIDDEN = ("/" + "Users/", "/" + "home/", "file" + "://", ".act" + "/", "." + "log")


class FinalEvidenceError(ValueError):
    pass


def load(relative: str) -> dict[str, object]:
    value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise FinalEvidenceError(f"object:{relative}")
    return value


def sha256(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def source_commit() -> str:
    return subprocess.run(
        ("git", "log", "-1", "--format=%H", "--", "crates", "tools", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "fixtures"),
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()


def validate(final: dict[str, object]) -> None:
    expected_top = {"schema", "decision", "implementation", "findings", "gates", "holds", "candidate"}
    if set(final) != expected_top or final.get("schema") != "nostr_automerge.final_assurance.v4":
        raise FinalEvidenceError("final_shape")
    if final.get("decision") != "implementation_remediation_required" or final.get("implementation") != "complete_for_local_code_scope":
        raise FinalEvidenceError("decision")
    findings = final.get("findings")
    if not isinstance(findings, list) or [item.get("id") for item in findings] != [f"FINDING_{number:03d}" for number in range(36, 44)]:
        raise FinalEvidenceError("finding_membership")
    expected_statuses = ["closed"] * 4 + ["held_external_nip_reconciliation"] + ["closed"] * 3
    if [item.get("status") for item in findings] != expected_statuses:
        raise FinalEvidenceError("finding_status")
    holds = final.get("holds")
    expected_holds = {
        "external_nip_reconciliation": "not_completed_out_of_scope",
        "sustained_native_fuzzing": "not_completed_release_hold",
        "independent_external_security_protocol_review": "not_completed_release_hold",
        "publication_authority": "not_authorized",
        "production_readiness": "not_claimed",
    }
    if holds != expected_holds:
        raise FinalEvidenceError("holds")
    candidate = load("reports/final_candidate_identity_v4.json")
    if final.get("candidate") != candidate:
        raise FinalEvidenceError("candidate_substitution")
    rust = candidate.get("rust", {})
    typescript = candidate.get("typescript", {})
    if rust.get("implementation_commit") != source_commit() or not HEX40.fullmatch(str(rust.get("evidence_base_commit"))):
        raise FinalEvidenceError("rust_candidate")
    if typescript != {
        "implementation_identity": "triesap/nostr_automerge_typescript",
        "implementation_commit": "436891eeb4054d397a5485edd4ee74ccf6937965",
        "dependency_lock_sha256": "d881757529b805b8ae4da935127730fe901b8b13a71382023be161016cd35a7d",
    }:
        raise FinalEvidenceError("typescript_candidate")
    expected_authority = {
        "requirements_sha256": sha256("spec/requirements.json"),
        "applicability_sha256": sha256("spec/requirements_applicability.json"),
        "fixture_distribution_sha256": sha256("fixtures/distribution/manifest_v5.json"),
    }
    if candidate.get("authority") != expected_authority:
        raise FinalEvidenceError("authority_binding")
    evidence = candidate.get("evidence", {})
    expected_evidence = {
        "requirements_coverage_sha256": sha256("reports/requirements_coverage_v5.json"),
        "test_evidence_manifest_sha256": sha256("reports/test_evidence_manifest_v5.json"),
        "interop_combined_sha256": sha256("reports/interop_combined_v4.json"),
        "mutation_campaign_sha256": sha256("reports/mutation_campaign_v4.json"),
    }
    if evidence != expected_evidence or any(not HEX64.fullmatch(str(value)) for value in evidence.values()):
        raise FinalEvidenceError("evidence_binding")
    resource = load("reports/resource_qualification_v5.json")
    supply = load("reports/package_supply_chain_v4.json")
    if resource.get("result") != "pass" or resource.get("rust", {}).get("implementation_commit") != source_commit():
        raise FinalEvidenceError("resource")
    if supply.get("result") != "pass" or supply.get("source_only_boundaries", {}).get("result") != "pass":
        raise FinalEvidenceError("supply_chain")
    supersession = load("reports/evidence_supersession_v4.json")
    paths = [item.get("path") for item in supersession.get("superseded", [])]
    required_stale = {
        "reports/final_assurance_v3.json",
        "reports/interop_combined_v3.json",
        "reports/requirements_coverage_v3.json",
        "reports/requirements_coverage_v4.json",
        "reports/resource_benchmarks.json",
    }
    if supersession.get("status") != "active" or not required_stale <= set(paths):
        raise FinalEvidenceError("supersession")
    for item in supersession["superseded"]:
        if item.get("sha256") != sha256(str(item.get("path"))):
            raise FinalEvidenceError("superseded_hash")
    serialized = json.dumps((final, candidate, resource, supply, supersession), sort_keys=True)
    if any(token in serialized for token in FORBIDDEN):
        raise FinalEvidenceError("private_material")


def self_test(final: dict[str, object]) -> None:
    mutations = []
    status = copy.deepcopy(final); status["decision"] = "code_complete_publication_held"; mutations.append(status)
    nip = copy.deepcopy(final); nip["findings"][4]["status"] = "closed"; mutations.append(nip)
    fuzz = copy.deepcopy(final); fuzz["holds"].pop("sustained_native_fuzzing"); mutations.append(fuzz)
    commit = copy.deepcopy(final); commit["candidate"]["rust"]["implementation_commit"] = "00" * 20; mutations.append(commit)
    caught = 0
    for mutation in mutations:
        try:
            validate(mutation)
        except FinalEvidenceError:
            caught += 1
            continue
        raise AssertionError("final evidence mutation survived")
    if caught != len(mutations):
        raise AssertionError("incomplete final evidence self-test")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    final = load("reports/final_assurance_v4.json")
    validate(final)
    if args.self_test:
        self_test(final)
    print("PASS: final remediation-v4 evidence is current and the held decision fails closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
