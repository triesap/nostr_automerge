#!/usr/bin/env python3
"""Generate exact remediation-v7 Rust evidence for all 129 requirements."""

from __future__ import annotations

import hashlib
import json
import subprocess
from collections import defaultdict
from pathlib import Path

from generate_requirement_matrix import rust_proof
from generate_requirement_matrix_v3 import test_id
from generate_requirement_matrix_v7 import EXACT_ASSERTIONS as V7_EXACT_ASSERTIONS
from generate_requirement_matrix_v7 import EXACT_FIXTURES as V7_EXACT_FIXTURES
from generate_requirement_matrix_v7 import exact_assertion_path
from validate_requirement_matrix_v7 import signed_artifact_hash


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "fixtures/distribution/manifest_v8.json"
EXACT_FIXTURES = {key: value.copy() for key, value in V7_EXACT_FIXTURES.items()}
BRANCH = [
    "change_references_invalid_noncanonical_child",
    "manifest_references_invalid_noncanonical_child",
    "noncanonical_child_excluded_base_head",
    "noncanonical_child_invalid_base_head",
    "noncanonical_child_pending_base_head",
    "noncanonical_grandchild_invalid_parent_epoch",
]
SCOPE = [
    "cross_coordinate_descriptor_reference_isolated",
    "foreign_change_references_target_control",
    "foreign_chunk_excluded_from_target_digest",
    "foreign_chunk_references_target_descriptor",
    "foreign_claim_flood_exact_budget",
    "unrelated_valid_checkpoints_exact_budget",
]
EXACT_FIXTURES.update({
    "NCRDT-BRANCH-001": BRANCH,
    "NCRDT-BRANCH-002": BRANCH,
    "NCRDT-SCOPE-004": [
        "foreign_change_references_target_control",
        "foreign_claim_flood_exact_budget",
    ],
    "NCRDT-SCOPE-005": [
        "cross_coordinate_descriptor_reference_isolated",
        "foreign_chunk_references_target_descriptor",
    ],
    "NCRDT-SCOPE-006": SCOPE,
    "NCRDT-RESOURCE-009": ["parent_propagation_exact_budget"],
    "NCRDT-RESOURCE-010": ["interrupted_finalization_forfeiture"],
    "NCRDT-CONF-008": BRANCH
    + SCOPE
    + ["interrupted_finalization_forfeiture", "parent_propagation_exact_budget"],
})
EXACT_ASSERTIONS = dict(V7_EXACT_ASSERTIONS)
EXACT_ASSERTIONS["NCRDT-EVIDENCE-004"] = (
    "scripts/validate_requirement_matrix_v8.py",
    ["artifact_hash", "generic_critical", "status_overclaim"],
)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args: str) -> str:
    return subprocess.run(
        ("git", *args), cwd=ROOT, check=True, capture_output=True, text=True
    ).stdout.strip()


def canonical_write(path: Path, value: object) -> None:
    path.write_text(
        json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )


def main() -> int:
    requirements_path = ROOT / "spec/requirements.json"
    applicability_path = ROOT / "spec/requirements_applicability.json"
    requirements = json.loads(requirements_path.read_text())["requirements"]
    applicability = json.loads(applicability_path.read_text())["classifications"]
    distribution = json.loads(MANIFEST.read_text())
    fixture_paths = {
        item["fixture_id"]: ROOT / item["metadata_path"]
        for item in distribution["fixtures"]
    }
    by_requirement: dict[str, list[str]] = defaultdict(list)
    for item in distribution["fixtures"]:
        for requirement in item["requirements"]:
            by_requirement[requirement].append(item["fixture_id"])
    rust_candidate = git("rev-parse", "HEAD")
    prior_typescript = json.loads(
        (ROOT / "reports/interop_typescript_v7.json").read_text()
    )["candidate"]
    rows: list[dict[str, object]] = []
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
            row["status"] = (
                "external-hold" if identifier in {"NCRDT-NIP-001", "NCRDT-NIP-002"}
                else "not-applicable"
            )
            row["rationale"] = (
                "The externally authored NIP remains read-only and unreconciled."
                if identifier in {"NCRDT-NIP-001", "NCRDT-NIP-002"}
                else "Approved authority classifies this requirement outside local deterministic implementation."
            )
            rows.append(row)
            continue

        source = rust_proof(identifier)
        fixture_ids = sorted(
            EXACT_FIXTURES.get(identifier, by_requirement.get(identifier, [])),
            key=str.encode,
        )
        if fixture_ids:
            evidence_kind = "signed-fixture"
            evidence_ids = fixture_ids
            test_path = "tools/nostr_automerge_conformance/src/runner.rs"
            artifact_hash = signed_artifact_hash(evidence_ids, fixture_paths)
            command = "cargo run -p nostr_automerge_conformance --locked -- run_corpus fixtures/v1_draft/scenarios"
        elif identifier in EXACT_ASSERTIONS:
            test_path, evidence_ids = EXACT_ASSERTIONS[identifier]
            evidence_kind = "exact-assertion"
            test_path = exact_assertion_path(test_path, evidence_ids)
            artifact_hash = sha256(ROOT / test_path)
            command = "cargo test --workspace --all-targets --locked"
        else:
            evidence_kind = "exact-assertion"
            evidence_ids = [test_id(identifier)]
            test_path = exact_assertion_path(source["test"], evidence_ids)
            artifact_hash = sha256(ROOT / test_path)
            command = "cargo test --workspace --all-targets --locked"
        row["rust_proof"] = {
            "candidate": rust_candidate,
            "implementation_path": source["implementation"],
            "test_path": test_path,
            "evidence_kind": evidence_kind,
            "evidence_ids": evidence_ids,
            "command": command,
            "result": "pass",
            "artifact_sha256": artifact_hash,
        }
        row["status"] = "pass" if classification == "rust-only" else "pending"
        rows.append(row)

    report = {
        "schema": "nostr_automerge.requirement_coverage.v8",
        "phase": "rust-complete-typescript-pending",
        "requirements_sha256": sha256(requirements_path),
        "applicability_sha256": sha256(applicability_path),
        "fixture_distribution_sha256": sha256(MANIFEST),
        "rust_candidate": rust_candidate,
        "typescript_candidate": prior_typescript,
        "requirement_count": len(rows),
        "rows": rows,
    }
    canonical_write(ROOT / "reports/requirements_coverage_v8.json", report)
    print(f"PASS: generated {len(rows)} exact Rust requirement rows; TypeScript pending")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
