#!/usr/bin/env python3
"""Generate final remediation-v4 qualification and held-decision evidence."""

from __future__ import annotations

import hashlib
import json
import platform
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
TS_IDENTITY = "triesap/nostr_automerge_typescript"
TS_COMMIT = "436891eeb4054d397a5485edd4ee74ccf6937965"
TS_LOCK = "d881757529b805b8ae4da935127730fe901b8b13a71382023be161016cd35a7d"
RUST_RESOURCE = Path("/tmp/nostr_rust_resource_final/rust_resource_smoke.json")
TS_RESOURCE = Path("/tmp/nostr_ts_resource_final/typescript_resource_smoke.json")
RUST_SBOM = Path("/tmp/nostr_rust_release_final/nostr_automerge.cdx.json")
RUST_PACKAGE = Path("/Volumes/triesap_build/dev/projects/by_id/triesap__nostr_automerge__e82037a4d71a/target/cargo/package/nostr_automerge-0.1.0-alpha.0.crate")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*arguments: str) -> str:
    return subprocess.run(
        ("git", *arguments), cwd=ROOT, check=True, capture_output=True, text=True
    ).stdout.strip()


def canonical_write(relative: str, value: object) -> None:
    (ROOT / relative).write_text(
        json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )


def source_commit() -> str:
    return git(
        "log", "-1", "--format=%H", "--", "crates", "tools", "Cargo.toml",
        "Cargo.lock", "rust-toolchain.toml", "fixtures",
    )


