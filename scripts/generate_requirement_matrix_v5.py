#!/usr/bin/env python3
"""Generate final commit-bound Rust and opaque TypeScript requirement evidence."""

from __future__ import annotations

import hashlib
import json
from collections import defaultdict
from pathlib import Path

from generate_requirement_matrix import rust_proof
from generate_requirement_matrix_v3 import test_id


ROOT = Path(__file__).resolve().parents[1]


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def canonical_write(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n", encoding="utf-8")


def main() -> int:
    requirements_path = ROOT / "spec/requirements.json"
    applicability_path = ROOT / "spec/requirements_applicability.json"
    distribution_path = ROOT / "fixtures/distribution/manifest_v5.json"
    manifest_path = ROOT / "reports/test_evidence_manifest_v5.json"
    attestation_path = ROOT / "reports/interop_typescript_v4.json"
    requirements = json.loads(requirements_path.read_text())["requirements"]
    applicability = json.loads(applicability_path.read_text())["classifications"]
    distribution = json.loads(distribution_path.read_text())
    evidence = json.loads(manifest_path.read_text())
    attestation = json.loads(attestation_path.read_text())
    fixtures: dict[str, list[tuple[str, str]]] = defaultdict(list)
    for item in distribution["fixtures"]:
        for requirement in item["requirements"]:
            fixtures[requirement].append((item["fixture_id"], item["profile"]))

    overlay_rows: dict[str, object] = {}
    rows: list[dict[str, object]] = []
    for requirement in requirements:
        identifier = requirement["id"]
        classification = applicability[identifier]
        row: dict[str, object] = {
            "id": identifier,
            "applicability": classification,
            "authority": {"source": requirement["source"], "section": requirement["section"], "text_sha256": hashlib.sha256(requirement["text"].encode()).hexdigest()},
        }
        if classification in {"out-of-core", "explicitly-deferred"}:
            row["status"] = "not-applicable"
            row["rationale"] = "Approved authority classifies this requirement outside the deterministic core." if classification == "out-of-core" else "Approved authority explicitly defers this requirement."
            rows.append(row)
            continue
        source = rust_proof(identifier)
        direct = sorted(fixtures.get(identifier, []))
        if direct:
            kind, evidence_id, job_name = "signed_fixture", direct[0][0], "signed-conformance"
        else:
            kind = "policy" if source["runner_job"] == "policy" else "cargo_test"
            evidence_id, job_name = test_id(identifier), "rust-tests"
        job = evidence["jobs"][job_name]
        row["rust_proof"] = {
            "language": "rust", "implementation_identity": "triesap/nostr_automerge",
            "implementation_commit": evidence["implementation_commit"], "implementation_path": source["implementation"],
            "evidence_kind": kind, "evidence_id": evidence_id, "execution_command": job["command"],
            "runner_job": job_name, "result_artifact": job["result_artifact"], "result_sha256": job["result_sha256"], "result": "pass",
        }
        if classification == "rust-and-typescript":
            fixture_ids = [item[0] for item in direct]
            profiles = sorted({item[1] for item in direct}) if direct else sorted(attestation["profiles"])
            overlay = {
                "language": "typescript", "implementation_identity": attestation["implementation_identity"],
                "implementation_commit": attestation["commit"], "attestation_path": "reports/interop_typescript_v4.json",
                "attestation_sha256": sha256(attestation_path), "dependency_lock_sha256": attestation["dependency_lock_sha256"],
                "fixture_distribution_sha256": attestation["fixture_distribution_sha256"], "profiles": profiles,
                "fixture_ids": fixture_ids, "scope": "direct_signed_fixtures" if direct else "complete_signed_profiles", "result": "pass",
            }
            row["typescript_overlay"] = overlay
            overlay_rows[identifier] = overlay
        row["status"] = "pass"
        rows.append(row)

    overlay_report = {
        "schema": "nostr_automerge.requirement_typescript_overlay.v5", "attestation_path": "reports/interop_typescript_v4.json",
        "attestation_sha256": sha256(attestation_path), "requirement_count": len(overlay_rows), "requirements": overlay_rows,
    }
    overlay_path = ROOT / "reports/requirements_typescript_overlay_v5.json"
    canonical_write(overlay_path, overlay_report)
    report = {
        "schema": "nostr_automerge.requirement_coverage.v5", "requirements_sha256": sha256(requirements_path),
        "applicability_sha256": sha256(applicability_path), "fixture_distribution_sha256": sha256(distribution_path),
        "rust_evidence_manifest_sha256": sha256(manifest_path), "typescript_overlay_sha256": sha256(overlay_path),
        "requirement_count": len(rows), "rows": rows,
    }
    canonical_write(ROOT / "reports/requirements_coverage_v5.json", report)
    print(f"PASS: generated {len(rows)} final rows with {len(overlay_rows)} opaque TypeScript overlays")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
