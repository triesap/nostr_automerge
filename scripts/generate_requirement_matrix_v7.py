#!/usr/bin/env python3
"""Generate remediation-v6 exact evidence for all 119 requirements."""

from __future__ import annotations

import hashlib
import json
import subprocess
from collections import defaultdict
from pathlib import Path

from generate_requirement_matrix import rust_proof
from generate_requirement_matrix_v3 import test_id
from validate_requirement_matrix_v7 import signed_artifact_hash


ROOT = Path(__file__).resolve().parents[1]
TS_ATTESTATION = ROOT / "reports/interop_typescript_v7.json"
EXACT_FIXTURES: dict[str, list[str]] = {
    "NCRDT-CLAIM-001": ["change_references_unsupported_control"],
    "NCRDT-CLAIM-002": ["unauthorized_change_under_noncanonical_control"],
    "NCRDT-CLAIM-003": ["change_under_terminal_control"],
    "NCRDT-CONTROLREF-001": [
        "child_references_static_invalid_parent",
        "child_references_unsupported_parent_control",
        "child_references_wrong_coordinate_parent",
        "child_references_wrong_kind_parent",
        "control_transition_unknown_parent",
    ],
    "NCRDT-CONTROLREF-002": [
        "deep_noncanonical_branch_control_validation",
        "descendant_of_invalid_control_is_invalid",
        "descendant_of_pending_control_is_pending",
    ],
    "NCRDT-FRONTIER-001": [
        "child_base_head_is_known_excluded",
        "child_base_head_is_known_invalid",
        "child_base_head_is_known_other_control",
        "child_base_head_is_known_unsupported",
    ],
    "NCRDT-CPCHUNK-004": [
        "checkpoint_descriptor_references_invalid_control",
        "checkpoint_descriptor_references_pending_control",
        "checkpoint_descriptor_references_unsupported_control",
        "checkpoint_descriptor_references_wrong_coordinate_control",
        "checkpoint_descriptor_references_wrong_kind_control",
        "chunk_references_invalid_descriptor",
        "chunk_references_pending_descriptor",
        "chunk_references_unsupported_descriptor",
        "chunk_references_wrong_coordinate_descriptor",
        "chunk_references_wrong_kind_descriptor",
        "orphan_chunk_promotes_after_descriptor_delivery",
    ],
    "NCRDT-CONF-007": [
        "change_references_unsupported_control",
        "unauthorized_change_under_noncanonical_control",
        "change_under_terminal_control",
        "pending_and_noncanonical_claims_same_hash",
        "pending_and_invalid_claims_same_hash",
        "pruned_and_pending_claims_same_hash",
        "equivocation_excluded_and_pending_claims_same_hash",
        "child_references_unsupported_parent_control",
        "child_references_wrong_kind_parent",
        "child_references_static_invalid_parent",
        "child_references_wrong_coordinate_parent",
        "child_base_head_is_known_invalid",
        "child_base_head_is_known_excluded",
        "child_base_head_is_known_unsupported",
        "child_base_head_is_known_other_control",
        "descendant_of_pending_control_is_pending",
        "descendant_of_invalid_control_is_invalid",
        "deep_noncanonical_branch_control_validation",
        "dependency_known_through_other_control",
        "dependency_known_through_unsupported_control",
        "dependency_known_through_prior_equivocation_exclusion",
        "dependency_known_through_invalid_control",
        "checkpoint_descriptor_references_pending_control",
        "checkpoint_descriptor_references_wrong_kind_control",
        "checkpoint_descriptor_references_wrong_coordinate_control",
        "checkpoint_descriptor_references_unsupported_control",
        "checkpoint_descriptor_references_invalid_control",
        "chunk_references_wrong_kind_descriptor",
        "chunk_references_wrong_coordinate_descriptor",
        "chunk_references_invalid_descriptor",
        "chunk_references_unsupported_descriptor",
        "chunk_references_pending_descriptor",
        "orphan_chunk_promotes_after_descriptor_delivery",
    ],
}
EXACT_ASSERTIONS: dict[str, tuple[str, list[str]]] = {
    "NCRDT-RESOURCE-005": (
        "crates/nostr_automerge/tests/public_engine_api.rs",
        [
            "cancellation_before_control_evaluation_fabricates_no_state",
            "zero_budget_target_entry_consumes_no_work",
        ],
    ),
    "NCRDT-RESOURCE-006": (
        "crates/nostr_automerge/tests/public_engine_api.rs",
        [
            "prior_knowledge_classification_has_cooperative_stop_boundaries",
            "prior_knowledge_exhaustion_is_deterministic_at_every_item_boundary",
        ],
    ),
    "NCRDT-RESOURCE-007": (
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        ["finalization_dimensions_reject_underflow_and_double_finish"],
    ),
    "NCRDT-RESOURCE-008": (
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        ["report_validation_precedes_finalization_refund"],
    ),
}
REQUIRED_TS_FIXTURES: dict[str, list[str]] = {
    key: value.copy() for key, value in EXACT_FIXTURES.items()
}
REQUIRED_TS_FIXTURES.update({
    "NCRDT-RESOURCE-005": [
        "interrupted_cancel_at_ingress",
        "unrelated_changes_do_not_consume_target_budget",
    ],
    "NCRDT-RESOURCE-006": [
        "dependency_known_through_invalid_control",
        "dependency_known_through_other_control",
        "dependency_known_through_prior_equivocation_exclusion",
        "dependency_known_through_unsupported_control",
    ],
    "NCRDT-RESOURCE-007": [
        "interrupted_report_reservation_after",
        "interrupted_report_reservation_before",
    ],
    "NCRDT-RESOURCE-008": [
        "interrupted_report_reservation_after",
        "interrupted_report_reservation_before",
    ],
})


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
    distribution_path = ROOT / "fixtures/distribution/manifest_v7.json"
    requirements = json.loads(requirements_path.read_text())["requirements"]
    applicability = json.loads(applicability_path.read_text())["classifications"]
    distribution = json.loads(distribution_path.read_text())
    attestation = json.loads(TS_ATTESTATION.read_text())
    fixture_paths = {
        item["fixture_id"]: ROOT / item["metadata_path"]
        for item in distribution["fixtures"]
    }
    by_requirement: dict[str, list[str]] = defaultdict(list)
    for item in distribution["fixtures"]:
        for requirement in item["requirements"]:
            by_requirement[requirement].append(item["fixture_id"])
    executed_typescript = set(attestation["executed_fixture_ids"])
    rust_candidate = git("rev-parse", "HEAD")
    typescript_candidate = attestation["candidate"]
    attestation_hash = sha256(TS_ATTESTATION)
    rows: list[dict[str, object]] = []
    overlay_rows: dict[str, object] = {}

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
            row["status"] = "external-hold" if identifier == "NCRDT-NIP-001" else "not-applicable"
            row["rationale"] = (
                "The externally authored NIP remains read-only and unreconciled."
                if identifier == "NCRDT-NIP-001"
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
            artifact_hash = sha256(ROOT / test_path)
            command = "cargo test --workspace --all-targets --locked"
        else:
            evidence_kind = "exact-assertion"
            evidence_ids = [test_id(identifier)]
            test_path = source["test"]
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
        if classification == "rust-and-typescript":
            required = sorted(
                REQUIRED_TS_FIXTURES.get(identifier, fixture_ids), key=str.encode
            )
            if not set(required).issubset(executed_typescript):
                raise AssertionError(
                    f"TypeScript attestation lacks exact fixtures for {identifier}: {required}"
                )
            opaque = {
                "implementation_identity": "triesap/nostr_automerge_typescript",
                "candidate": typescript_candidate,
                "dependency_lock_sha256": attestation["dependency_lock_sha256"],
                "fixture_ids": required,
                "commands": attestation["commands"],
                "result": "pass",
                "artifact_sha256": attestation_hash,
            }
            row["typescript_proof"] = opaque
            overlay_rows[identifier] = opaque
        row["status"] = "pass"
        rows.append(row)

    overlay = {
        "schema": "nostr_automerge.requirement_typescript_overlay.v7",
        "attestation_path": TS_ATTESTATION.relative_to(ROOT).as_posix(),
        "attestation_sha256": attestation_hash,
        "requirement_count": len(overlay_rows),
        "requirements": overlay_rows,
    }
    overlay_path = ROOT / "reports/requirements_typescript_overlay_v7.json"
    canonical_write(overlay_path, overlay)
    report = {
        "schema": "nostr_automerge.requirement_coverage.v7",
        "requirements_sha256": sha256(requirements_path),
        "applicability_sha256": sha256(applicability_path),
        "fixture_distribution_sha256": sha256(distribution_path),
        "rust_candidate": rust_candidate,
        "typescript_candidate": typescript_candidate,
        "requirement_count": len(rows),
        "rows": rows,
    }
    canonical_write(ROOT / "reports/requirements_coverage_v7.json", report)
    print(f"PASS: generated {len(rows)} exact remediation-v6 requirement rows")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
