#!/usr/bin/env python3
"""Validate the active remediation-v12 authority and runtime cursor."""

from __future__ import annotations

import copy
import hashlib
import json
import pathlib
import subprocess


ROOT = pathlib.Path(__file__).resolve().parents[1]
AUTHORITY_PATH = ROOT / "spec/remediation_v12_authority.json"
LEDGER_PATH = ROOT / "implementation/runtime_ledger_v12.json"
FINDINGS_PATH = ROOT / "spec/remediation_findings_v12.json"
REPRODUCTIONS_PATH = ROOT / "spec/remediation_v12_reproductions.json"
EVIDENCE_POLICY_PATH = ROOT / "spec/remediation_v12_evidence_policy.json"
AUTHORITY_GATE_PATH = ROOT / "reports/remediation_v12_authority_gate.json"
PROJECTION_GATE_PATH = ROOT / "reports/trusted_epoch_projection_gate_v12.json"

REVIEWED_CANDIDATE = "9e99af892764ccb165a12b8bb186935bd599d561"
REVIEWED_TREE = "4b684dc123f371ded75c1469505b130c36359f93"
PLAN_CANDIDATE = "d1b9202be6bf9deb643ca7d81f89c5c3281eb523"
PLAN_TREE = "739068407a059b071655cc63bcf1b570285fbaf7"
PLAN_PATH = "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v12.md"
PLAN_SHA256 = "aa8ea9bc6801175dd247dd283521a4ad8f0735eafcd280151a842c37418e5585"
REQUIREMENTS_BASELINE_CANDIDATE = "4a5abe6f0bff2dbe147d9805f4cd3de844874ab6"
REQUIREMENTS_CANDIDATE = "fd9ed9103879d4933832766c0c8dadb57262a49f"
PROJECTION_CANDIDATE = "25b540e176d291c9de823e8106e074a5d4eff48b"
TRAVERSAL_CANDIDATE = "eafa932ff4cfa7a4356827f2b97037a8d35c89f3"
LOOKUP_CANDIDATE = "6e7d13e735017ce310670ca70d56bd4e5225ac61"
PUBLICATION_CANDIDATE = "bb2f600aeb08c74dd7c8556c1bfc14baa4568ce6"
SEMANTIC_MATRIX_CANDIDATE = "41be17ed694f1e9848c47acd99a79f4513dfc2e4"
WORK_CONTRACT_CANDIDATE = "5b3a386160e3310071e644a7030ade80248640d5"
PROJECTION_GATE_CANDIDATE = "187a260858bdd41f9dd967e32279b9f21454ede2"
ACTOR_DECISION_CANDIDATE = "629747ab2537593ddf0a4f689a3e14ea6e576039"
HOLDS = [
    "external_assurance",
    "event_kind_allocation",
    "nip_submission",
    "production_qualification",
    "publication",
    "release",
    "remote_mutation",
]
ACTIVE_SCOPE = [
    "crates/nostr_automerge/src/graph/actor_state.rs",
    "crates/nostr_automerge/src/engine/reference_evaluator.rs",
    "crates/nostr_automerge/src/reference/epoch_engine.rs",
    "docs/execution/remediation_v12/ledger.md",
    "implementation/runtime_ledger_v12.json",
    "reports/spec_baseline.txt",
    "scripts/reproduce_remediation_v12.py",
    "scripts/validate_remediation_v12.py",
    "spec/remediation_v12_reproductions.json",
    "tools/nostr_automerge_conformance/src/fixture_generation.rs",
]

EVIDENCE_REQUIREMENTS = [
    "NCRDT-RESOURCE-017",
    "NCRDT-RESOURCE-018",
    "NCRDT-RESOURCE-019",
    "NCRDT-EVIDENCE-007",
]
OWNER_MODES = ["item_metered", "exact_reserved", "sealed_constant_time"]
ROW_FIELDS = [
    "id", "family", "source_path", "source_symbol", "owner_mode",
    "requirements", "test", "command", "candidate", "artifact_sha256",
    "mutation",
]
APPROVED_ROOTS = [
    "docs/adr", "docs/execution/remediation_v12", "fixtures", "implementation",
    "reports", "scripts", "spec", "tests", "tools/validation",
]
OPAQUE_ALLOWED = [
    "artifact_sha256", "candidate", "counts", "identity_sha256", "result_classes",
]
OPAQUE_PROHIBITED = [
    "commands", "credentials", "logs", "package_layout", "paths", "source", "urls",
]
GATE_CHAIN = [
    ("plan_v12", "d1b9202be6bf9deb643ca7d81f89c5c3281eb523"),
    ("step_1364", "22cb8f0c77637647ce485e4d6f206316113e429a"),
    ("step_1365", "4e6b9e2c189d407b29a478c5445405b922789aa0"),
    ("step_1366", "00fca7681ba079e98ebf8d116bc7fa12926d1a87"),
    ("step_1367", "1de9769b36b5fa610483c3f0ffcd0e7e6ee2768c"),
    ("step_1368", "bb8b8fd4560eaf141ff599ed440edeb68c30a33f"),
    ("step_1369", "4819b9ae58650f8b5decfb19e0f8d895dc47c7d2"),
    ("step_1370", "670e300c4cb029ce8b67468c6009b756ea703002"),
]
GATE_ARTIFACTS = [
    ("spec/remediation_v12_reproductions.json", "d61707db33ffefdf30ee5293bc6a5e994a67ece2a397613afc7e39cdf4def8c1"),
    ("spec/remediation_v12_evidence_policy.json", "159d25262833f4acb062bce1366aa741465c088e03cbb689b69d6fb681ab0492"),
    ("docs/adr/README.md", "5e2fea1a448da14bfcdd177e65f72a7f790bc7692ed6291f17caa74d5f07b7e1"),
    ("scripts/reproduce_remediation_v12.py", "76ccd7244006da4595f152d31f9e59787cc39647e6e2be03ce727fa5ba02c29e"),
]


