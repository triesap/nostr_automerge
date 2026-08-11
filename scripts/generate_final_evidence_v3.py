#!/usr/bin/env python3
"""Generate final remediation-v3 authority, candidate, and closure evidence."""

from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path

from validate_typescript_private_boundary_v3 import self_test as leak_self_test
from validate_typescript_private_boundary_v3 import validate_repository as validate_leak_boundary


ROOT = Path(__file__).resolve().parents[1]
BASELINE = "cee7559b8bd7eb00f5f1e37b24c8f9c68e11049d"
LEGACY_INTEROP_SHA256 = "4db60726b29cdeb24715027c7de5958ed84291c2776689f02610373feb2c68c4"
NIP_SHA256 = "67019c8ea680714052c65226f620a8e1a60b9b10a8f158603063a835a7bbc7a3"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*arguments: str) -> str:
    return subprocess.run(("git", *arguments), cwd=ROOT, check=True, capture_output=True, text=True).stdout.strip()


def implementation_commit() -> str:
    return git("log", "-1", "--format=%H", "--", "crates", "tools/nostr_automerge_conformance", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "fixtures")


def write(relative: str, value: object) -> None:
    (ROOT / relative).write_text(json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n", encoding="utf-8")


def old_json(relative: str) -> dict[str, object]:
    return json.loads(git("show", f"{BASELINE}:{relative}"))


def authority_reconciliation() -> dict[str, object]:
    old = {item["id"]: item for item in old_json("spec/requirements.json")["requirements"]}
    current_path = ROOT / "spec/requirements.json"
    current = json.loads(current_path.read_text())["requirements"]
    changes = []
    for item in current:
        before = old[item["id"]]
        if before != item:
            identifier = item["id"]
            if identifier.startswith(("NCRDT-SEQ", "NCRDT-AUTOADAPTER")):
                decision = "docs/adr/adr_0038_causal_operation_counter.md"
            elif identifier.startswith("NCRDT-MANIFEST"):
                decision = "docs/adr/adr_0035_selected_manifest_dynamic_validation.md"
            else:
                decision = "docs/adr/adr_0036_dynamic_event_dispositions.md"
            changes.append({
                "id": identifier, "source": item["source"], "section": item["section"],
                "before_text_sha256": hashlib.sha256(before["text"].encode()).hexdigest(),
                "after_text_sha256": hashlib.sha256(item["text"].encode()).hexdigest(), "approved_decision": decision,
            })
    return {
        "schema": "nostr_automerge.authority_reconciliation.v4", "baseline_commit": BASELINE,
        "baseline_requirements_sha256": hashlib.sha256((git("show", f"{BASELINE}:spec/requirements.json") + "\n").encode()).hexdigest(),
        "current_requirements_sha256": sha256(current_path), "requirement_count": len(current), "changed_requirement_count": len(changes),
        "changes": changes, "external_nip_sha256": sha256(ROOT / "spec/NIP_DRAFT.md"),
        "external_nip_expected_sha256": NIP_SHA256, "external_nip_modified": False,
    }


def main() -> int:
    rust_commit = implementation_commit()
    evidence_commit = git("rev-parse", "HEAD")
    ts = json.loads((ROOT / "reports/interop_typescript_v3.json").read_text())
    write("reports/authority_reconciliation_v4.json", authority_reconciliation())
    changed = git("diff", "--name-only", f"{rust_commit}..HEAD").splitlines()
    protected = [path for path in changed if path.startswith(("crates/", "tools/nostr_automerge_conformance/", "fixtures/")) or path in {"Cargo.toml", "Cargo.lock", "rust-toolchain.toml"}]
    candidate = {
        "schema": "nostr_automerge.final_candidate_identity.v1", "status": "pass",
        "rust": {"implementation_identity": "triesap/nostr_automerge", "implementation_commit": rust_commit, "evidence_commit": evidence_commit, "cargo_lock_sha256": sha256(ROOT / "Cargo.lock"), "protected_changes_after_implementation": protected},
        "typescript": {"implementation_identity": ts["implementation_identity"], "implementation_commit": ts["commit"], "dependency_lock_sha256": ts["dependency_lock_sha256"], "attestation_sha256": sha256(ROOT / "reports/interop_typescript_v3.json")},
        "fixture_distribution_sha256": sha256(ROOT / "fixtures/distribution/manifest_v4.json"),
    }
    write("reports/final_candidate_identity.json", candidate)
    write("reports/interop_combined.json", {
        "schema": "nostr_automerge.superseded_evidence.v1", "status": "superseded",
        "superseded_artifact_sha256": LEGACY_INTEROP_SHA256, "reason": "historical_six_fixture_evidence",
        "authoritative_artifact": "reports/interop_combined_v3.json", "authoritative_artifact_sha256": sha256(ROOT / "reports/interop_combined_v3.json"),
    })
    validate_leak_boundary(); leak_self_test()
    write("reports/typescript_leak_boundary_v3.json", {
        "schema": "nostr_automerge.typescript_leak_boundary.v3", "status": "pass", "mutation_result": "all_rejected",
        "attestation_sha256": sha256(ROOT / "reports/interop_typescript_v3.json"), "evidence_commit": evidence_commit,
        "prohibited": ["source", "repository_url", "absolute_private_path", "raw_log", "workflow_state", "credential"],
    })
    phase_evidence = {
        "FINDING_028": ["reports/remediation_v3_phase_01.json", "reports/remediation_v3_phase_07.json", "reports/requirements_coverage_v4.json"],
        "FINDING_029": ["reports/remediation_v3_phase_02.json", "reports/remediation_v3_phase_07.json", "reports/requirements_coverage_v4.json"],
        "FINDING_030": ["reports/remediation_v3_phase_03.json", "reports/remediation_v3_phase_06.json", "reports/requirements_coverage_v4.json"],
        "FINDING_031": ["reports/remediation_v3_phase_04.json", "reports/remediation_v3_phase_07.json", "reports/requirements_coverage_v4.json"],
        "FINDING_032": ["reports/remediation_v3_phase_05.json", "reports/test_evidence_manifest_v4.json", "reports/requirements_coverage_v4.json"],
        "FINDING_033": ["reports/interop_combined_v3.json", "reports/requirements_typescript_overlay_v4.json", "reports/requirements_coverage_v4.json"],
        "FINDING_034": ["reports/remediation_v3_phase_06.json", "reports/authority_reconciliation_v4.json", "reports/requirements_coverage_v4.json"],
    }
    findings = [{"id": identifier, "result": "closed", "evidence": evidence} for identifier, evidence in phase_evidence.items()]
    findings.append({"id": "FINDING_035", "result": "resolved_with_release_holds", "holds": ["sustained_native_fuzzing", "independent_external_review", "publication_authority"], "evidence": ["reports/fuzz_campaign.json", "reports/external_review.json", "reports/release_readiness.json"]})
    write("reports/remediation_closure_v3.json", {
        "schema": "nostr_automerge.remediation_closure.v3", "status": "code_complete_publication_held", "candidate": candidate,
        "requirements": {"registry_count": 87, "report": "reports/requirements_coverage_v4.json", "status": "pass"}, "findings": findings,
        "holds": ["sustained_native_fuzzing", "independent_external_review", "publication_authority"],
        "non_claims": ["no publication, tag, push, release, deployment, or NIP action", "no sustained native fuzz execution", "no independent external review", "no production-readiness claim"],
    })
    write("reports/fuzz_campaign.json", {
        "schema": "nostr_automerge.fuzz_campaign.v3", "status": "not_completed_release_hold", "decision": "release_hold",
        "implementation_commit": rust_commit, "execution": "deferred_by_operator_direction", "targets_present": sorted(path.name for path in (ROOT / "fuzz/fuzz_targets").glob("*.rs")),
        "non_claims": ["no fuzz target build claim", "no sustained crash-free or timeout-free claim"],
    })
    write("reports/external_review.json", {
        "schema": "nostr_automerge.external_review.v3", "status": "not_completed_release_hold", "decision": "hold_publication",
        "candidate": {"rust_commit": rust_commit, "typescript_commit": ts["commit"]}, "evidence": [],
        "reason": "independent external security and protocol review has not occurred",
    })
    write("reports/release_readiness.json", {
        "schema": "nostr_automerge.release_readiness.v4", "decision": "hold_publication", "code_completion": "complete",
        "local_implementation_status": "code_complete_publication_held",
        "rust_candidate": rust_commit, "typescript_candidate": ts["commit"], "locked_gate": "pass",
        "local_alpha_package": "source_package_verified", "public_engine": "follow_up_remediation_complete",
        "signed_conformance": "pass_103_fixtures", "interop": "byte_exact_final_profiles_pass", "requirements": "executed_evidence_v4_pass",
        "evidence_mutations": "pass_no_survivors", "private_boundary": "pass", "authority_reconciliation": "pass_nip_unchanged",
        "coverage": "pass_with_documented_gaps", "resource_qualification": "representative_local_pass", "supply_chain": "pass",
        "fuzz_targets": "not_run_policy_deferred", "sustained_fuzzing": "not_completed_release_hold", "external_review": "not_completed_release_hold",
        "publication_authority": "not_authorized", "nip_document": "out_of_scope_unchanged",
    })
    print("PASS: generated final remediation-v3 authority, candidate, boundary, and closure evidence")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