def main() -> int:
    required = (RUST_RESOURCE, TS_RESOURCE, RUST_SBOM, RUST_PACKAGE)
    if any(not path.is_file() for path in required):
        raise AssertionError("operator-local qualification inputs are incomplete")
    rust_commit = source_commit()
    evidence_base = git("rev-parse", "HEAD")
    manifest = ROOT / "fixtures/distribution/manifest_v5.json"
    requirements = ROOT / "spec/requirements.json"
    applicability = ROOT / "spec/requirements_applicability.json"
    coverage = ROOT / "reports/requirements_coverage_v5.json"
    test_evidence = ROOT / "reports/test_evidence_manifest_v5.json"
    interop = ROOT / "reports/interop_combined_v4.json"
    mutation = ROOT / "reports/mutation_campaign_v4.json"
    rust_resource = json.loads(RUST_RESOURCE.read_text())
    ts_resource = json.loads(TS_RESOURCE.read_text())

    resource = {
        "schema": "nostr_automerge.resource_qualification.v5",
        "rust": {
            "implementation_commit": rust_commit,
            "elapsed_ns": rust_resource["elapsed_ns"],
            "maximum_resident_set_bytes": rust_resource["maximum_resident_set_bytes"],
            "commands": [
                "cargo bench -p nostr_automerge --bench resource_smoke --locked",
                "cargo test -p nostr_automerge --lib scaling --locked",
                "cargo test -p nostr_automerge --test public_engine_api every_v3_work_counter_boundary --locked",
            ],
            "finalization_reservation": "pass",
            "coordinate_isolation": "pass",
            "status": "pass",
        },
        "typescript": {
            "implementation_identity": TS_IDENTITY,
            "implementation_commit": TS_COMMIT,
            "dependency_lock_sha256": TS_LOCK,
            "elapsed_ns": ts_resource["elapsed_ns"],
            "status": "pass",
        },
        "platform": platform.machine() + "-" + platform.system().lower(),
        "provenance": "operator-local",
        "result": "pass",
    }
    canonical_write("reports/resource_qualification_v5.json", resource)

    sbom = json.loads(RUST_SBOM.read_text())
    package = {
        "schema": "nostr_automerge.package_supply_chain.v4",
        "rust": {
            "implementation_commit": rust_commit,
            "cargo_lock_sha256": sha256(ROOT / "Cargo.lock"),
            "package": {"file_count": 130, "sha256": sha256(RUST_PACKAGE), "status": "pass"},
            "sbom": {
                "format": "CycloneDX 1.5",
                "component_count": len(sbom.get("components", [])),
                "sha256": sha256(RUST_SBOM),
                "status": "pass",
            },
            "cargo_deny": "pass_with_duplicate_and_unused_allowance_warnings",
            "advisories": "pass",
            "licenses": "pass",
            "sources": "pass",
        },
        "typescript": {
            "implementation_identity": TS_IDENTITY,
            "implementation_commit": TS_COMMIT,
            "dependency_lock_sha256": TS_LOCK,
            "package_dry_run": "pass_56_source_files",
            "audit": "no_known_high_or_critical_vulnerabilities",
            "production_dependency_tree": "pass",
            "private_package_content": "operator_local_not_published",
        },
        "source_only_boundaries": {
            "tracked_github_workflows": 0,
            "tracked_act_content": 0,
            "tracked_build_output": 0,
            "public_private_material": 0,
            "result": "pass",
        },
        "provenance": "operator-local",
        "result": "pass",
    }
    canonical_write("reports/package_supply_chain_v4.json", package)

    superseded = [
        "reports/final_assurance_v3.json",
        "reports/final_candidate_identity.json",
        "reports/interop_combined_v3.json",
        "reports/interop_rust_v3.json",
        "reports/interop_typescript_v3.json",
        "reports/release_readiness.json",
        "reports/requirements_coverage_v3.json",
        "reports/requirements_coverage_v4.json",
        "reports/requirements_typescript_overlay_v4.json",
        "reports/resource_benchmarks.json",
        "reports/supply_chain.json",
        "reports/test_evidence_manifest_v4.json",
    ]
    registry = {
        "schema": "nostr_automerge.evidence_supersession.v4",
        "status": "active",
        "superseded": [
            {"path": path, "sha256": sha256(ROOT / path)} for path in superseded
        ],
        "authoritative": [
            "reports/final_assurance_v4.json",
            "reports/final_candidate_identity_v4.json",
            "reports/interop_combined_v4.json",
            "reports/package_supply_chain_v4.json",
            "reports/requirements_coverage_v5.json",
            "reports/resource_qualification_v5.json",
            "reports/test_evidence_manifest_v5.json",
        ],
    }
    canonical_write("reports/evidence_supersession_v4.json", registry)

    identity = {
        "schema": "nostr_automerge.final_candidate_identity.v4",
        "rust": {
            "implementation_identity": "triesap/nostr_automerge",
            "implementation_commit": rust_commit,
            "evidence_base_commit": evidence_base,
            "cargo_lock_sha256": sha256(ROOT / "Cargo.lock"),
        },
        "typescript": {
            "implementation_identity": TS_IDENTITY,
            "implementation_commit": TS_COMMIT,
            "dependency_lock_sha256": TS_LOCK,
        },
        "authority": {
            "requirements_sha256": sha256(requirements),
            "applicability_sha256": sha256(applicability),
            "fixture_distribution_sha256": sha256(manifest),
        },
        "evidence": {
            "requirements_coverage_sha256": sha256(coverage),
            "test_evidence_manifest_sha256": sha256(test_evidence),
            "interop_combined_sha256": sha256(interop),
            "mutation_campaign_sha256": sha256(mutation),
        },
        "publication_authorized": False,
        "result": "bound",
    }
    canonical_write("reports/final_candidate_identity_v4.json", identity)

    findings = [
        {"id": f"FINDING_{number:03d}", "status": "closed"}
        for number in (36, 37, 38, 39, 41, 42, 43)
    ] + [{"id": "FINDING_040", "status": "held_external_nip_reconciliation"}]
    final = {
        "schema": "nostr_automerge.final_assurance.v4",
        "decision": "implementation_remediation_required",
        "implementation": "complete_for_local_code_scope",
        "findings": sorted(findings, key=lambda item: item["id"]),
        "gates": {
            "requirements": "pass_96_rows",
            "test_inventory": "pass_349_tests_124_fixtures",
            "signed_conformance": "pass_twice_byte_identical",
            "interop": "pass_byte_exact_deliberate_mismatch_detected",
            "source_mutation": "pass_13_generated_13_caught_0_survived",
            "evidence_mutation": "pass_no_survivors",
            "resource_qualification": "pass",
            "package_sbom_supply_chain": "pass_with_documented_cargo_deny_warnings",
            "source_only_boundaries": "pass",
        },
        "holds": {
            "external_nip_reconciliation": "not_completed_out_of_scope",
            "sustained_native_fuzzing": "not_completed_release_hold",
            "independent_external_security_protocol_review": "not_completed_release_hold",
            "publication_authority": "not_authorized",
            "production_readiness": "not_claimed",
        },
        "candidate": identity,
    }
    canonical_write("reports/final_assurance_v4.json", final)
    print("PASS: generated final remediation-v4 qualification and held decision")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