class EvidenceError(RuntimeError):
    pass


def require_keys(value: object, keys: list[str], label: str) -> dict[str, object]:
    if not isinstance(value, dict) or list(value) != keys:
        raise EvidenceError(f"{label}:shape")
    return value


def require_equal(actual: object, expected: object, label: str) -> None:
    if actual != expected:
        raise EvidenceError(label)


def sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args: str) -> str:
    result = subprocess.run(
        ["git", *args], cwd=ROOT, capture_output=True, text=True, check=False
    )
    if result.returncode:
        raise EvidenceError("git:" + ":".join(args))
    return result.stdout.strip()


def git_file_sha(candidate: str, path: str) -> str:
    result = subprocess.run(
        ["git", "show", f"{candidate}:{path}"],
        cwd=ROOT,
        capture_output=True,
        check=False,
    )
    if result.returncode:
        raise EvidenceError("git:file:" + path)
    return hashlib.sha256(result.stdout).hexdigest()


def validate_authority(authority: object) -> None:
    record = require_keys(
        authority,
        [
            "schema",
            "status",
            "reviewed_public",
            "governing_plan",
            "historical_v11",
            "active_sequence",
            "counts",
            "frozen_sha256",
            "holds",
            "result",
        ],
        "authority",
    )
    require_equal(record["schema"], "nostr_automerge.remediation_v12_authority.v1", "authority:schema")
    require_equal(record["status"], "authority_and_reproduction_correction_required", "authority:status")
    reviewed = require_keys(record["reviewed_public"], ["candidate", "tree"], "authority:reviewed")
    require_equal(reviewed, {"candidate": REVIEWED_CANDIDATE, "tree": REVIEWED_TREE}, "authority:reviewed")
    plan = require_keys(record["governing_plan"], ["candidate", "tree", "path", "sha256"], "authority:plan")
    require_equal(plan, {"candidate": PLAN_CANDIDATE, "tree": PLAN_TREE, "path": PLAN_PATH, "sha256": PLAN_SHA256}, "authority:plan")
    historical = require_keys(record["historical_v11"], ["final_decision_sha256", "runtime_ledger_sha256", "authority_sha256", "status"], "authority:historical")
    require_equal(historical["status"], "immutable_history", "authority:historical_status")
    sequence = require_keys(record["active_sequence"], ["rcld_first", "rcld_last", "step_first", "step_last", "step_count"], "authority:sequence")
    require_equal(sequence, {"rcld_first": 109, "rcld_last": 115, "step_first": "step_1364", "step_last": "step_1419", "step_count": 56}, "authority:sequence")
    counts = require_keys(record["counts"], ["requirements_current", "requirements_target", "scenarios_current", "scenarios_target"], "authority:counts")
    require_equal(counts, {"requirements_current": 152, "requirements_target": 156, "scenarios_current": 198, "scenarios_target": 204}, "authority:counts")
    frozen = require_keys(record["frozen_sha256"], ["nip", "requirements", "report_contract"], "authority:frozen")
    require_equal(frozen, {
        "nip": "8262bf32cb70b7c0e46210441120652e52504fb73839641ac19dddfed840acf8",
        "requirements": "840822a1acf171c887b9a9aba79ddf159ffcd9c5d7a74bd74d7e0bac5c6161f4",
        "report_contract": "636bd1ff32673a00dc0f41440bde61f2b0f8d86f853a7feaaf119de1ff2ce189",
    }, "authority:frozen")
    require_equal(record["holds"], HOLDS, "authority:holds")
    require_equal(record["result"], "pass", "authority:result")


