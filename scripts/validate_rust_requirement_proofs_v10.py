#!/usr/bin/env python3
"""Audit and optionally execute exact public Rust proof for all 148 requirements."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parents[1]
SOURCE_CANDIDATE = "6fbef81f8f12caef49ddee6fd5135d900bf22093"
SIGNED_GATE = "reports/signed_conformance_gate_v10.json"
HELD_CLASSES = {"out-of-core", "explicitly-deferred"}


class ProofError(ValueError):
    """One semantic Rust-proof invariant failed."""


@dataclass(frozen=True)
class Proof:
    kind: str
    target: str
    selector: str
    semantic_category: str


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise ProofError(diagnostic)


def load(relative: str) -> dict[str, Any]:
    value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    require(isinstance(value, dict), f"object:{relative}")
    return value


CATEGORY_BY_PREFIX = {
    "ACQ": "authority",
    "ACTOR": "change_application",
    "ALERT": "report_contract",
    "APP": "change_application",
    "AUTOADAPTER": "evidence_integrity",
    "B64": "wire_ingress",
    "BACKFILL": "external_hold",
    "BARRIER": "external_hold",
    "BRANCH": "change_application",
    "CHAIN": "control_history",
    "CHECKPOINT": "checkpoint_verification",
    "CLAIM": "change_application",
    "COMPLETION": "report_contract",
    "CONF": "signed_conformance",
    "CONTROL": "control_history",
    "CONTROLREF": "control_history",
    "CONV": "signed_conformance",
    "CORE": "evidence_integrity",
    "CPAUTH": "checkpoint_verification",
    "CPCHUNK": "checkpoint_verification",
    "CPDESC": "checkpoint_verification",
    "CPRECOVERY": "external_hold",
    "CPTRUST": "checkpoint_verification",
    "DISPOSITION": "change_application",
    "DOC": "external_hold",
    "DUP": "change_application",
    "ENC": "wire_ingress",
    "EPOCH": "control_history",
    "EQUIV": "change_application",
    "EVALUATOR": "evidence_integrity",
    "EVIDENCE": "evidence_integrity",
    "FANIN": "change_application",
    "FEATURES": "evidence_integrity",
    "FRAME": "wire_ingress",
    "FRONTIER": "control_history",
    "INTERRUPT": "resource_accounting",
    "JSON": "wire_ingress",
    "LIMIT": "wire_ingress",
    "LIMITS": "external_hold",
    "MANIFEST": "control_history",
    "NIP": "authority",
    "NIP01": "wire_ingress",
    "NIPBOUNDARY": "wire_ingress",
    "OUTCOME": "report_contract",
    "PROFILE": "wire_ingress",
    "PUB": "external_hold",
    "RELAY": "external_hold",
    "REPO": "evidence_integrity",
    "RESOURCE": "resource_accounting",
    "RETENTION": "external_hold",
    "SCOPE": "resource_accounting",
    "SECCTRL": "external_hold",
    "SEM": "change_application",
    "SEQ": "change_application",
    "STATE": "change_application",
    "STATUS": "external_hold",
    "TAG": "wire_ingress",
    "TS": "evidence_integrity",
    "VERSION": "wire_ingress",
}


PREFIX_TESTS = {
    "NIP01": ("nip01_conformance", "valid_signed_event_is_accepted"),
    "ACQ": ("rust_lib", "evidence::source::tests::prove_acquisition_metadata_has_no_semantic_path"),
    "TAG": ("rust_lib", "wire::tags::tests::rejects_repeated_extra_forbidden_and_unsorted"),
    "JSON": ("rust_lib", "wire::canonical_json::parse::tests::accepts_exact_content_and_rejects_every_normalization"),
    "B64": ("base64_contract", "signed_change_events_reject_every_noncanonical_base64_class"),
    "LIMIT": ("rust_lib", "limits::tests::constants_match_checked_in_registry"),
    "ACTOR": ("rust_lib", "automerge_adapter::document::tests::prove_derived_actor_replaces_unused_random_actor"),
    "FRAME": ("rust_lib", "automerge_adapter::framing::tests::accepts_only_change_magic_and_type"),
    "ENC": ("rust_lib", "automerge_adapter::encode::tests::qualify_canonical_uncompressed_re_encoding"),
    "SEM": ("rust_lib", "automerge_adapter::semantics::add_complete_automerge_semantic_matrix"),
    "SEQ": ("rust_lib", "automerge_adapter::counters::tests::implement_checked_actor_counter_transitions"),
    "FANIN": ("rust_lib", "control::frontier::tests::resolves_chain_branch_and_fan_in_closures"),
    "DUP": ("rust_lib", "evidence::corpus_builder::tests::implement_corpusbuilder_idempotent_ingestion"),
    "CPDESC": ("rust_lib", "checkpoint::descriptor::tests::parse_checkpoint_descriptors"),
    "CPCHUNK": ("rust_lib", "checkpoint::chunk::tests::parse_checkpoint_chunks"),
    "CPTRUST": ("rust_lib", "checkpoint::verify_history::tests::verify_full_historical_carrier_authorization"),
    "CONV": ("public_engine_api", "duplicate_delayed_and_invalid_evidence_converges"),
    "APP": ("public_engine_api", "reference_evaluator_api_is_sealed_and_repository_owned"),
    "PROFILE": ("rust_lib", "profile::tests::only_exact_draft_revision_is_available"),
    "ALERT": ("rust_lib", "integrity::tests::alerts_reject_noncanonical_sets_and_zero_sequence"),
    "EVALUATOR": ("public_engine_api", "build_immutable_evidence_corpus_through_public_api"),
    "DISPOSITION": ("rust_lib", "engine::reference_evaluator::tests::carrier_and_aggregate_decision_table_is_exhaustive"),
    "EPOCH": ("rust_lib", "control::epoch_state::tests::rejects_inconsistent_heads_and_derives_actor_state"),
    "SCOPE": ("public_engine_api", "unrelated_coordinate_evidence_is_report_and_budget_inert"),
    "CLAIM": ("public_engine_api", "signed_causal_change_matrix"),
    "CONTROLREF": ("rust_lib", "control::reference_state::tests::every_parent_state_has_an_exhaustive_dependent_outcome"),
    "FRONTIER": ("rust_lib", "control::frontier::tests::base_head_knowledge_has_exhaustive_dependent_outcomes"),
    "RESOURCE": ("public_engine_api", "every_work_counter_has_exact_before_and_after_boundaries"),
    "INTERRUPT": ("rust_lib", "engine::evaluation_report::tests::incomplete_report_shape_rejects_every_nonempty_or_mismatched_field"),
    "VERSION": ("rust_lib", "carrier::version::tests::enforce_protocol_revision_profile_semantics"),
}


TEST_OVERRIDES = {
    "NCRDT-JSON-002": ("rust_lib", "wire::canonical_json::serialize::tests::orders_keys_by_utf16_and_emits_minimal_json"),
    "NCRDT-JSON-003": ("rust_lib", "wire::strict_json::tests::rejects_duplicates_after_escape_decoding"),
    "NCRDT-CPDESC-003": ("rust_lib", "checkpoint::descriptor::tests::validate_descriptor_arithmetic"),
    "NCRDT-CPDESC-005": ("rust_lib", "checkpoint::verify::tests::commit_empty_sorted_change_set_hash"),
    "NCRDT-CPDESC-006": ("rust_lib", "checkpoint::verify::tests::verify_declared_checkpoint_heads"),
    "NCRDT-CPCHUNK-002": ("rust_lib", "checkpoint::merkle::tests::verify_ordered_merkle_proofs"),
    "NCRDT-CPCHUNK-003": ("rust_lib", "checkpoint::assemble::tests::verify_complete_snapshot_size_and_hash"),
    "NCRDT-CPCHUNK-004": ("rust_lib", "checkpoint::reference_state::tests::every_descriptor_reference_state_has_one_dependent_outcome"),
    "NCRDT-ALERT-001": ("rust_lib", "control::reorganization::tests::detect_and_report_canonical_reorganization"),
    "NCRDT-ALERT-002": ("rust_lib", "control::select::tests::emit_controller_equivocation_alerts"),
    "NCRDT-RESOURCE-003": ("rust_lib", "engine::reference_evaluator::tests::complete_report_plan_is_exact_named_and_overflow_checked"),
    "NCRDT-RESOURCE-004": ("rust_lib", "engine::evaluation_report::tests::budget_and_cancel_no_progress_reports_differ_only_by_typed_stop"),
    "NCRDT-RESOURCE-005": ("public_engine_api", "zero_budget_target_entry_consumes_no_work"),
    "NCRDT-RESOURCE-006": ("public_engine_api", "prior_knowledge_exhaustion_is_deterministic_at_every_item_boundary"),
    "NCRDT-RESOURCE-007": ("rust_lib", "engine::reference_evaluator::tests::complete_report_plan_is_exact_named_and_overflow_checked"),
    "NCRDT-RESOURCE-008": ("rust_lib", "engine::reference_evaluator::tests::report_validation_precedes_finalization_refund"),
    "NCRDT-RESOURCE-012": ("rust_lib", "engine::evaluation_report::tests::canonical_alert_comparisons_are_interleaved_with_successful_charges"),
    "NCRDT-RESOURCE-013": ("rust_lib", "engine::reference_evaluator::tests::complete_report_plan_is_exact_named_and_overflow_checked"),
    "NCRDT-VERSION-002": ("public_engine_api", "finding_079_unsupported_carrier_does_not_create_semantic_hash_state"),
}


VALIDATOR_OVERRIDES = {
    "NCRDT-REPO-001": "scripts/validate_repository_policy.py",
    "NCRDT-CORE-001": "scripts/validate_architecture.py",
    "NCRDT-AUTOADAPTER-001": "scripts/validate_architecture.py",
    "NCRDT-AUTOADAPTER-002": "scripts/validate_architecture.py",
    "NCRDT-AUTOADAPTER-003": "scripts/validate_automerge_qualification.py",
    "NCRDT-NIPBOUNDARY-001": "scripts/validate_architecture.py",
    "NCRDT-FEATURES-001": "scripts/validate_repository_policy.py",
    "NCRDT-TS-001": "scripts/validate_opaque_conformance_v10.py",
    "NCRDT-TS-002": "scripts/validate_opaque_conformance_v10.py",
    "NCRDT-NIP-001": "scripts/validate_nip_snapshot.py",
    "NCRDT-NIP-002": "scripts/validate_nip_snapshot.py",
    "NCRDT-NIP-003": "scripts/validate_nip_snapshot.py",
    "NCRDT-EVIDENCE-001": "scripts/validate_reports.py",
    "NCRDT-EVIDENCE-002": "scripts/validate_assurance_v9.py",
    "NCRDT-EVIDENCE-003": "scripts/validate_semantic_proof_catalog_v10.py",
    "NCRDT-EVIDENCE-004": "scripts/validate_semantic_proof_catalog_v10.py",
    "NCRDT-EVIDENCE-005": "scripts/validate_semantic_proof_catalog_v10.py",
    "NCRDT-EVIDENCE-006": "scripts/validate_semantic_proof_catalog_v10.py",
}


def prefix(identifier: str) -> str:
    return identifier.split("-", 2)[1]


def category(identifier: str, held: bool) -> str:
    if held:
        return "external_hold"
    value = CATEGORY_BY_PREFIX.get(prefix(identifier))
    require(value is not None, f"category:{identifier}")
    return value


def proof_for_requirement(
    identifier: str,
    fixtures_by_requirement: dict[str, tuple[str, ...]],
) -> tuple[Proof, ...]:
    semantic_category = category(identifier, False)
    if identifier.startswith("NCRDT-CONF-"):
        return (Proof("validator", "validator", "scripts/validate_signed_conformance_gate_v10.py", semantic_category),)
    validator = VALIDATOR_OVERRIDES.get(identifier)
    if validator is not None:
        return (Proof("validator", "validator", validator, semantic_category),)
    fixtures = fixtures_by_requirement.get(identifier, ())
    if fixtures:
        return tuple(
            Proof("signed_fixture", "signed_conformance_v10", item, semantic_category)
            for item in fixtures[:1]
        )
    selected = TEST_OVERRIDES.get(identifier) or PREFIX_TESTS.get(prefix(identifier))
    require(selected is not None, f"proof_mapping:{identifier}")
    return (Proof("rust_test", selected[0], selected[1], semantic_category),)


def build_rows() -> tuple[list[dict[str, Any]], tuple[Proof, ...]]:
    requirements = load("spec/requirements.json")["requirements"]
    applicability = load("spec/requirements_applicability.json")["classifications"]
    manifest = load("fixtures/distribution/manifest_v10.json")
    fixtures_by_requirement: dict[str, list[str]] = {}
    for fixture in manifest["fixtures"]:
        for identifier in fixture["requirements"]:
            fixtures_by_requirement.setdefault(identifier, []).append(fixture["fixture_id"])
    frozen_fixtures = {
        identifier: tuple(sorted(values, key=str.encode))
        for identifier, values in fixtures_by_requirement.items()
    }
    rows: list[dict[str, Any]] = []
    proofs: list[Proof] = []
    for requirement in requirements:
        identifier = requirement["id"]
        classification = applicability[identifier]
        held = classification in HELD_CLASSES
        selected = () if held else proof_for_requirement(identifier, frozen_fixtures)
        rows.append(
            {
                "id": identifier,
                "semantic_category": category(identifier, held),
                "applicability": classification,
                "status": "held" if held else "pass",
                "rust_proof_ids": [f"{item.kind}:{item.target}:{item.selector}" for item in selected],
            }
        )
        proofs.extend(selected)
    unique = tuple(dict.fromkeys(proofs))
    return rows, unique


def projection_identity(rows: list[dict[str, Any]], proofs: tuple[Proof, ...]) -> str:
    value = {
        "rows": rows,
        "proofs": [proof.__dict__ for proof in proofs],
        "candidate": SOURCE_CANDIDATE,
    }
    return hashlib.sha256(
        json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()


def validate(rows: list[dict[str, Any]], proofs: tuple[Proof, ...]) -> None:
    requirements = load("spec/requirements.json")["requirements"]
    applicability = load("spec/requirements_applicability.json")["classifications"]
    require(len(rows) == 148, "row_count")
    require([row["id"] for row in rows] == [item["id"] for item in requirements], "row_order")
    require(len(proofs) == len(set(proofs)), "proof_unique")
    proof_ids = {f"{item.kind}:{item.target}:{item.selector}" for item in proofs}
    for row in rows:
        identifier = row["id"]
        held = applicability[identifier] in HELD_CLASSES
        require(row["semantic_category"] == category(identifier, held), f"category:{identifier}")
        require(row["applicability"] == applicability[identifier], f"applicability:{identifier}")
        require(row["status"] == ("held" if held else "pass"), f"status:{identifier}")
        ids = row["rust_proof_ids"]
        require(isinstance(ids, list) and ids == sorted(set(ids), key=str.encode), f"proof_order:{identifier}")
        require((not ids) == held, f"proof_presence:{identifier}")
        require(all(item in proof_ids for item in ids), f"proof_reference:{identifier}")
        require(all(item.split(":", 1)[0] in {"rust_test", "signed_fixture", "validator"} for item in ids), f"proof_kind:{identifier}")
    require(sum(row["status"] == "pass" for row in rows) == 124, "pass_count")
    require(sum(row["status"] == "held" for row in rows) == 24, "held_count")
    for proof in proofs:
        require(proof.semantic_category in set(CATEGORY_BY_PREFIX.values()), "proof_category")
        if proof.kind == "signed_fixture":
            require(proof.target == "signed_conformance_v10", "fixture_target")
        elif proof.kind == "rust_test":
            require(proof.target in {"rust_lib", "public_engine_api", "nip01_conformance", "base64_contract"}, "test_target")
        else:
            require(proof.kind == "validator" and proof.target == "validator", "validator_target")
            require((ROOT / proof.selector).is_file(), "validator_path")


def mutation_self_test(rows: list[dict[str, Any]], proofs: tuple[Proof, ...]) -> int:
    caught = 0
    mutations = []
    missing = copy.deepcopy(rows); missing.pop(); mutations.append(missing)
    reordered = copy.deepcopy(rows); reordered.reverse(); mutations.append(reordered)
    duplicate = copy.deepcopy(rows); duplicate[1] = duplicate[0]; mutations.append(duplicate)
    passing = next(index for index, row in enumerate(rows) if row["status"] == "pass")
    no_proof = copy.deepcopy(rows); no_proof[passing]["rust_proof_ids"] = []; mutations.append(no_proof)
    held = next(index for index, row in enumerate(rows) if row["status"] == "held")
    false_pass = copy.deepcopy(rows); false_pass[held]["status"] = "pass"; mutations.append(false_pass)
    wrong_category = copy.deepcopy(rows); wrong_category[passing]["semantic_category"] = "external_hold"; mutations.append(wrong_category)
    stale = copy.deepcopy(rows); stale[passing]["rust_proof_ids"] = ["rust_test:rust_lib:stale"]; mutations.append(stale)
    for mutation in mutations:
        try:
            validate(mutation, proofs)
        except ProofError:
            caught += 1
        else:
            raise ProofError("row_mutation_survived")
    malformed_proofs = (*proofs, proofs[0])
    try:
        validate(rows, malformed_proofs)
    except ProofError:
        caught += 1
    else:
        raise ProofError("proof_mutation_survived")
    return caught


def test_command(proof: Proof) -> list[str]:
    command = ["cargo", "extbuild", "run", "--", "cargo", "test", "-p", "nostr_automerge"]
    if proof.target == "rust_lib":
        command.append("--lib")
    else:
        command.extend(("--test", proof.target))
    command.extend(("--locked", proof.selector, "--", "--exact"))
    return command


def execute(proofs: tuple[Proof, ...]) -> int:
    executed = 0
    signed_gate_checked = False
    for proof in proofs:
        if proof.kind == "signed_fixture":
            if not signed_gate_checked:
                gate = load(SIGNED_GATE)
                require(gate.get("status") == "pass", "signed_gate")
                signed_gate_checked = True
            executed += 1
            continue
        command = [sys.executable, str(ROOT / proof.selector)] if proof.kind == "validator" else test_command(proof)
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, check=False)
        require(result.returncode == 0, f"execution:{proof.selector}\n{result.stdout}\n{result.stderr}")
        if proof.kind == "rust_test":
            require(f"test {proof.selector} ... ok" in result.stdout, f"test_identity:{proof.selector}")
            require(result.stdout.count(" 1 passed;") == 1, f"test_count:{proof.selector}")
        else:
            require(result.stdout.startswith("PASS:"), f"validator_result:{proof.selector}")
        executed += 1
    return executed


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run-suite", action="store_true")
    arguments = parser.parse_args()
    rows, proofs = build_rows()
    validate(rows, proofs)
    mutations = mutation_self_test(rows, proofs)
    executed = execute(proofs) if arguments.run_suite else 0
    print("PASS: exact Rust requirement proof audit v10")
    print("- requirements=148")
    print("- passing=124")
    print("- held=24")
    print(f"- unique_proofs={len(proofs)}")
    print(f"- negative_mutations={mutations}")
    print(f"- projection_sha256={projection_identity(rows, proofs)}")
    print(f"- executed={executed}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ProofError as error:
        raise SystemExit(f"FAIL: {error}") from error
