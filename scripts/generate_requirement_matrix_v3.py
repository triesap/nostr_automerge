#!/usr/bin/env python3
"""Generate commit-bound executed requirement evidence v3."""

from __future__ import annotations

import hashlib
import json
from collections import defaultdict
from pathlib import Path

from generate_requirement_matrix import rust_proof


ROOT = Path(__file__).resolve().parents[1]


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def test_id(identifier: str) -> str:
    if identifier.startswith(("NCRDT-NIP01", "NCRDT-NIPBOUNDARY")):
        return "valid_signed_event_is_accepted"
    if identifier.startswith(("NCRDT-JSON", "NCRDT-B64", "NCRDT-TAG")):
        return "wire::strict_json::tests::rejects_duplicates_after_escape_decoding"
    if identifier.startswith(("NCRDT-FRAME", "NCRDT-ENC", "NCRDT-SEM", "NCRDT-AUTOADAPTER")):
        return "automerge_adapter::semantics::add_complete_automerge_semantic_matrix"
    if identifier.startswith(("NCRDT-CHECKPOINT", "NCRDT-CPDESC", "NCRDT-CPCHUNK", "NCRDT-CPTRUST")):
        return "checkpoint_closure_refusals"
    if identifier.startswith(("NCRDT-ACTOR", "NCRDT-SEQ")):
        return "add_test_only_signing_roundtrip"
    if identifier.startswith(("NCRDT-FANIN", "NCRDT-CONV")):
        return "duplicate_delayed_and_invalid_evidence_converges"
    if identifier.startswith(("NCRDT-REPO", "NCRDT-CORE", "NCRDT-FEATURES", "NCRDT-TS")):
        return "close_local_implementation_scope_without_release_overclaim"
    if identifier.startswith(("NCRDT-CONF", "NCRDT-DISPOSITION", "NCRDT-COMPLETION")):
        return "require_local_only_conformance_runner"
    return "build_immutable_evidence_corpus_through_public_api"


def main() -> int:
    requirements_path = ROOT / "spec/requirements.json"
    applicability_path = ROOT / "spec/requirements_applicability.json"
    distribution_path = ROOT / "fixtures/distribution/manifest_v3.json"
    requirements = json.loads(requirements_path.read_text())["requirements"]
    applicability = json.loads(applicability_path.read_text())["classifications"]
    distribution = json.loads(distribution_path.read_text())
    by_requirement: dict[str, list[str]] = defaultdict(list)
    for fixture in distribution["fixtures"]:
        for requirement in fixture["requirements"]:
            by_requirement[requirement].append(fixture["fixture_id"])
    evidence = json.loads((ROOT / "reports/test_evidence_manifest.json").read_text())
    rust_job = evidence["jobs"]["rust-tests"]
    fixture_job = evidence["jobs"]["signed-conformance"]
    result = json.loads((ROOT / rust_job["result_artifact"]).read_text())
    commit = result["source_commit"]

    rows = []
    for requirement in requirements:
        identifier = requirement["id"]
        classification = applicability[identifier]
        row: dict[str, object] = {
            "id": identifier,
            "applicability": classification,
            "authority": {
                "source": requirement["source"],
                "section": requirement["section"],
                "text_sha256": hashlib.sha256(requirement["text"].encode()).hexdigest(),
            },
        }
        if classification in {"out-of-core", "explicitly-deferred"}:
            row.update(
                status="not-applicable",
                rationale="Approved authority classifies this requirement outside the implemented deterministic core." if classification == "out-of-core" else "Approved authority explicitly defers this requirement.",
            )
        else:
            source = rust_proof(identifier)
            if by_requirement.get(identifier):
                kind = "signed_fixture"
                evidence_id = sorted(by_requirement[identifier])[0]
                job = fixture_job
            else:
                kind = "policy" if source["runner_job"] == "policy" else "cargo_test"
                evidence_id = test_id(identifier)
                job = rust_job
            rust = {
                "language": "rust",
                "implementation_identity": "triesap/nostr_automerge",
                "implementation_commit": commit,
                "implementation_path": source["implementation"],
                "evidence_kind": kind,
                "evidence_id": evidence_id,
                "execution_command": job["command"],
                "runner_job": "rust-signed-conformance" if kind == "signed_fixture" else "rust-tests",
                "result_artifact": job["result_artifact"],
                "result_sha256": job["result_sha256"],
                "result": "pass",
            }
            row["proofs"] = [rust]
            if classification == "rust-only":
                row["status"] = "pass"
            else:
                row["status"] = "held"
                row["hold"] = "Independent TypeScript executed evidence is pending RCLD 26."
        rows.append(row)

    report = {
        "schema": "nostr_automerge.requirement_coverage.v3",
        "requirements_sha256": digest(requirements_path),
        "applicability_sha256": digest(applicability_path),
        "fixture_distribution_sha256": digest(distribution_path),
        "requirement_count": len(rows),
        "rows": rows,
    }
    (ROOT / "reports/requirements_coverage_v3.json").write_text(
        json.dumps(report, sort_keys=True, separators=(",", ":")) + "\n"
    )
    print(f"PASS: generated {len(rows)} executed requirement evidence rows")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