def validate_ledger(ledger: object) -> None:
    record = require_keys(ledger, ["schema", "status", "authority", "cursor", "findings", "requirements", "active_checkpoint_scope", "predecessors"], "ledger")
    require_equal(record["schema"], "nostr_automerge.runtime_ledger.v12.v1", "ledger:schema")
    require_equal(record["status"], "implementation_in_progress", "ledger:status")
    require_equal(record["authority"], "spec/remediation_v12_authority.json", "ledger:authority")
    cursor = require_keys(record["cursor"], ["active_rcld", "active_step", "next_step", "last_planned_step", "remaining_checkpoint_count", "remaining_rcld_count"], "ledger:cursor")
    require_equal(cursor, {"active_rcld": 111, "active_step": "step_1381", "next_step": "step_1382", "last_planned_step": "step_1419", "remaining_checkpoint_count": 38, "remaining_rcld_count": 5}, "ledger:cursor")
    findings = require_keys(record["findings"], ["open", "held"], "ledger:findings")
    require_equal(findings, {"open": ["FINDING_100", "FINDING_101", "FINDING_102", "FINDING_103"], "held": ["FINDING_080"]}, "ledger:findings")
    require_equal(record["requirements"], EVIDENCE_REQUIREMENTS, "ledger:requirements")
    require_equal(record["active_checkpoint_scope"], ACTIVE_SCOPE, "ledger:scope")
    predecessors = record["predecessors"]
    if not isinstance(predecessors, list) or len(predecessors) != 19:
        raise EvidenceError("ledger:predecessors")
    require_equal(predecessors[0], {"step": "step_1363", "candidate": REVIEWED_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_v11")
    require_equal(predecessors[1], {"step": "plan_v12", "candidate": PLAN_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_plan")
    require_equal(predecessors[2], {"step": "step_1364", "candidate": "22cb8f0c77637647ce485e4d6f206316113e429a", "owner_class": "public", "result": "pass"}, "ledger:predecessor_1364")
    require_equal(predecessors[3], {"step": "step_1365", "candidate": "4e6b9e2c189d407b29a478c5445405b922789aa0", "owner_class": "public", "result": "pass"}, "ledger:predecessor_1365")
    require_equal(predecessors[4], {"step": "step_1366", "candidate": "00fca7681ba079e98ebf8d116bc7fa12926d1a87", "owner_class": "public", "result": "pass"}, "ledger:predecessor_1366")
    require_equal(predecessors[5], {"step": "step_1367", "candidate": "1de9769b36b5fa610483c3f0ffcd0e7e6ee2768c", "owner_class": "public", "result": "pass"}, "ledger:predecessor_1367")
    require_equal(predecessors[6], {"step": "step_1368", "candidate": "bb8b8fd4560eaf141ff599ed440edeb68c30a33f", "owner_class": "public", "result": "pass"}, "ledger:predecessor_1368")
    require_equal(predecessors[7], {"step": "step_1369", "candidate": "4819b9ae58650f8b5decfb19e0f8d895dc47c7d2", "owner_class": "public", "result": "pass"}, "ledger:predecessor_1369")
    require_equal(predecessors[8], {"step": "step_1370", "candidate": "670e300c4cb029ce8b67468c6009b756ea703002", "owner_class": "public", "result": "pass"}, "ledger:predecessor_1370")
    require_equal(predecessors[9], {"step": "step_1371", "candidate": REQUIREMENTS_BASELINE_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_1371")
    require_equal(predecessors[10], {"step": "step_1372", "candidate": REQUIREMENTS_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_1372")
    require_equal(predecessors[11], {"step": "step_1373", "candidate": PROJECTION_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_1373")
    require_equal(predecessors[12], {"step": "step_1374", "candidate": TRAVERSAL_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_1374")
    require_equal(predecessors[13], {"step": "step_1375", "candidate": LOOKUP_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_1375")
    require_equal(predecessors[14], {"step": "step_1376", "candidate": PUBLICATION_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_1376")
    require_equal(predecessors[15], {"step": "step_1377", "candidate": SEMANTIC_MATRIX_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_1377")
    require_equal(predecessors[16], {"step": "step_1378", "candidate": WORK_CONTRACT_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_1378")
    require_equal(predecessors[17], {"step": "step_1379", "candidate": PROJECTION_GATE_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_1379")
    require_equal(predecessors[18], {"step": "step_1380", "candidate": ACTOR_DECISION_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_1380")


def validate_trusted_projection() -> None:
    source = (ROOT / "crates/nostr_automerge/src/graph/actor_state.rs").read_text()
    production = source.split("#[cfg(test)]", 1)[0]
    required = [
        "pub(crate) struct TrustedEpochProjection<'a>",
        "pub(crate) struct TrustedEpochView",
        "branch_membership: &'a BTreeMap<ChangeHash, ChangeCandidate>",
        "expected_sequence: u64",
    ]
    if any(token not in production for token in required):
        raise EvidenceError("projection:shape")
    if production.count("causal_next_op: u64") != 2:
        raise EvidenceError("projection:causal_counter_shape")
    if production.count("accepted_closure: &'a BTreeSet<ChangeHash>") < 2:
        raise EvidenceError("projection:closure_shape")
    if production.count("Ok(TrustedEpochProjection {") != 1:
        raise EvidenceError("projection:constructor")
    for prohibited in (
        "pub struct TrustedEpochProjection",
        "pub struct TrustedEpochView",
        "&mut TrustedEpochProjection",
        "pub(crate) actor_states:",
        "pub(crate) dependencies:",
    ):
        if prohibited in production:
            raise EvidenceError("projection:visibility")
    require_equal(
        git("diff", "--name-only", REQUIREMENTS_CANDIDATE, "--", "crates/nostr_automerge/src/lib.rs"),
        "",
        "projection:public_api",
    )


def validate_charged_projection_traversal() -> None:
    source = (ROOT / "crates/nostr_automerge/src/graph/actor_state.rs").read_text()
    production = source.split("#[cfg(test)]", 1)[0]
    required = [
        "trait EpochProjectionSource<'a>",
        "struct CanonicalEpochProjectionSource<'a>",
        "ActorStateError::NoncanonicalInput",
        "let Some(hash) = source.next_member()",
        "if !source.accepted_member(&hash)",
        "let Some(candidate) = source.candidate(&hash)",
        "let Some(dependency) = source.dependency(candidate, index)",
        "if !source.accepted_member(&dependency)",
        "previous >= hash",
        "previous >= dependency",
    ]
    if any(token not in production for token in required):
        raise EvidenceError("projection:charged_traversal")
    body = production.split("fn build_trusted_epoch_projection", 1)[1]
    for prohibited in (".sort(", ".sort_by(", ".sort_unstable(", ".dedup("):
        if prohibited in body:
            raise EvidenceError("projection:repair")
    tests = source.split("#[cfg(test)]", 1)[1]
    for name in (
        "charged_projection_traversal_stops_before_every_source_read",
        "projection_rejects_noncanonical_members_and_dependencies_without_repair",
    ):
        if tests.count(f"fn {name}()") != 1:
            raise EvidenceError("projection:test_inventory")


def validate_metered_projection_lookups() -> None:
    source = (ROOT / "crates/nostr_automerge/src/graph/actor_state.rs").read_text()
    production, tests = source.split("#[cfg(test)]", 1)
    required = [
        "pub(crate) fn candidate_metered<E>(",
        "fn candidate_metered_observed<E>(",
        "ProjectionLookupOperation::BranchMembership",
        "ProjectionLookupOperation::AcceptedMembership",
        "ProjectionLookupOperation::ActorState",
        "ProjectionLookupOperation::DirectDependency",
        "ProjectionLookupOperation::PredecessorCandidate",
        "ProjectionLookupOperation::ActorIdentityComparison",
        "ProjectionLookupOperation::ExpectedSequence",
        "ProjectionLookupOperation::SequenceComparison",
        "ProjectionLookupOperation::ExpectedNextComparison",
    ]
    if any(token not in production for token in required):
        raise EvidenceError("projection:metered_lookup")
    body = production.split("fn candidate_metered_observed", 1)[1].split(
        "pub(crate) fn dependencies", 1
    )[0]
    if body.count("charge(WorkCounter::GraphNode)") != 8:
        raise EvidenceError("projection:node_lookup_charges")
    if body.count("charge(WorkCounter::GraphEdge)") != 1:
        raise EvidenceError("projection:edge_lookup_charges")
    if "pub(crate) fn candidate(" in production:
        raise EvidenceError("projection:unmetered_candidate_lookup")
    if tests.count(
        "fn projection_lookups_and_semantic_comparisons_are_immediately_charged()"
    ) != 1:
        raise EvidenceError("projection:lookup_test_inventory")


def validate_metered_projection_publication() -> None:
    source = (ROOT / "crates/nostr_automerge/src/graph/actor_state.rs").read_text()
    production, tests = source.split("#[cfg(test)]", 1)
    required = [
        "enum ProjectionPublicationOperation",
        "fn build_trusted_epoch_projection_observed<'a, E>(",
        "ProjectionPublicationOperation::CandidateDependency",
        "ProjectionPublicationOperation::DependantBucket",
        "ProjectionPublicationOperation::ReadyCandidate",
        "ProjectionPublicationOperation::RemainingDependencies",
        "ProjectionPublicationOperation::Dependencies",
        "ProjectionPublicationOperation::FrontierHead",
        "ProjectionPublicationOperation::ActorState",
        "ProjectionPublicationOperation::WriterContribution",
        "ProjectionPublicationOperation::CausalCounter",
        "ProjectionPublicationOperation::ReadyDependant",
        "ProjectionPublicationOperation::Projection",
    ]
    if any(token not in production for token in required):
        raise EvidenceError("projection:metered_publication")
    body = production.split("fn build_trusted_epoch_projection_observed", 1)[1].split(
        "#[cfg(test)]", 1
    )[0]
    for prohibited in ("with_capacity(", ".reserve(", ".clone()", ".collect::<"):
        if prohibited in body:
            raise EvidenceError("projection:eager_publication")
    if tests.count(
        "fn projection_allocation_insertion_and_publication_are_charged_before_work()"
    ) != 1:
        raise EvidenceError("projection:publication_test_inventory")


def validate_projection_semantic_matrix() -> None:
    source = (ROOT / "crates/nostr_automerge/src/graph/actor_state.rs").read_text()
    tests = source.split("#[cfg(test)]", 1)[1]
    name = "projection_semantic_matrix_is_complete_and_order_invariant"
    if tests.count(f"fn {name}()") != 1:
        raise EvidenceError("projection:semantic_matrix_inventory")
    body = tests.split(f"fn {name}()", 1)[1].split("\n    #[test]", 1)[0]
    required = [
        '"empty"',
        '"single"',
        '"deep_predecessor"',
        '"unrelated_dependency"',
        '"actor_gap"',
        '"actor_rollback"',
        "last_sequence: u64::MAX",
        ".eq(wide_hashes.iter().copied())",
        "wide_projection.writer_contributions()",
        "accepted.into_iter().rev().collect()",
        "ExpectedProjectionCase",
    ]
    if any(token not in body for token in required):
        raise EvidenceError("projection:semantic_matrix")


def validate_projection_work_contract() -> None:
    source = (ROOT / "crates/nostr_automerge/src/graph/actor_state.rs").read_text()
    tests = source.split("#[cfg(test)]", 1)[1]
    name = "projection_work_contract_preserves_first_stop_and_predecessor_output"
    if tests.count(f"fn {name}()") != 1:
        raise EvidenceError("projection:work_contract_inventory")
    body = tests.split(f"fn {name}()", 1)[1].split("\n    #[test]", 1)[0]
    required = [
        "const TOTAL_CHARGES: usize = 41;",
        "const GRAPH_NODES: usize = 32;",
        "const GRAPH_EDGES: usize = 9;",
        "for successful_limit in 0..TOTAL_CHARGES",
        "Completion::BudgetExhausted, Completion::Cancelled",
        "successful_limit + 1",
        "actor_state_bytes(&metered_states)",
        "actor_state_bytes(&predecessor_states)",
        "core::ptr::eq(error, &injected)",
        "std::panic::panic_any(PANIC_IDENTITY)",
    ]
    if any(token not in body for token in required):
        raise EvidenceError("projection:work_contract")
    inventory = (ROOT / "scripts/validate_resource_operation_inventory_v10.py").read_text()
    for token in (
        "PROJECTION_WORK_CONTRACT_ANCHORS",
        "def validate_projection_work_contract(source: str)",
        "validate_projection_work_contract(sources[PROJECTION_WORK_CONTRACT_PATH])",
    ):
        if token not in inventory:
            raise EvidenceError("projection:operation_inventory")


def validate_projected_actor_sequence_decision() -> None:
    source = (ROOT / "crates/nostr_automerge/src/graph/actor_state.rs").read_text()
    production, tests = source.split("#[cfg(test)]", 1)
    required = [
        "pub(crate) fn actor_sequence_decision_metered<E>(",
        "let view = self.candidate_metered(candidate, charge)?;",
        "if !view.actor_identity_matches()",
        "None if candidate.sequence == 1 => Ok(())",
        "ActorStateError::SequenceRollback",
        "ActorStateError::MissingPredecessor",
    ]
    if any(token not in production for token in required):
        raise EvidenceError("actor_sequence:decision")
    name = "projected_actor_sequence_decision_is_nonmutating_and_complete"
    if tests.count(f"fn {name}()") != 1:
        raise EvidenceError("actor_sequence:test_inventory")
    body = tests.split(f"fn {name}()", 1)[1].split("\n    #[test]", 1)[0]
    for token in (
        "let genesis",
        "let mut deep",
        "let mut unrelated",
        "let mut gap",
        "let mut rollback",
        "let mut duplicate",
        "last_sequence: u64::MAX",
        "const LOOKUP_CHARGES: usize = 9;",
        "Completion::BudgetExhausted, Completion::Cancelled",
        "LOOKUP_CHARGES + 1",
    ):
        if token not in body:
            raise EvidenceError("actor_sequence:matrix")


def validate_projected_actor_sequence_production_path(
    actor_source: str | None = None,
    engine_source: str | None = None,
) -> None:
    actor_source = actor_source or (
        ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
    ).read_text()
    engine_source = engine_source or (
        ROOT / "crates/nostr_automerge/src/reference/epoch_engine.rs"
    ).read_text()
    actor_production = actor_source.split("#[cfg(test)]\npub(crate) mod tests", 1)[0]
    engine_production = engine_source.split("#[cfg(test)]\nmod tests", 1)[0]
    if "fn validate_actor_predecessor(" in actor_production:
        raise EvidenceError("actor_sequence:legacy_scan")
    if "validate_actor_predecessor" in engine_production:
        raise EvidenceError("actor_sequence:legacy_call")
    if engine_production.count(".actor_sequence_decision_metered(&candidate") != 1:
        raise EvidenceError("actor_sequence:production_route")
    route = engine_production.split(
        ".actor_sequence_decision_metered(&candidate", 1
    )[0]
    if "initialize_actor_states_metered(known, &all_candidates" not in route[-600:]:
        raise EvidenceError("actor_sequence:projection_route")
    tests = actor_source.split("#[cfg(test)]\npub(crate) mod tests", 1)[1]
    name = "finding_100_actor_predecessor_scan_reproduction"
    if tests.count(f"fn {name}()") != 1:
        raise EvidenceError("actor_sequence:public_regression")
    prefix = tests.split(f"fn {name}()", 1)[0].rsplit("#[test]", 1)[-1]
    if "#[ignore" in prefix:
        raise EvidenceError("actor_sequence:ignored_regression")


def actor_sequence_source_mutation_self_test() -> int:
    actor_source = (
        ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
    ).read_text()
    engine_source = (
        ROOT / "crates/nostr_automerge/src/reference/epoch_engine.rs"
    ).read_text()
    mutations = (
        (actor_source, engine_source.replace(".actor_sequence_decision_metered(&candidate", ".candidate_metered(&candidate", 1)),
        (actor_source.replace("fn causal_next_op", "fn validate_actor_predecessor() {}\n\nfn causal_next_op", 1), engine_source),
    )
    caught = 0
    for changed_actor, changed_engine in mutations:
        try:
            validate_projected_actor_sequence_production_path(
                changed_actor, changed_engine
            )
        except EvidenceError:
            caught += 1
            continue
        raise EvidenceError("actor_sequence:source_mutation_survived")
    return caught


def validate_reproductions(reproductions: object) -> None:
    record = require_keys(reproductions, ["schema", "cases", "result"], "reproductions")
    require_equal(record["schema"], "nostr_automerge.remediation_v12_reproductions.v1", "reproductions:schema")
    rows = record["cases"]
    if not isinstance(rows, list) or len(rows) != 10:
        raise EvidenceError("reproductions:rows")
    expected = [
        ("actor_predecessor", "crates/nostr_automerge/src/graph/actor_state.rs", "graph::actor_state::tests::finding_100_actor_predecessor_scan_reproduction", "unmetered actor predecessor collection remains"),
        ("causal_next_op", "crates/nostr_automerge/src/graph/actor_state.rs", "graph::actor_state::tests::finding_100_causal_next_op_scan_reproduction", "unmetered causal next-op scan remains"),
        ("empty_frontier", "crates/nostr_automerge/src/graph/actor_state.rs", "graph::actor_state::tests::finding_100_empty_frontier_work_reproduction", "unmetered empty-frontier allocation remains"),
        ("epoch_ancestry", "crates/nostr_automerge/src/graph/epoch.rs", "graph::epoch::tests::finding_100_epoch_ancestry_work_reproduction", "unmetered epoch ancestry materialization remains"),
        ("epoch_writer_authorization", "crates/nostr_automerge/src/reference/epoch_engine.rs", "reference::epoch_engine::tests::finding_100_epoch_writer_authorization_work_reproduction", "unmetered epoch writer authorization scan remains"),
        ("dependency_closure", "crates/nostr_automerge/src/graph/closure.rs", "graph::closure::tests::finding_100_dependency_closure_work_reproduction", "unmetered dependency-closure preparation remains"),
        ("schedule_readiness", "crates/nostr_automerge/src/graph/schedule.rs", "graph::schedule::tests::finding_100_schedule_readiness_work_reproduction", "unmetered schedule readiness and pop preparation remains"),
        ("schedule_publication", "crates/nostr_automerge/src/graph/schedule.rs", "graph::schedule::tests::finding_100_schedule_publication_work_reproduction", "unmetered schedule insertion and result publication remains"),
        ("quarantine_overlays", "crates/nostr_automerge/src/reference/epoch_engine.rs", "reference::epoch_engine::tests::finding_100_quarantine_overlay_work_reproduction", "unmetered selected and fallback quarantine overlays remain"),
        ("zero_post_stop", "crates/nostr_automerge/src/reference/epoch_engine.rs", "reference::epoch_engine::tests::finding_100_zero_post_stop_work_reproduction", "unmetered target preparation remains before the first stop"),
    ]
    for index, (family, path, test, diagnostic) in enumerate(expected):
        row = require_keys(rows[index], ["finding", "family", "kind", "path", "test", "diagnostic", "expected"], f"reproductions:row:{index}")
        require_equal(row, {
            "finding": "FINDING_100",
            "family": family,
            "kind": "rust_failure",
            "path": path,
            "test": test,
            "diagnostic": diagnostic,
            "expected": "fixed_pass" if index == 0 else "open_failure",
        }, f"reproductions:{family}")
    require_equal(record["result"], "pass", "reproductions:result")


def validate_findings(findings: object) -> None:
    record = require_keys(findings, ["schema", "status", "findings", "result"], "findings")
    require_equal(record["schema"], "nostr_automerge.remediation_findings.v12.v1", "findings:schema")
    require_equal(record["status"], "implementation_in_progress", "findings:status")
    rows = record["findings"]
    if not isinstance(rows, list) or len(rows) != 5:
        raise EvidenceError("findings:rows")
    expected_ids = ["FINDING_100", "FINDING_101", "FINDING_102", "FINDING_103", "FINDING_080"]
    require_equal([row.get("id") if isinstance(row, dict) else None for row in rows], expected_ids, "findings:ids")
    for row in rows:
        require_keys(row, ["id", "severity", "class", "title", "requirements", "source_paths", "closure", "status"], "findings:row")
        if not isinstance(row["title"], str) or not row["title"]:
            raise EvidenceError("findings:title")
        if not isinstance(row["closure"], str) or not row["closure"]:
            raise EvidenceError("findings:closure")
        if not isinstance(row["requirements"], list) or not isinstance(row["source_paths"], list):
            raise EvidenceError("findings:vectors")
    require_equal([row["status"] for row in rows], ["open", "open", "open", "open", "held"], "findings:statuses")
    require_equal(rows[-1]["severity"], "hold", "findings:held_severity")
    require_equal(record["result"], "pass", "findings:result")


def validate_evidence_policy(policy: object) -> None:
    record = require_keys(policy, [
        "schema", "status", "authority", "policy", "decisions", "requirements",
        "owner_modes", "required_row_fields", "approved_roots",
        "opaque_allowed_fields", "opaque_prohibited_fields", "holds", "result",
    ], "evidence_policy")
    require_equal(record["schema"], "nostr_automerge.remediation_v12_evidence_policy.v1", "evidence_policy:schema")
    require_equal(record["status"], "approved_staged", "evidence_policy:status")
    require_equal(record["authority"], "spec/remediation_v12_authority.json", "evidence_policy:authority")
    require_equal(require_keys(record["policy"], ["path", "sha256"], "evidence_policy:policy"), {
        "path": "spec/EVIDENCE_POLICY.md",
        "sha256": "43f99e4151b037682f2135d1f80e4e254fcc59d4097fc2032b7a8be519bd51fc",
    }, "evidence_policy:policy")
    decisions = record["decisions"]
    require_equal(decisions, [
        {
            "id": "ADR-0076",
            "path": "docs/adr/adr_0076_authoritative_epoch_semantic_work.md",
            "sha256": "35876ef2f7d8c189d535c104bbd4baa57bd2e94d432f7b04147373f976f3463a",
        },
        {
            "id": "ADR-0077",
            "path": "docs/adr/adr_0077_complete_runtime_operation_inventory.md",
            "sha256": "8f4f4d51c763272e84f1a16d93fe2428461d3658c8377e975f318875881bb6db",
        },
    ], "evidence_policy:decisions")
    for index, decision in enumerate(decisions):
        require_keys(decision, ["id", "path", "sha256"], f"evidence_policy:decision:{index}")
    require_equal(record["requirements"], EVIDENCE_REQUIREMENTS, "evidence_policy:requirements")
    require_equal(record["owner_modes"], OWNER_MODES, "evidence_policy:owner_modes")
    require_equal(record["required_row_fields"], ROW_FIELDS, "evidence_policy:row_fields")
    require_equal(record["approved_roots"], APPROVED_ROOTS, "evidence_policy:roots")
    require_equal(record["opaque_allowed_fields"], OPAQUE_ALLOWED, "evidence_policy:opaque_allowed")
    require_equal(record["opaque_prohibited_fields"], OPAQUE_PROHIBITED, "evidence_policy:opaque_prohibited")
    require_equal(record["holds"], HOLDS, "evidence_policy:holds")
    require_equal(record["result"], "pass", "evidence_policy:result")


def validate_authority_gate(gate: object) -> None:
    record = require_keys(gate, [
        "schema", "status", "rcld", "candidate_chain", "counts", "findings",
        "artifacts", "holds", "result",
    ], "authority_gate")
    require_equal(record["schema"], "nostr_automerge.remediation_v12_authority_gate.v1", "authority_gate:schema")
    require_equal(record["status"], "rcld_109_complete", "authority_gate:status")
    require_equal(record["rcld"], 109, "authority_gate:rcld")
    expected_chain = [{"step": step, "candidate": candidate} for step, candidate in GATE_CHAIN]
    require_equal(record["candidate_chain"], expected_chain, "authority_gate:chain")
    for index, row in enumerate(record["candidate_chain"]):
        require_keys(row, ["step", "candidate"], f"authority_gate:chain:{index}")
        require_equal(git("rev-parse", f"{row['candidate']}^{{commit}}"), row["candidate"], f"authority_gate:candidate:{index}")
    counts = require_keys(record["counts"], [
        "authority_mutations", "adr_mutations", "reproduction_cases",
        "reproduction_mutations", "fixed_reproductions", "open_reproductions",
    ], "authority_gate:counts")
    require_equal(counts, {
        "authority_mutations": 22, "adr_mutations": 15,
        "reproduction_cases": 10, "reproduction_mutations": 15,
        "fixed_reproductions": 0, "open_reproductions": 10,
    }, "authority_gate:counts")
    require_equal(require_keys(record["findings"], ["open", "held"], "authority_gate:findings"), {
        "open": ["FINDING_100", "FINDING_101", "FINDING_102", "FINDING_103"],
        "held": ["FINDING_080"],
    }, "authority_gate:findings")
    expected_artifacts = [{"path": path, "sha256": digest} for path, digest in GATE_ARTIFACTS]
    require_equal(record["artifacts"], expected_artifacts, "authority_gate:artifacts")
    source_candidate = GATE_CHAIN[-1][1]
    for index, row in enumerate(record["artifacts"]):
        require_keys(row, ["path", "sha256"], f"authority_gate:artifact:{index}")
        require_equal(git_file_sha(source_candidate, row["path"]), row["sha256"], f"authority_gate:artifact_hash:{index}")
    require_equal(record["holds"], HOLDS, "authority_gate:holds")
    require_equal(record["result"], "pass", "authority_gate:result")


def validate_files() -> None:
    require_equal(git("rev-parse", f"{REVIEWED_CANDIDATE}^{{tree}}"), REVIEWED_TREE, "git:reviewed_tree")
    require_equal(git("rev-parse", f"{PLAN_CANDIDATE}^{{tree}}"), PLAN_TREE, "git:plan_tree")
    require_equal(sha256(ROOT / PLAN_PATH), PLAN_SHA256, "file:plan")
    require_equal(sha256(ROOT / "spec/NIP_DRAFT.md"), "8262bf32cb70b7c0e46210441120652e52504fb73839641ac19dddfed840acf8", "file:nip")
    require_equal(git_file_sha(REQUIREMENTS_BASELINE_CANDIDATE, "spec/requirements.json"), "840822a1acf171c887b9a9aba79ddf159ffcd9c5d7a74bd74d7e0bac5c6161f4", "file:historical_requirements")
    require_equal(sha256(ROOT / "spec/REPORT_CONTRACT.md"), "636bd1ff32673a00dc0f41440bde61f2b0f8d86f853a7feaaf119de1ff2ce189", "file:report_contract")
    instructions = (ROOT / "AGENTS.md").read_text()
    if "nostr_automerge_v1_multi_rcld_v12.md" not in instructions or "RCLDs 109 through 115" not in instructions:
        raise EvidenceError("file:instructions")
    require_equal(sha256(ROOT / "spec/EVIDENCE_POLICY.md"), "43f99e4151b037682f2135d1f80e4e254fcc59d4097fc2032b7a8be519bd51fc", "file:evidence_policy")
    require_equal(sha256(ROOT / "docs/adr/adr_0076_authoritative_epoch_semantic_work.md"), "35876ef2f7d8c189d535c104bbd4baa57bd2e94d432f7b04147373f976f3463a", "file:adr_0076")
    require_equal(sha256(ROOT / "docs/adr/adr_0077_complete_runtime_operation_inventory.md"), "8f4f4d51c763272e84f1a16d93fe2428461d3658c8377e975f318875881bb6db", "file:adr_0077")
    require_equal(sha256(ROOT / "tools/validation/remediation_v12_evidence_policy.schema.json"), "f4be03d9d38af88277182d951a8e67ff2a34f7090b9c413e66a7a373b50ba669", "file:evidence_policy_schema")
    require_equal(sha256(ROOT / "tools/validation/remediation_v12_authority_gate.schema.json"), "3c7c30205feff1347c9b2bb68ab308c4cc81d4d6e71bc72f010f16c507a6f6bd", "file:authority_gate_schema")
    require_equal(sha256(PROJECTION_GATE_PATH), "64e1c650a806ce6ef79b6e67009621388f797340d9e3ece0b6361ded2d875bff", "file:projection_gate")
    require_equal(sha256(ROOT / "tools/validation/trusted_epoch_projection_gate_v12.schema.json"), "8451ac4a647b8f2f12eca0bdbddf37becc1757deac765bf3558dc0f6cbce4577", "file:projection_gate_schema")


def mutation_self_test(authority: object, ledger: object, findings: object, reproductions: object, evidence_policy: object, authority_gate: object) -> int:
    mutations: list[tuple[str, object, object]] = []
    for label, path, value in (
        ("reviewed", ("reviewed_public", "candidate"), "0" * 40),
        ("plan", ("governing_plan", "sha256"), "0" * 64),
        ("count", ("counts", "scenarios_target"), 205),
        ("hold", ("holds",), HOLDS[:-1]),
    ):
        changed = copy.deepcopy(authority)
        target = changed
        for key in path[:-1]:
            target = target[key]
        target[path[-1]] = value
        mutations.append((label, changed, ledger))
    extra = copy.deepcopy(authority)
    extra["unapproved"] = False
    mutations.append(("authority_extra", extra, ledger))
    reordered = copy.deepcopy(authority)
    reordered["schema"] = reordered.pop("schema")
    mutations.append(("authority_order", reordered, ledger))
    for label, field, value in (
        ("cursor", "next_step", "step_1383"),
        ("scope", "active_checkpoint_scope", ACTIVE_SCOPE[:-1]),
        ("finding", "findings", {"open": ["FINDING_100"], "held": ["FINDING_080"]}),
        ("requirements", "requirements", EVIDENCE_REQUIREMENTS[:-1]),
    ):
        changed = copy.deepcopy(ledger)
        if field == "next_step":
            changed["cursor"][field] = value
        else:
            changed[field] = value
        mutations.append((label, authority, changed))
    for label, changed_authority, changed_ledger in mutations:
        try:
            validate_authority(changed_authority)
            validate_ledger(changed_ledger)
        except EvidenceError:
            continue
        raise EvidenceError("mutation_survived:" + label)
    finding_mutations = []
    missing = copy.deepcopy(findings)
    missing["findings"].pop(1)
    finding_mutations.append(("finding_missing", missing))
    closed = copy.deepcopy(findings)
    closed["findings"][0]["status"] = "closed"
    finding_mutations.append(("finding_closed", closed))
    unheld = copy.deepcopy(findings)
    unheld["findings"][-1]["status"] = "open"
    finding_mutations.append(("finding_unheld", unheld))
    extra_finding_key = copy.deepcopy(findings)
    extra_finding_key["findings"][0]["unapproved"] = False
    finding_mutations.append(("finding_extra", extra_finding_key))
    for label, changed in finding_mutations:
        try:
            validate_findings(changed)
        except EvidenceError:
            continue
        raise EvidenceError("mutation_survived:" + label)
    reproduction_mutations = []
    missing_reproduction = copy.deepcopy(reproductions)
    missing_reproduction["cases"].clear()
    reproduction_mutations.append(("reproduction_missing", missing_reproduction))
    reopened = copy.deepcopy(reproductions)
    reopened["cases"][0]["expected"] = "open_failure"
    reproduction_mutations.append(("reproduction_status", reopened))
    for label, changed in reproduction_mutations:
        try:
            validate_reproductions(changed)
        except EvidenceError:
            continue
        raise EvidenceError("mutation_survived:" + label)
    policy_mutations = []
    for label, path, value in (
        ("policy_root", ("approved_roots",), APPROVED_ROOTS[:-1]),
        ("policy_mode", ("owner_modes",), OWNER_MODES[::-1]),
        ("policy_row", ("required_row_fields",), ROW_FIELDS[:-1]),
        ("policy_hold", ("holds",), HOLDS[:-1]),
        ("policy_hash", ("policy", "sha256"), "0" * 64),
    ):
        changed = copy.deepcopy(evidence_policy)
        target = changed
        for key in path[:-1]:
            target = target[key]
        target[path[-1]] = value
        policy_mutations.append((label, changed))
    extra_policy = copy.deepcopy(evidence_policy)
    extra_policy["unapproved"] = False
    policy_mutations.append(("policy_extra", extra_policy))
    reordered_policy = copy.deepcopy(evidence_policy)
    reordered_policy["schema"] = reordered_policy.pop("schema")
    policy_mutations.append(("policy_order", reordered_policy))
    for label, changed in policy_mutations:
        try:
            validate_evidence_policy(changed)
        except EvidenceError:
            continue
        raise EvidenceError("mutation_survived:" + label)
    gate_mutations = []
    missing_candidate = copy.deepcopy(authority_gate)
    missing_candidate["candidate_chain"].pop()
    gate_mutations.append(("gate_candidate", missing_candidate))
    wrong_count = copy.deepcopy(authority_gate)
    wrong_count["counts"]["open_reproductions"] = 9
    gate_mutations.append(("gate_count", wrong_count))
    closed_finding = copy.deepcopy(authority_gate)
    closed_finding["findings"]["open"].pop()
    gate_mutations.append(("gate_finding", closed_finding))
    wrong_artifact = copy.deepcopy(authority_gate)
    wrong_artifact["artifacts"][0]["sha256"] = "0" * 64
    gate_mutations.append(("gate_artifact", wrong_artifact))
    extra_gate = copy.deepcopy(authority_gate)
    extra_gate["unapproved"] = False
    gate_mutations.append(("gate_extra", extra_gate))
    reordered_gate = copy.deepcopy(authority_gate)
    reordered_gate["schema"] = reordered_gate.pop("schema")
    gate_mutations.append(("gate_order", reordered_gate))
    for label, changed in gate_mutations:
        try:
            validate_authority_gate(changed)
        except EvidenceError:
            continue
        raise EvidenceError("mutation_survived:" + label)
    return len(mutations) + len(finding_mutations) + len(reproduction_mutations) + len(policy_mutations) + len(gate_mutations)


def main() -> None:
    authority = json.loads(AUTHORITY_PATH.read_text())
    ledger = json.loads(LEDGER_PATH.read_text())
    findings = json.loads(FINDINGS_PATH.read_text())
    reproductions = json.loads(REPRODUCTIONS_PATH.read_text())
    evidence_policy = json.loads(EVIDENCE_POLICY_PATH.read_text())
    authority_gate = json.loads(AUTHORITY_GATE_PATH.read_text())
    validate_authority(authority)
    validate_ledger(ledger)
    validate_findings(findings)
    validate_reproductions(reproductions)
    validate_evidence_policy(evidence_policy)
    validate_authority_gate(authority_gate)
    validate_files()
    validate_trusted_projection()
    validate_charged_projection_traversal()
    validate_metered_projection_lookups()
    validate_metered_projection_publication()
    validate_projection_semantic_matrix()
    validate_projection_work_contract()
    validate_projected_actor_sequence_decision()
    validate_projected_actor_sequence_production_path()
    source_mutations = actor_sequence_source_mutation_self_test()
    mutation_count = mutation_self_test(authority, ledger, findings, reproductions, evidence_policy, authority_gate)
    print("PASS: remediation v12 authority")
    print(f"- mutations={mutation_count}")
    print(f"- source_mutations={source_mutations}")
    print("- active=RCLD111/step_1381")


if __name__ == "__main__":
    main()
