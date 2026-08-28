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
ACTOR_GATE_PATH = ROOT / "reports/remediation_v12_actor_gate.json"

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
ACTOR_ROUTE_CANDIDATE = "3ced1446a04a422f5760d00d9786dd18ace76930"
ACTOR_SIGNED_CANDIDATE = "63661f2f1df8394d7526ec2f5d2fa2be60e65efe"
CAUSAL_DECISION_CANDIDATE = "287b0967751f7575faf3a8ee38c15c65aa428290"
CAUSAL_ROUTE_CANDIDATE = "5377b9d276b4deabd0fd3c6f6dac1734d213e74d"
FRONTIER_CANDIDATE = "502c5701d73c4452906ef92f0a324908b9d039a8"
COMBINED_CANDIDATE = "6a190cefa11f84e069ece47644b66992dc82e8f3"
ACTOR_GATE_CANDIDATE = "e98ec40d582c5e8d5e54b856681b260dce716183"
COMPACT_ANCESTRY_CANDIDATE = "43884403ee71c5a0b6fbf7a9b91b4617dd53b43c"
METERED_ANCESTRY_CANDIDATE = "89009e315cfe8596f3a639a0af9e359a7c0a40d7"
ANCESTRY_ROUTE_CANDIDATE = "4f4d43c3aca9d4d959edb2464039d50a983e70a0"
AUTHORIZATION_HELPER_CANDIDATE = "d3b1d462ee4691741821067fb51d33d6d8eb24d6"
AUTHORIZATION_ROUTE_CANDIDATE = "b7a72c9c0be884fa821cd4224fe523fa02e03426"
DEPENDENCY_CLOSURE_CANDIDATE = "6de8d68c83996009962b315306ada3c339f12844"
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
    "crates/nostr_automerge/src/engine/reference_evaluator.rs",
    "crates/nostr_automerge/src/graph/scaling.rs",
    "crates/nostr_automerge/src/graph/schedule.rs",
    "crates/nostr_automerge/src/reference/epoch_engine.rs",
    "crates/nostr_automerge/src/reference/evaluate.rs",
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
    require_equal(cursor, {"active_rcld": 112, "active_step": "step_1394", "next_step": "step_1395", "last_planned_step": "step_1419", "remaining_checkpoint_count": 25, "remaining_rcld_count": 4}, "ledger:cursor")
    findings = require_keys(record["findings"], ["open", "held"], "ledger:findings")
    require_equal(findings, {"open": ["FINDING_100", "FINDING_101", "FINDING_102", "FINDING_103"], "held": ["FINDING_080"]}, "ledger:findings")
    require_equal(record["requirements"], EVIDENCE_REQUIREMENTS, "ledger:requirements")
    require_equal(record["active_checkpoint_scope"], ACTIVE_SCOPE, "ledger:scope")
    predecessors = record["predecessors"]
    if not isinstance(predecessors, list) or len(predecessors) != 32:
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
    require_equal(predecessors[19], {"step": "step_1381", "candidate": ACTOR_ROUTE_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_1381")
    require_equal(predecessors[20], {"step": "step_1382", "candidate": ACTOR_SIGNED_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_1382")
    require_equal(predecessors[21], {"step": "step_1383", "candidate": CAUSAL_DECISION_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_1383")
    require_equal(predecessors[22], {"step": "step_1384", "candidate": CAUSAL_ROUTE_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_1384")
    require_equal(predecessors[23], {"step": "step_1385", "candidate": FRONTIER_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_1385")
    require_equal(predecessors[24], {"step": "step_1386", "candidate": COMBINED_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_1386")
    require_equal(predecessors[25], {"step": "step_1387", "candidate": ACTOR_GATE_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_1387")
    require_equal(predecessors[26], {"step": "step_1388", "candidate": COMPACT_ANCESTRY_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_1388")
    require_equal(predecessors[27], {"step": "step_1389", "candidate": METERED_ANCESTRY_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_1389")
    require_equal(predecessors[28], {"step": "step_1390", "candidate": ANCESTRY_ROUTE_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_1390")
    require_equal(predecessors[29], {"step": "step_1391", "candidate": AUTHORIZATION_HELPER_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_1391")
    require_equal(predecessors[30], {"step": "step_1392", "candidate": AUTHORIZATION_ROUTE_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_1392")
    require_equal(predecessors[31], {"step": "step_1393", "candidate": DEPENDENCY_CLOSURE_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_1393")


def validate_trusted_projection() -> None:
    source = (ROOT / "crates/nostr_automerge/src/graph/actor_state.rs").read_text()
    production = source.split("#[cfg(test)]\npub(crate) mod tests", 1)[0]
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
    production = source.split("#[cfg(test)]\npub(crate) mod tests", 1)[0]
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
    tests = source.split("#[cfg(test)]\npub(crate) mod tests", 1)[1]
    for name in (
        "charged_projection_traversal_stops_before_every_source_read",
        "projection_rejects_noncanonical_members_and_dependencies_without_repair",
    ):
        if tests.count(f"fn {name}()") != 1:
            raise EvidenceError("projection:test_inventory")


def validate_metered_projection_lookups() -> None:
    source = (ROOT / "crates/nostr_automerge/src/graph/actor_state.rs").read_text()
    production, tests = source.split("#[cfg(test)]\npub(crate) mod tests", 1)
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
    production, tests = source.split("#[cfg(test)]\npub(crate) mod tests", 1)
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
    tests = source.split("#[cfg(test)]\npub(crate) mod tests", 1)[1]
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
    tests = source.split("#[cfg(test)]\npub(crate) mod tests", 1)[1]
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
    production, tests = source.split("#[cfg(test)]\npub(crate) mod tests", 1)
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
    if actor_production.count(
        "self.actor_sequence_decision_metered(candidate, &mut charge)?;"
    ) != 1:
        raise EvidenceError("actor_sequence:combined_decision")
    if engine_production.count(".actor_sequence_decision_metered(") != 0:
        raise EvidenceError("actor_sequence:production_route")
    if engine_production.count(".candidate_semantics_decision_metered(") != 1:
        raise EvidenceError("actor_sequence:combined_route")
    route = engine_production.split(
        ".candidate_semantics_decision_metered(", 1
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
        (actor_source.replace("self.actor_sequence_decision_metered(candidate, &mut charge)?;", "self.candidate_metered(candidate, &mut charge)?;", 1), engine_source),
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


def validate_signed_transitive_actor_constructions() -> None:
    source = (
        ROOT / "crates/nostr_automerge/tests/public_engine_api.rs"
    ).read_text()
    name = (
        "signed_transitive_actor_predecessor_is_order_independent_"
        "across_chain_and_fork"
    )
    if source.count(f"fn {name}()") != 1:
        raise EvidenceError("actor_sequence:signed_inventory")
    body = source.split(f"fn {name}()", 1)[1].split("\n#[test]", 1)[0]
    required = [
        "evaluate_case(0xd1, false)",
        "evaluate_case(0xd2, true)",
        "[0_usize, 1, 2, 3, 4]",
        "[4, 3, 2, 1, 0]",
        "[2, 0, 4, 1, 3]",
        "event_disposition(&report, event_id)",
        ".accepted_changes()",
        "report.heads(), [returning_hash]",
        "reports.windows(2).all",
    ]
    if any(token not in body for token in required):
        raise EvidenceError("actor_sequence:signed_matrix")
    prefix = source.split(f"fn {name}()", 1)[0].rsplit("#[test]", 1)[-1]
    if "#[ignore" in prefix:
        raise EvidenceError("actor_sequence:signed_ignored")


def validate_projected_causal_next_decision(source: str | None = None) -> None:
    source = source or (
        ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
    ).read_text()
    production, tests = source.split(
        "#[cfg(test)]\npub(crate) mod tests", 1
    )
    if production.count("pub(crate) fn causal_next_decision_metered<E>(") != 1:
        raise EvidenceError("causal_next:decision_inventory")
    method = production.split(
        "pub(crate) fn causal_next_decision_metered<E>(", 1
    )[1].split("pub(crate) fn candidate_metered<E>(", 1)[0]
    required = [
        "let causal_next_op = self.causal_next_op;",
        "candidate.start_op == causal_next_op",
        "causal_next_op.checked_add(candidate.operation_count)",
        "ActorStateError::OperationCounter",
    ]
    if any(token not in method for token in required):
        raise EvidenceError("causal_next:decision_shape")
    if method.count("charge(WorkCounter::GraphNode)") != 3:
        raise EvidenceError("causal_next:charge_count")
    if "actor_states" in method or ".values()" in method or "causal_next_op(" in method:
        raise EvidenceError("causal_next:state_scan")
    name = (
        "projected_causal_next_decision_is_checked_constant_size_"
        "and_exactly_metered"
    )
    if tests.count(f"fn {name}()") != 1:
        raise EvidenceError("causal_next:test_inventory")
    prefix, body = tests.split(f"fn {name}()", 1)
    if "#[ignore" in prefix.rsplit("#[test]", 1)[-1]:
        raise EvidenceError("causal_next:test_ignored")
    body = body.split("\n    #[test]", 1)[0]
    test_requirements = [
        "1_u8..=64",
        "candidate(1, 1, 1, 0)",
        "gap.start_op = 66",
        "duplicate.start_op = 64",
        "u64::MAX",
        "overflow.operation_count = 1",
        "Completion::BudgetExhausted",
        "Completion::Cancelled",
        "DECISION_CHARGES",
        "assert_eq!(legacy_states[&next.actor].next_op, 66)",
    ]
    if any(token not in body for token in test_requirements):
        raise EvidenceError("causal_next:test_matrix")


def causal_next_source_mutation_self_test() -> int:
    source = (
        ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
    ).read_text()
    mutations = (
        source.replace(
            "causal_next_op.checked_add(candidate.operation_count)",
            "causal_next_op.saturating_add(candidate.operation_count)",
            1,
        ),
        source.replace(
            "let causal_next_op = self.causal_next_op;",
            "let causal_next_op = self.actor_states.values().map(|state| state.next_op).max().unwrap_or(1);",
            1,
        ),
    )
    caught = 0
    for changed in mutations:
        try:
            validate_projected_causal_next_decision(changed)
        except EvidenceError:
            caught += 1
            continue
        raise EvidenceError("causal_next:source_mutation_survived")
    return caught


def validate_projected_causal_next_production_path(
    actor_source: str | None = None,
    engine_source: str | None = None,
) -> None:
    actor_source = actor_source or (
        ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
    ).read_text()
    engine_source = engine_source or (
        ROOT / "crates/nostr_automerge/src/reference/epoch_engine.rs"
    ).read_text()
    actor_production = actor_source.split(
        "#[cfg(test)]\npub(crate) mod tests", 1
    )[0]
    engine_production = engine_source.split("#[cfg(test)]\nmod tests", 1)[0]
    forbidden = [
        "pub(crate) fn apply_empty_counter",
        "pub(crate) fn apply_nonempty_counter",
        "fn causal_next_op(states:",
        "legacy_counter_is_valid",
    ]
    if any(token in actor_production for token in forbidden):
        raise EvidenceError("causal_next:legacy_production")
    if actor_production.count(
        "self.causal_next_decision_metered(candidate, &mut charge)?;"
    ) != 1:
        raise EvidenceError("causal_next:combined_decision")
    if engine_production.count(".causal_next_decision_metered(") != 0:
        raise EvidenceError("causal_next:production_route")
    if engine_production.count(".candidate_semantics_decision_metered(") != 1:
        raise EvidenceError("causal_next:combined_route")
    if "legacy_counter_is_valid" in engine_production:
        raise EvidenceError("causal_next:legacy_call")
    route = engine_production.split(
        ".candidate_semantics_decision_metered(", 1
    )[0]
    if "initialize_actor_states_metered(known, &all_candidates" not in route[-1000:]:
        raise EvidenceError("causal_next:projection_route")
    if engine_production.count(".empty_frontier_decision_metered(") != 0:
        raise EvidenceError("causal_next:frontier_isolation")
    tests = actor_source.split("#[cfg(test)]\npub(crate) mod tests", 1)[1]
    name = "finding_100_causal_next_op_scan_reproduction"
    if tests.count(f"fn {name}()") != 1:
        raise EvidenceError("causal_next:public_regression")
    prefix = tests.split(f"fn {name}()", 1)[0].rsplit("#[test]", 1)[-1]
    if "#[ignore" in prefix:
        raise EvidenceError("causal_next:ignored_regression")


def causal_next_route_source_mutation_self_test() -> int:
    actor_source = (
        ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
    ).read_text()
    engine_source = (
        ROOT / "crates/nostr_automerge/src/reference/epoch_engine.rs"
    ).read_text()
    mutations = (
        (
            actor_source.replace(
                "self.causal_next_decision_metered(candidate, &mut charge)?;",
                "self.legacy_counter_is_valid(candidate, &mut charge)?;",
                1,
            ),
            engine_source,
        ),
        (
            actor_source.replace(
                "#[cfg(test)]\nfn reference_apply_nonempty_counter",
                "pub(crate) fn apply_nonempty_counter",
                1,
            ),
            engine_source,
        ),
    )
    caught = 0
    for changed_actor, changed_engine in mutations:
        try:
            validate_projected_causal_next_production_path(
                changed_actor, changed_engine
            )
        except EvidenceError:
            caught += 1
            continue
        raise EvidenceError("causal_next:route_mutation_survived")
    return caught


def validate_streaming_empty_frontier(
    actor_source: str | None = None,
    engine_source: str | None = None,
) -> None:
    actor_source = actor_source or (
        ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
    ).read_text()
    engine_source = engine_source or (
        ROOT / "crates/nostr_automerge/src/reference/epoch_engine.rs"
    ).read_text()
    production, tests = actor_source.split(
        "#[cfg(test)]\npub(crate) mod tests", 1
    )
    if production.count("pub(crate) fn empty_frontier_decision_metered<E>(") != 1:
        raise EvidenceError("frontier:decision_inventory")
    method = production.split(
        "fn empty_frontier_decision_metered_observed", 1
    )[1].split("pub(crate) fn into_accepted_state_parts", 1)[0]
    required = [
        "FrontierComparisonOperation::CandidateKindComparison",
        "FrontierComparisonOperation::CandidateCount",
        "FrontierComparisonOperation::ProjectionCount",
        "FrontierComparisonOperation::BaseCount",
        "FrontierComparisonOperation::CandidatePull",
        "FrontierComparisonOperation::CandidateOrderComparison",
        "FrontierComparisonOperation::ProjectionPull",
        "FrontierComparisonOperation::BasePull",
        "FrontierComparisonOperation::BaseAcceptedLookup",
        "FrontierComparisonOperation::ExpectedSourceComparison",
        "FrontierComparisonOperation::FrontierEqualityComparison",
    ]
    if any(token not in method for token in required):
        raise EvidenceError("frontier:operation_inventory")
    for prohibited in (".collect::<", ".clone()", ".sort", ".dedup", "with_capacity"):
        if prohibited in method:
            raise EvidenceError("frontier:allocation_or_repair")
    helper = production.split("fn metered_frontier_operation", 1)[1].split(
        "#[cfg(test)]", 1
    )[0]
    if (
        "charge(counter).map_err(MeteredActorStateError::Work)?;\n"
        "    let result = target();\n"
        "    observed(operation);"
    ) not in helper:
        raise EvidenceError("frontier:charge_order")
    engine_production = engine_source.split("#[cfg(test)]\nmod tests", 1)[0]
    if actor_source.split("#[cfg(test)]\npub(crate) mod tests", 1)[0].count(
        "self.empty_frontier_decision_metered(candidate, base_frontier, charge)?;"
    ) != 1:
        raise EvidenceError("frontier:combined_decision")
    if engine_production.count(".empty_frontier_decision_metered(") != 0:
        raise EvidenceError("frontier:production_route")
    if engine_production.count(".candidate_semantics_decision_metered(") != 1:
        raise EvidenceError("frontier:combined_route")
    for name in (
        "empty_frontier_comparison_is_streaming_exact_and_immediately_metered",
        "finding_100_empty_frontier_work_reproduction",
    ):
        if tests.count(f"fn {name}()") != 1:
            raise EvidenceError("frontier:test_inventory")
        prefix = tests.split(f"fn {name}()", 1)[0].rsplit("#[test]", 1)[-1]
        if "#[ignore" in prefix:
            raise EvidenceError("frontier:test_ignored")


def streaming_frontier_source_mutation_self_test() -> int:
    actor_source = (
        ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
    ).read_text()
    engine_source = (
        ROOT / "crates/nostr_automerge/src/reference/epoch_engine.rs"
    ).read_text()
    mutations = (
        (
            actor_source.replace(
                "charge(counter).map_err(MeteredActorStateError::Work)?;\n    let result = target();",
                "let result = target();\n    charge(counter).map_err(MeteredActorStateError::Work)?;",
                1,
            ),
            engine_source,
        ),
        (
            actor_source.replace(
                "let dependency_count = metered_frontier_operation(",
                "let _hidden = candidate.dependencies.iter().copied().collect::<BTreeSet<_>>();\n        let dependency_count = metered_frontier_operation(",
                1,
            ),
            engine_source,
        ),
        (
            actor_source.replace(
                "self.empty_frontier_decision_metered(candidate, base_frontier, charge)?;",
                "self.legacy_empty_frontier_is_valid(candidate, base_frontier, charge)?;",
                1,
            ),
            engine_source,
        ),
    )
    caught = 0
    for changed_actor, changed_engine in mutations:
        try:
            validate_streaming_empty_frontier(changed_actor, changed_engine)
        except EvidenceError:
            caught += 1
            continue
        raise EvidenceError("frontier:source_mutation_survived")
    return caught


def validate_combined_candidate_semantics(
    actor_source: str | None = None,
    engine_source: str | None = None,
    runner_source: str | None = None,
) -> None:
    actor_source = actor_source or (
        ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
    ).read_text()
    engine_source = engine_source or (
        ROOT / "crates/nostr_automerge/src/reference/epoch_engine.rs"
    ).read_text()
    runner_source = runner_source or (
        ROOT / "tools/nostr_automerge_conformance/src/runner.rs"
    ).read_text()
    actor_production, actor_tests = actor_source.split(
        "#[cfg(test)]\npub(crate) mod tests", 1
    )
    if actor_production.count(
        "pub(crate) fn candidate_semantics_decision_metered<E>("
    ) != 1:
        raise EvidenceError("combined:decision_inventory")
    method = actor_production.split(
        "fn candidate_semantics_decision_metered_observed<E>(", 1
    )[1].split("/// Decides actor-sequence continuity", 1)[0]
    ordered = [
        "self.actor_sequence_decision_metered(candidate, &mut charge)?;",
        "self.causal_next_decision_metered(candidate, &mut charge)?;",
        "self.empty_frontier_decision_metered(candidate, base_frontier, charge)?;",
    ]
    positions = [method.find(token) for token in ordered]
    if any(position < 0 for position in positions) or positions != sorted(positions):
        raise EvidenceError("combined:decision_order")
    stages = [
        "CandidateSemanticStage::ActorSequence",
        "CandidateSemanticStage::CausalCounter",
        "CandidateSemanticStage::EmptyFrontier",
    ]
    stage_positions = [method.find(stage) for stage in stages]
    if any(position < 0 for position in stage_positions) or stage_positions != sorted(stage_positions):
        raise EvidenceError("combined:stage_order")
    engine_production = engine_source.split("#[cfg(test)]\nmod tests", 1)[0]
    if engine_production.count(".candidate_semantics_decision_metered(") != 1:
        raise EvidenceError("combined:production_route")
    for bypass in (
        ".actor_sequence_decision_metered(",
        ".causal_next_decision_metered(",
        ".empty_frontier_decision_metered(",
    ):
        if bypass in engine_production:
            raise EvidenceError("combined:production_bypass")
    if "let actor_counter_frontier_valid = if let Some(known) = complete_closure" not in engine_production:
        raise EvidenceError("combined:production_result")
    test_name = (
        "complete_candidate_semantics_preserve_precedence_and_every_stop_boundary"
    )
    if actor_tests.count(f"fn {test_name}()") != 1:
        raise EvidenceError("combined:test_inventory")
    prefix, body = actor_tests.split(f"fn {test_name}()", 1)
    if "#[ignore" in prefix.rsplit("#[test]", 1)[-1]:
        raise EvidenceError("combined:test_ignored")
    body = body.split("\n    #[test]", 1)[0]
    required = [
        "Completion::BudgetExhausted, Completion::Cancelled",
        "ActorStateError::MissingPredecessor",
        "ActorStateError::OperationCounter",
        "ActorStateError::DependencyFrontier",
        "core::ptr::eq(error, &injected)",
        "completed.windows(2)",
    ]
    if any(token not in body for token in required):
        raise EvidenceError("combined:test_matrix")
    signed_name = "actor_counter_frontier_reports_match_predecessor_bytes"
    if runner_source.count(f"fn {signed_name}()") != 1:
        raise EvidenceError("combined:signed_inventory")
    signed_prefix, signed_body = runner_source.split(f"fn {signed_name}()", 1)
    if "#[ignore" in signed_prefix.rsplit("#[test]", 1)[-1]:
        raise EvidenceError("combined:signed_ignored")
    signed_body = signed_body.split("\n    #[test]", 1)[0]
    fixture_ids = [
        "actor_counter_sequence_start",
        "actor_counter_exact_predecessor",
        "actor_counter_missing_predecessor",
        "actor_counter_sequence_gap",
        "actor_counter_sequence_rollback",
        "actor_counter_start_op",
        "actor_counter_empty_preservation",
        "actor_counter_empty_frontier",
    ]
    if any(signed_body.count(f'"{fixture_id}"') != 1 for fixture_id in fixture_ids):
        raise EvidenceError("combined:signed_matrix")
    for token in ("run_fixture(&fixture)", 'format!("{fixture_id}.expected.json")'):
        if token not in signed_body:
            raise EvidenceError("combined:signed_bytes")


def combined_candidate_source_mutation_self_test() -> int:
    actor_source = (
        ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
    ).read_text()
    engine_source = (
        ROOT / "crates/nostr_automerge/src/reference/epoch_engine.rs"
    ).read_text()
    runner_source = (
        ROOT / "tools/nostr_automerge_conformance/src/runner.rs"
    ).read_text()
    mutations = (
        (
            actor_source.replace(
                "self.actor_sequence_decision_metered(candidate, &mut charge)?;",
                "self.causal_next_decision_metered(candidate, &mut charge)?;",
                1,
            ),
            engine_source,
            runner_source,
        ),
        (
            actor_source.replace(
                "self.empty_frontier_decision_metered(candidate, base_frontier, charge)?;",
                "Ok(())?;",
                1,
            ),
            engine_source,
            runner_source,
        ),
        (
            actor_source,
            engine_source.replace(
                ".candidate_semantics_decision_metered(",
                ".actor_sequence_decision_metered(",
                1,
            ),
            runner_source,
        ),
        (
            actor_source,
            engine_source,
            runner_source.replace(
                '"actor_counter_empty_frontier",',
                '"actor_counter_sequence_start",',
                1,
            ),
        ),
    )
    caught = 0
    for changed_actor, changed_engine, changed_runner in mutations:
        try:
            validate_combined_candidate_semantics(
                changed_actor, changed_engine, changed_runner
            )
        except EvidenceError:
            caught += 1
            continue
        raise EvidenceError("combined:source_mutation_survived")
    return caught


def validate_compact_epoch_ancestry(source: str | None = None) -> None:
    source = source or (ROOT / "crates/nostr_automerge/src/graph/epoch.rs").read_text()
    production, tests = source.split("#[cfg(test)]\nmod tests", 1)
    required = [
        "pub(crate) enum EpochAncestry {\n    Valid,\n    PendingMissing,\n    InvalidOmission,\n}",
        "enum EpochAncestryObservation {\n    Missing,\n    Complete { omits_base_head: bool },\n}",
        "const fn from_observation(observation: EpochAncestryObservation) -> Self",
        "EpochAncestryObservation::Missing => Self::PendingMissing",
        "omits_base_head: false,\n            } => Self::Valid",
        "omits_base_head: true,\n            } => Self::InvalidOmission",
    ]
    if any(token not in production for token in required):
        raise EvidenceError("ancestry:compact_shape")
    for prohibited in (
        "PendingMissing(Vec<ChangeHash>)",
        "InvalidOmission(Vec<ChangeHash>)",
        "pub enum EpochAncestry",
        "pub(crate) enum EpochAncestryObservation",
    ):
        if prohibited in production:
            raise EvidenceError("ancestry:compact_boundary")
    test_name = "compact_epoch_ancestry_outcomes_are_closed_and_unambiguous"
    if tests.count(f"fn {test_name}()") != 1:
        raise EvidenceError("ancestry:compact_test_inventory")
    prefix, body = tests.split(f"fn {test_name}()", 1)
    if "#[ignore" in prefix.rsplit("#[test]", 1)[-1]:
        raise EvidenceError("ancestry:compact_test_ignored")
    body = body.split("\n    #[test]", 1)[0]
    for token in (
        "size_of::<EpochAncestry>()",
        "EpochAncestryObservation::Missing",
        "omits_base_head: false",
        "omits_base_head: true",
        "EpochAncestry::PendingMissing",
        "EpochAncestry::InvalidOmission",
        "EpochAncestry::Valid",
    ):
        if token not in body:
            raise EvidenceError("ancestry:compact_test_matrix")


def compact_epoch_ancestry_source_mutation_self_test() -> int:
    source = (ROOT / "crates/nostr_automerge/src/graph/epoch.rs").read_text()
    mutations = (
        source.replace("    PendingMissing,", "    PendingMissing(Vec<ChangeHash>),", 1),
        source.replace("    InvalidOmission,", "    InvalidOmission(Vec<ChangeHash>),", 1),
        source.replace(
            "    Missing,\n    Complete { omits_base_head: bool },",
            "    Missing { omits_base_head: bool },\n    Complete { omits_base_head: bool },",
            1,
        ),
        source.replace(
            "pub(crate) enum EpochAncestry {",
            "pub enum EpochAncestry {",
            1,
        ),
    )
    caught = 0
    for changed in mutations:
        try:
            validate_compact_epoch_ancestry(changed)
        except EvidenceError:
            caught += 1
            continue
        raise EvidenceError("ancestry:compact_source_mutation_survived")
    return caught


def validate_metered_epoch_ancestry(source: str | None = None) -> None:
    source = source or (ROOT / "crates/nostr_automerge/src/graph/epoch.rs").read_text()
    production, tests = source.split("#[cfg(test)]\nmod tests", 1)
    body = production.split("fn classify_epoch_ancestry_metered_observed", 1)[1].split(
        "fn ancestry_operation", 1
    )[0]
    required = [
        "EpochAncestryOperation::MissingDependencyPull",
        "EpochAncestryOperation::BaseHeadPull",
        "EpochAncestryOperation::AcceptedClosureLookup",
        "EpochAncestryOperation::InclusionComparison",
        "EpochAncestryOperation::StateTransition",
        "WorkCounter::GraphNode",
        "WorkCounter::GraphEdge",
        "missing.next().copied()",
        "base.next().copied()",
        "dependency_closure.contains(&head)",
    ]
    if any(token not in body for token in required):
        raise EvidenceError("ancestry:metered_operations")
    expected_counts = {
        "EpochAncestryOperation::MissingDependencyPull": 1,
        "EpochAncestryOperation::BaseHeadPull": 1,
        "EpochAncestryOperation::AcceptedClosureLookup": 1,
        "EpochAncestryOperation::InclusionComparison": 1,
        "EpochAncestryOperation::StateTransition": 3,
    }
    if any(body.count(token) != count for token, count in expected_counts.items()):
        raise EvidenceError("ancestry:metered_operation_counts")
    for prohibited in ("Vec::", ".collect", ".sort", ".dedup", ".difference", ".is_empty"):
        if prohibited in body:
            raise EvidenceError("ancestry:metered_allocation_or_repair")
    operation = production.split("fn ancestry_operation", 1)[1].split(
        "pub(crate) fn validate_epoch_ancestry", 1
    )[0]
    ordered = ["charge(counter)?;", "let result = target();", "observed(operation);", "Ok(result)"]
    positions = [operation.find(token) for token in ordered]
    if any(position < 0 for position in positions) or positions != sorted(positions):
        raise EvidenceError("ancestry:charge_order")
    test_name = "ancestry_classification_is_nonallocating_streaming_and_exactly_metered"
    if tests.count(f"fn {test_name}()") != 1:
        raise EvidenceError("ancestry:metered_test_inventory")
    prefix, test = tests.split(f"fn {test_name}()", 1)
    if "#[ignore" in prefix.rsplit("#[test]", 1)[-1]:
        raise EvidenceError("ancestry:metered_test_ignored")
    test = test.split("\n    #[test]", 1)[0]
    for token in (
        "TestStop::BudgetExhausted, TestStop::Cancelled",
        "for limit in 0..exact",
        "assert_eq!(observed, expected_trace[..limit])",
        "(1..=64).map(hash).collect()",
        "EpochAncestry::PendingMissing",
        "EpochAncestry::InvalidOmission",
        "EpochAncestry::Valid",
    ):
        if token not in test:
            raise EvidenceError("ancestry:metered_test_matrix")


def metered_epoch_ancestry_source_mutation_self_test() -> int:
    source = (ROOT / "crates/nostr_automerge/src/graph/epoch.rs").read_text()
    mutations = (
        source.replace("charge(counter)?;", "let _ = counter;", 1),
        source.replace(
            "charge(counter)?;\n    let result = target();",
            "let result = target();\n    charge(counter)?;",
            1,
        ),
        source.replace("missing.next().copied()", "missing.collect::<Vec<_>>().first().copied().copied()", 1),
        source.replace("dependency_closure.contains(&head)", "true", 1),
        source.replace(
            "EpochAncestryOperation::StateTransition,\n            || EpochAncestry::from_observation(EpochAncestryObservation::Missing)",
            "EpochAncestryOperation::BaseHeadPull,\n            || EpochAncestry::from_observation(EpochAncestryObservation::Missing)",
            1,
        ),
    )
    caught = 0
    for changed in mutations:
        try:
            validate_metered_epoch_ancestry(changed)
        except EvidenceError:
            caught += 1
            continue
        raise EvidenceError("ancestry:metered_source_mutation_survived")
    return caught


def validate_epoch_ancestry_production_path(
    epoch_source: str | None = None,
    engine_source: str | None = None,
    public_source: str | None = None,
) -> None:
    epoch_source = epoch_source or (
        ROOT / "crates/nostr_automerge/src/graph/epoch.rs"
    ).read_text()
    engine_source = engine_source or (
        ROOT / "crates/nostr_automerge/src/reference/epoch_engine.rs"
    ).read_text()
    public_source = public_source or (
        ROOT / "crates/nostr_automerge/tests/public_engine_api.rs"
    ).read_text()
    epoch_production, epoch_tests = epoch_source.split("#[cfg(test)]\nmod tests", 1)
    engine_production = engine_source.split("#[cfg(test)]\nmod tests", 1)[0]
    if "pub(crate) fn validate_epoch_ancestry(" in epoch_production:
        raise EvidenceError("ancestry:legacy_helper")
    if engine_production.count("classify_epoch_ancestry_metered(") != 1:
        raise EvidenceError("ancestry:production_route")
    route = engine_production.split("classify_epoch_ancestry_metered(", 1)[1].split(
        "let prior_dependencies_valid", 1
    )[0]
    for token in (
        "input.accepted_base().frontier_heads()",
        "&closure.known",
        "&closure.missing",
        "charge_epoch_item(counter, budget, cancellation)",
        ".map_err(EpochEvaluationError::Schedule)",
        "EpochAncestry::InvalidOmission",
    ):
        if token not in route:
            raise EvidenceError("ancestry:production_route_shape")
    for prohibited in ("Infallible", "Ok::<(), Infallible>", "validate_epoch_ancestry("):
        if prohibited in epoch_production or prohibited in engine_production:
            raise EvidenceError("ancestry:production_bypass")
    reproduction = "finding_100_epoch_ancestry_work_reproduction"
    if epoch_tests.count(f"fn {reproduction}()") != 1:
        raise EvidenceError("ancestry:reproduction_inventory")
    prefix = epoch_tests.split(f"fn {reproduction}()", 1)[0].rsplit("#[test]", 1)[-1]
    if "#[ignore" in prefix:
        raise EvidenceError("ancestry:reproduction_ignored")
    for test_name in (
        "base_omission_cannot_poison_valid_same_sequence_change",
        "missing_dependency_promotes_after_delivery",
    ):
        if public_source.count(f"fn {test_name}()") != 1:
            raise EvidenceError("ancestry:signed_inventory")
        attributes = public_source.split(f"fn {test_name}()", 1)[0].rsplit("#[test]", 1)[-1]
        if "#[ignore" in attributes:
            raise EvidenceError("ancestry:signed_ignored")


def epoch_ancestry_route_source_mutation_self_test() -> int:
    epoch_source = (ROOT / "crates/nostr_automerge/src/graph/epoch.rs").read_text()
    engine_source = (
        ROOT / "crates/nostr_automerge/src/reference/epoch_engine.rs"
    ).read_text()
    public_source = (
        ROOT / "crates/nostr_automerge/tests/public_engine_api.rs"
    ).read_text()
    mutations = (
        (
            epoch_source,
            engine_source.replace(
                "classify_epoch_ancestry_metered(", "validate_epoch_ancestry(", 1
            ),
            public_source,
        ),
        (
            epoch_source,
            engine_source.replace(
                "charge_epoch_item(counter, budget, cancellation)\n                    .map_err(EpochEvaluationError::Schedule)",
                "Ok::<(), EpochEvaluationError>(())",
                1,
            ),
            public_source,
        ),
        (
            epoch_source.replace(
                "#[test]\n    fn finding_100_epoch_ancestry_work_reproduction()",
                "#[test]\n    #[ignore = \"restored\"]\n    fn finding_100_epoch_ancestry_work_reproduction()",
                1,
            ),
            engine_source,
            public_source,
        ),
        (
            epoch_source,
            engine_source,
            public_source.replace(
                "fn missing_dependency_promotes_after_delivery()",
                "fn missing_dependency_after_delivery()",
                1,
            ),
        ),
    )
    caught = 0
    for changed_epoch, changed_engine, changed_public in mutations:
        try:
            validate_epoch_ancestry_production_path(
                changed_epoch, changed_engine, changed_public
            )
        except EvidenceError:
            caught += 1
            continue
        raise EvidenceError("ancestry:route_source_mutation_survived")
    return caught


def validate_shared_control_member_authorization(
    authorization_source: str | None = None,
    checkpoint_source: str | None = None,
    evaluator_source: str | None = None,
    runner_source: str | None = None,
) -> None:
    authorization_source = authorization_source or (
        ROOT / "crates/nostr_automerge/src/control/authorize.rs"
    ).read_text()
    checkpoint_source = checkpoint_source or (
        ROOT / "crates/nostr_automerge/src/checkpoint/authorize.rs"
    ).read_text()
    evaluator_source = evaluator_source or (
        ROOT / "crates/nostr_automerge/src/engine/reference_evaluator.rs"
    ).read_text()
    runner_source = runner_source or (
        ROOT / "tools/nostr_automerge_conformance/src/fixture_generation.rs"
    ).read_text()
    production, tests = authorization_source.split("#[cfg(test)]\nmod tests", 1)
    body = production.split("pub(crate) fn any_control_member_metered", 1)[1]
    required = [
        "visit(WorkCounter::Control)?;\n        let Some(member) = members.next()",
        "visit(WorkCounter::Control)?;\n        if predicate(member)",
        "return Ok(false)",
        "return Ok(true)",
    ]
    if any(token not in body for token in required):
        raise EvidenceError("authorization:charged_order")
    if body.count("visit(WorkCounter::Control)?;") != 2:
        raise EvidenceError("authorization:charge_count")
    for prohibited in (".any(", ".collect", ".sort", ".dedup", "with_capacity"):
        if prohibited in body:
            raise EvidenceError("authorization:eager_or_bypass")
    for test_name in (
        "member_authorization_charges_each_pull_and_predicate_before_work",
        "member_authorization_preserves_every_budget_and_cancellation_boundary",
    ):
        if tests.count(f"fn {test_name}()") != 1:
            raise EvidenceError("authorization:test_inventory")
        attributes = tests.split(f"fn {test_name}()", 1)[0].rsplit("#[test]", 1)[-1]
        if "#[ignore" in attributes:
            raise EvidenceError("authorization:test_ignored")
    for token in (
        "(5, false, 9, 4)",
        "(10, true, 2, 1)",
        "(20, true, 4, 2)",
        "(40, true, 8, 4)",
        "for capacity in 0..=required",
        "for cancel_at in 0..required",
        "predicates.get(), cancel_at / 2",
    ):
        if token not in tests:
            raise EvidenceError("authorization:test_matrix")
    checkpoint_production = checkpoint_source.split("#[cfg(test)]\nmod tests", 1)[0]
    evaluator_production = evaluator_source.split("#[cfg(test)]\nmod tests", 1)[0]
    if checkpoint_production.count("any_control_member_metered") != 2:
        raise EvidenceError("authorization:checkpoint_route")
    if evaluator_production.count("any_control_member_metered") != 3:
        raise EvidenceError("authorization:evaluator_route")
    if "fn any_control_member_metered" in checkpoint_production or "fn any_control_member_metered" in evaluator_production:
        raise EvidenceError("authorization:duplicate_helper")
    budget_binding = runner_source.split(
        'let current_delta = if fixture_id == "unrelated_valid_checkpoints_exact_budget"',
        1,
    )
    if len(budget_binding) != 2 or "{\n                2\n            } else {\n                1\n            };" not in budget_binding[1]:
        raise EvidenceError("authorization:resource_delta")


def shared_control_member_authorization_source_mutation_self_test() -> int:
    authorization_source = (
        ROOT / "crates/nostr_automerge/src/control/authorize.rs"
    ).read_text()
    checkpoint_source = (
        ROOT / "crates/nostr_automerge/src/checkpoint/authorize.rs"
    ).read_text()
    evaluator_source = (
        ROOT / "crates/nostr_automerge/src/engine/reference_evaluator.rs"
    ).read_text()
    runner_source = (
        ROOT / "tools/nostr_automerge_conformance/src/fixture_generation.rs"
    ).read_text()
    mutations = (
        (
            authorization_source.replace(
                "visit(WorkCounter::Control)?;\n        let Some(member)",
                "let Some(member)",
                1,
            ),
            checkpoint_source,
            evaluator_source,
            runner_source,
        ),
        (
            authorization_source.replace(
                "visit(WorkCounter::Control)?;\n        if predicate(member)",
                "if predicate(member)",
                1,
            ),
            checkpoint_source,
            evaluator_source,
            runner_source,
        ),
        (
            authorization_source,
            checkpoint_source.replace("any_control_member_metered(", "control.members().iter().any(", 1),
            evaluator_source,
            runner_source,
        ),
        (
            authorization_source,
            checkpoint_source,
            evaluator_source.replace("any_control_member_metered(", "legacy_member_scan(", 1),
            runner_source,
        ),
        (
            authorization_source.replace(
                "#[test]\n    fn member_authorization_preserves_every_budget",
                "#[test]\n    #[ignore]\n    fn member_authorization_preserves_every_budget",
                1,
            ),
            checkpoint_source,
            evaluator_source,
            runner_source,
        ),
        (
            authorization_source,
            checkpoint_source,
            evaluator_source,
            runner_source.replace(
                'fixture_id == "unrelated_valid_checkpoints_exact_budget" {\n                2',
                'fixture_id == "unrelated_valid_checkpoints_exact_budget" {\n                1',
                1,
            ),
        ),
    )
    caught = 0
    for changed_authorization, changed_checkpoint, changed_evaluator, changed_runner in mutations:
        try:
            validate_shared_control_member_authorization(
                changed_authorization, changed_checkpoint, changed_evaluator, changed_runner
            )
        except EvidenceError:
            caught += 1
            continue
        raise EvidenceError("authorization:source_mutation_survived")
    return caught


def validate_authorization_production_routes(
    epoch_source: str | None = None,
    evaluator_source: str | None = None,
    checkpoint_source: str | None = None,
    public_source: str | None = None,
) -> None:
    epoch_source = epoch_source or (
        ROOT / "crates/nostr_automerge/src/reference/epoch_engine.rs"
    ).read_text()
    evaluator_source = evaluator_source or (
        ROOT / "crates/nostr_automerge/src/engine/reference_evaluator.rs"
    ).read_text()
    checkpoint_source = checkpoint_source or (
        ROOT / "crates/nostr_automerge/src/checkpoint/authorize.rs"
    ).read_text()
    public_source = public_source or (
        ROOT / "crates/nostr_automerge/tests/public_engine_api.rs"
    ).read_text()
    epoch_production, epoch_tests = epoch_source.split("#[cfg(test)]\nmod tests", 1)
    evaluator_production = evaluator_source.split("#[cfg(test)]\nmod tests", 1)[0]
    checkpoint_production = checkpoint_source.split("#[cfg(test)]\nmod tests", 1)[0]
    if epoch_production.count("any_control_member_metered") != 2:
        raise EvidenceError("authorization:epoch_route")
    if evaluator_production.count("any_control_member_metered") != 3:
        raise EvidenceError("authorization:change_routes")
    if checkpoint_production.count("any_control_member_metered") != 2:
        raise EvidenceError("authorization:checkpoint_route_current")
    route = epoch_production.split("pub(crate) fn evaluate_epoch(", 1)[1].split(
        "let mut dispositions = resolve_epoch", 1
    )[0]
    ordered = [
        "if terminal {",
        "let authorized = any_control_member_metered(",
        "if !authorized {",
        "candidate_dependency_closure(",
    ]
    positions = [route.find(token) for token in ordered]
    if any(position < 0 for position in positions) or positions != sorted(positions):
        raise EvidenceError("authorization:epoch_precedence")
    if route.count("continue;") < 2:
        raise EvidenceError("authorization:epoch_refusal_exit")
    change_route = evaluator_production.split("fn control_authorizes_change(", 1)[1].split(
        "fn reduce_change_dispositions", 1
    )[0]
    if change_route.find("if control.terminal()") > change_route.find("any_control_member_metered("):
        raise EvidenceError("authorization:terminal_precedence")
    for prohibited in (
        "selected.content().members.iter().any",
        "let mut members = control.members().iter()",
        "for _ in 0..control.members().len()",
    ):
        if prohibited in epoch_production or prohibited in evaluator_production:
            raise EvidenceError("authorization:production_bypass")
    for test_name in (
        "finding_100_epoch_writer_authorization_work_reproduction",
        "epoch_writer_refusal_precedes_dependency_work_and_preserves_typed_stops",
    ):
        if epoch_tests.count(f"fn {test_name}()") != 1:
            raise EvidenceError("authorization:epoch_test_inventory")
        attributes = epoch_tests.split(f"fn {test_name}()", 1)[0].rsplit("#[test]", 1)[-1]
        if "#[ignore" in attributes:
            raise EvidenceError("authorization:epoch_test_ignored")
    refusal = epoch_tests.split(
        "fn epoch_writer_refusal_precedes_dependency_work_and_preserves_typed_stops()", 1
    )[1].split("\n    #[test]", 1)[0]
    for token in (
        "WorkCounter::Control), 3",
        "WorkCounter::ApplyChange), 0",
        "ScheduleError::BudgetExhausted",
        "ScheduleError::Cancelled",
        "[(2, 0), (3, 1), (4, 2)]",
    ):
        if token not in refusal:
            raise EvidenceError("authorization:epoch_refusal_matrix")
    for test_name in (
        "signed_checkpoint_role_gate_is_exact_and_delivery_order_independent",
        "noncanonical_authorization_is_enforced_before_exclusion",
        "signed_causal_change_matrix",
    ):
        if public_source.count(f"fn {test_name}()") != 1:
            raise EvidenceError("authorization:signed_inventory")
        attributes = public_source.split(f"fn {test_name}()", 1)[0].rsplit("#[test]", 1)[-1]
        if "#[ignore" in attributes:
            raise EvidenceError("authorization:signed_ignored")


def authorization_production_source_mutation_self_test() -> int:
    epoch_source = (
        ROOT / "crates/nostr_automerge/src/reference/epoch_engine.rs"
    ).read_text()
    evaluator_source = (
        ROOT / "crates/nostr_automerge/src/engine/reference_evaluator.rs"
    ).read_text()
    checkpoint_source = (
        ROOT / "crates/nostr_automerge/src/checkpoint/authorize.rs"
    ).read_text()
    public_source = (
        ROOT / "crates/nostr_automerge/tests/public_engine_api.rs"
    ).read_text()
    mutations = (
        (epoch_source.replace("any_control_member_metered(", "legacy_member_scan(", 1), evaluator_source, checkpoint_source, public_source),
        (epoch_source.replace("if !authorized {", "if authorized {", 1), evaluator_source, checkpoint_source, public_source),
        (epoch_source.replace("candidate_dependency_closure(", "legacy_candidate_closure(", 1), evaluator_source, checkpoint_source, public_source),
        (epoch_source, evaluator_source.replace("any_control_member_metered(", "legacy_member_scan(", 1), checkpoint_source, public_source),
        (epoch_source, evaluator_source, checkpoint_source.replace("any_control_member_metered(", "legacy_member_scan(", 1), public_source),
        (epoch_source.replace("#[test]\n    fn epoch_writer_refusal", "#[test]\n    #[ignore]\n    fn epoch_writer_refusal", 1), evaluator_source, checkpoint_source, public_source),
    )
    caught = 0
    for changed_epoch, changed_evaluator, changed_checkpoint, changed_public in mutations:
        try:
            validate_authorization_production_routes(
                changed_epoch, changed_evaluator, changed_checkpoint, changed_public
            )
        except EvidenceError:
            caught += 1
            continue
        raise EvidenceError("authorization:production_source_mutation_survived")
    return caught


def validate_metered_candidate_dependency_closure(source: str | None = None) -> None:
    source = source or (ROOT / "crates/nostr_automerge/src/graph/closure.rs").read_text()
    production, tests = source.split("#[cfg(test)]\nmod tests", 1)
    implementation = production.split("fn candidate_dependency_closure_observed", 1)[1].split(
        "fn charge_closure_work", 1
    )[0]
    wrapper = production.split("pub(crate) fn candidate_dependency_closure(", 1)[1].split(
        "fn candidate_dependency_closure_observed", 1
    )[0]
    if wrapper.count("candidate_dependency_closure_observed(") != 1:
        raise EvidenceError("closure:production_route")
    helper = implementation.split("fn closure_operation", 1)[1]
    ordered = ["charge(counter)?;", "let value = target();", "observed(operation);", "Ok(value)"]
    positions = [helper.find(token) for token in ordered]
    if any(position < 0 for position in positions) or positions != sorted(positions):
        raise EvidenceError("closure:charge_target_order")
    required_operations = [
        "ResultConstruction", "PendingStackConstruction", "DependencyPull", "PendingPush",
        "PendingPull", "KnownLookup", "KnownInsert", "CandidateLookup", "KnownRemove",
        "MissingInsert", "IndegreeMapConstruction", "KnownPull", "IndegreeInsert",
        "DependencyKnownComparison", "IndegreeLookup", "IndegreeIncrement",
        "DependantMapConstruction", "DependantLookup", "DependantBucketInsert",
        "DependantInsert", "ReadySetConstruction", "IndegreePull", "ReadinessComparison",
        "ReadyInsert", "ReadyPull", "OrderedPush", "DependantChildrenLookup",
        "DependantPull", "IndegreeDecrement", "OrderedSetConstruction", "OrderedPull",
        "OrderedInsert", "OrderedMembershipComparison", "CyclicInsert", "ResultPublication",
    ]
    for operation in required_operations:
        if production.count(f"CandidateClosureOperation::{operation}") < 1:
            raise EvidenceError("closure:operation:" + operation)
    prohibited = [
        "Vec::with_capacity(candidate.dependencies.len())",
        ".collect::<Result<BTreeMap<_, _>, _>>()",
        ".collect::<Result<Vec<_>, _>>()?",
        "while let Some(hash) = pending.pop()",
        "result.known.difference(&ordered)",
    ]
    if any(token in implementation for token in prohibited):
        raise EvidenceError("closure:unmetered_preparation")
    for test_name in (
        "candidate_dependency_closure_charges_immediately_before_every_target_operation",
        "candidate_dependency_closure_scales_across_deep_wide_cycle_and_missing_graphs",
        "finding_100_dependency_closure_work_reproduction",
    ):
        if tests.count(f"fn {test_name}()") != 1:
            raise EvidenceError("closure:test_inventory")
        attributes = tests.split(f"fn {test_name}()", 1)[0].rsplit("#[test]", 1)[-1]
        if "#[ignore" in attributes:
            raise EvidenceError("closure:test_ignored")
    trace_test = tests.split(
        "fn candidate_dependency_closure_charges_immediately_before_every_target_operation()", 1
    )[1].split("\n    #[test]", 1)[0]
    for token in (
        "Trace::Charge(counter)",
        "Trace::Operation(operation)",
        "for allowance in 0..operations.len()",
        "Err(Stop::BudgetExhausted)",
        "Err(Stop::Cancelled)",
        "operations[..allowance]",
    ):
        if token not in trace_test:
            raise EvidenceError("closure:boundary_matrix")


def candidate_dependency_closure_source_mutation_self_test() -> int:
    source = (ROOT / "crates/nostr_automerge/src/graph/closure.rs").read_text()
    mutations = (
        source.replace("charge(counter)?;", "let _ = counter;", 1),
        source.replace(
            "charge(counter)?;\n    let value = target();",
            "let value = target();\n    charge(counter)?;",
            1,
        ),
        source.replace(
            "CandidateClosureOperation::PendingStackConstruction",
            "CandidateClosureOperation::ResultConstruction",
            1,
        ),
        source.replace(
            "CandidateClosureOperation::DependantMapConstruction",
            "CandidateClosureOperation::ResultConstruction",
            1,
        ),
        source.replace(
            "#[test]\n    fn candidate_dependency_closure_charges",
            "#[test]\n    #[ignore]\n    fn candidate_dependency_closure_charges",
            1,
        ),
        source.replace(
            "fn candidate_dependency_closure_scales_across_deep_wide_cycle_and_missing_graphs()",
            "fn removed_graph_shape_matrix()",
            1,
        ),
        source.replace(
            "CandidateClosureOperation::ResultPublication",
            "CandidateClosureOperation::ResultConstruction",
            1,
        ),
    )
    caught = 0
    for changed in mutations:
        try:
            validate_metered_candidate_dependency_closure(changed)
        except EvidenceError:
            caught += 1
            continue
        raise EvidenceError("closure:source_mutation_survived")
    return caught


def validate_metered_candidate_schedule(
    source: str | None = None,
    scaling_source: str | None = None,
) -> None:
    source = source or (ROOT / "crates/nostr_automerge/src/graph/schedule.rs").read_text()
    scaling_source = scaling_source or (
        ROOT / "crates/nostr_automerge/src/graph/scaling.rs"
    ).read_text()
    production, tests = source.split("#[cfg(test)]\nmod tests", 1)
    implementation = production.split("fn schedule_candidates_observed", 1)[1].split(
        "fn charge_schedule_work", 1
    )[0]
    helper = implementation.split("fn schedule_operation", 1)[1]
    ordered = ["charge(counter)?;", "let value = target();", "observed(operation);", "Ok(value)"]
    positions = [helper.find(token) for token in ordered]
    if any(position < 0 for position in positions) or positions != sorted(positions):
        raise EvidenceError("schedule:charge_target_order")
    if production.split("pub(crate) fn schedule_candidates(", 1)[1].split(
        "fn schedule_candidates_observed", 1
    )[0].count("schedule_candidates_observed(") != 1:
        raise EvidenceError("schedule:production_route")
    operations = [
        "ScheduleConstruction", "RemainingMapConstruction", "CandidatePull",
        "RemainingInsert", "CandidateHashSetConstruction", "RemainingKeyPull",
        "CandidateHashInsert", "UnresolvedMapConstruction", "DependantMapConstruction",
        "RemainingEntryPull", "DependencyPull", "AcceptedLookup", "UnresolvedIncrement",
        "CandidateLookup", "DependantLookup", "DependantBucketInsert", "DependantInsert",
        "UnresolvedInsert", "ReadySetConstruction", "UnresolvedPull",
        "ReadinessComparison", "ReadyInsert", "OrderedVecConstruction", "ReadyPeek",
        "ReadyTieComparison", "ReadyPop", "RemainingRemove", "OrderedPush",
        "ChildrenLookup", "ChildPull", "UnresolvedLookup", "UnresolvedDecrement",
        "MissingSetConstruction", "RemainingValuePull", "MissingCandidateLookup",
        "MissingAcceptedLookup", "MissingInsert", "PendingSetConstruction",
        "MissingLookup", "PendingInsert", "BlockedStackConstruction", "PendingPull",
        "BlockedPush", "BlockedPull", "RemainingLookup", "PendingLookup",
        "CyclicSetConstruction", "CyclicCandidatePull", "CyclicPendingLookup",
        "CyclicInsert", "ResultPublication",
    ]
    for operation in operations:
        if production.count(f"ScheduleOperation::{operation}") < 1:
            raise EvidenceError("schedule:operation:" + operation)
    for prohibited in (
        "remaining.keys().copied().collect::<BTreeSet<_>>()",
        "pending.iter().copied().collect::<Vec<_>>()",
        "while let Some(hash) = blocked.pop()",
        ".filter_map(|(hash, count)|",
    ):
        if prohibited in implementation:
            raise EvidenceError("schedule:unmetered_preparation")
    for test_name in (
        "scheduling_charges_immediately_before_every_operation_and_preserves_typed_stops",
        "finding_100_schedule_readiness_work_reproduction",
        "finding_100_schedule_publication_work_reproduction",
    ):
        if tests.count(f"fn {test_name}()") != 1:
            raise EvidenceError("schedule:test_inventory")
        attributes = tests.split(f"fn {test_name}()", 1)[0].rsplit("#[test]", 1)[-1]
        if "#[ignore" in attributes:
            raise EvidenceError("schedule:test_ignored")
    matrix = tests.split(
        "fn scheduling_charges_immediately_before_every_operation_and_preserves_typed_stops()", 1
    )[1].split("\n    #[test]", 1)[0]
    for token in (
        "Trace::Charge(counter)", "Trace::Operation(operation)",
        "for allowance in 0..operations.len()", "Stop::BudgetExhausted",
        "Stop::Cancelled", "operations[..allowance]", "inputs.into_iter().rev()",
    ):
        if token not in matrix:
            raise EvidenceError("schedule:boundary_matrix")
    for token in (
        "(128, 0, 2_186, 1_273)", "(128, 0, 2_186, 1_147)",
        "(0, 1, 29, 13)", "(0, 2, 39, 28)",
    ):
        if token not in scaling_source:
            raise EvidenceError("schedule:scaling")


def candidate_schedule_source_mutation_self_test() -> int:
    source = (ROOT / "crates/nostr_automerge/src/graph/schedule.rs").read_text()
    scaling = (ROOT / "crates/nostr_automerge/src/graph/scaling.rs").read_text()
    mutations = (
        (source.replace("charge(counter)?;", "let _ = counter;", 1), scaling),
        (source.replace(
            "charge(counter)?;\n    let value = target();",
            "let value = target();\n    charge(counter)?;",
            1,
        ), scaling),
        (source.replace("ScheduleOperation::ReadyTieComparison", "ScheduleOperation::ReadyPeek", 1), scaling),
        (source.replace("ScheduleOperation::ReadyPop", "ScheduleOperation::ReadyPeek", 1), scaling),
        (source.replace("ScheduleOperation::ResultPublication", "ScheduleOperation::ScheduleConstruction", 1), scaling),
        (source.replace(
            "#[test]\n    fn scheduling_charges",
            "#[test]\n    #[ignore]\n    fn scheduling_charges",
            1,
        ), scaling),
        (source.replace("fn finding_100_schedule_readiness_work_reproduction()", "fn removed_readiness_proof()", 1), scaling),
        (source, scaling.replace("(128, 0, 2_186, 1_273)", "(128, 0, 256, 254)")),
    )
    caught = 0
    for index, (changed_source, changed_scaling) in enumerate(mutations):
        try:
            validate_metered_candidate_schedule(changed_source, changed_scaling)
        except EvidenceError:
            caught += 1
            continue
        raise EvidenceError(f"schedule:source_mutation_survived:{index}")
    return caught


def validate_active_causal_budget_deltas(
    evaluator_source: str | None = None,
    fixture_source: str | None = None,
) -> None:
    evaluator_source = evaluator_source or (
        ROOT / "crates/nostr_automerge/src/engine/reference_evaluator.rs"
    ).read_text()
    fixture_source = fixture_source or (
        ROOT / "tools/nostr_automerge_conformance/src/fixture_generation.rs"
    ).read_text()
    if "fixture_items.checked_add(226)" not in evaluator_source:
        raise EvidenceError("causal_next:post_branch_delta")
    if any(token not in fixture_source for token in (
        '("deep_delta_root_lookup_exact_budget", 17, 9, 8, 0, 2_160)',
        '("deep_delta_absent_lookup_exact_budget", 16, 8, 8, 0, 2_145)',
        '("deep_delta_extend_exact_budget", 17, 9, 1, 7, 1_999)',
        "signed.budget.max_items.checked_add(active_delta)",
    )):
        raise EvidenceError("causal_next:persistent_delta")


def causal_budget_source_mutation_self_test() -> int:
    evaluator_source = (
        ROOT / "crates/nostr_automerge/src/engine/reference_evaluator.rs"
    ).read_text()
    fixture_source = (
        ROOT / "tools/nostr_automerge_conformance/src/fixture_generation.rs"
    ).read_text()
    mutations = (
        (evaluator_source.replace("checked_add(226)", "checked_add(225)", 1), fixture_source),
        (evaluator_source, fixture_source.replace("2_160)", "2_159)", 1)),
        (evaluator_source, fixture_source.replace("2_145)", "2_144)", 1)),
        (evaluator_source, fixture_source.replace("1_999)", "1_998)", 1)),
    )
    caught = 0
    for changed_evaluator, changed_fixture in mutations:
        try:
            validate_active_causal_budget_deltas(changed_evaluator, changed_fixture)
        except EvidenceError:
            caught += 1
            continue
        raise EvidenceError("causal_next:budget_mutation_survived")
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
            "expected": "fixed_pass" if index <= 7 else "open_failure",
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


def validate_actor_gate(gate: object) -> None:
    record = require_keys(gate, [
        "schema", "status", "rcld", "candidate_chain", "requirements",
        "decisions", "work_contract", "reproductions", "source_bindings",
        "findings", "holds", "result",
    ], "actor_gate")
    require_equal(record["schema"], "nostr_automerge.remediation_v12_actor_gate.v1", "actor_gate:schema")
    require_equal(record["status"], "rcld_111_complete", "actor_gate:status")
    require_equal(record["rcld"], 111, "actor_gate:rcld")
    require_equal(record["candidate_chain"], [
        {"step": step, "candidate": candidate}
        for step, candidate in (
            ("step_1379", PROJECTION_GATE_CANDIDATE),
            ("step_1380", ACTOR_DECISION_CANDIDATE),
            ("step_1381", ACTOR_ROUTE_CANDIDATE),
            ("step_1382", ACTOR_SIGNED_CANDIDATE),
            ("step_1383", CAUSAL_DECISION_CANDIDATE),
            ("step_1384", CAUSAL_ROUTE_CANDIDATE),
            ("step_1385", FRONTIER_CANDIDATE),
            ("step_1386", COMBINED_CANDIDATE),
        )
    ], "actor_gate:chain")
    require_equal(record["requirements"], EVIDENCE_REQUIREMENTS, "actor_gate:requirements")
    require_equal(require_keys(record["decisions"], [
        "actor_lookup_operations", "causal_counter_operations",
        "frontier_operations", "combined_stages", "signed_scenarios",
        "delivery_order_minimum", "typed_precedence",
    ], "actor_gate:decisions"), {
        "actor_lookup_operations": 9,
        "causal_counter_operations": 3,
        "frontier_operations": 11,
        "combined_stages": 3,
        "signed_scenarios": 8,
        "delivery_order_minimum": 2,
        "typed_precedence": "actor_then_counter_then_frontier",
    }, "actor_gate:decisions")
    require_equal(require_keys(record["work_contract"], [
        "budget_matrix", "cancellation_matrix", "first_stop_preserved",
        "zero_later_stage_work", "unexpected_identity",
        "predecessor_output_bytes", "production_bypasses",
    ], "actor_gate:work"), {
        "budget_matrix": "pass",
        "cancellation_matrix": "pass",
        "first_stop_preserved": True,
        "zero_later_stage_work": True,
        "unexpected_identity": "preserved",
        "predecessor_output_bytes": "equal",
        "production_bypasses": 0,
    }, "actor_gate:work")
    require_equal(require_keys(record["reproductions"], [
        "fixed_families", "remaining_finding_100_families", "finding_100_status",
    ], "actor_gate:reproductions"), {
        "fixed_families": ["actor_predecessor", "causal_next_op", "empty_frontier"],
        "remaining_finding_100_families": 7,
        "finding_100_status": "open",
    }, "actor_gate:reproductions")
    require_equal(record["source_bindings"], [
        {"path": path, "sha256": digest}
        for path, digest in (
            ("crates/nostr_automerge/src/graph/actor_state.rs", "b33733c9c84a7b7a6247967172a562c2c3f0da68a64514a50b27b715127c8290"),
            ("crates/nostr_automerge/src/reference/epoch_engine.rs", "3ffd16c4fb3b8d6de7c7a6aa49e2a7f8f57ef23df39e54f1a1f2f511b9bf68ac"),
            ("tools/nostr_automerge_conformance/src/runner.rs", "dcf1826785ff35fd636838e55b67a0b40bf40d4ba3ff5dca1875e64d56233b5a"),
            ("scripts/validate_remediation_v12.py", "be0e3cd28d9aa2d9e3973d4a950d53d9a39b804378a8fea8780afd18b1e7bd75"),
            ("scripts/reproduce_remediation_v12.py", "ad3e3c6df4f20963efb2bea22aceac373e7b014d9b11d91ec5028f39d65019a4"),
        )
    ], "actor_gate:bindings")
    require_equal(require_keys(record["findings"], ["open", "held"], "actor_gate:findings"), {
        "open": ["FINDING_100", "FINDING_101", "FINDING_102", "FINDING_103"],
        "held": ["FINDING_080"],
    }, "actor_gate:findings")
    require_equal(record["holds"], HOLDS, "actor_gate:holds")
    require_equal(record["result"], "pass", "actor_gate:result")


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


def mutation_self_test(authority: object, ledger: object, findings: object, reproductions: object, evidence_policy: object, authority_gate: object, actor_gate: object) -> int:
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
        ("cursor", "next_step", "step_1396"),
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
    actor_gate_mutations = []
    wrong_status = copy.deepcopy(actor_gate)
    wrong_status["status"] = "implementation_in_progress"
    actor_gate_mutations.append(("actor_gate_status", wrong_status))
    wrong_remaining = copy.deepcopy(actor_gate)
    wrong_remaining["reproductions"]["remaining_finding_100_families"] = 6
    actor_gate_mutations.append(("actor_gate_remaining", wrong_remaining))
    closed_finding_100 = copy.deepcopy(actor_gate)
    closed_finding_100["reproductions"]["finding_100_status"] = "closed"
    actor_gate_mutations.append(("actor_gate_finding", closed_finding_100))
    wrong_work = copy.deepcopy(actor_gate)
    wrong_work["work_contract"]["production_bypasses"] = 1
    actor_gate_mutations.append(("actor_gate_work", wrong_work))
    wrong_binding = copy.deepcopy(actor_gate)
    wrong_binding["source_bindings"][0]["sha256"] = "0" * 64
    actor_gate_mutations.append(("actor_gate_binding", wrong_binding))
    for label, changed in actor_gate_mutations:
        try:
            validate_actor_gate(changed)
        except EvidenceError:
            continue
        raise EvidenceError("mutation_survived:" + label)
    return len(mutations) + len(finding_mutations) + len(reproduction_mutations) + len(policy_mutations) + len(gate_mutations) + len(actor_gate_mutations)


def main() -> None:
    authority = json.loads(AUTHORITY_PATH.read_text())
    ledger = json.loads(LEDGER_PATH.read_text())
    findings = json.loads(FINDINGS_PATH.read_text())
    reproductions = json.loads(REPRODUCTIONS_PATH.read_text())
    evidence_policy = json.loads(EVIDENCE_POLICY_PATH.read_text())
    authority_gate = json.loads(AUTHORITY_GATE_PATH.read_text())
    actor_gate = json.loads(ACTOR_GATE_PATH.read_text())
    validate_authority(authority)
    validate_ledger(ledger)
    validate_findings(findings)
    validate_reproductions(reproductions)
    validate_evidence_policy(evidence_policy)
    validate_authority_gate(authority_gate)
    validate_actor_gate(actor_gate)
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
    validate_signed_transitive_actor_constructions()
    validate_projected_causal_next_decision()
    source_mutations += causal_next_source_mutation_self_test()
    validate_projected_causal_next_production_path()
    source_mutations += causal_next_route_source_mutation_self_test()
    validate_streaming_empty_frontier()
    source_mutations += streaming_frontier_source_mutation_self_test()
    validate_combined_candidate_semantics()
    source_mutations += combined_candidate_source_mutation_self_test()
    validate_compact_epoch_ancestry()
    source_mutations += compact_epoch_ancestry_source_mutation_self_test()
    validate_metered_epoch_ancestry()
    source_mutations += metered_epoch_ancestry_source_mutation_self_test()
    validate_epoch_ancestry_production_path()
    source_mutations += epoch_ancestry_route_source_mutation_self_test()
    validate_shared_control_member_authorization()
    source_mutations += shared_control_member_authorization_source_mutation_self_test()
    validate_authorization_production_routes()
    source_mutations += authorization_production_source_mutation_self_test()
    validate_metered_candidate_dependency_closure()
    source_mutations += candidate_dependency_closure_source_mutation_self_test()
    validate_metered_candidate_schedule()
    source_mutations += candidate_schedule_source_mutation_self_test()
    validate_active_causal_budget_deltas()
    source_mutations += causal_budget_source_mutation_self_test()
    mutation_count = mutation_self_test(authority, ledger, findings, reproductions, evidence_policy, authority_gate, actor_gate)
    print("PASS: remediation v12 authority")
    print(f"- mutations={mutation_count}")
    print(f"- source_mutations={source_mutations}")
    print("- active=RCLD112/step_1394")


if __name__ == "__main__":
    main()
